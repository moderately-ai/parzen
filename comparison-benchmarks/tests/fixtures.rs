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
