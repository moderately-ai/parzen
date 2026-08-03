use crate::{HarnessResult, fixtures::Value, scenarios::Scenario};

pub fn evaluate(scenario: Scenario, params: &[Value]) -> HarnessResult<f64> {
    let value = match scenario {
        Scenario::LinearFloat | Scenario::IndependentFloat | Scenario::SteppedFloat => params
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let target = 1.5 - index as f64 * 0.1;
                let delta = float(*value)? - target;
                Ok(delta * delta)
            })
            .sum::<HarnessResult<f64>>()?,
        Scenario::Categorical => match categorical(params[0])? {
            7 => 0.0,
            6 | 8 => 0.1,
            _ => 1.0,
        },
        Scenario::Integer => (int(params[0])? - 17).pow(2) as f64,
        Scenario::SteppedInteger => (int(params[0])? - 15).pow(2) as f64,
        Scenario::LogFloat => {
            let ratio = (float(params[0])? / 1e-3).ln();
            ratio * ratio
        }
        Scenario::MixedIndependent | Scenario::CorrelatedMixed => {
            let category = categorical(params[0])?;
            let x = float(params[1])?;
            let n = int(params[2])?;
            let category_cost = if category == 2 { 0.0 } else { 1.0 };
            category_cost + (x - 2.5).powi(2) + ((n - 15) as f64 / 10.0).powi(2)
        }
        Scenario::Conditional => {
            let parent = categorical(params[0])?;
            if parent == 0 {
                0.25
            } else {
                (float(params[1])? - 3.0).powi(2)
            }
        }
        Scenario::CorrelatedNumeric => rosenbrock(params)?,
    };
    if value.is_finite() {
        Ok(value)
    } else {
        Err("objective produced a non-finite value".into())
    }
}

#[must_use]
pub const fn optimum(_scenario: Scenario) -> f64 {
    0.0
}

fn rosenbrock(params: &[Value]) -> HarnessResult<f64> {
    if params.len() < 2 {
        return Err("Rosenbrock requires at least two dimensions".into());
    }
    params.windows(2).try_fold(0.0, |sum, pair| {
        let x = float(pair[0])?;
        let y = float(pair[1])?;
        Ok(sum + 100.0 * (y - x * x).powi(2) + (1.0 - x).powi(2))
    })
}

fn float(value: Value) -> HarnessResult<f64> {
    value
        .as_float()
        .ok_or_else(|| "expected active float parameter".into())
}

fn int(value: Value) -> HarnessResult<i64> {
    value
        .as_int()
        .ok_or_else(|| "expected integer parameter".into())
}

fn categorical(value: Value) -> HarnessResult<u32> {
    value
        .as_categorical()
        .ok_or_else(|| "expected categorical parameter".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documented_optima_are_zero() {
        let cases = [
            (Scenario::LinearFloat, vec![Value::Float(1.5)]),
            (Scenario::Integer, vec![Value::Int(17)]),
            (Scenario::SteppedFloat, vec![Value::Float(1.5)]),
            (Scenario::SteppedInteger, vec![Value::Int(15)]),
            (Scenario::LogFloat, vec![Value::Float(1e-3)]),
            (
                Scenario::CorrelatedNumeric,
                vec![Value::Float(1.0), Value::Float(1.0)],
            ),
        ];
        for (scenario, params) in cases {
            assert_eq!(evaluate(scenario, &params).expect("valid objective"), 0.0);
        }
    }
}
