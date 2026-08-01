use parzen_comparison_benchmarks::{
    fixtures::Value,
    objectives::{evaluate, optimum},
    scenarios::Scenario,
};

#[test]
fn known_optima_evaluate_to_declared_values() {
    let cases = [
        (Scenario::LinearFloat, vec![Value::Float(1.5)]),
        (Scenario::Integer, vec![Value::Int(17)]),
        (Scenario::Categorical, vec![Value::Categorical(7)]),
        (Scenario::LogFloat, vec![Value::Float(1e-3)]),
        (
            Scenario::Conditional,
            vec![Value::Categorical(1), Value::Float(3.0)],
        ),
        (
            Scenario::CorrelatedNumeric,
            vec![Value::Float(1.0), Value::Float(1.0)],
        ),
    ];
    for (scenario, values) in cases {
        assert!(
            (evaluate(scenario, &values).expect("objective") - optimum(scenario)).abs() < 1e-12
        );
    }
}
