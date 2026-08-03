use parzen_comparison_benchmarks::{
    fixtures::{Fixture, Value},
    scenarios::Scenario,
};

#[test]
fn fixture_checksum_is_stable() {
    let fixture = Fixture::generate(Scenario::MixedIndependent, 3, 16, 42).expect("fixture");
    assert_eq!(fixture.checksum, 16_368_768_401_313_490_165);
}

#[test]
fn conditional_fixture_marks_only_inactive_children() {
    let fixture = Fixture::generate(Scenario::Conditional, 2, 256, 91).expect("fixture");
    for trial in fixture.trials {
        let parent = trial.params[0]
            .as_categorical()
            .expect("categorical parent");
        assert_eq!(trial.params[1] == Value::Inactive, parent == 0);
    }
}

#[test]
fn integer_cardinality_controls_the_exact_fixture_domain() {
    for cardinality in [8, 64, 4_096, 100_001] {
        let fixture = Fixture::generate_with_integer_cardinality(
            Scenario::Integer,
            1,
            cardinality.min(10_000),
            42,
            cardinality,
        )
        .expect("fixture");
        let high = -100 + cardinality as i64 - 1;
        assert!(fixture.trials.iter().all(|trial| {
            trial.params[0]
                .as_int()
                .is_some_and(|value| (-100..=high).contains(&value))
        }));
    }
}
