// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use parzen::{
    CategoricalDistribution, Condition, Direction, Distribution, FloatDistribution,
    IntDistribution, ModelStrategy, ParamValue, ParzenError, SearchSpace, Study, TpeSampler,
    TpeSamplerConfig, TrialInput,
};

fn categorical_space(choices: u32) -> SearchSpace {
    let mut space = SearchSpace::new();
    space
        .add(
            "x",
            Distribution::Categorical(CategoricalDistribution::new(choices).unwrap()),
        )
        .unwrap();
    space
}

#[test]
fn categorical_round_trip_and_constant_time_best_view() {
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(42).startup_trials(2)).unwrap();
    let mut study = Study::new(Direction::Maximize, sampler, categorical_space(5)).unwrap();
    for _ in 0..20 {
        let x = study.suggest_categorical("x").unwrap();
        study
            .complete_trial(if x == 2 { 1.0 } else { 0.0 })
            .unwrap();
    }
    let best = study.best_trial().unwrap();
    assert_eq!(best.value(), 1.0);
    assert_eq!(best.get("x"), Some(ParamValue::Categorical(2)));
    assert_eq!(study.trials().len(), 20);
    assert_eq!(
        study.trial(best.id()).unwrap().to_record(),
        best.to_record()
    );
}

#[test]
fn float_integer_and_log_suggestions_obey_distributions() {
    let mut space = SearchSpace::new();
    space
        .add(
            "float",
            Distribution::Float(
                FloatDistribution::linear(-1.0, 1.0)
                    .unwrap()
                    .with_step(0.25)
                    .unwrap(),
            ),
        )
        .unwrap();
    space
        .add(
            "log_float",
            Distribution::Float(FloatDistribution::log(1e-6, 1.0).unwrap()),
        )
        .unwrap();
    space
        .add(
            "int",
            Distribution::Int(
                IntDistribution::linear(-10, 10)
                    .unwrap()
                    .with_step(4)
                    .unwrap(),
            ),
        )
        .unwrap();
    space
        .add(
            "log_int",
            Distribution::Int(IntDistribution::log(1, 1_000).unwrap()),
        )
        .unwrap();
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(7).startup_trials(3)).unwrap();
    let mut study = Study::new(Direction::Minimize, sampler, space).unwrap();
    for _ in 0..50 {
        let float = study.suggest_float("float").unwrap();
        let log_float = study.suggest_float("log_float").unwrap();
        let int = study.suggest_int("int").unwrap();
        let log_int = study.suggest_int("log_int").unwrap();
        assert!((-1.0..=1.0).contains(&float));
        assert!((((float + 1.0) / 0.25).round() - ((float + 1.0) / 0.25)).abs() < 1e-10);
        assert!((1e-6..=1.0).contains(&log_float));
        assert!((-10..=10).contains(&int));
        assert_eq!((int + 10) % 4, 0);
        assert!((1..=1_000).contains(&log_int));
        study
            .complete_trial(float.abs() + (log_float - 0.01).abs() + (int - 2).abs() as f64)
            .unwrap();
    }
}

#[test]
fn conditional_parameters_require_parent_and_validate_completion() {
    let mut space = SearchSpace::new();
    let parent = space
        .add(
            "kind",
            Distribution::Categorical(CategoricalDistribution::new(2).unwrap()),
        )
        .unwrap();
    let child = space
        .add(
            "depth",
            Distribution::Int(IntDistribution::linear(1, 5).unwrap()),
        )
        .unwrap();
    space
        .add_condition(
            child,
            Condition::CategoricalIn {
                parent,
                choices: vec![1].into_boxed_slice(),
            },
        )
        .unwrap();
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(3)).unwrap();
    let mut study = Study::new(Direction::Maximize, sampler, space).unwrap();
    assert!(matches!(
        study.suggest_int("depth"),
        Err(ParzenError::UnresolvedCondition(_))
    ));
    let kind = study.suggest_categorical("kind").unwrap();
    if kind == 1 {
        study.suggest_int("depth").unwrap();
    } else {
        assert!(matches!(
            study.suggest_int("depth"),
            Err(ParzenError::InactiveParameter(_))
        ));
    }
    study.complete_trial(1.0).unwrap();
}

#[test]
fn grouped_suggestions_are_cached_as_one_vector() {
    let mut space = SearchSpace::new();
    let x = space
        .add(
            "x",
            Distribution::Categorical(CategoricalDistribution::new(10).unwrap()),
        )
        .unwrap();
    let y = space
        .add(
            "y",
            Distribution::Categorical(CategoricalDistribution::new(10).unwrap()),
        )
        .unwrap();
    space.add_group([x, y]).unwrap();
    let sampler = TpeSampler::new(
        TpeSamplerConfig::performance(11).model(ModelStrategy::Grouped { max_group_size: 8 }),
    )
    .unwrap();
    let mut study = Study::new(Direction::Maximize, sampler, space).unwrap();
    let first = study.suggest_categorical("x").unwrap();
    let y = study.suggest_categorical("y").unwrap();
    assert_eq!(study.suggest_categorical("x").unwrap(), first);
    study
        .complete_trial(f64::from(first == 3 && y == 7))
        .unwrap();
}

#[test]
fn mixed_group_is_cached_and_returns_canonical_values() {
    let mut space = SearchSpace::new();
    let category = space
        .add(
            "category",
            Distribution::Categorical(CategoricalDistribution::new(3).unwrap()),
        )
        .unwrap();
    let value = space
        .add(
            "value",
            Distribution::Float(
                FloatDistribution::linear(0.0, 1.0)
                    .unwrap()
                    .with_step(0.2)
                    .unwrap(),
            ),
        )
        .unwrap();
    space.add_group([category, value]).unwrap();
    let sampler = TpeSampler::new(
        TpeSamplerConfig::performance(12).model(ModelStrategy::Grouped { max_group_size: 8 }),
    )
    .unwrap();
    let mut study = Study::new(Direction::Minimize, sampler, space).unwrap();
    let first_category = study.suggest_categorical("category").unwrap();
    let first_value = study.suggest_float("value").unwrap();
    assert_eq!(
        study.suggest_categorical("category").unwrap(),
        first_category
    );
    assert_eq!(study.suggest_float("value").unwrap(), first_value);
    let index = (first_value / 0.2).round();
    assert!((first_value - 0.2_f64.mul_add(index, 0.0)).abs() <= f64::EPSILON);
    study.complete_trial(first_value).unwrap();
}

#[test]
fn injected_trials_are_validated() {
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(1)).unwrap();
    let mut study = Study::new(Direction::Maximize, sampler, categorical_space(3)).unwrap();
    study
        .add_trial(TrialInput {
            params: vec![("x".into(), ParamValue::Categorical(2))],
            value: 0.5,
        })
        .unwrap();
    assert!(matches!(
        study.add_trial(TrialInput {
            params: vec![("x".into(), ParamValue::Categorical(3))],
            value: 0.6
        }),
        Err(ParzenError::ValueOutsideDistribution(_))
    ));
    assert_eq!(study.best_value(), Some(0.5));
}

#[test]
#[cfg(feature = "serde")]
fn serde_round_trip_owned_trial() {
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(1)).unwrap();
    let mut study = Study::new(Direction::Maximize, sampler, categorical_space(2)).unwrap();
    study
        .add_trial(TrialInput {
            params: vec![("x".into(), ParamValue::Categorical(1))],
            value: 0.5,
        })
        .unwrap();
    let record = study.best_trial().unwrap().to_record();
    let json = serde_json::to_string(&record).unwrap();
    assert_eq!(
        serde_json::from_str::<parzen::TrialRecord>(&json).unwrap(),
        record
    );
}

#[test]
fn packed_history_has_a_compact_measured_capacity() {
    let mut space = SearchSpace::new();
    for name in ["a", "b", "c", "d"] {
        space
            .add(
                name,
                Distribution::Categorical(CategoricalDistribution::new(10).unwrap()),
            )
            .unwrap();
    }
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(1)).unwrap();
    let mut study = Study::new(Direction::Maximize, sampler, space).unwrap();
    for trial in 0..100_000_u32 {
        study
            .add_trial(TrialInput {
                params: ["a", "b", "c", "d"]
                    .into_iter()
                    .map(|name| (name.into(), ParamValue::Categorical(trial % 10)))
                    .collect(),
                value: f64::from(trial),
            })
            .unwrap();
    }
    assert!(study.history_capacity_bytes() < 24 * 1024 * 1024);
    assert!(study.estimator_history_len() <= 537 * 4);
}

#[test]
fn optimized_build_is_deterministic_for_the_same_seed() {
    fn study(seed: u64) -> Study {
        let mut space = SearchSpace::new();
        for name in ["a", "b", "c", "d"] {
            space
                .add(
                    name,
                    Distribution::Float(FloatDistribution::linear(-5.0, 5.0).unwrap()),
                )
                .unwrap();
        }
        let sampler = TpeSampler::new(
            TpeSamplerConfig::performance(seed)
                .startup_trials(10)
                .model(ModelStrategy::Independent),
        )
        .unwrap();
        Study::new(Direction::Minimize, sampler, space).unwrap()
    }

    let mut first = study(0x5eed);
    let mut second = study(0x5eed);
    for iteration in 0..80 {
        let mut first_values = Vec::new();
        let mut second_values = Vec::new();
        for name in ["a", "b", "c", "d"] {
            first_values.push(first.suggest_float(name).unwrap());
            second_values.push(second.suggest_float(name).unwrap());
        }
        assert_eq!(first_values, second_values, "iteration {iteration}");
        let objective = first_values.iter().map(|value| value * value).sum();
        first.complete_trial(objective).unwrap();
        second.complete_trial(objective).unwrap();
    }
}
