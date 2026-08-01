use std::{num::NonZeroUsize, sync::Arc};

use parzen::{
    CategoricalDistribution, Condition, Direction, Distribution, FloatDistribution, GammaStrategy,
    HistoryPolicy, IntDistribution, ModelStrategy, ParamValue, SearchSpace, Study, TpeSampler,
    TpeSamplerConfig, TrialInput, WeightStrategy,
};

use crate::{
    HarnessResult,
    adapter::{Backend, Support},
    cli::RunConfig,
    fixtures::{FixtureTrial, Value},
    scenarios::{ParzenHistory, Scenario},
};

pub struct ParzenBackend {
    study: Study,
    scenario: Scenario,
    dimensions: usize,
}

impl Backend for ParzenBackend {
    const NAME: &'static str = "parzen";
    const VERSION: &'static str = "0.2.0";

    fn support(_scenario: Scenario) -> Support {
        Support::yes()
    }

    fn semantics(config: &RunConfig) -> Vec<String> {
        vec![
            "Gaussian product kernels; explicit groups use joint trial-aligned mixtures".into(),
            "fixed gamma = ceil(0.10 * observations), minimum one".into(),
            "24 expected-improvement candidates; uniform observation weights".into(),
            format!(
                "history policy: {}",
                match config.parzen_history {
                    ParzenHistory::Full => "full",
                    ParzenHistory::Bounded => "bounded (25 good, 512 bad, 64 recent bad)",
                }
            ),
        ]
    }

    fn create(config: &RunConfig) -> HarnessResult<Self> {
        let (space, dimensions) = make_space(config.scenario, config.dimensions)?;
        let history = match config.parzen_history {
            ParzenHistory::Full => HistoryPolicy::Full,
            ParzenHistory::Bounded => HistoryPolicy::Bounded {
                max_good_trials: NonZeroUsize::new(25).ok_or("invalid bounded good limit")?,
                max_bad_trials: NonZeroUsize::new(512).ok_or("invalid bounded bad limit")?,
                recent_bad_trials: 64,
            },
        };
        let model = if matches!(
            config.scenario,
            Scenario::CorrelatedNumeric | Scenario::CorrelatedMixed
        ) {
            ModelStrategy::Grouped { max_group_size: 8 }
        } else {
            ModelStrategy::Independent
        };
        let sampler = TpeSampler::new(
            TpeSamplerConfig::performance(config.seed)
                .startup_trials(10)
                .ei_candidates(NonZeroUsize::new(24).ok_or("invalid candidate count")?)
                .gamma(GammaStrategy::Custom(Arc::new(|n| {
                    ((n as f64) * 0.1).ceil().max(1.0) as usize
                })))
                .weights(WeightStrategy::Uniform)
                .model(model)
                .history(history),
        )?;
        Ok(Self {
            study: Study::new(Direction::Minimize, sampler, space)?,
            scenario: config.scenario,
            dimensions,
        })
    }

    fn ingest(&mut self, trial: &FixtureTrial) -> HarnessResult<()> {
        let params = trial
            .params
            .iter()
            .enumerate()
            .filter_map(|(index, value)| {
                to_parzen(*value).map(|value| (parameter_name(index), value))
            })
            .collect();
        self.study.add_trial(TrialInput {
            params,
            value: trial.objective,
        })?;
        Ok(())
    }

    fn suggest(&mut self) -> HarnessResult<Vec<Value>> {
        suggest_study(&mut self.study, self.scenario, self.dimensions)
    }

    fn complete(&mut self, objective: f64) -> HarnessResult<()> {
        self.study.complete_trial(objective)?;
        Ok(())
    }

    fn abort(&mut self) -> HarnessResult<()> {
        self.study.abort_trial();
        Ok(())
    }

    fn observations(&self) -> usize {
        self.study.trials().len()
    }
}

fn make_space(
    scenario: Scenario,
    requested_dimensions: usize,
) -> HarnessResult<(SearchSpace, usize)> {
    let dimensions = if scenario == Scenario::CorrelatedNumeric {
        requested_dimensions.max(2)
    } else {
        requested_dimensions
    };
    let mut space = SearchSpace::new();
    let mut ids = Vec::new();
    match scenario {
        Scenario::LinearFloat | Scenario::IndependentFloat | Scenario::CorrelatedNumeric => {
            let bounds = if scenario == Scenario::CorrelatedNumeric {
                (-3.0, 3.0)
            } else {
                (-10.0, 10.0)
            };
            for index in 0..dimensions {
                ids.push(space.add(
                    parameter_name(index),
                    Distribution::Float(FloatDistribution::linear(bounds.0, bounds.1)?),
                )?);
            }
        }
        Scenario::Categorical => {
            space.add(
                parameter_name(0),
                Distribution::Categorical(CategoricalDistribution::new(20)?),
            )?;
        }
        Scenario::Integer => {
            space.add(
                parameter_name(0),
                Distribution::Int(IntDistribution::linear(-100, 100)?),
            )?;
        }
        Scenario::SteppedInteger => {
            space.add(
                parameter_name(0),
                Distribution::Int(IntDistribution::linear(-100, 100)?.with_step(5)?),
            )?;
        }
        Scenario::LogFloat => {
            space.add(
                parameter_name(0),
                Distribution::Float(FloatDistribution::log(1e-6, 1.0)?),
            )?;
        }
        Scenario::MixedIndependent | Scenario::CorrelatedMixed => {
            ids.push(space.add(
                parameter_name(0),
                Distribution::Categorical(CategoricalDistribution::new(5)?),
            )?);
            ids.push(space.add(
                parameter_name(1),
                Distribution::Float(FloatDistribution::linear(-10.0, 10.0)?),
            )?);
            ids.push(space.add(
                parameter_name(2),
                Distribution::Int(IntDistribution::linear(-100, 100)?),
            )?);
        }
        Scenario::Conditional => {
            let parent = space.add(
                parameter_name(0),
                Distribution::Categorical(CategoricalDistribution::new(2)?),
            )?;
            let child = space.add(
                parameter_name(1),
                Distribution::Float(FloatDistribution::linear(-10.0, 10.0)?),
            )?;
            space.add_condition(
                child,
                Condition::CategoricalIn {
                    parent,
                    choices: vec![1].into_boxed_slice(),
                },
            )?;
        }
    }
    if matches!(
        scenario,
        Scenario::CorrelatedNumeric | Scenario::CorrelatedMixed
    ) {
        space.add_group(ids)?;
    }
    Ok((space, dimensions))
}

fn suggest_study(
    study: &mut Study,
    scenario: Scenario,
    dimensions: usize,
) -> HarnessResult<Vec<Value>> {
    let values = match scenario {
        Scenario::LinearFloat | Scenario::IndependentFloat | Scenario::CorrelatedNumeric => (0
            ..dimensions)
            .map(|index| {
                study
                    .suggest_float(&parameter_name(index))
                    .map(Value::Float)
            })
            .collect::<Result<Vec<_>, _>>()?,
        Scenario::Categorical => vec![Value::Categorical(study.suggest_categorical("p0")?)],
        Scenario::Integer | Scenario::SteppedInteger => vec![Value::Int(study.suggest_int("p0")?)],
        Scenario::LogFloat => vec![Value::Float(study.suggest_float("p0")?)],
        Scenario::MixedIndependent | Scenario::CorrelatedMixed => vec![
            Value::Categorical(study.suggest_categorical("p0")?),
            Value::Float(study.suggest_float("p1")?),
            Value::Int(study.suggest_int("p2")?),
        ],
        Scenario::Conditional => {
            let parent = study.suggest_categorical("p0")?;
            vec![
                Value::Categorical(parent),
                if parent == 1 {
                    Value::Float(study.suggest_float("p1")?)
                } else {
                    Value::Inactive
                },
            ]
        }
    };
    Ok(values)
}

fn parameter_name(index: usize) -> String {
    format!("p{index}")
}

fn to_parzen(value: Value) -> Option<ParamValue> {
    match value {
        Value::Float(value) => Some(ParamValue::Float(value)),
        Value::Int(value) => Some(ParamValue::Int(value)),
        Value::Categorical(value) => Some(ParamValue::Categorical(value)),
        Value::Inactive => None,
    }
}
