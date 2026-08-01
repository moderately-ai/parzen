// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use parzen::{
    CategoricalDistribution, Direction, Distribution, FloatDistribution, IntDistribution,
    SearchSpace, Study, TpeSampler, TpeSamplerConfig,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn stepped_integer_suggestions_are_aligned(low in -1_000_i64..1_000, width in 1_i64..1_000, step in 1_u64..100, seed in any::<u64>()) {
        let high = low.saturating_add(width);
        let mut space = SearchSpace::new();
        space.add("x", Distribution::Int(IntDistribution::linear(low, high).unwrap().with_step(step).unwrap())).unwrap();
        let sampler = TpeSampler::new(TpeSamplerConfig::performance(seed).startup_trials(0)).unwrap();
        let mut study = Study::new(Direction::Minimize, sampler, space).unwrap();
        for _ in 0..10 {
            let value = study.suggest_int("x").unwrap();
            prop_assert!((low..=high).contains(&value));
            prop_assert_eq!((i128::from(value) - i128::from(low)) % i128::from(step), 0);
            study.complete_trial(value.unsigned_abs() as f64).unwrap();
        }
    }
}

proptest! {
    #[test]
    fn every_distribution_family_suggests_self_canonical_values(seed in any::<u64>()) {
        let mut space = SearchSpace::new();
        space.add("category", Distribution::Categorical(CategoricalDistribution::new(7).unwrap())).unwrap();
        space.add("float", Distribution::Float(FloatDistribution::linear(-3.0, 4.0).unwrap())).unwrap();
        space.add("log_float", Distribution::Float(FloatDistribution::log(0.01, 100.0).unwrap())).unwrap();
        space.add("step_float", Distribution::Float(FloatDistribution::linear(-1.0, 1.0).unwrap().with_step(0.2).unwrap())).unwrap();
        space.add("integer", Distribution::Int(IntDistribution::linear(i64::MIN, i64::MAX).unwrap().with_step(3).unwrap())).unwrap();
        space.add("log_integer", Distribution::Int(IntDistribution::log(1, 100_000).unwrap())).unwrap();
        let sampler = TpeSampler::new(TpeSamplerConfig::performance(seed).startup_trials(0)).unwrap();
        let mut study = Study::new(Direction::Minimize, sampler, space).unwrap();
        for _ in 0..12 {
            let category = study.suggest_categorical("category").unwrap();
            let float = study.suggest_float("float").unwrap();
            let log_float = study.suggest_float("log_float").unwrap();
            let step_float = study.suggest_float("step_float").unwrap();
            let integer = study.suggest_int("integer").unwrap();
            let log_integer = study.suggest_int("log_integer").unwrap();
            prop_assert!(category < 7);
            prop_assert!(float.is_finite() && (-3.0..=4.0).contains(&float));
            prop_assert!(log_float.is_finite() && (0.01..=100.0).contains(&log_float));
            prop_assert!(step_float.is_finite() && (-1.0..=1.0).contains(&step_float));
            let step_index = ((step_float + 1.0) / 0.2).round();
            prop_assert!((step_float - 0.2_f64.mul_add(step_index, -1.0)).abs() < 1.0e-12);
            prop_assert!((i128::from(integer) - i128::from(i64::MIN)) % 3 == 0);
            prop_assert!((1..=100_000).contains(&log_integer));
            study.complete_trial(float.abs() + log_float.ln().abs()).unwrap();
        }
    }
}
