use std::collections::HashMap;

use optimizer::{
    Study, Trial,
    parameter::{CategoricalParam, FloatParam, IntParam, ParamId, ParamValue, Parameter},
    sampler::tpe::{MultivariateTpeSamplerBuilder, TpeSamplerBuilder},
};

use crate::{
    HarnessResult,
    adapter::{Backend, Support},
    cli::RunConfig,
    fixtures::{FixtureTrial, Value},
    scenarios::Scenario,
};

#[derive(Clone)]
enum ParameterDef {
    Float(FloatParam),
    Int(IntParam),
    Categorical(CategoricalParam<u32>),
}

impl ParameterDef {
    fn id(&self) -> ParamId {
        match self {
            Self::Float(p) => p.id(),
            Self::Int(p) => p.id(),
            Self::Categorical(p) => p.id(),
        }
    }

    fn suggest(&self, trial: &mut Trial) -> optimizer::Result<Value> {
        match self {
            Self::Float(param) => param.suggest(trial).map(Value::Float),
            Self::Int(param) => param.suggest(trial).map(Value::Int),
            Self::Categorical(param) => param.suggest(trial).map(Value::Categorical),
        }
    }
}

pub struct OptimizerBackend {
    study: Study<f64>,
    parameters: Vec<ParameterDef>,
    scenario: Scenario,
    pending: Option<Trial>,
}

impl Backend for OptimizerBackend {
    const NAME: &'static str = "optimizer";
    const VERSION: &'static str = "1.0.1";

    fn support(scenario: Scenario) -> Support {
        if scenario == Scenario::CorrelatedMixed {
            Support::no(
                "optimizer 1.0.1 multivariate TPE samples categorical parameters independently",
            )
        } else {
            Support::yes()
        }
    }

    fn semantics(config: &RunConfig) -> Vec<String> {
        let mut semantics = vec![
            if config.scenario == Scenario::CorrelatedNumeric { "MultivariateTpeSampler with a cached joint numeric vector".into() } else { "univariate TpeSampler; parameters sampled independently".into() },
            "Gaussian KDE with optimizer's bandwidth policy and categorical probability smoothing".into(),
            "fixed gamma = 0.10; 10 startup observations; 24 candidates; full retained history".into(),
            "fixture history is injected through public enqueue, ask/suggest, and complete_trial calls".into(),
        ];
        if config.scenario == Scenario::Integer {
            semantics.push(format!(
                "integer domain: -100..={} ({} exact values)",
                -100 + config.integer_cardinality as i64 - 1,
                config.integer_cardinality
            ));
        }
        if config.scenario == Scenario::SteppedFloat {
            semantics.push("stepped-float domain: -10..=10 with exact step 0.5".into());
        }
        semantics
    }

    fn create(config: &RunConfig) -> HarnessResult<Self> {
        let study = if config.scenario == Scenario::CorrelatedNumeric {
            let sampler = MultivariateTpeSamplerBuilder::new()
                .gamma(0.10)
                .n_startup_trials(10)
                .n_ei_candidates(24)
                .seed(config.seed)
                .build()?;
            Study::minimize(sampler)
        } else {
            let sampler = TpeSamplerBuilder::new()
                .gamma(0.10)
                .n_startup_trials(10)
                .n_ei_candidates(24)
                .seed(config.seed)
                .build()?;
            Study::minimize(sampler)
        };
        Ok(Self {
            study,
            parameters: make_parameters(config),
            scenario: config.scenario,
            pending: None,
        })
    }

    fn ingest(&mut self, fixture: &FixtureTrial) -> HarnessResult<()> {
        let fixed = self
            .parameters
            .iter()
            .zip(&fixture.params)
            .filter_map(|(parameter, value)| {
                to_optimizer(*value).map(|value| (parameter.id(), value))
            })
            .collect::<HashMap<_, _>>();
        self.study.enqueue(fixed);
        let mut trial = self.study.ask();
        let _ = suggest_parameters(&self.parameters, self.scenario, &mut trial)?;
        self.study.complete_trial(trial, fixture.objective);
        Ok(())
    }

    fn suggest(&mut self) -> HarnessResult<Vec<Value>> {
        let mut trial = self.study.ask();
        let values = suggest_parameters(&self.parameters, self.scenario, &mut trial)?;
        self.pending = Some(trial);
        Ok(values)
    }

    fn complete(&mut self, objective: f64) -> HarnessResult<()> {
        let trial = self
            .pending
            .take()
            .ok_or("no pending optimizer suggestion")?;
        self.study.complete_trial(trial, objective);
        Ok(())
    }

    fn abort(&mut self) -> HarnessResult<()> {
        if let Some(trial) = self.pending.take() {
            self.study.fail_trial(trial, "benchmark abort");
        }
        Ok(())
    }

    fn observations(&self) -> usize {
        self.study.n_trials()
    }
}

fn make_parameters(config: &RunConfig) -> Vec<ParameterDef> {
    let scenario = config.scenario;
    let dimensions = config.dimensions;
    match scenario {
        Scenario::LinearFloat | Scenario::IndependentFloat => (0..dimensions)
            .map(|i| ParameterDef::Float(FloatParam::new(-10.0, 10.0).name(format!("p{i}"))))
            .collect(),
        Scenario::CorrelatedNumeric => (0..dimensions.max(2))
            .map(|i| ParameterDef::Float(FloatParam::new(-3.0, 3.0).name(format!("p{i}"))))
            .collect(),
        Scenario::Categorical => vec![ParameterDef::Categorical(
            CategoricalParam::new((0..20).collect()).name("p0"),
        )],
        Scenario::SteppedFloat => vec![ParameterDef::Float(
            FloatParam::new(-10.0, 10.0).step(0.5).name("p0"),
        )],
        Scenario::Integer => vec![ParameterDef::Int(
            IntParam::new(-100, -100 + config.integer_cardinality as i64 - 1).name("p0"),
        )],
        Scenario::SteppedInteger => vec![ParameterDef::Int(
            IntParam::new(-100, 100).step(5).name("p0"),
        )],
        Scenario::LogFloat => vec![ParameterDef::Float(
            FloatParam::new(1e-6, 1.0).log_scale().name("p0"),
        )],
        Scenario::MixedIndependent | Scenario::CorrelatedMixed => vec![
            ParameterDef::Categorical(CategoricalParam::new((0..5).collect()).name("p0")),
            ParameterDef::Float(FloatParam::new(-10.0, 10.0).name("p1")),
            ParameterDef::Int(IntParam::new(-100, 100).name("p2")),
        ],
        Scenario::Conditional => vec![
            ParameterDef::Categorical(CategoricalParam::new(vec![0, 1]).name("p0")),
            ParameterDef::Float(FloatParam::new(-10.0, 10.0).name("p1")),
        ],
    }
}

fn suggest_parameters(
    parameters: &[ParameterDef],
    scenario: Scenario,
    trial: &mut Trial,
) -> HarnessResult<Vec<Value>> {
    let mut values: Vec<Value> = Vec::with_capacity(parameters.len());
    for (index, parameter) in parameters.iter().enumerate() {
        if scenario == Scenario::Conditional
            && index == 1
            && values.first().and_then(|value| value.as_categorical()) == Some(0)
        {
            values.push(Value::Inactive);
        } else {
            values.push(parameter.suggest(trial)?);
        }
    }
    Ok(values)
}

fn to_optimizer(value: Value) -> Option<ParamValue> {
    match value {
        Value::Float(value) => Some(ParamValue::Float(value)),
        Value::Int(value) => Some(ParamValue::Int(value)),
        Value::Categorical(value) => Some(ParamValue::Categorical(value as usize)),
        Value::Inactive => None,
    }
}
