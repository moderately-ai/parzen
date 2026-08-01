use std::{
    ffi::OsString,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use parzen_comparison_benchmarks::{
    HarnessResult,
    output::BenchmarkRecord,
    report::{read_jsonl, write_markdown},
    scenarios::{Operation, ParzenHistory, QUALITY_SEEDS, Scenario},
};

#[derive(Clone, Copy)]
struct BackendSpec {
    label: &'static str,
    binary: &'static str,
    history: ParzenHistory,
}

const BACKENDS: [BackendSpec; 5] = [
    BackendSpec {
        label: "parzen/full",
        binary: "bench-parzen",
        history: ParzenHistory::Full,
    },
    BackendSpec {
        label: "parzen/bounded",
        binary: "bench-parzen",
        history: ParzenHistory::Bounded,
    },
    BackendSpec {
        label: "tpe",
        binary: "bench-tpe",
        history: ParzenHistory::Full,
    },
    BackendSpec {
        label: "hyperopt",
        binary: "bench-hyperopt",
        history: ParzenHistory::Full,
    },
    BackendSpec {
        label: "optimizer",
        binary: "bench-optimizer",
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
}

struct DriverCli {
    command: String,
    report_input: Option<PathBuf>,
    backend: String,
    output: Option<PathBuf>,
    machine_label: String,
    rounds: usize,
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
    let mut writer = BufWriter::new(File::create(&output)?);
    let cases = cases_for(&cli.command)?;
    let timing_rounds = if cli.command == "smoke" {
        1
    } else {
        cli.rounds
    };

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
                let mut record = invoke_backend(binary_dir, backend, case, &cli.machine_label)?;
                record.comparison_round = Some(if case.operation == Operation::Quality {
                    case_index
                } else {
                    round
                });
                record.invocation_order = Some(order);
                serde_json::to_writer(&mut writer, &record)?;
                writeln!(writer)?;
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
        "smoke" | "timing" | "scaling" | "quality" | "memory" | "full" | "report"
    ) {
        return Err(format!("unknown command `{command}`\n{}", usage()).into());
    }
    let mut report_input = None;
    if command == "report" {
        report_input = args.next().map(PathBuf::from);
    }
    let mut cli = DriverCli {
        command,
        report_input,
        backend: "all".into(),
        output: None,
        machine_label: "unlabelled".into(),
        rounds: 8,
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
            "--rounds" => {
                cli.rounds = value
                    .into_string()
                    .map_err(|_| "rounds must be UTF-8")?
                    .parse()?
            }
            "--memory-bin-dir" => cli.memory_binary_dir = Some(PathBuf::from(value)),
            _ => return Err(format!("unknown argument `{flag}`\n{}", usage()).into()),
        }
    }
    if cli.rounds == 0 {
        return Err("rounds must be positive".into());
    }
    Ok(cli)
}

fn usage() -> &'static str {
    "compare <smoke|timing|scaling|quality|memory|full|report JSONL> \
     [--backend all|NAME] [--output PATH] [--machine-label LABEL] [--rounds N] \
     [--memory-bin-dir PATH]"
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
) -> HarnessResult<BenchmarkRecord> {
    let binary = dir.join(backend.binary);
    if !binary.is_file() {
        return Err(format!(
            "missing {}; build all binaries before measurement",
            binary.display()
        )
        .into());
    }
    let output = Command::new(&binary)
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
            "--parzen-history",
            match backend.history {
                ParzenHistory::Full => "full",
                ParzenHistory::Bounded => "bounded",
            },
        ])
        .args(["--machine-label", machine_label, "--format", "json"])
        .output()?;
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

fn base_case(scenario: Scenario, operation: Operation) -> Case {
    Case {
        scenario,
        operation,
        history: 1_000,
        dimensions: scenario.default_dimensions(),
        budget: 100,
        seed: 42,
        iterations: 0,
        samples: 10,
        warmup: 3,
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
