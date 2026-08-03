use rand::{SeedableRng, rngs::StdRng};
use tpe::{TpeOptimizer, TpeOptimizerBuilder, density_estimation::DefaultEstimatorBuilder};

use crate::{
    HarnessResult,
    adapter::{Backend, Support},
    cli::RunConfig,
    fixtures::{FixtureTrial, Value},
    scenarios::Scenario,
};

pub struct TpeBackend {
    optimizers: Vec<TpeOptimizer<DefaultEstimatorBuilder>>,
    rng: StdRng,
    scenario: Scenario,
    pending: Option<Vec<Value>>,
    observations: usize,
}

impl Backend for TpeBackend {
    const NAME: &'static str = "tpe";
    const VERSION: &'static str = "0.3.1";

    fn support(scenario: Scenario) -> Support {
        match scenario {
            Scenario::Integer | Scenario::SteppedFloat | Scenario::SteppedInteger => {
                Support::no("tpe 0.3.1 has no equivalent stepped or discrete numeric model")
            }
            Scenario::LogFloat => Support::no("tpe 0.3.1 has no native log-scaled range"),
            Scenario::MixedIndependent | Scenario::CorrelatedMixed => Support::no(
                "fixture includes a discrete integer that tpe 0.3.1 cannot model equivalently",
            ),
            Scenario::CorrelatedNumeric => {
                Support::no("one independent TpeOptimizer per parameter does not model correlation")
            }
            _ => Support::yes(),
        }
    }

    fn semantics(_config: &RunConfig) -> Vec<String> {
        vec![
            "one independent TpeOptimizer per modeled parameter".into(),
            "Parzen estimator for continuous parameters; histogram estimator for categories".into(),
            "gamma = 0.10; 24 candidates; inactive conditional values are told as NaN".into(),
            "full observation history; no startup phase in the public optimizer".into(),
        ]
    }

    fn create(config: &RunConfig) -> HarnessResult<Self> {
        let mut builder = TpeOptimizerBuilder::new();
        builder.gamma(0.10).candidates(24);
        let dimensions = match config.scenario {
            Scenario::IndependentFloat => config.dimensions,
            Scenario::Conditional => 2,
            _ => 1,
        };
        let mut optimizers = Vec::with_capacity(dimensions);
        for index in 0..dimensions {
            let optimizer = if config.scenario == Scenario::Categorical
                || (config.scenario == Scenario::Conditional && index == 0)
            {
                builder.build(
                    tpe::histogram_estimator(),
                    tpe::categorical_range(if config.scenario == Scenario::Conditional {
                        2
                    } else {
                        20
                    })?,
                )?
            } else {
                builder.build(tpe::parzen_estimator(), tpe::range(-10.0, 10.0)?)?
            };
            optimizers.push(optimizer);
        }
        Ok(Self {
            optimizers,
            rng: StdRng::seed_from_u64(config.seed),
            scenario: config.scenario,
            pending: None,
            observations: 0,
        })
    }

    fn ingest(&mut self, trial: &FixtureTrial) -> HarnessResult<()> {
        for (optimizer, value) in self.optimizers.iter_mut().zip(&trial.params) {
            optimizer.tell(to_f64(*value), trial.objective)?;
        }
        self.observations += 1;
        Ok(())
    }

    fn suggest(&mut self) -> HarnessResult<Vec<Value>> {
        let mut values: Vec<Value> = Vec::with_capacity(self.optimizers.len());
        for (index, optimizer) in self.optimizers.iter_mut().enumerate() {
            if self.scenario == Scenario::Conditional
                && index == 1
                && values.first().and_then(|value| value.as_categorical()) == Some(0)
            {
                values.push(Value::Inactive);
                continue;
            }
            let value = optimizer.ask(&mut self.rng)?;
            values.push(
                if self.scenario == Scenario::Categorical
                    || (self.scenario == Scenario::Conditional && index == 0)
                {
                    Value::Categorical(value as u32)
                } else {
                    Value::Float(value)
                },
            );
        }
        self.pending = Some(values.clone());
        Ok(values)
    }

    fn complete(&mut self, objective: f64) -> HarnessResult<()> {
        let pending = self.pending.take().ok_or("no pending tpe suggestion")?;
        for (optimizer, value) in self.optimizers.iter_mut().zip(pending) {
            optimizer.tell(to_f64(value), objective)?;
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

fn to_f64(value: Value) -> f64 {
    match value {
        Value::Float(value) => value,
        Value::Int(value) => value as f64,
        Value::Categorical(value) => f64::from(value),
        Value::Inactive => f64::NAN,
    }
}
