// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use parzen::{
    CategoricalDistribution, Direction, Distribution, FloatDistribution, IntDistribution,
    ModelStrategy, SearchSpace, Study, TpeSampler, TpeSamplerConfig,
};

fn categorical_study(seed: u64, choices: u32) -> Study {
    let mut space = SearchSpace::new();
    space
        .add(
            "x",
            Distribution::Categorical(CategoricalDistribution::new(choices).unwrap()),
        )
        .unwrap();
    Study::new(
        Direction::Maximize,
        TpeSampler::new(TpeSamplerConfig::performance(seed)).unwrap(),
        space,
    )
    .unwrap()
}

#[test]
fn grouped_model_learns_bimodal_numeric_correlation() {
    let mut grouped_regret = 0.0;
    for seed in 0..100 {
        let mut space = SearchSpace::new();
        let x = space
            .add(
                "x",
                Distribution::Float(FloatDistribution::linear(0.0, 1.0).unwrap()),
            )
            .unwrap();
        let y = space
            .add(
                "y",
                Distribution::Float(FloatDistribution::linear(0.0, 1.0).unwrap()),
            )
            .unwrap();
        space.add_group([x, y]).unwrap();
        let sampler = TpeSampler::new(
            TpeSamplerConfig::performance(seed).model(ModelStrategy::Grouped { max_group_size: 8 }),
        )
        .unwrap();
        let mut study = Study::new(Direction::Minimize, sampler, space).unwrap();
        for _ in 0..80 {
            let x = study.suggest_float("x").unwrap();
            let y = study.suggest_float("y").unwrap();
            let first = (x - 0.2).powi(2) + (y - 0.8).powi(2);
            let second = (x - 0.8).powi(2) + (y - 0.2).powi(2);
            study.complete_trial(first.min(second)).unwrap();
        }
        grouped_regret += study.best_value().unwrap();
    }
    assert!(
        grouped_regret / 100.0 < 0.005,
        "mean regret: {}",
        grouped_regret / 100.0
    );
}

#[test]
fn discrete_models_converge_on_integer_and_log_objectives() {
    let mut integer_regret = 0.0;
    let mut log_regret = 0.0;
    for seed in 0..100 {
        let mut space = SearchSpace::new();
        space
            .add(
                "integer",
                Distribution::Int(
                    IntDistribution::linear(-20, 20)
                        .unwrap()
                        .with_step(2)
                        .unwrap(),
                ),
            )
            .unwrap();
        space
            .add(
                "log_integer",
                Distribution::Int(IntDistribution::log(1, 10_000).unwrap()),
            )
            .unwrap();
        let mut study = Study::new(
            Direction::Minimize,
            TpeSampler::new(TpeSamplerConfig::performance(seed)).unwrap(),
            space,
        )
        .unwrap();
        for _ in 0..80 {
            let integer = study.suggest_int("integer").unwrap();
            let log_integer = study.suggest_int("log_integer").unwrap();
            let integer_loss = (integer - 6).unsigned_abs() as f64;
            let log_loss = ((log_integer as f64).ln() - 500_f64.ln()).abs();
            study.complete_trial(integer_loss + log_loss).unwrap();
            integer_regret += f64::from(integer == 6);
            log_regret += f64::from((450..=550).contains(&log_integer));
        }
    }
    assert!(integer_regret > 100.0);
    assert!(log_regret > 100.0);
}

#[test]
fn reliably_finds_one_of_twenty_categories() {
    let mut successes = 0;
    for seed in 0..500 {
        let mut study = categorical_study(seed, 20);
        for _ in 0..50 {
            let x = study.suggest_categorical("x").unwrap();
            study.complete_trial(f64::from(x == 7)).unwrap();
        }
        successes += usize::from(study.best_value() == Some(1.0));
    }
    assert!(successes >= 490, "successes: {successes}/500");
}

#[test]
fn grouped_model_finds_sparse_interaction() {
    let mut successes = 0;
    for seed in 0..500 {
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
        let config =
            TpeSamplerConfig::performance(seed).model(ModelStrategy::Grouped { max_group_size: 8 });
        let mut study =
            Study::new(Direction::Maximize, TpeSampler::new(config).unwrap(), space).unwrap();
        for _ in 0..50 {
            let x = study.suggest_categorical("x").unwrap();
            let y = study.suggest_categorical("y").unwrap();
            study.complete_trial(f64::from(x == 3 && y == 7)).unwrap();
        }
        successes += usize::from(study.best_value() == Some(1.0));
    }
    assert!(successes >= 230, "successes: {successes}/500");
}

#[test]
fn continuous_tpe_converges_on_quadratic() {
    let mut final_regret = 0.0;
    for seed in 0..100 {
        let mut space = SearchSpace::new();
        space
            .add(
                "x",
                Distribution::Float(FloatDistribution::linear(-10.0, 10.0).unwrap()),
            )
            .unwrap();
        let mut study = Study::new(
            Direction::Minimize,
            TpeSampler::new(TpeSamplerConfig::performance(seed)).unwrap(),
            space,
        )
        .unwrap();
        for _ in 0..60 {
            let x = study.suggest_float("x").unwrap();
            study.complete_trial((x - 2.5).powi(2)).unwrap();
        }
        final_regret += study.best_value().unwrap();
    }
    assert!(
        final_regret / 100.0 < 0.02,
        "mean regret: {}",
        final_regret / 100.0
    );
}
