// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use parzen::{
    CategoricalDistribution, Condition, Direction, Distribution, FloatDistribution, HistoryPolicy,
    IntDistribution, ModelStrategy, ParamValue, SearchSpace, Study, TpeSampler, TpeSamplerConfig,
    TrialInput,
};

fn benchmark_suggestion(c: &mut Criterion) {
    let mut group = c.benchmark_group("categorical_suggestion");
    for &(trials, choices) in &[(100, 20_u32), (1_000, 20), (10_000, 20), (10_000, 100)] {
        let mut space = SearchSpace::new();
        space
            .add(
                "x",
                Distribution::Categorical(CategoricalDistribution::new(choices).unwrap()),
            )
            .unwrap();
        let sampler = TpeSampler::new(TpeSamplerConfig::performance(42).startup_trials(0)).unwrap();
        let mut study = Study::new(Direction::Maximize, sampler, space).unwrap();
        for index in 0..trials {
            study
                .add_trial(TrialInput {
                    params: vec![("x".into(), ParamValue::Categorical(index % choices))],
                    value: f64::from(index % 997),
                })
                .unwrap();
        }
        group.bench_with_input(
            BenchmarkId::new(format!("{choices}_choices"), trials),
            &trials,
            |b, _| {
                b.iter(|| {
                    let _ = study.suggest_categorical("x").unwrap();
                    study.abort_trial();
                });
            },
        );
    }
    group.finish();

    let mut space = SearchSpace::new();
    space
        .add(
            "x",
            Distribution::Float(FloatDistribution::linear(-10.0, 10.0).unwrap()),
        )
        .unwrap();
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(42).startup_trials(0)).unwrap();
    let mut study = Study::new(Direction::Minimize, sampler, space).unwrap();
    for index in 0..10_000 {
        let x = f64::from(index % 10_000) / 500.0 - 10.0;
        study
            .add_trial(TrialInput {
                params: vec![("x".into(), ParamValue::Float(x))],
                value: (x - 2.5).powi(2),
            })
            .unwrap();
    }
    c.bench_function("numeric_suggestion/10000", |b| {
        b.iter(|| {
            let _ = study.suggest_float("x").unwrap();
            study.abort_trial();
        });
    });

    let mut grouped = c.benchmark_group("grouped_numeric_suggestion");
    for dimensions in [2_usize, 8] {
        let mut space = SearchSpace::new();
        let params: Vec<_> = (0..dimensions)
            .map(|index| {
                space
                    .add(
                        format!("x{index}"),
                        Distribution::Float(FloatDistribution::linear(-10.0, 10.0).unwrap()),
                    )
                    .unwrap()
            })
            .collect();
        space.add_group(params).unwrap();
        let sampler = TpeSampler::new(
            TpeSamplerConfig::performance(42)
                .startup_trials(0)
                .model(ModelStrategy::Grouped { max_group_size: 8 }),
        )
        .unwrap();
        let mut study = Study::new(Direction::Minimize, sampler, space).unwrap();
        for trial in 0..10_000_u32 {
            study
                .add_trial(TrialInput {
                    params: (0..dimensions)
                        .map(|index| {
                            let value = f64::from((trial + index as u32) % 10_000) / 500.0 - 10.0;
                            (format!("x{index}"), ParamValue::Float(value))
                        })
                        .collect(),
                    value: f64::from(trial % 997),
                })
                .unwrap();
        }
        grouped.bench_with_input(
            BenchmarkId::from_parameter(dimensions),
            &dimensions,
            |b, _| {
                b.iter(|| {
                    let _ = study.suggest_float("x0").unwrap();
                    study.abort_trial();
                });
            },
        );
    }
    grouped.finish();

    let mut sequential = c.benchmark_group("bounded_suggest_complete");
    for trials in [10_000_u32, 100_000, 1_000_000] {
        let mut space = SearchSpace::new();
        space
            .add(
                "x",
                Distribution::Categorical(CategoricalDistribution::new(20).unwrap()),
            )
            .unwrap();
        let sampler = TpeSampler::new(TpeSamplerConfig::performance(7).startup_trials(0)).unwrap();
        let mut study = Study::new(Direction::Maximize, sampler, space).unwrap();
        for trial in 0..trials {
            study
                .add_trial(TrialInput {
                    params: vec![("x".into(), ParamValue::Categorical(trial % 20))],
                    value: f64::from(trial % 997),
                })
                .unwrap();
        }
        sequential.bench_with_input(BenchmarkId::from_parameter(trials), &trials, |b, _| {
            b.iter(|| {
                let x = study.suggest_categorical("x").unwrap();
                study.complete_trial(f64::from(x)).unwrap();
            });
        });
    }
    sequential.finish();
}

fn benchmark_cold_and_grouped(c: &mut Criterion) {
    c.bench_function("cold_first_suggestion", |b| {
        b.iter_batched(
            || {
                let mut space = SearchSpace::new();
                space
                    .add(
                        "x",
                        Distribution::Float(FloatDistribution::linear(-10.0, 10.0).unwrap()),
                    )
                    .unwrap();
                Study::new(
                    Direction::Minimize,
                    TpeSampler::new(TpeSamplerConfig::performance(91)).unwrap(),
                    space,
                )
                .unwrap()
            },
            |mut study| {
                let _ = study.suggest_float("x").unwrap();
            },
            BatchSize::SmallInput,
        );
    });

    let mut categorical = c.benchmark_group("grouped_categorical_suggestion");
    for dimensions in [2_usize, 8] {
        let mut space = SearchSpace::new();
        let params: Vec<_> = (0..dimensions)
            .map(|index| {
                space
                    .add(
                        format!("x{index}"),
                        Distribution::Categorical(CategoricalDistribution::new(20).unwrap()),
                    )
                    .unwrap()
            })
            .collect();
        space.add_group(params).unwrap();
        let sampler = TpeSampler::new(
            TpeSamplerConfig::performance(92)
                .startup_trials(0)
                .model(ModelStrategy::Grouped { max_group_size: 8 }),
        )
        .unwrap();
        let mut study = Study::new(Direction::Maximize, sampler, space).unwrap();
        for trial in 0..10_000_u32 {
            study
                .add_trial(TrialInput {
                    params: (0..dimensions)
                        .map(|index| {
                            (
                                format!("x{index}"),
                                ParamValue::Categorical((trial + index as u32) % 20),
                            )
                        })
                        .collect(),
                    value: f64::from(trial % 997),
                })
                .unwrap();
        }
        categorical.bench_with_input(
            BenchmarkId::from_parameter(dimensions),
            &dimensions,
            |b, _| {
                b.iter(|| {
                    let _ = study.suggest_categorical("x0").unwrap();
                    study.abort_trial();
                });
            },
        );
    }
    categorical.finish();
}

fn benchmark_discrete_mixed_and_conditional(c: &mut Criterion) {
    let mut space = SearchSpace::new();
    let category = space
        .add(
            "category",
            Distribution::Categorical(CategoricalDistribution::new(20).unwrap()),
        )
        .unwrap();
    let continuous = space
        .add(
            "continuous",
            Distribution::Float(FloatDistribution::linear(-10.0, 10.0).unwrap()),
        )
        .unwrap();
    let integer = space
        .add(
            "integer",
            Distribution::Int(
                IntDistribution::linear(-100, 100)
                    .unwrap()
                    .with_step(5)
                    .unwrap(),
            ),
        )
        .unwrap();
    let stepped = space
        .add(
            "stepped",
            Distribution::Float(
                FloatDistribution::linear(0.0, 1.0)
                    .unwrap()
                    .with_step(0.05)
                    .unwrap(),
            ),
        )
        .unwrap();
    space
        .add_group([category, continuous, integer, stepped])
        .unwrap();
    let sampler = TpeSampler::new(
        TpeSamplerConfig::performance(93)
            .startup_trials(0)
            .model(ModelStrategy::Grouped { max_group_size: 8 }),
    )
    .unwrap();
    let mut study = Study::new(Direction::Minimize, sampler, space).unwrap();
    for trial in 0..10_000_u32 {
        study
            .add_trial(TrialInput {
                params: vec![
                    ("category".into(), ParamValue::Categorical(trial % 20)),
                    (
                        "continuous".into(),
                        ParamValue::Float(f64::from(trial % 400) / 20.0 - 10.0),
                    ),
                    (
                        "integer".into(),
                        ParamValue::Int(-100 + i64::from(trial % 41) * 5),
                    ),
                    (
                        "stepped".into(),
                        ParamValue::Float(0.05 * f64::from(trial % 21)),
                    ),
                ],
                value: f64::from(trial % 997),
            })
            .unwrap();
    }
    c.bench_function("mixed_group_suggestion/4/10000", |b| {
        b.iter(|| {
            let _ = study.suggest_float("stepped").unwrap();
            study.abort_trial();
        });
    });

    let mut space = SearchSpace::new();
    let parent = space
        .add(
            "parent",
            Distribution::Categorical(CategoricalDistribution::new(1).unwrap()),
        )
        .unwrap();
    let child = space
        .add(
            "child",
            Distribution::Int(IntDistribution::linear(0, 100).unwrap()),
        )
        .unwrap();
    space
        .add_condition(
            child,
            Condition::CategoricalIn {
                parent,
                choices: vec![0].into_boxed_slice(),
            },
        )
        .unwrap();
    let sampler = TpeSampler::new(TpeSamplerConfig::performance(94).startup_trials(0)).unwrap();
    let mut conditional = Study::new(Direction::Minimize, sampler, space).unwrap();
    for trial in 0..10_000_u32 {
        conditional
            .add_trial(TrialInput {
                params: vec![
                    ("parent".into(), ParamValue::Categorical(0)),
                    ("child".into(), ParamValue::Int(i64::from(trial % 101))),
                ],
                value: f64::from(trial % 997),
            })
            .unwrap();
    }
    c.bench_function("conditional_suggestion/10000", |b| {
        b.iter(|| {
            let _ = conditional.suggest_categorical("parent").unwrap();
            let _ = conditional.suggest_int("child").unwrap();
            conditional.abort_trial();
        });
    });
}

fn benchmark_full_history(c: &mut Criterion) {
    let mut space = SearchSpace::new();
    space
        .add(
            "x",
            Distribution::Float(FloatDistribution::linear(-10.0, 10.0).unwrap()),
        )
        .unwrap();
    let sampler = TpeSampler::new(
        TpeSamplerConfig::performance(95)
            .startup_trials(0)
            .history(HistoryPolicy::Full),
    )
    .unwrap();
    let mut study = Study::new(Direction::Minimize, sampler, space).unwrap();
    for trial in 0..10_000_u32 {
        let value = f64::from(trial % 10_000) / 500.0 - 10.0;
        study
            .add_trial(TrialInput {
                params: vec![("x".into(), ParamValue::Float(value))],
                value: value.abs(),
            })
            .unwrap();
    }
    c.bench_function("full_history_suggest_complete/10000", |b| {
        b.iter(|| {
            let value = study.suggest_float("x").unwrap();
            study.complete_trial(value.abs()).unwrap();
        });
    });
}

criterion_group!(
    benches,
    benchmark_suggestion,
    benchmark_cold_and_grouped,
    benchmark_discrete_mixed_and_conditional,
    benchmark_full_history
);
criterion_main!(benches);
