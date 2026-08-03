use hyperopt::{
    Optimizer,
    kernel::{continuous::Gaussian, discrete::Binomial, universal::Uniform},
};
use ordered_float::OrderedFloat;

use crate::{
    HarnessResult,
    adapter::{Backend, Support},
    cli::RunConfig,
    fixtures::{FixtureTrial, Value},
    scenarios::Scenario,
};

type Float = OrderedFloat<f64>;
type FloatOptimizer = Optimizer<Uniform<Float, Float>, Float, Float>;
type IntOptimizer = Optimizer<Uniform<i32, OrderedFloat<f64>>, i32, OrderedFloat<f64>>;

enum ParameterOptimizer {
    Float(FloatOptimizer),
    Int(IntOptimizer),
}

pub struct HyperoptBackend {
    optimizers: Vec<ParameterOptimizer>,
    pending: Option<Vec<Value>>,
    observations: usize,
}

impl Backend for HyperoptBackend {
    const NAME: &'static str = "hyperopt";
    const VERSION: &'static str = "0.0.17";

    fn support(scenario: Scenario) -> Support {
        match scenario {
            Scenario::LinearFloat | Scenario::IndependentFloat | Scenario::Integer => {
                Support::yes()
            }
            Scenario::Categorical
            | Scenario::MixedIndependent
            | Scenario::Conditional
            | Scenario::CorrelatedMixed => {
                Support::no("hyperopt 0.0.17 has no categorical or conditional public model")
            }
            Scenario::SteppedFloat | Scenario::SteppedInteger => {
                Support::no("hyperopt 0.0.17 has no equivalent stepped numeric model")
            }
            Scenario::LogFloat => Support::no("hyperopt 0.0.17 has no native log-scaled range"),
            Scenario::CorrelatedNumeric => {
                Support::no("independent Optimizer instances do not model numeric correlation")
            }
        }
    }

    fn semantics(config: &RunConfig) -> Vec<String> {
        let mut semantics = vec![
            "one independent Optimizer per modeled parameter".into(),
            "Gaussian continuous kernel; Binomial discrete integer kernel".into(),
            "ordered-float wrappers are included in measured lifecycle calls".into(),
            "gamma cutoff = 0.10 using round; 24 candidates; full retained history".into(),
            "integer values are shifted by +100 because the Binomial kernel requires a nonnegative coordinate".into(),
        ];
        if config.scenario == Scenario::Integer {
            semantics.push(format!(
                "integer domain: -100..={} ({} exact values)",
                -100 + config.integer_cardinality as i64 - 1,
                config.integer_cardinality
            ));
        }
        semantics
    }

    fn create(config: &RunConfig) -> HarnessResult<Self> {
        let dimensions = if config.scenario == Scenario::IndependentFloat {
            config.dimensions
        } else {
            1
        };
        let mut optimizers = Vec::with_capacity(dimensions);
        for index in 0..dimensions {
            let seed = config.seed.wrapping_add(index as u64);
            if config.scenario == Scenario::Integer {
                let optimizer = Optimizer::new(
                    0_i32..=i32::try_from(config.integer_cardinality - 1)?,
                    Uniform::with_bounds(0_i32..=i32::try_from(config.integer_cardinality - 1)?),
                    fastrand::Rng::with_seed(seed),
                )
                .cutoff(0.10)
                .n_candidates(24_usize);
                optimizers.push(ParameterOptimizer::Int(optimizer));
            } else {
                let low = OrderedFloat(-10.0);
                let high = OrderedFloat(10.0);
                let optimizer = Optimizer::new(
                    low..=high,
                    Uniform::with_bounds(low..=high),
                    fastrand::Rng::with_seed(seed),
                )
                .cutoff(0.10)
                .n_candidates(24_usize);
                optimizers.push(ParameterOptimizer::Float(optimizer));
            }
        }
        Ok(Self {
            optimizers,
            pending: None,
            observations: 0,
        })
    }

    fn ingest(&mut self, trial: &FixtureTrial) -> HarnessResult<()> {
        for (optimizer, value) in self.optimizers.iter_mut().zip(&trial.params) {
            feed(optimizer, *value, trial.objective)?;
        }
        self.observations += 1;
        Ok(())
    }

    fn suggest(&mut self) -> HarnessResult<Vec<Value>> {
        let values = self
            .optimizers
            .iter_mut()
            .map(|optimizer| match optimizer {
                ParameterOptimizer::Float(optimizer) => {
                    Value::Float(optimizer.new_trial::<Gaussian<Float>>().into_inner())
                }
                ParameterOptimizer::Int(optimizer) => Value::Int(i64::from(
                    optimizer.new_trial::<Binomial<i32, OrderedFloat<f64>>>() - 100,
                )),
            })
            .collect::<Vec<_>>();
        self.pending = Some(values.clone());
        Ok(values)
    }

    fn complete(&mut self, objective: f64) -> HarnessResult<()> {
        let pending = self
            .pending
            .take()
            .ok_or("no pending hyperopt suggestion")?;
        for (optimizer, value) in self.optimizers.iter_mut().zip(pending) {
            feed(optimizer, value, objective)?;
        }
        self.observations += 1;
        Ok(())
    }

    fn abort(&mut self) -> HarnessResult<()> {
        self.pending = None;
        Ok(())
    }

    fn observations(&self) -> usize {
        self.observations
    }
}

fn feed(optimizer: &mut ParameterOptimizer, value: Value, objective: f64) -> HarnessResult<()> {
    match (optimizer, value) {
        (ParameterOptimizer::Float(optimizer), Value::Float(value)) => {
            optimizer.feed_back(OrderedFloat(value), OrderedFloat(objective))
        }
        (ParameterOptimizer::Int(optimizer), Value::Int(value)) => {
            optimizer.feed_back(i32::try_from(value + 100)?, OrderedFloat(objective))
        }
        _ => return Err("fixture value does not match hyperopt parameter model".into()),
    }
    Ok(())
}
