use parzen_comparison_benchmarks::{
    adapter::Backend,
    backends::{
        hyperopt_backend::HyperoptBackend, optimizer_backend::OptimizerBackend,
        parzen_backend::ParzenBackend, tpe_backend::TpeBackend,
    },
    cli::RunConfig,
    fixtures::Fixture,
    scenarios::Scenario,
};

fn validate_history<B: Backend>(scenario: Scenario, dimensions: usize) {
    let config = RunConfig {
        scenario,
        dimensions,
        history: 32,
        ..RunConfig::default()
    };
    let fixture = Fixture::generate(scenario, dimensions, 32, 42).expect("fixture");
    let mut backend = B::create(&config).expect("backend");
    for trial in &fixture.trials {
        backend.ingest(trial).expect("ingest");
    }
    assert_eq!(backend.observations(), fixture.trials.len());
    let suggestion = backend.suggest().expect("suggestion");
    assert_eq!(suggestion.len(), fixture.trials[0].params.len());
    backend.abort().expect("abort");
}

#[test]
fn continuous_fixture_is_accepted_by_every_backend() {
    validate_history::<ParzenBackend>(Scenario::IndependentFloat, 4);
    validate_history::<TpeBackend>(Scenario::IndependentFloat, 4);
    validate_history::<HyperoptBackend>(Scenario::IndependentFloat, 4);
    validate_history::<OptimizerBackend>(Scenario::IndependentFloat, 4);
}

#[test]
fn conditional_fixture_is_accepted_where_semantics_exist() {
    validate_history::<ParzenBackend>(Scenario::Conditional, 2);
    validate_history::<TpeBackend>(Scenario::Conditional, 2);
    validate_history::<OptimizerBackend>(Scenario::Conditional, 2);
    assert!(!HyperoptBackend::support(Scenario::Conditional).supported);
}

#[test]
fn correlated_numeric_uses_supported_joint_adapters() {
    validate_history::<ParzenBackend>(Scenario::CorrelatedNumeric, 4);
    validate_history::<OptimizerBackend>(Scenario::CorrelatedNumeric, 4);
    assert!(!TpeBackend::support(Scenario::CorrelatedNumeric).supported);
    assert!(!HyperoptBackend::support(Scenario::CorrelatedNumeric).supported);
}

fn validate_all_supported_domains<B: Backend>() {
    for scenario in Scenario::COMPARATIVE {
        if !B::support(scenario).supported {
            continue;
        }
        let dimensions = scenario.default_dimensions();
        let config = RunConfig {
            scenario,
            dimensions,
            history: 12,
            ..RunConfig::default()
        };
        let fixture = Fixture::generate(scenario, dimensions, 12, 7).expect("fixture");
        let mut backend = B::create(&config).expect("backend");
        for trial in &fixture.trials {
            backend.ingest(trial).expect("ingest");
        }
        let suggestion = backend.suggest().expect("suggestion");
        assert_eq!(suggestion.len(), fixture.trials[0].params.len());
        backend.abort().expect("abort");
    }
}

#[test]
fn every_declared_supported_domain_executes() {
    validate_all_supported_domains::<ParzenBackend>();
    validate_all_supported_domains::<TpeBackend>();
    validate_all_supported_domains::<HyperoptBackend>();
    validate_all_supported_domains::<OptimizerBackend>();
}
