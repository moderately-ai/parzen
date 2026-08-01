use parzen_comparison_benchmarks::{
    backends::parzen_backend::ParzenBackend,
    backends::tpe_backend::TpeBackend,
    cli::{BackendCli, OutputFormat, RunConfig},
    measurement::execute,
    output::{BenchmarkRecord, SCHEMA_VERSION},
    report::write_markdown,
    scenarios::{Operation, Scenario},
};

#[test]
fn record_round_trips_through_versioned_json() {
    let cli = BackendCli {
        config: RunConfig {
            scenario: Scenario::LinearFloat,
            operation: Operation::Cycle,
            history: 10,
            dimensions: 1,
            iterations: 1,
            samples: 1,
            warmup: 0,
            ..RunConfig::default()
        },
        format: OutputFormat::Json,
    };
    let record = execute::<ParzenBackend>(&cli).expect("record");
    let json = serde_json::to_string(&record).expect("serialize");
    let decoded: BenchmarkRecord = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded.schema_version, SCHEMA_VERSION);
    assert_eq!(decoded.fixture_checksum, record.fixture_checksum);
    assert_eq!(decoded.observations, 11);
}

#[test]
fn unsupported_cases_are_structured_records() {
    let cli = BackendCli {
        config: RunConfig {
            scenario: Scenario::Integer,
            dimensions: 1,
            ..RunConfig::default()
        },
        format: OutputFormat::Json,
    };
    let record = execute::<TpeBackend>(&cli).expect("unsupported record");
    assert!(!record.supported);
    assert!(record.unsupported_reason.is_some());
}

#[test]
fn timeouts_are_distinct_from_unsupported_semantics() {
    let config = RunConfig {
        scenario: Scenario::LinearFloat,
        operation: Operation::ColdSuggest,
        history: 1_000,
        dimensions: 1,
        ..RunConfig::default()
    };
    let environment = parzen_comparison_benchmarks::output::Environment::capture_preflight("test");
    let record = BenchmarkRecord::timed_out("hyperopt", "0.0.17", config, environment, 10);
    assert!(record.supported);
    assert!(record.unsupported_reason.is_none());
    assert!(record.execution_error.is_some());
}

#[test]
fn cycle_samples_start_from_fresh_history() {
    let cli = BackendCli {
        config: RunConfig {
            scenario: Scenario::LinearFloat,
            operation: Operation::Cycle,
            history: 10,
            dimensions: 1,
            iterations: 3,
            samples: 2,
            warmup: 0,
            ..RunConfig::default()
        },
        format: OutputFormat::Json,
    };
    let record = execute::<ParzenBackend>(&cli).expect("record");
    assert_eq!(record.observations, 13);
}

#[test]
fn ingest_batch_counts_every_inserted_observation() {
    let cli = BackendCli {
        config: RunConfig {
            scenario: Scenario::LinearFloat,
            operation: Operation::Ingest,
            history: 10,
            dimensions: 1,
            iterations: 3,
            samples: 1,
            warmup: 0,
            ..RunConfig::default()
        },
        format: OutputFormat::Json,
    };
    let record = execute::<ParzenBackend>(&cli).expect("record");
    assert_eq!(record.observations, 10);
    assert_eq!(record.timing.expect("timing").operations_per_sample, 30);
}

#[test]
fn cold_suggest_is_not_automatically_batched() {
    let cli = BackendCli {
        config: RunConfig {
            scenario: Scenario::LinearFloat,
            operation: Operation::ColdSuggest,
            history: 10,
            dimensions: 1,
            iterations: 0,
            samples: 1,
            warmup: 0,
            ..RunConfig::default()
        },
        format: OutputFormat::Json,
    };
    let record = execute::<ParzenBackend>(&cli).expect("record");
    assert_eq!(record.timing.expect("timing").operations_per_sample, 1);
}

#[cfg(not(feature = "dhat-heap"))]
#[test]
fn memory_requires_explicit_dhat_feature() {
    let cli = BackendCli {
        config: RunConfig {
            scenario: Scenario::LinearFloat,
            operation: Operation::Memory,
            history: 10,
            dimensions: 1,
            iterations: 1,
            ..RunConfig::default()
        },
        format: OutputFormat::Json,
    };
    assert!(execute::<ParzenBackend>(&cli).is_err());
}

#[test]
fn markdown_generation_is_deterministic_for_fixed_records() {
    let cli = BackendCli {
        config: RunConfig {
            scenario: Scenario::LinearFloat,
            operation: Operation::Cycle,
            history: 10,
            dimensions: 1,
            iterations: 1,
            samples: 1,
            warmup: 0,
            ..RunConfig::default()
        },
        format: OutputFormat::Json,
    };
    let record = execute::<ParzenBackend>(&cli).expect("record");
    let records = [record];
    let mut first = Vec::new();
    let mut second = Vec::new();
    write_markdown(&records, &mut first).expect("first report");
    write_markdown(&records, &mut second).expect("second report");
    assert_eq!(first, second);
}
