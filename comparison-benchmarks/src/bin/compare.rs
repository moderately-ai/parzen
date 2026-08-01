use std::{
    collections::HashSet,
    ffi::OsString,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use parzen_comparison_benchmarks::{
    HarnessResult,
    cli::RunConfig,
    output::{BenchmarkRecord, ENVIRONMENT_SNAPSHOT_VAR, Environment},
    report::{read_jsonl, write_markdown},
    scenarios::{Operation, ParzenHistory, QUALITY_SEEDS, Scenario},
};
use wait_timeout::ChildExt;

#[derive(Clone, Copy)]
struct BackendSpec {
    label: &'static str,
    binary: &'static str,
    version: &'static str,
    history: ParzenHistory,
}

const BACKENDS: [BackendSpec; 5] = [
    BackendSpec {
        label: "parzen/full",
        binary: "bench-parzen",
        version: "0.2.0",
        history: ParzenHistory::Full,
    },
    BackendSpec {
        label: "parzen/bounded",
        binary: "bench-parzen",
        version: "0.2.0",
        history: ParzenHistory::Bounded,
    },
    BackendSpec {
        label: "tpe",
        binary: "bench-tpe",
        version: "0.3.1",
        history: ParzenHistory::Full,
    },
    BackendSpec {
        label: "hyperopt",
        binary: "bench-hyperopt",
        version: "0.0.17",
        history: ParzenHistory::Full,
    },
    BackendSpec {
        label: "optimizer",
        binary: "bench-optimizer",
        version: "1.0.1",
        history: ParzenHistory::Full,
    },
];

#[derive(Clone)]
struct Case {
    scenario: Scenario,
    operation: Operation,
    history: usize,
    dimensions: usize,
    budget: usize,
    seed: u64,
    iterations: usize,
    samples: usize,
    warmup: usize,
    calibration_ms: u64,
}

struct DriverCli {
    command: String,
    report_input: Option<PathBuf>,
    backend: String,
    output: Option<PathBuf>,
    machine_label: String,
    scenario: Option<Scenario>,
    operation: Option<Operation>,
    history: Option<usize>,
    rounds: usize,
    samples: Option<usize>,
    warmup: Option<usize>,
    calibration_ms: Option<u64>,
    quality_seeds: usize,
    timeout_seconds: u64,
    memory_binary_dir: Option<PathBuf>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("compare: {error}");
        std::process::exit(2);
    }
}

fn run() -> HarnessResult<()> {
    let cli = parse_cli(std::env::args_os().skip(1))?;
    if cli.command == "report" {
        let input = cli.report_input.ok_or("report requires a JSONL path")?;
        let output = cli.output.unwrap_or_else(|| input.with_extension("md"));
        write_report(&input, &output)?;
        println!("wrote {}", output.display());
        return Ok(());
    }
    if cfg!(feature = "dhat-heap") && cli.command != "memory" {
        return Err("allocator-instrumented binaries may only run the memory suite".into());
    }
    if !cfg!(feature = "dhat-heap") && cli.command == "memory" {
        return Err("memory suite requires binaries built with `--features dhat-heap`".into());
    }
    if cli.command == "full" && cli.memory_binary_dir.is_none() {
        return Err(
            "full suite requires `--memory-bin-dir` pointing to a separate dhat-heap build".into(),
        );
    }

    let backends = select_backends(&cli.backend)?;
    let output = cli.output.unwrap_or_else(default_output_path);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let executable_dir = std::env::current_exe()?
        .parent()
        .ok_or("compare executable has no parent directory")?
        .to_owned();
    let mut cases = cases_for(&cli.command)?;
    cases.retain(|case| {
        cli.scenario
            .is_none_or(|scenario| case.scenario == scenario)
    });
    cases.retain(|case| {
        cli.operation
            .is_none_or(|operation| case.operation == operation)
    });
    for case in &mut cases {
        if let Some(history) = cli.history {
            case.history = history;
        }
        if let Some(samples) = cli.samples {
            case.samples = samples;
        }
        if let Some(warmup) = cli.warmup {
            case.warmup = warmup;
        }
        if let Some(calibration_ms) = cli.calibration_ms {
            case.calibration_ms = calibration_ms;
        }
    }
    cases.retain(|case| {
        case.operation != Operation::Quality
            || QUALITY_SEEDS[..cli.quality_seeds].contains(&case.seed)
    });
    if cases.is_empty() {
        return Err("the selected filters produced no benchmark cases".into());
    }
    let timing_rounds = if matches!(cli.command.as_str(), "smoke" | "characterize") {
        1
    } else {
        cli.rounds
    };
    print_work_plan(&cli.command, &cases, backends.len(), timing_rounds);
    let suite_environment = Environment::capture_preflight(&cli.machine_label);
    let environment_snapshot = serde_json::to_string(&suite_environment)?;
    let mut writer = BufWriter::new(File::create(&output)?);
    let mut failed_operations = HashSet::new();
    let mut failed_suggestions = HashSet::new();

    for (case_index, case) in cases.iter().enumerate() {
        let rounds = if matches!(case.operation, Operation::Quality | Operation::Memory) {
            1
        } else {
            timing_rounds
        };
        for round in 0..rounds {
            let rotation = (case_index + round) % backends.len();
            for order in 0..backends.len() {
                let backend = backends[(rotation + order) % backends.len()];
                eprintln!(
                    "{} round {}: {} {} {}",
                    cli.command,
                    round + 1,
                    backend.label,
                    case.scenario,
                    case.operation
                );
                let binary_dir = if case.operation == Operation::Memory {
                    cli.memory_binary_dir.as_deref().unwrap_or(&executable_dir)
                } else {
                    &executable_dir
                };
                let operation_key = (backend.label, case.scenario, case.operation);
                let suggestion_key = (backend.label, case.scenario);
                let skip = failed_operations.contains(&operation_key)
                    || (matches!(case.operation, Operation::Suggest | Operation::Cycle)
                        && failed_suggestions.contains(&suggestion_key));
                let mut record = if skip {
                    BenchmarkRecord::execution_failed(
                        backend_name(backend),
                        backend.version,
                        run_config(backend, case, &cli.machine_label),
                        suite_environment.clone(),
                        "skipped after an earlier timeout for the same backend and scenario"
                            .to_owned(),
                    )
                } else {
                    invoke_backend(
                        binary_dir,
                        backend,
                        case,
                        &cli.machine_label,
                        &environment_snapshot,
                        &suite_environment,
                        Duration::from_secs(cli.timeout_seconds),
                    )?
                };
                if record.execution_error.is_some() && !skip {
                    failed_operations.insert(operation_key);
                    if matches!(case.operation, Operation::ColdSuggest | Operation::Suggest) {
                        failed_suggestions.insert(suggestion_key);
                    }
                }
                record.comparison_round = Some(if case.operation == Operation::Quality {
                    case_index
                } else {
                    round
                });
                record.invocation_order = Some(order);
                serde_json::to_writer(&mut writer, &record)?;
                writeln!(writer)?;
                writer.flush()?;
            }
        }
    }
    writer.flush()?;
    let markdown = output.with_extension("md");
    write_report(&output, &markdown)?;
    println!("wrote {} and {}", output.display(), markdown.display());
    Ok(())
}

fn parse_cli<I, S>(args: I) -> HarnessResult<DriverCli>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut args = args.into_iter().map(Into::into);
    let command = args
        .next()
        .ok_or_else(|| usage().to_owned())?
        .into_string()
        .map_err(|_| "arguments must be UTF-8")?;
    if !matches!(
        command.as_str(),
        "smoke" | "characterize" | "timing" | "scaling" | "quality" | "memory" | "full" | "report"
    ) {
        return Err(format!("unknown command `{command}`\n{}", usage()).into());
    }
    let mut report_input = None;
    if command == "report" {
        report_input = args.next().map(PathBuf::from);
    }
    let timeout_seconds = if command == "characterize" { 10 } else { 120 };
    let mut cli = DriverCli {
        command,
        report_input,
        backend: "all".into(),
        output: None,
        machine_label: "unlabelled".into(),
        scenario: None,
        operation: None,
        history: None,
        rounds: 3,
        samples: None,
        warmup: None,
        calibration_ms: None,
        quality_seeds: 8,
        timeout_seconds,
        memory_binary_dir: None,
    };
    while let Some(flag) = args.next() {
        let flag = flag.into_string().map_err(|_| "arguments must be UTF-8")?;
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for `{flag}`"))?;
        match flag.as_str() {
            "--backend" => {
                cli.backend = value.into_string().map_err(|_| "backend must be UTF-8")?
            }
            "--output" => cli.output = Some(PathBuf::from(value)),
            "--machine-label" => {
                cli.machine_label = value
                    .into_string()
                    .map_err(|_| "machine label must be UTF-8")?
            }
            "--scenario" => {
                cli.scenario = Some(
                    value
                        .into_string()
                        .map_err(|_| "scenario must be UTF-8")?
                        .parse()?,
                )
            }
            "--operation" => {
                cli.operation = Some(
                    value
                        .into_string()
                        .map_err(|_| "operation must be UTF-8")?
                        .parse()?,
                )
            }
            "--history" => {
                cli.history = Some(
                    value
                        .into_string()
                        .map_err(|_| "history must be UTF-8")?
                        .parse()?,
                )
            }
            "--rounds" => {
                cli.rounds = value
                    .into_string()
                    .map_err(|_| "rounds must be UTF-8")?
                    .parse()?
            }
            "--samples" => {
                cli.samples = Some(
                    value
                        .into_string()
                        .map_err(|_| "samples must be UTF-8")?
                        .parse()?,
                )
            }
            "--warmup" => {
                cli.warmup = Some(
                    value
                        .into_string()
                        .map_err(|_| "warmup must be UTF-8")?
                        .parse()?,
                )
            }
            "--calibration-ms" => {
                cli.calibration_ms = Some(
                    value
                        .into_string()
                        .map_err(|_| "calibration duration must be UTF-8")?
                        .parse()?,
                )
            }
            "--timeout-seconds" => {
                cli.timeout_seconds = value
                    .into_string()
                    .map_err(|_| "timeout must be UTF-8")?
                    .parse()?
            }
            "--quality-seeds" => {
                cli.quality_seeds = value
                    .into_string()
                    .map_err(|_| "quality seed count must be UTF-8")?
                    .parse()?
            }
            "--memory-bin-dir" => cli.memory_binary_dir = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown argument `{flag}`\n{}", usage()).into()),
        }
    }
    if cli.rounds == 0 {
        return Err("rounds must be positive".into());
    }
    if cli.history == Some(0) {
        return Err("history must be positive".into());
    }
    if cli.samples == Some(0) {
        return Err("samples must be positive".into());
    }
    if cli.calibration_ms == Some(0) {
        return Err("calibration duration must be positive".into());
    }
    if cli.timeout_seconds == 0 {
        return Err("timeout must be positive".into());
    }
    if cli.quality_seeds == 0 || cli.quality_seeds > QUALITY_SEEDS.len() {
        return Err(format!(
            "quality seed count must be between 1 and {}",
            QUALITY_SEEDS.len()
        )
        .into());
    }
    Ok(cli)
}

fn usage() -> &'static str {
    "compare <smoke|characterize|timing|scaling|quality|memory|full|report JSONL> \
     [--backend all|NAME] [--scenario NAME] [--operation NAME] [--history N] \
     [--output PATH] [--machine-label LABEL] [--rounds N] \
     [--samples N] [--warmup N] [--calibration-ms N] [--timeout-seconds N] \
     [--quality-seeds N] [--memory-bin-dir PATH]"
}

fn select_backends(selection: &str) -> HarnessResult<Vec<BackendSpec>> {
    if selection == "all" {
        return Ok(BACKENDS.to_vec());
    }
    let selected = BACKENDS
        .iter()
        .copied()
        .filter(|backend| {
            backend.label == selection
                || (selection == "parzen" && backend.label.starts_with("parzen/"))
        })
        .collect::<Vec<_>>();
    if selected.is_empty() {
        Err(format!("unknown backend `{selection}`").into())
    } else {
        Ok(selected)
    }
}

fn invoke_backend(
    dir: &Path,
    backend: BackendSpec,
    case: &Case,
    machine_label: &str,
    environment_snapshot: &str,
    suite_environment: &Environment,
    timeout: Duration,
) -> HarnessResult<BenchmarkRecord> {
    let binary = dir.join(backend.binary);
    if !binary.is_file() {
        return Err(format!(
            "missing {}; build all binaries before measurement",
            binary.display()
        )
        .into());
    }
    let config = run_config(backend, case, machine_label);
    let mut child = Command::new(&binary)
        .env(ENVIRONMENT_SNAPSHOT_VAR, environment_snapshot)
        .args([
            "--scenario",
            &case.scenario.to_string(),
            "--operation",
            &case.operation.to_string(),
        ])
        .args([
            "--history",
            &case.history.to_string(),
            "--dimensions",
            &case.dimensions.to_string(),
        ])
        .args([
            "--iterations",
            &case.iterations.to_string(),
            "--budget",
            &case.budget.to_string(),
        ])
        .args([
            "--seed",
            &case.seed.to_string(),
            "--samples",
            &case.samples.to_string(),
        ])
        .args([
            "--warmup",
            &case.warmup.to_string(),
            "--calibration-ms",
            &case.calibration_ms.to_string(),
            "--parzen-history",
            match backend.history {
                ParzenHistory::Full => "full",
                ParzenHistory::Bounded => "bounded",
            },
        ])
        .args(["--machine-label", machine_label, "--format", "json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if child.wait_timeout(timeout)?.is_none() {
        child.kill()?;
        let _ = child.wait_with_output()?;
        return Ok(BenchmarkRecord::timed_out(
            backend_name(backend),
            backend.version,
            config,
            suite_environment.clone(),
            timeout.as_secs(),
        ));
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(format!(
            "{} failed: {}",
            backend.label,
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn backend_name(backend: BackendSpec) -> &'static str {
    if backend.label.starts_with("parzen/") {
        "parzen"
    } else {
        backend.label
    }
}

fn run_config(backend: BackendSpec, case: &Case, machine_label: &str) -> RunConfig {
    RunConfig {
        scenario: case.scenario,
        operation: case.operation,
        history: case.history,
        dimensions: case.dimensions,
        iterations: case.iterations,
        budget: case.budget,
        seed: case.seed,
        samples: case.samples,
        warmup: case.warmup,
        calibration_ms: case.calibration_ms,
        profile_seconds: 30,
        parzen_history: backend.history,
        machine_label: machine_label.to_owned(),
    }
}

fn base_case(scenario: Scenario, operation: Operation) -> Case {
    Case {
        scenario,
        operation,
        history: 1_000,
        dimensions: scenario.default_dimensions(),
        budget: 100,
        seed: 42,
        iterations: 0,
        samples: 5,
        warmup: 1,
        calibration_ms: 100,
    }
}

fn cases_for(command: &str) -> HarnessResult<Vec<Case>> {
    let cases = match command {
        "smoke" => vec![Case {
            history: 10,
            iterations: 2,
            samples: 1,
            warmup: 0,
            ..base_case(Scenario::LinearFloat, Operation::Cycle)
        }],
        "characterize" => Scenario::COMPARATIVE
            .into_iter()
            .flat_map(|scenario| {
                [
                    Operation::Construct,
                    Operation::Ingest,
                    Operation::ColdSuggest,
                    Operation::Suggest,
                    Operation::Update,
                    Operation::Cycle,
                ]
                .into_iter()
                .map(move |operation| Case {
                    iterations: 1,
                    samples: 1,
                    warmup: 0,
                    calibration_ms: 1,
                    ..base_case(scenario, operation)
                })
            })
            .collect(),
        "timing" => Scenario::COMPARATIVE
            .into_iter()
            .flat_map(|scenario| {
                [
                    Operation::Construct,
                    Operation::Ingest,
                    Operation::ColdSuggest,
                    Operation::Suggest,
                    Operation::Update,
                    Operation::Cycle,
                ]
                .into_iter()
                .map(move |operation| base_case(scenario, operation))
            })
            .collect(),
        "scaling" => {
            let mut cases = Vec::new();
            for scenario in [
                Scenario::LinearFloat,
                Scenario::IndependentFloat,
                Scenario::CorrelatedNumeric,
            ] {
                for history in [10, 100, 1_000, 10_000, 100_000] {
                    for dimensions in [1, 4, 8, 16] {
                        if scenario == Scenario::LinearFloat && dimensions != 1 {
                            continue;
                        }
                        if scenario == Scenario::CorrelatedNumeric && dimensions == 1 {
                            continue;
                        }
                        cases.push(Case {
                            history,
                            dimensions,
                            ..base_case(scenario, Operation::Suggest)
                        });
                    }
                }
            }
            cases
        }
        "quality" => Scenario::COMPARATIVE
            .into_iter()
            .flat_map(|scenario| {
                [25, 50, 100, 250].into_iter().flat_map(move |budget| {
                    QUALITY_SEEDS.into_iter().map(move |seed| Case {
                        budget,
                        seed,
                        samples: 1,
                        warmup: 0,
                        ..base_case(scenario, Operation::Quality)
                    })
                })
            })
            .collect(),
        "memory" => Scenario::COMPARATIVE
            .into_iter()
            .flat_map(|scenario| {
                [1_000, 10, 100_000, 100, 10_000]
                    .into_iter()
                    .map(move |history| Case {
                        history,
                        iterations: 100,
                        samples: 1,
                        warmup: 0,
                        ..base_case(scenario, Operation::Memory)
                    })
            })
            .collect(),
        "full" => {
            let mut cases = cases_for("timing")?;
            cases.extend(cases_for("scaling")?);
            cases.extend(cases_for("quality")?);
            cases.extend(cases_for("memory")?);
            cases
        }
        _ => return Err(format!("cannot create cases for `{command}`").into()),
    };
    Ok(cases)
}

fn print_work_plan(command: &str, cases: &[Case], backend_count: usize, timing_rounds: usize) {
    let invocations = cases
        .iter()
        .map(|case| {
            let rounds = if matches!(case.operation, Operation::Quality | Operation::Memory) {
                1
            } else {
                timing_rounds
            };
            rounds * backend_count
        })
        .sum::<usize>();
    let timed_batch_ms = cases
        .iter()
        .filter(|case| case.iterations == 0 && case.operation.is_batchable())
        .map(|case| case.samples as u128 * case.calibration_ms as u128)
        .sum::<u128>()
        * backend_count as u128
        * timing_rounds as u128;
    let adaptive_evaluations = cases
        .iter()
        .filter(|case| case.operation == Operation::Quality)
        .map(|case| case.budget)
        .sum::<usize>()
        * backend_count;
    let memory_observations = cases
        .iter()
        .filter(|case| case.operation == Operation::Memory)
        .map(|case| case.history.saturating_add(case.iterations))
        .sum::<usize>()
        * backend_count;
    eprintln!(
        "plan: {command} has {} cases and {invocations} backend invocations; calibrated timed batches request up to {:.1}s if every selected backend supports every case, excluding calibration and setup; up to {adaptive_evaluations} adaptive quality evaluations and {memory_observations} memory observations",
        cases.len(),
        timed_batch_ms as f64 / 1_000.0
    );
}

fn default_output_path() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs());
    PathBuf::from(format!(
        "comparison-benchmarks/results/raw/run-{timestamp}.jsonl"
    ))
}

fn write_report(input: &Path, output: &Path) -> HarnessResult<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let records = read_jsonl(input)?;
    let mut markdown = Vec::new();
    write_markdown(&records, &mut markdown)?;
    fs::write(output, markdown)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn characterization_executes_each_timing_case_once_without_calibration() {
        let cases = cases_for("characterize").expect("cases");
        assert_eq!(cases.len(), Scenario::COMPARATIVE.len() * 6);
        assert!(
            cases
                .iter()
                .all(|case| { case.iterations == 1 && case.samples == 1 && case.warmup == 0 })
        );
    }

    #[test]
    fn routine_timing_defaults_are_smaller_than_curated_protocol() {
        let cases = cases_for("timing").expect("cases");
        assert!(
            cases.iter().all(|case| {
                case.samples == 5 && case.warmup == 1 && case.calibration_ms == 100
            })
        );
        let cli = parse_cli(["timing"]).expect("CLI");
        assert_eq!(cli.rounds, 3);
    }

    #[test]
    fn driver_overrides_reject_zero_work() {
        assert!(parse_cli(["timing", "--samples", "0"]).is_err());
        assert!(parse_cli(["timing", "--calibration-ms", "0"]).is_err());
        assert!(parse_cli(["timing", "--rounds", "0"]).is_err());
        assert!(parse_cli(["timing", "--timeout-seconds", "0"]).is_err());
        assert!(parse_cli(["quality", "--quality-seeds", "0"]).is_err());
        assert!(parse_cli(["quality", "--quality-seeds", "33"]).is_err());
        assert!(parse_cli(["timing", "--history", "0"]).is_err());
    }

    #[test]
    fn driver_parses_case_filters() {
        let cli = parse_cli([
            "timing",
            "--scenario",
            "independent-float",
            "--operation",
            "suggest",
            "--history",
            "100",
        ])
        .expect("CLI");
        assert_eq!(cli.scenario, Some(Scenario::IndependentFloat));
        assert_eq!(cli.operation, Some(Operation::Suggest));
        assert_eq!(cli.history, Some(100));
    }
}
