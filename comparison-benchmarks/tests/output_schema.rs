use parzen_comparison_benchmarks::{
    backends::parzen_backend::ParzenBackend,
    backends::tpe_backend::TpeBackend,
    cli::{BackendCli, BenchmarkProtocol, OutputFormat, ProfileWorkload, RunConfig},
    measurement::execute,
    output::{BenchmarkRecord, SCHEMA_VERSION},
    report::{read_jsonl, write_markdown},
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
    assert_eq!(decoded.benchmark_protocol, BenchmarkProtocol::Quick);
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

#[test]
fn fixed_suggest_profile_keeps_history_constant() {
    let cli = BackendCli {
        config: RunConfig {
            scenario: Scenario::LinearFloat,
            operation: Operation::Profile,
            profile_workload: ProfileWorkload::FixedSuggest,
            profile_seconds: 1,
            history: 10,
            dimensions: 1,
            ..RunConfig::default()
        },
        format: OutputFormat::Json,
    };
    let record = execute::<ParzenBackend>(&cli).expect("fixed profile");
    assert_eq!(record.profile_workload, Some(ProfileWorkload::FixedSuggest));
    assert_eq!(record.profile_start_observations, Some(10));
    assert_eq!(record.profile_end_observations, Some(10));
    assert!(
        record
            .profile_operations
            .is_some_and(|operations| operations > 0)
    );
    assert_ne!(record.result_checksum, 0);
}

#[test]
fn cycle_profile_records_exact_history_growth() {
    let cli = BackendCli {
        config: RunConfig {
            scenario: Scenario::LinearFloat,
            operation: Operation::Profile,
            profile_workload: ProfileWorkload::Cycle,
            profile_seconds: 1,
            history: 10,
            dimensions: 1,
            ..RunConfig::default()
        },
        format: OutputFormat::Json,
    };
    let record = execute::<ParzenBackend>(&cli).expect("cycle profile");
    let operations = record.profile_operations.expect("operations");
    assert!(operations > 0);
    assert_eq!(record.profile_start_observations, Some(10));
    assert_eq!(record.profile_end_observations, Some(10 + operations));
}

#[test]
fn profile_report_identifies_the_workload() {
    let cli = BackendCli {
        config: RunConfig {
            scenario: Scenario::LinearFloat,
            operation: Operation::Profile,
            profile_workload: ProfileWorkload::FixedSuggest,
            profile_seconds: 1,
            history: 10,
            dimensions: 1,
            ..RunConfig::default()
        },
        format: OutputFormat::Json,
    };
    let mut competitor = execute::<ParzenBackend>(&cli).expect("parzen profile");
    competitor.backend = "comparison-probe".to_owned();
    let records = [execute::<ParzenBackend>(&cli).expect("profile"), competitor];
    let mut markdown = Vec::new();
    write_markdown(&records, &mut markdown).expect("report");
    let markdown = String::from_utf8(markdown).expect("utf8");
    assert!(markdown.contains("Profile workload: `fixed-suggest`"));
    assert!(markdown.contains("Start observations"));
}

#[test]
fn report_reader_rejects_old_schema_versions() {
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
    let mut record = execute::<ParzenBackend>(&cli).expect("record");
    record.schema_version = SCHEMA_VERSION - 1;
    let path = std::env::temp_dir().join(format!(
        "parzen-schema-test-{}-{}.jsonl",
        std::process::id(),
        record.fixture_checksum
    ));
    std::fs::write(
        &path,
        format!("{}\n", serde_json::to_string(&record).expect("json")),
    )
    .expect("write fixture");
    let error = read_jsonl(&path).expect_err("old schema must fail");
    std::fs::remove_file(path).expect("remove fixture");
    assert!(
        error
            .to_string()
            .contains("unsupported JSONL schema version")
    );
}
