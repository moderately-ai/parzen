// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use parzen::{
    CategoricalDistribution, Direction, Distribution, FloatDistribution, IntDistribution,
    ParamValue, ParzenError, SearchSpace, Study, TpeSampler, TpeSamplerConfig, TrialInput,
};
use parzen::{GammaStrategy, HistoryPolicy};
use std::{num::NonZeroUsize, sync::Arc};

#[test]
fn malformed_inputs_are_errors_not_panics() {
    assert!(CategoricalDistribution::new(0).is_err());
    assert!(FloatDistribution::linear(f64::NAN, 1.0).is_err());
    assert!(FloatDistribution::log(0.0, 1.0).is_err());
    assert!(
        FloatDistribution::linear(0.0, 1.0)
            .unwrap()
            .with_step(0.0)
            .is_err()
    );
    assert!(IntDistribution::linear(2, 1).is_err());
    assert!(IntDistribution::log(0, 10).is_err());
    assert!(TpeSampler::new(TpeSamplerConfig::performance(0).prior_weight(f64::NAN)).is_err());
    assert!(TpeSampler::new(TpeSamplerConfig::performance(0).prior_weight(0.0)).is_err());
}

#[test]
fn non_finite_objectives_are_rejected() {
    let mut space = SearchSpace::new();
    space
        .add(
            "x",
            Distribution::Categorical(CategoricalDistribution::new(2).unwrap()),
        )
        .unwrap();
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(0).startup_trials(0)).unwrap();
    let mut study = Study::new(Direction::Maximize, sampler, space).unwrap();
    study.suggest_categorical("x").unwrap();
    assert_eq!(
        study.complete_trial(f64::NAN),
        Err(ParzenError::NonFiniteObjective)
    );
    assert!(study.abort_trial());
    assert_eq!(
        study.add_trial(TrialInput {
            params: vec![("x".into(), ParamValue::Categorical(0))],
            value: f64::INFINITY
        }),
        Err(ParzenError::NonFiniteObjective)
    );
}

#[test]
fn zero_startup_and_empty_history_use_prior() {
    let mut space = SearchSpace::new();
    space
        .add(
            "x",
            Distribution::Categorical(CategoricalDistribution::new(2).unwrap()),
        )
        .unwrap();
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(0).startup_trials(0)).unwrap();
    let mut study = Study::new(Direction::Maximize, sampler, space).unwrap();
    assert!(study.suggest_categorical("x").unwrap() < 2);
}

#[test]
fn repeated_suggestion_does_not_resample() {
    let mut space = SearchSpace::new();
    space
        .add(
            "x",
            Distribution::Categorical(CategoricalDistribution::new(100).unwrap()),
        )
        .unwrap();
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(42)).unwrap();
    let mut study = Study::new(Direction::Maximize, sampler, space).unwrap();
    let first = study.suggest_categorical("x").unwrap();
    for _ in 0..20 {
        assert_eq!(study.suggest_categorical("x").unwrap(), first);
    }
}

#[test]
fn unaligned_float_high_never_escapes_the_step_grid() {
    let mut space = SearchSpace::new();
    space
        .add(
            "x",
            Distribution::Float(
                FloatDistribution::linear(0.0, 1.0)
                    .unwrap()
                    .with_step(0.6)
                    .unwrap(),
            ),
        )
        .unwrap();
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(4).startup_trials(0)).unwrap();
    let mut study = Study::new(Direction::Maximize, sampler, space).unwrap();
    for _ in 0..100 {
        let value = study.suggest_float("x").unwrap();
        assert!(value == 0.0 || (value - 0.6).abs() < f64::EPSILON);
        study.complete_trial(value).unwrap();
    }
}

#[test]
fn injection_does_not_mix_with_a_pending_trial() {
    let mut space = SearchSpace::new();
    space
        .add(
            "x",
            Distribution::Categorical(CategoricalDistribution::new(2).unwrap()),
        )
        .unwrap();
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(0)).unwrap();
    let mut study = Study::new(Direction::Maximize, sampler, space).unwrap();
    study.suggest_categorical("x").unwrap();
    assert_eq!(
        study.add_trial(TrialInput {
            params: vec![("x".into(), ParamValue::Categorical(0))],
            value: 1.0,
        }),
        Err(ParzenError::PendingTrial)
    );
}

#[test]
fn finite_bounds_with_infinite_width_are_rejected() {
    assert!(FloatDistribution::linear(-f64::MAX, f64::MAX).is_err());
    assert!(
        FloatDistribution::linear(0.0, 1.0)
            .unwrap()
            .with_step(f64::MIN_POSITIVE)
            .is_err()
    );
}

#[cfg(feature = "serde")]
#[test]
fn deserialization_cannot_bypass_distribution_validation() {
    let invalid = [
        r#"{"Categorical":{"num_choices":0}}"#,
        r#"{"Int":{"low":0,"high":10,"scale":"Linear","step":0}}"#,
        r#"{"Int":{"low":0,"high":10,"scale":"Log","step":1}}"#,
        r#"{"Float":{"low":0.0,"high":1.0,"scale":"Linear","step":0.0}}"#,
        r#"{"Float":{"low":0.0,"high":1.0,"scale":"Linear","step":null,"extra":1}}"#,
    ];
    for json in invalid {
        assert!(
            serde_json::from_str::<Distribution>(json).is_err(),
            "{json}"
        );
    }
}

#[cfg(feature = "serde")]
#[test]
fn every_distribution_shape_round_trips_through_validation() {
    let distributions = [
        Distribution::Categorical(CategoricalDistribution::new(4).unwrap()),
        Distribution::Float(FloatDistribution::linear(-2.0, 3.0).unwrap()),
        Distribution::Float(FloatDistribution::log(0.01, 100.0).unwrap()),
        Distribution::Float(
            FloatDistribution::linear(0.0, 1.0)
                .unwrap()
                .with_step(0.3)
                .unwrap(),
        ),
        Distribution::Int(IntDistribution::linear(-7, 9).unwrap()),
        Distribution::Int(
            IntDistribution::linear(-7, 9)
                .unwrap()
                .with_step(4)
                .unwrap(),
        ),
        Distribution::Int(IntDistribution::log(1, 10_000).unwrap()),
    ];
    for distribution in distributions {
        let json = serde_json::to_string(&distribution).unwrap();
        assert_eq!(
            serde_json::from_str::<Distribution>(&json).unwrap(),
            distribution
        );
    }
}

#[test]
fn step_larger_than_range_is_a_valid_one_point_grid() {
    let distribution = FloatDistribution::linear(2.0, 3.0)
        .unwrap()
        .with_step(10.0)
        .unwrap();
    let mut space = SearchSpace::new();
    space.add("x", Distribution::Float(distribution)).unwrap();
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(7)).unwrap();
    let mut study = Study::new(Direction::Minimize, sampler, space).unwrap();
    for _ in 0..32 {
        assert_eq!(study.suggest_float("x").unwrap(), 2.0);
        study.complete_trial(0.0).unwrap();
    }
}

#[test]
fn injected_stepped_float_is_stored_canonically() {
    let mut space = SearchSpace::new();
    space
        .add(
            "x",
            Distribution::Float(
                FloatDistribution::linear(0.0, 1.0)
                    .unwrap()
                    .with_step(0.1)
                    .unwrap(),
            ),
        )
        .unwrap();
    let canonical = 0.1_f64.mul_add(3.0, 0.0);
    let nearby = f64::from_bits(canonical.to_bits() + 1);
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(1)).unwrap();
    let mut study = Study::new(Direction::Minimize, sampler, space).unwrap();
    let id = study
        .add_trial(TrialInput {
            params: vec![("x".into(), ParamValue::Float(nearby))],
            value: 0.0,
        })
        .unwrap();
    assert_eq!(
        study.trial(id).unwrap().get("x"),
        Some(ParamValue::Float(canonical))
    );

    let outside_tolerance = f64::from_bits(canonical.to_bits() + 5);
    assert!(matches!(
        study.add_trial(TrialInput {
            params: vec![("x".into(), ParamValue::Float(outside_tolerance))],
            value: 0.0,
        }),
        Err(ParzenError::ValueOutsideDistribution(_))
    ));
}

#[test]
fn bounded_custom_gamma_reports_capacity_error() {
    let mut space = SearchSpace::new();
    space
        .add(
            "x",
            Distribution::Categorical(CategoricalDistribution::new(3).unwrap()),
        )
        .unwrap();
    let history = HistoryPolicy::Bounded {
        max_good_trials: NonZeroUsize::new(3).unwrap(),
        max_bad_trials: NonZeroUsize::new(8).unwrap(),
        recent_bad_trials: 2,
    };
    let config = TpeSamplerConfig::performance(1)
        .startup_trials(0)
        .history(history)
        .gamma(GammaStrategy::Custom(Arc::new(|_| 4)));
    let mut study =
        Study::new(Direction::Maximize, TpeSampler::new(config).unwrap(), space).unwrap();
    for index in 0..5 {
        study
            .add_trial(TrialInput {
                params: vec![("x".into(), ParamValue::Categorical(index % 3))],
                value: f64::from(index),
            })
            .unwrap();
    }
    assert_eq!(
        study.suggest_categorical("x"),
        Err(ParzenError::GammaExceedsHistoryLimit {
            requested: 4,
            limit: 3,
        })
    );
}
