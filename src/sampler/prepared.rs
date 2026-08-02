use std::num::NonZeroU64;

use smallvec::SmallVec;

use crate::{Distribution, ParamValue, ParzenError, SearchSpace, search_space::ParamId};

use super::EstimatorKey;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum PreparedValue {
    Continuous(f64),
    Discrete { grid_index: u64, transformed: f64 },
    Categorical(u32),
}

#[derive(Debug, Clone)]
pub(crate) struct PreparedParam {
    pub id: ParamId,
    pub position: usize,
    pub distribution: Distribution,
    pub kernel: PreparedKernel,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum PreparedKernel {
    Continuous(PreparedContinuous),
    Discrete(PreparedDiscrete),
    Categorical(PreparedCategorical),
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PreparedContinuous {
    pub transformed_low: f64,
    pub transformed_high: f64,
    pub range: f64,
    pub prior_center: f64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PreparedDiscrete {
    pub adapted_low: f64,
    pub adapted_high: f64,
    pub range: f64,
    pub prior_center: f64,
    /// `None` represents the one valid cardinality above `u64::MAX`: 2^64.
    pub grid_cardinality: Option<NonZeroU64>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PreparedCategorical {
    pub choices: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PreparedEstimatorKind {
    Continuous1,
    Discrete1,
    Categorical1,
    ContinuousGroup,
    GeneralGroup,
}

pub(crate) struct PreparedEstimator {
    pub key: EstimatorKey,
    pub params: SmallVec<[PreparedParam; 8]>,
    pub kind: PreparedEstimatorKind,
}

impl PreparedEstimator {
    pub(crate) fn new(
        key: EstimatorKey,
        params: &[ParamId],
        space: &SearchSpace,
    ) -> Result<Self, ParzenError> {
        let params = params
            .iter()
            .copied()
            .enumerate()
            .map(|(position, id)| PreparedParam::new(id, position, space))
            .collect::<Result<SmallVec<_>, _>>()?;
        let kind = match params.as_slice() {
            [
                PreparedParam {
                    kernel: PreparedKernel::Continuous(_),
                    ..
                },
            ] => PreparedEstimatorKind::Continuous1,
            [
                PreparedParam {
                    kernel: PreparedKernel::Discrete(_),
                    ..
                },
            ] => PreparedEstimatorKind::Discrete1,
            [
                PreparedParam {
                    kernel: PreparedKernel::Categorical(_),
                    ..
                },
            ] => PreparedEstimatorKind::Categorical1,
            values
                if values
                    .iter()
                    .all(|param| matches!(param.kernel, PreparedKernel::Continuous(_))) =>
            {
                PreparedEstimatorKind::ContinuousGroup
            }
            _ => PreparedEstimatorKind::GeneralGroup,
        };
        Ok(Self { key, params, kind })
    }

    pub(crate) fn all_categorical(&self) -> bool {
        matches!(self.kind, PreparedEstimatorKind::Categorical1)
            || self
                .params
                .iter()
                .all(|param| matches!(param.kernel, PreparedKernel::Categorical(_)))
    }
}

impl PreparedParam {
    pub(crate) fn new(
        id: ParamId,
        position: usize,
        space: &SearchSpace,
    ) -> Result<Self, ParzenError> {
        let distribution = space.parameters[id.0 as usize].distribution.clone();
        let kernel = match distribution {
            Distribution::Categorical(dist) => PreparedKernel::Categorical(PreparedCategorical {
                choices: dist.num_choices(),
            }),
            Distribution::Float(dist) => {
                if let Some(step) = dist.step() {
                    let highest = dist.grid_value(dist.max_step_index().unwrap_or(0));
                    PreparedKernel::Discrete(PreparedDiscrete::new(
                        dist.low() - step / 2.0,
                        highest + step / 2.0,
                        dist.max_step_index().unwrap_or(0),
                    )?)
                } else {
                    PreparedKernel::Continuous(PreparedContinuous::new(
                        dist.transform(dist.low()),
                        dist.transform(dist.high()),
                    )?)
                }
            }
            Distribution::Int(dist) => {
                let half_step = dist.step() as f64 / 2.0;
                let raw_low = dist.low() as f64 - half_step;
                let raw_high = dist.grid_value(dist.max_step_index()) as f64 + half_step;
                let (low, high) = match dist.scale() {
                    crate::IntScale::Linear => (raw_low, raw_high),
                    crate::IntScale::Log => (raw_low.ln(), raw_high.ln()),
                };
                PreparedKernel::Discrete(PreparedDiscrete::new(low, high, dist.max_step_index())?)
            }
        };
        Ok(Self {
            id,
            position,
            distribution,
            kernel,
        })
    }

    #[cfg(test)]
    pub(crate) fn prepare(&self, value: ParamValue) -> Option<PreparedValue> {
        if self.distribution.canonicalize(value) != Some(value) {
            return None;
        }
        self.prepare_canonical(value)
    }

    pub(crate) fn prepare_canonical(&self, value: ParamValue) -> Option<PreparedValue> {
        debug_assert_eq!(self.distribution.canonicalize(value), Some(value));
        match (&self.distribution, value) {
            (Distribution::Categorical(dist), ParamValue::Categorical(value))
                if value < dist.num_choices() =>
            {
                Some(PreparedValue::Categorical(value))
            }
            (Distribution::Float(dist), ParamValue::Float(value)) => {
                if let Some(step) = dist.step() {
                    let grid_index = ((value - dist.low()) / step).round() as u64;
                    debug_assert!(matches!(
                        self.kernel,
                        PreparedKernel::Discrete(prepared)
                            if prepared.grid_cardinality.is_none_or(|count| grid_index < count.get())
                    ));
                    Some(PreparedValue::Discrete {
                        grid_index,
                        transformed: value,
                    })
                } else {
                    Some(PreparedValue::Continuous(dist.transform(value)))
                }
            }
            (Distribution::Int(dist), ParamValue::Int(value)) => {
                let delta = i128::from(value) - i128::from(dist.low());
                let grid_index = u64::try_from(delta / i128::from(dist.step())).ok()?;
                debug_assert!(matches!(
                    self.kernel,
                    PreparedKernel::Discrete(prepared)
                        if prepared.grid_cardinality.is_none_or(|count| grid_index < count.get())
                ));
                Some(PreparedValue::Discrete {
                    grid_index,
                    transformed: dist.transform(value),
                })
            }
            _ => None,
        }
    }
}

impl PreparedContinuous {
    fn new(low: f64, high: f64) -> Result<Self, ParzenError> {
        let range = high - low;
        if !range.is_finite() || range <= 0.0 {
            return Err(ParzenError::InternalModel(
                "prepared continuous domain is not finite and positive".into(),
            ));
        }
        Ok(Self {
            transformed_low: low,
            transformed_high: high,
            range,
            prior_center: (low + high) * 0.5,
        })
    }
}

impl PreparedDiscrete {
    fn new(low: f64, high: f64, max_grid_index: u64) -> Result<Self, ParzenError> {
        let range = high - low;
        if !range.is_finite() || range <= 0.0 {
            return Err(ParzenError::InternalModel(
                "prepared discrete domain is invalid".into(),
            ));
        }
        Ok(Self {
            adapted_low: low,
            adapted_high: high,
            range,
            prior_center: (low + high) * 0.5,
            grid_cardinality: NonZeroU64::new(max_grid_index.wrapping_add(1)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CategoricalDistribution, FloatDistribution, IntDistribution};

    #[test]
    fn prepared_coordinates_match_distribution_transforms() {
        let mut space = SearchSpace::new();
        let linear = space
            .add(
                "linear",
                Distribution::Float(FloatDistribution::linear(-2.0, 4.0).unwrap()),
            )
            .unwrap();
        let log = space
            .add(
                "log",
                Distribution::Float(FloatDistribution::log(0.01, 100.0).unwrap()),
            )
            .unwrap();
        let discrete = space
            .add(
                "discrete",
                Distribution::Int(
                    IntDistribution::linear(-5, 15)
                        .unwrap()
                        .with_step(5)
                        .unwrap(),
                ),
            )
            .unwrap();
        let prepared = PreparedEstimator::new(
            EstimatorKey::Group(crate::GroupId(0)),
            &[linear, log, discrete],
            &space,
        )
        .unwrap();
        assert_eq!(
            prepared.params[0].prepare(ParamValue::Float(1.0)),
            Some(PreparedValue::Continuous(1.0))
        );
        assert_eq!(
            prepared.params[1].prepare(ParamValue::Float(1.0)),
            Some(PreparedValue::Continuous(0.0))
        );
        assert_eq!(
            prepared.params[2].prepare(ParamValue::Int(10)),
            Some(PreparedValue::Discrete {
                grid_index: 3,
                transformed: 10.0
            })
        );
    }

    #[test]
    fn prepared_metadata_covers_every_kernel_family() {
        let mut space = SearchSpace::new();
        let continuous = space
            .add(
                "continuous",
                Distribution::Float(FloatDistribution::log(0.01, 100.0).unwrap()),
            )
            .unwrap();
        let stepped = space
            .add(
                "stepped",
                Distribution::Float(
                    FloatDistribution::linear(-1.0, 1.0)
                        .unwrap()
                        .with_step(0.5)
                        .unwrap(),
                ),
            )
            .unwrap();
        let integer = space
            .add(
                "integer",
                Distribution::Int(IntDistribution::log(1, 1_000).unwrap()),
            )
            .unwrap();
        let categorical = space
            .add(
                "categorical",
                Distribution::Categorical(CategoricalDistribution::new(7).unwrap()),
            )
            .unwrap();
        let full_integer = space
            .add(
                "full-integer",
                Distribution::Int(IntDistribution::linear(i64::MIN, i64::MAX).unwrap()),
            )
            .unwrap();

        let continuous = PreparedParam::new(continuous, 0, &space).unwrap();
        let PreparedKernel::Continuous(metadata) = continuous.kernel else {
            panic!("log float should use a continuous kernel");
        };
        assert_eq!(metadata.transformed_low, 0.01_f64.ln());
        assert_eq!(metadata.transformed_high, 100.0_f64.ln());
        assert_eq!(
            metadata.range,
            metadata.transformed_high - metadata.transformed_low
        );

        let stepped = PreparedParam::new(stepped, 0, &space).unwrap();
        let PreparedKernel::Discrete(metadata) = stepped.kernel else {
            panic!("stepped float should use a discrete kernel");
        };
        assert_eq!(metadata.adapted_low, -1.25);
        assert_eq!(metadata.adapted_high, 1.25);
        assert_eq!(metadata.grid_cardinality.map(NonZeroU64::get), Some(5));

        let integer = PreparedParam::new(integer, 0, &space).unwrap();
        let PreparedKernel::Discrete(metadata) = integer.kernel else {
            panic!("integer should use a discrete kernel");
        };
        assert_eq!(metadata.grid_cardinality.map(NonZeroU64::get), Some(1_000));
        assert_eq!(
            integer.prepare(ParamValue::Int(10)),
            Some(PreparedValue::Discrete {
                grid_index: 9,
                transformed: 10.0_f64.ln(),
            })
        );

        let categorical = PreparedParam::new(categorical, 0, &space).unwrap();
        assert!(matches!(
            categorical.kernel,
            PreparedKernel::Categorical(PreparedCategorical { choices: 7 })
        ));

        let full_integer = PreparedParam::new(full_integer, 0, &space).unwrap();
        assert!(matches!(
            full_integer.kernel,
            PreparedKernel::Discrete(PreparedDiscrete {
                grid_cardinality: None,
                ..
            })
        ));
    }
}
