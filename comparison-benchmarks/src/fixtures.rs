use serde::{Deserialize, Serialize};

use crate::{HarnessResult, objectives::evaluate, scenarios::Scenario};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Value {
    Float(f64),
    Int(i64),
    Categorical(u32),
    Inactive,
}

impl Value {
    #[must_use]
    pub const fn as_float(self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_int(self) -> Option<i64> {
        match self {
            Self::Int(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_categorical(self) -> Option<u32> {
        match self {
            Self::Categorical(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FixtureTrial {
    pub id: u64,
    pub params: Vec<Value>,
    pub objective: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fixture {
    pub scenario: Scenario,
    pub dimensions: usize,
    pub seed: u64,
    pub trials: Vec<FixtureTrial>,
    pub checksum: u64,
}

impl Fixture {
    pub fn generate(
        scenario: Scenario,
        dimensions: usize,
        count: usize,
        seed: u64,
    ) -> HarnessResult<Self> {
        let mut rng = SplitMix64::new(seed);
        let mut trials = Vec::with_capacity(count);
        for id in 0..count {
            let params = sample_params(scenario, dimensions, &mut rng);
            let objective = evaluate(scenario, &params)?;
            trials.push(FixtureTrial {
                id: id as u64,
                params,
                objective,
            });
        }
        let checksum = checksum_trials(&trials);
        Ok(Self {
            scenario,
            dimensions,
            seed,
            trials,
            checksum,
        })
    }
}

fn sample_params(scenario: Scenario, dimensions: usize, rng: &mut SplitMix64) -> Vec<Value> {
    match scenario {
        Scenario::LinearFloat | Scenario::IndependentFloat => (0..dimensions)
            .map(|_| Value::Float(rng.range_f64(-10.0, 10.0)))
            .collect(),
        Scenario::Categorical => vec![Value::Categorical(rng.range_u64(20) as u32)],
        Scenario::Integer => vec![Value::Int(rng.range_i64(-100, 100))],
        Scenario::SteppedInteger => {
            vec![Value::Int(-100 + 5 * rng.range_u64(41) as i64)]
        }
        Scenario::LogFloat => {
            vec![Value::Float(
                rng.range_f64(1e-6_f64.ln(), 1.0_f64.ln()).exp(),
            )]
        }
        Scenario::MixedIndependent | Scenario::CorrelatedMixed => vec![
            Value::Categorical(rng.range_u64(5) as u32),
            Value::Float(rng.range_f64(-10.0, 10.0)),
            Value::Int(rng.range_i64(-100, 100)),
        ],
        Scenario::Conditional => {
            let parent = rng.range_u64(2) as u32;
            vec![
                Value::Categorical(parent),
                if parent == 1 {
                    Value::Float(rng.range_f64(-10.0, 10.0))
                } else {
                    Value::Inactive
                },
            ]
        }
        Scenario::CorrelatedNumeric => (0..dimensions.max(2))
            .map(|_| Value::Float(rng.range_f64(-3.0, 3.0)))
            .collect(),
    }
}

#[must_use]
pub fn checksum_values(values: &[Value]) -> u64 {
    values.iter().fold(0xcbf2_9ce4_8422_2325, |hash, value| {
        let encoded = match value {
            Value::Float(value) => value.to_bits(),
            Value::Int(value) => *value as u64,
            Value::Categorical(value) => u64::from(*value),
            Value::Inactive => u64::MAX,
        };
        (hash ^ encoded).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[must_use]
pub fn checksum_trials(trials: &[FixtureTrial]) -> u64 {
    trials.iter().fold(0xcbf2_9ce4_8422_2325, |hash, trial| {
        (hash ^ checksum_values(&trial.params) ^ trial.objective.to_bits())
            .wrapping_mul(0x0000_0100_0000_01b3)
    })
}

#[derive(Debug, Clone, Copy)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn unit_f64(&mut self) -> f64 {
        (self.next() >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0)
    }

    fn range_f64(&mut self, low: f64, high: f64) -> f64 {
        self.unit_f64().mul_add(high - low, low)
    }

    fn range_u64(&mut self, count: u64) -> u64 {
        if count == 0 { 0 } else { self.next() % count }
    }

    fn range_i64(&mut self, low: i64, high: i64) -> i64 {
        low + self.range_u64((high - low + 1) as u64) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixtures_are_deterministic() {
        let first =
            Fixture::generate(Scenario::MixedIndependent, 3, 100, 42).expect("valid fixture");
        let second =
            Fixture::generate(Scenario::MixedIndependent, 3, 100, 42).expect("valid fixture");
        assert_eq!(first, second);
        assert_ne!(first.checksum, 0);
    }

    #[test]
    fn conditional_masks_match_parent() {
        let fixture = Fixture::generate(Scenario::Conditional, 2, 100, 7).expect("valid fixture");
        for trial in fixture.trials {
            let parent = trial.params[0].as_categorical().expect("parent");
            assert_eq!(matches!(trial.params[1], Value::Inactive), parent == 0);
        }
    }
}
