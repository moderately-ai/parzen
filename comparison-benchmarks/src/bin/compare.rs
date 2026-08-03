use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use parzen_comparison_benchmarks::{
    HarnessResult,
    cli::{BenchmarkProtocol, ProfileWorkload, RunConfig},
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
    integer_cardinality: usize,
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
    dimensions: Option<usize>,
    integer_cardinality: Option<usize>,
    protocol: BenchmarkProtocol,
    rounds: Option<usize>,
    samples: Option<usize>,
    warmup: Option<usize>,
    calibration_ms: Option<u64>,
    quality_seeds: usize,
    case_timeout_seconds: Option<u64>,
    suite_timeout_seconds: Option<u64>,
    allow_long_run: bool,
    plan_only: bool,
    resume: bool,
    shard: Option<Shard>,
    memory_binary_dir: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Shard {
    index: usize,
    count: usize,
}

impl std::str::FromStr for Shard {
    type Err = parzen_comparison_benchmarks::HarnessError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let (index, count) = value
            .split_once('/')
            .ok_or("shard must use INDEX/COUNT syntax")?;
        let index = index.parse::<usize>()?;
        let count = count.parse::<usize>()?;
        if index == 0 || count == 0 || index > count {
            return Err("shard index must be between 1 and its positive count".into());
        }
        Ok(Self {
            index: index - 1,
            count,
        })
    }
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
    let output = cli.output.clone().unwrap_or_else(default_output_path);
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
    apply_history_filter(&cli.command, cli.history, &mut cases);
    apply_dimension_filter(&cli.command, cli.dimensions, &mut cases);
    for case in &mut cases {
        if let Some(history) = cli.history
            && !matches!(cli.command.as_str(), "scaling" | "memory" | "full")
        {
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
        if let Some(cardinality) = cli.integer_cardinality {
            case.integer_cardinality = cardinality;
        }
    }
    cases.retain(|case| {
        case.operation != Operation::Quality
            || QUALITY_SEEDS[..cli.quality_seeds].contains(&case.seed)
    });
    if cases.is_empty() {
        return Err("the selected filters produced no benchmark cases".into());
    }
    if let Some(shard) = cli.shard {
        cases = select_shard(cases, shard);
        if cases.is_empty() {
            return Err("the selected shard contains no benchmark cases".into());
        }
    }
    let protocol = cli.protocol;
    for case in &mut cases {
        if cli.samples.is_none() {
            case.samples = protocol.samples();
        }
        if cli.warmup.is_none() {
            case.warmup = protocol.warmups();
        }
        if cli.calibration_ms.is_none() {
            case.calibration_ms = protocol.calibration_ms();
        }
    }
    let timing_rounds = if matches!(cli.command.as_str(), "smoke" | "characterize") {
        1
    } else {
        cli.rounds.unwrap_or_else(|| protocol.rounds())
    };
    let case_timeout_seconds = cli
        .case_timeout_seconds
        .unwrap_or_else(|| protocol.case_timeout_seconds());
    let suite_timeout_seconds = cli
        .suite_timeout_seconds
        .unwrap_or_else(|| protocol.suite_timeout_seconds());
    let estimate = print_work_plan(
        &cli,
        &output,
        &cases,
        backends.len(),
        timing_rounds,
        case_timeout_seconds,
        suite_timeout_seconds,
    );
    if cli.plan_only {
        return Ok(());
    }
    validate_estimated_duration(estimate, cli.shard.is_some(), cli.allow_long_run)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let suite_environment = Environment::capture_preflight(&cli.machine_label);
    let environment_snapshot = serde_json::to_string(&suite_environment)?;
    let existing = if cli.resume && output.exists() {
        read_jsonl(&output)?
    } else {
        Vec::new()
    };
    let mut completed = existing
        .iter()
        .filter_map(completed_key)
        .collect::<HashSet<_>>();
    let mut calibrations = existing
        .iter()
        .filter_map(|record| {
            Some((
                calibration_key(record.binary_checksum.as_deref()?, &record.config),
                record.calibration_iterations?,
            ))
        })
        .collect::<HashMap<_, _>>();
    let file = if cli.resume {
        OpenOptions::new().create(true).append(true).open(&output)?
    } else {
        File::create(&output)?
    };
    let mut writer = BufWriter::new(file);
    let mut failed_operations = HashSet::new();
    let mut failed_suggestions = HashSet::new();
    let suite_started = Instant::now();
    let mut suite_expired = false;

    for (case_index, case) in cases.iter().enumerate() {
        let rounds = if matches!(case.operation, Operation::Quality | Operation::Memory) {
            1
        } else {
            timing_rounds
        };
        for round in 0..rounds {
            let rotation = (case_index + round) % backends.len();
            for order in 0..backends.len() {
                if suite_started.elapsed() >= Duration::from_secs(suite_timeout_seconds) {
                    suite_expired = true;
                    break;
                }
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
                let binary = binary_dir.join(backend.binary);
                let binary_checksum = sha256_file(&binary)?;
                let config = run_config(backend, case, &cli.machine_label, protocol);
                let calibration_key = calibration_key(&binary_checksum, &config);
                let reused_calibration = (case.iterations == 0 && case.operation.is_batchable())
                    .then(|| calibrations.get(&calibration_key).copied())
                    .flatten();
                let key = result_key(
                    &suite_environment.git_commit,
                    &binary_checksum,
                    backend,
                    &config,
                    round,
                );
                if completed.contains(&key) {
                    eprintln!("resume: skipping completed {key}");
                    continue;
                }
                let operation_key = (backend.label, case.scenario, case.operation);
                let suggestion_key = (backend.label, case.scenario);
                let skip = failed_operations.contains(&operation_key)
                    || (matches!(case.operation, Operation::Suggest | Operation::Cycle)
                        && failed_suggestions.contains(&suggestion_key));
                let mut record = if skip {
                    BenchmarkRecord::execution_failed(
                        backend_name(backend),
                        backend.version,
                        config.clone(),
                        suite_environment.clone(),
                        "skipped after an earlier timeout for the same backend and scenario"
                            .to_owned(),
                    )
                } else {
                    invoke_backend(
                        binary_dir,
                        backend,
                        &config,
                        &environment_snapshot,
                        &suite_environment,
                        Duration::from_secs(case_timeout_seconds),
                        reused_calibration,
                    )?
                };
                record.benchmark_protocol = protocol;
                record.case_timeout_seconds = Some(case_timeout_seconds);
                record.suite_timeout_seconds = Some(suite_timeout_seconds);
                record.shard = cli
                    .shard
                    .map(|shard| format!("{}/{}", shard.index + 1, shard.count));
                record.binary_checksum = Some(binary_checksum);
                record.mix_driver_metadata_checksum();
                if record.execution_error.is_none()
                    && let Some(iterations) = record.calibration_iterations
                {
                    calibrations.insert(calibration_key, iterations);
                }
                if record.execution_error.is_some() && !skip {
                    failed_operations.insert(operation_key);
                    if matches!(case.operation, Operation::ColdSuggest | Operation::Suggest) {
                        failed_suggestions.insert(suggestion_key);
                    }
                }
                record.comparison_round = Some(round);
                record.invocation_order = Some(order);
                serde_json::to_writer(&mut writer, &record)?;
                writeln!(writer)?;
                writer.flush()?;
                completed.insert(key);
            }
            if suite_expired {
                break;
            }
        }
        if suite_expired {
            break;
        }
    }
    writer.flush()?;
    let markdown = output.with_extension("md");
    write_report(&output, &markdown)?;
    println!("wrote {} and {}", output.display(), markdown.display());
    if suite_expired {
        eprintln!(
            "suite timeout reached after {:.1}s; completed records were preserved and may be continued with --resume",
            suite_started.elapsed().as_secs_f64()
        );
    }
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
    let characterize = command == "characterize";
    let mut cli = DriverCli {
        command,
        report_input,
        backend: "all".into(),
        output: None,
        machine_label: "unlabelled".into(),
        scenario: None,
        operation: None,
        history: None,
        dimensions: None,
        integer_cardinality: None,
        protocol: BenchmarkProtocol::Quick,
        rounds: None,
        samples: None,
        warmup: None,
        calibration_ms: None,
        quality_seeds: 8,
        case_timeout_seconds: characterize.then_some(10),
        suite_timeout_seconds: None,
        allow_long_run: false,
        plan_only: false,
        resume: false,
        shard: None,
        memory_binary_dir: None,
    };
    while let Some(flag) = args.next() {
        let flag = flag.into_string().map_err(|_| "arguments must be UTF-8")?;
        match flag.as_str() {
            "--allow-long-run" => {
                cli.allow_long_run = true;
                continue;
            }
            "--plan" => {
                cli.plan_only = true;
                continue;
            }
            "--resume" => {
                cli.resume = true;
                continue;
            }
            _ => {}
        }
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for `{flag}`"))?;
        match flag.as_str() {
            "--protocol" => {
                cli.protocol = value
                    .into_string()
                    .map_err(|_| "protocol must be UTF-8")?
                    .parse()?
            }
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
            "--dimensions" => {
                cli.dimensions = Some(
                    value
                        .into_string()
                        .map_err(|_| "dimensions must be UTF-8")?
                        .parse()?,
                )
            }
            "--integer-cardinality" => {
                cli.integer_cardinality = Some(
                    value
                        .into_string()
                        .map_err(|_| "integer cardinality must be UTF-8")?
                        .parse()?,
                )
            }
            "--rounds" => {
                cli.rounds = Some(
                    value
                        .into_string()
                        .map_err(|_| "rounds must be UTF-8")?
                        .parse()?,
                )
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
            "--case-timeout-seconds" | "--timeout-seconds" => {
                cli.case_timeout_seconds = Some(
                    value
                        .into_string()
                        .map_err(|_| "timeout must be UTF-8")?
                        .parse()?,
                )
            }
            "--suite-timeout-seconds" => {
                cli.suite_timeout_seconds = Some(
                    value
                        .into_string()
                        .map_err(|_| "suite timeout must be UTF-8")?
                        .parse()?,
                )
            }
            "--quality-seeds" => {
                cli.quality_seeds = value
                    .into_string()
                    .map_err(|_| "quality seed count must be UTF-8")?
                    .parse()?
            }
            "--memory-bin-dir" => cli.memory_binary_dir = Some(PathBuf::from(value)),
            "--shard" => {
                cli.shard = Some(
                    value
                        .into_string()
                        .map_err(|_| "shard must be UTF-8")?
                        .parse()?,
                )
            }
            _ => return Err(format!("unknown argument `{flag}`\n{}", usage()).into()),
        }
    }
    if cli.rounds == Some(0) {
        return Err("rounds must be positive".into());
    }
    if cli.history == Some(0) {
        return Err("history must be positive".into());
    }
    if cli.dimensions == Some(0) {
        return Err("dimensions must be positive".into());
    }
    if cli.integer_cardinality.is_some_and(|value| value < 2) {
        return Err("integer cardinality must be at least two".into());
    }
    if cli.samples == Some(0) {
        return Err("samples must be positive".into());
    }
    if cli.calibration_ms == Some(0) {
        return Err("calibration duration must be positive".into());
    }
    if cli.case_timeout_seconds == Some(0) {
        return Err("timeout must be positive".into());
    }
    if cli.suite_timeout_seconds == Some(0) {
        return Err("suite timeout must be positive".into());
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
     [--protocol quick|checkpoint|curated] [--plan] [--resume] [--shard INDEX/COUNT] \
     [--backend all|NAME[,NAME...]] [--scenario NAME] [--operation NAME] \
     [--history N] [--dimensions N] [--integer-cardinality N] \
     [--output PATH] [--machine-label LABEL] [--rounds N] \
     [--samples N] [--warmup N] [--calibration-ms N] \
     [--case-timeout-seconds N] [--suite-timeout-seconds N] [--allow-long-run] \
     [--quality-seeds N] [--memory-bin-dir PATH]"
}

fn select_backends(selection: &str) -> HarnessResult<Vec<BackendSpec>> {
    if selection == "all" {
        return Ok(BACKENDS.to_vec());
    }
    let requested = selection.split(',').collect::<Vec<_>>();
    if requested.iter().any(|name| name.is_empty()) {
        return Err("backend selection contains an empty name".into());
    }
    for name in &requested {
        if *name != "parzen" && !BACKENDS.iter().any(|backend| backend.label == *name) {
            return Err(format!("unknown backend `{name}`").into());
        }
    }
    let selected = BACKENDS
        .iter()
        .copied()
        .filter(|backend| {
            requested.iter().any(|name| {
                backend.label == *name
                    || (*name == "parzen" && backend.label.starts_with("parzen/"))
            })
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
    config: &RunConfig,
    environment_snapshot: &str,
    suite_environment: &Environment,
    timeout: Duration,
    calibrated_iterations: Option<usize>,
) -> HarnessResult<BenchmarkRecord> {
    let binary = dir.join(backend.binary);
    if !binary.is_file() {
        return Err(format!(
            "missing {}; build all binaries before measurement",
            binary.display()
        )
        .into());
    }
    let mut command = Command::new(&binary);
    command
        .env(ENVIRONMENT_SNAPSHOT_VAR, environment_snapshot)
        .args([
            "--protocol",
            &config.protocol.to_string(),
            "--scenario",
            &config.scenario.to_string(),
            "--operation",
            &config.operation.to_string(),
        ])
        .args([
            "--history",
            &config.history.to_string(),
            "--dimensions",
            &config.dimensions.to_string(),
            "--integer-cardinality",
            &config.integer_cardinality.to_string(),
        ])
        .args([
            "--iterations",
            &config.iterations.to_string(),
            "--budget",
            &config.budget.to_string(),
        ])
        .args([
            "--seed",
            &config.seed.to_string(),
            "--samples",
            &config.samples.to_string(),
        ])
        .args([
            "--warmup",
            &config.warmup.to_string(),
            "--calibration-ms",
            &config.calibration_ms.to_string(),
            "--parzen-history",
            match backend.history {
                ParzenHistory::Full => "full",
                ParzenHistory::Bounded => "bounded",
            },
        ])
        .args(["--machine-label", &config.machine_label, "--format", "json"]);
    if let Some(iterations) = calibrated_iterations {
        command.args(["--calibrated-iterations", &iterations.to_string()]);
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    if child.wait_timeout(timeout)?.is_none() {
        child.kill()?;
        let _ = child.wait_with_output()?;
        return Ok(BenchmarkRecord::timed_out(
            backend_name(backend),
            backend.version,
            config.clone(),
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

fn run_config(
    backend: BackendSpec,
    case: &Case,
    machine_label: &str,
    protocol: BenchmarkProtocol,
) -> RunConfig {
    RunConfig {
        protocol,
        scenario: case.scenario,
        operation: case.operation,
        history: case.history,
        dimensions: case.dimensions,
        integer_cardinality: case.integer_cardinality,
        iterations: case.iterations,
        budget: case.budget,
        seed: case.seed,
        samples: case.samples,
        warmup: case.warmup,
        calibration_ms: case.calibration_ms,
        profile_seconds: 30,
        profile_workload: ProfileWorkload::Cycle,
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
        integer_cardinality: 201,
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

fn apply_history_filter(command: &str, history: Option<usize>, cases: &mut Vec<Case>) {
    if let Some(history) = history {
        if matches!(command, "scaling" | "memory" | "full") {
            cases.retain(|case| case.history == history);
        } else {
            for case in cases {
                case.history = history;
            }
        }
    }
}

fn apply_dimension_filter(command: &str, dimensions: Option<usize>, cases: &mut Vec<Case>) {
    if let Some(dimensions) = dimensions {
        if matches!(command, "scaling" | "full") {
            cases.retain(|case| case.dimensions == dimensions);
        } else {
            for case in cases {
                case.dimensions = dimensions;
            }
        }
    }
}

fn print_work_plan(
    cli: &DriverCli,
    output: &Path,
    cases: &[Case],
    backend_count: usize,
    timing_rounds: usize,
    case_timeout_seconds: u64,
    suite_timeout_seconds: u64,
) -> Duration {
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
    let timing_seconds = timed_batch_ms as f64 / 1_000.0;
    let setup_seconds = invocations as f64 * 0.25;
    let quality_seconds = adaptive_evaluations as f64 * 0.005;
    let memory_seconds = memory_observations as f64 * 0.000_02;
    let estimated_seconds = timing_seconds + setup_seconds + quality_seconds + memory_seconds;
    let worst_case_seconds = invocations as u64 * case_timeout_seconds;
    eprintln!(
        "plan: {} protocol={} cases={} backends={} invocations={} output={} shard={}\n  timed batches: {:.1}s; adaptive evaluations: {}; memory observations: {}\n  estimated duration: {:.1}s; timeout ceiling: {:.1}s; per-case timeout: {}s; suite timeout: {}s; allow-long-run required: {}",
        cli.command,
        cli.protocol,
        cases.len(),
        backend_count,
        invocations,
        output.display(),
        cli.shard.map_or_else(
            || "all".to_owned(),
            |shard| format!("{}/{}", shard.index + 1, shard.count)
        ),
        timing_seconds,
        adaptive_evaluations,
        memory_observations,
        estimated_seconds,
        worst_case_seconds,
        case_timeout_seconds,
        suite_timeout_seconds,
        estimated_seconds > 45.0 * 60.0,
    );
    Duration::from_secs_f64(estimated_seconds)
}

fn validate_estimated_duration(
    estimate: Duration,
    sharded: bool,
    allow_long_run: bool,
) -> HarnessResult<()> {
    if allow_long_run {
        return Ok(());
    }
    if sharded && estimate > Duration::from_secs(20 * 60) {
        return Err(format!(
            "estimated shard duration {:.1} minutes exceeds 20 minutes; increase the shard count or pass --allow-long-run",
            estimate.as_secs_f64() / 60.0
        )
        .into());
    }
    if estimate > Duration::from_secs(45 * 60) {
        return Err(format!(
            "estimated suite duration {:.1} minutes exceeds 45 minutes; narrow or shard the run, or pass --allow-long-run",
            estimate.as_secs_f64() / 60.0
        )
        .into());
    }
    Ok(())
}

fn select_shard(cases: Vec<Case>, shard: Shard) -> Vec<Case> {
    let mut weighted = cases
        .into_iter()
        .enumerate()
        .map(|(original, case)| (estimated_case_weight(&case), original, case))
        .collect::<Vec<_>>();
    weighted.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));
    let mut loads = vec![0_u128; shard.count];
    let mut buckets = (0..shard.count).map(|_| Vec::new()).collect::<Vec<_>>();
    for (weight, original, case) in weighted {
        let target = loads
            .iter()
            .enumerate()
            .min_by_key(|(index, load)| (**load, *index))
            .map_or(0, |(index, _)| index);
        loads[target] = loads[target].saturating_add(weight);
        buckets[target].push((original, case));
    }
    let mut selected = std::mem::take(&mut buckets[shard.index]);
    selected.sort_by_key(|(original, _)| *original);
    selected.into_iter().map(|(_, case)| case).collect()
}

fn estimated_case_weight(case: &Case) -> u128 {
    match case.operation {
        Operation::Quality => case.budget as u128 * 5_000,
        Operation::Memory => case.history.saturating_add(case.iterations) as u128 * 20,
        _ => case.samples as u128 * case.calibration_ms as u128 * 1_000,
    }
}

fn sha256_file(path: &Path) -> HarnessResult<String> {
    if !path.is_file() {
        return Err(format!(
            "missing {}; build all binaries before measurement",
            path.display()
        )
        .into());
    }
    let output = if cfg!(target_os = "macos") {
        Command::new("shasum")
            .args(["-a", "256"])
            .arg(path)
            .output()?
    } else {
        Command::new("sha256sum").arg(path).output()?
    };
    if !output.status.success() {
        return Err(format!("failed to checksum {}", path.display()).into());
    }
    String::from_utf8(output.stdout)?
        .split_whitespace()
        .next()
        .map(str::to_owned)
        .ok_or_else(|| "checksum command returned no digest".into())
}

fn result_key(
    commit: &str,
    binary_checksum: &str,
    backend: BackendSpec,
    config: &RunConfig,
    round: usize,
) -> String {
    format!(
        "{commit}:{binary_checksum}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{round}",
        backend.label,
        config.scenario,
        config.operation,
        config.history,
        config.dimensions,
        config.integer_cardinality,
        config.seed,
        config.budget,
        config.protocol,
        config.iterations,
        config.samples,
        config.warmup,
        config.calibration_ms,
        format_args!("{:?}", config.parzen_history),
    )
}

fn calibration_key(binary_checksum: &str, config: &RunConfig) -> String {
    format!(
        "{binary_checksum}:{}:{}:{}:{}:{}:{}:{}:{}:{:?}",
        config.scenario,
        config.operation,
        config.history,
        config.dimensions,
        config.integer_cardinality,
        config.seed,
        config.protocol,
        config.calibration_ms,
        config.parzen_history,
    )
}

fn completed_key(record: &BenchmarkRecord) -> Option<String> {
    let checksum = record.binary_checksum.as_deref()?;
    let backend = if record.backend == "parzen" {
        match record.config.parzen_history {
            ParzenHistory::Full => BACKENDS[0],
            ParzenHistory::Bounded => BACKENDS[1],
        }
    } else {
        BACKENDS
            .iter()
            .copied()
            .find(|backend| backend.label == record.backend)?
    };
    Some(result_key(
        &record.environment.git_commit,
        checksum,
        backend,
        &record.config,
        record.comparison_round.unwrap_or(0),
    ))
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
        assert_eq!(cli.rounds, None);
        assert_eq!(cli.protocol.rounds(), 2);
    }

    #[test]
    fn driver_overrides_reject_zero_work() {
        assert!(parse_cli(["timing", "--samples", "0"]).is_err());
        assert!(parse_cli(["timing", "--calibration-ms", "0"]).is_err());
        assert!(parse_cli(["timing", "--rounds", "0"]).is_err());
        assert!(parse_cli(["timing", "--timeout-seconds", "0"]).is_err());
        assert!(parse_cli(["timing", "--suite-timeout-seconds", "0"]).is_err());
        assert!(parse_cli(["quality", "--quality-seeds", "0"]).is_err());
        assert!(parse_cli(["quality", "--quality-seeds", "33"]).is_err());
        assert!(parse_cli(["timing", "--history", "0"]).is_err());
        assert!(parse_cli(["timing", "--dimensions", "0"]).is_err());
    }

    #[test]
    fn protocols_supply_bounded_defaults_and_flags_do_not_consume_values() {
        let quick = parse_cli(["timing", "--protocol", "quick", "--plan"]).expect("quick");
        assert_eq!(quick.protocol, BenchmarkProtocol::Quick);
        assert!(quick.plan_only);
        assert_eq!(quick.protocol.suite_timeout_seconds(), 8 * 60);

        let curated = parse_cli([
            "timing",
            "--protocol",
            "curated",
            "--resume",
            "--allow-long-run",
        ])
        .expect("curated");
        assert_eq!(curated.protocol, BenchmarkProtocol::Curated);
        assert!(curated.resume);
        assert!(curated.allow_long_run);
    }

    #[test]
    fn shards_parse_and_partition_every_case_once() {
        assert_eq!(
            "1/3".parse::<Shard>().expect("shard"),
            Shard { index: 0, count: 3 }
        );
        assert!("0/3".parse::<Shard>().is_err());
        assert!("4/3".parse::<Shard>().is_err());
        let cases = cases_for("scaling").expect("cases");
        let expected = cases.len();
        let selected = (0..3)
            .flat_map(|index| select_shard(cases.clone(), Shard { index, count: 3 }))
            .count();
        assert_eq!(selected, expected);
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
            "--dimensions",
            "8",
            "--integer-cardinality",
            "4096",
        ])
        .expect("CLI");
        assert_eq!(cli.scenario, Some(Scenario::IndependentFloat));
        assert_eq!(cli.operation, Some(Operation::Suggest));
        assert_eq!(cli.history, Some(100));
        assert_eq!(cli.dimensions, Some(8));
        assert_eq!(cli.integer_cardinality, Some(4_096));
    }

    #[test]
    fn duration_guard_caps_shards_and_unsharded_suites() {
        assert!(validate_estimated_duration(Duration::from_secs(20 * 60), true, false).is_ok());
        assert!(
            validate_estimated_duration(Duration::from_secs(20 * 60 + 1), true, false).is_err()
        );
        assert!(validate_estimated_duration(Duration::from_secs(45 * 60), false, false).is_ok());
        assert!(
            validate_estimated_duration(Duration::from_secs(45 * 60 + 1), false, false).is_err()
        );
        assert!(validate_estimated_duration(Duration::from_secs(3 * 60 * 60), true, true).is_ok());
    }

    #[test]
    fn result_keys_include_timing_overrides_but_calibration_keys_do_not() {
        let backend = BACKENDS[0];
        let first = RunConfig {
            dimensions: 1,
            ..RunConfig::default()
        };
        let mut second = first.clone();
        second.samples += 1;
        assert_ne!(
            result_key("commit", "binary", backend, &first, 0),
            result_key("commit", "binary", backend, &second, 0)
        );
        assert_eq!(
            calibration_key("binary", &first),
            calibration_key("binary", &second)
        );
    }

    #[test]
    fn backend_selection_accepts_exact_comma_separated_subset() {
        let selected = select_backends("parzen,optimizer").expect("backends");
        assert_eq!(
            selected
                .iter()
                .map(|backend| backend.label)
                .collect::<Vec<_>>(),
            vec!["parzen/full", "parzen/bounded", "optimizer"]
        );
        assert!(select_backends("parzen,unknown").is_err());
        assert!(select_backends("parzen,").is_err());
    }

    #[test]
    fn memory_history_filter_selects_one_case() {
        let mut cases = cases_for("memory").expect("cases");
        cases.retain(|case| case.scenario == Scenario::IndependentFloat);
        apply_history_filter("memory", Some(1_000), &mut cases);
        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].history, 1_000);
    }

    #[test]
    fn timing_history_filter_overrides_the_default_case() {
        let mut timing = vec![base_case(Scenario::IndependentFloat, Operation::Cycle)];
        apply_history_filter("timing", Some(10_000), &mut timing);
        assert_eq!(timing[0].history, 10_000);
    }

    #[test]
    fn dimension_filter_selects_scaling_case_and_overrides_timing_case() {
        let mut scaling = cases_for("scaling").expect("cases");
        scaling.retain(|case| case.scenario == Scenario::IndependentFloat);
        apply_dimension_filter("scaling", Some(8), &mut scaling);
        assert_eq!(scaling.len(), 5);
        assert!(scaling.iter().all(|case| case.dimensions == 8));

        let mut timing = vec![base_case(Scenario::IndependentFloat, Operation::Cycle)];
        apply_dimension_filter("timing", Some(16), &mut timing);
        assert_eq!(timing[0].dimensions, 16);
    }
}
