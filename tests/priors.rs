// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use parzen::{
    Direction, Distribution, FloatDistribution, IntDistribution, SearchSpace, Study, TpeSampler,
    TpeSamplerConfig,
};

#[test]
fn linear_integer_startup_prior_is_uniform_over_grid() {
    let mut space = SearchSpace::new();
    space
        .add(
            "x",
            Distribution::Int(IntDistribution::linear(0, 2).unwrap()),
        )
        .unwrap();
    let sampler =
        TpeSampler::new(TpeSamplerConfig::performance(19).startup_trials(usize::MAX)).unwrap();
    let mut study = Study::new(Direction::Minimize, sampler, space).unwrap();
    let mut counts = [0_u32; 3];
    for _ in 0..30_000 {
        counts[study.suggest_int("x").unwrap() as usize] += 1;
        study.abort_trial();
    }
    for count in counts {
        assert!((9_700..=10_300).contains(&count), "counts: {counts:?}");
    }
}

#[test]
fn stepped_float_startup_prior_is_uniform_over_grid() {
    let mut space = SearchSpace::new();
    space
        .add(
            "x",
            Distribution::Float(
                FloatDistribution::linear(0.0, 1.0)
                    .unwrap()
                    .with_step(0.5)
                    .unwrap(),
            ),
        )
        .unwrap();
    let sampler =
        TpeSampler::new(TpeSamplerConfig::performance(23).startup_trials(usize::MAX)).unwrap();
    let mut study = Study::new(Direction::Minimize, sampler, space).unwrap();
    let mut counts = [0_u32; 3];
    for _ in 0..30_000 {
        counts[(study.suggest_float("x").unwrap() * 2.0) as usize] += 1;
        study.abort_trial();
    }
    for count in counts {
        assert!((9_700..=10_300).contains(&count), "counts: {counts:?}");
    }
}

#[test]
fn log_integer_startup_prior_matches_log_cell_mass() {
    let mut space = SearchSpace::new();
    space
        .add("x", Distribution::Int(IntDistribution::log(1, 4).unwrap()))
        .unwrap();
    let sampler =
        TpeSampler::new(TpeSamplerConfig::performance(31).startup_trials(100_000)).unwrap();
    let mut study = Study::new(Direction::Minimize, sampler, space).unwrap();
    let mut counts = [0_usize; 4];
    const SAMPLES: usize = 100_000;
    for _ in 0..SAMPLES {
        let value = study.suggest_int("x").unwrap();
        counts[(value - 1) as usize] += 1;
        study.complete_trial(0.0).unwrap();
    }
    let total_log_width = (4.5_f64 / 0.5).ln();
    for (index, count) in counts.into_iter().enumerate() {
        let value = index as f64 + 1.0;
        let expected = ((value + 0.5) / (value - 0.5)).ln() / total_log_width;
        let observed = count as f64 / SAMPLES as f64;
        assert!(
            (observed - expected).abs() < 0.006,
            "value {}: observed {observed}, expected {expected}",
            index + 1
        );
    }
}
