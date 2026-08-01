// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::f64::consts::TAU;

use rand::{Rng, distr::Distribution as _, rngs::StdRng};
use rand_distr::Normal;
use smallvec::SmallVec;

use super::{WeightStrategy, math};
use crate::{
    Distribution, FloatDistribution, IntDistribution, ParamValue, ParzenError, SearchSpace,
    TrialId, search_space::ParamId, storage::TrialStorage,
};

enum KernelSet {
    Categorical {
        choices: u32,
        observed: Vec<u32>,
        log_hit: f64,
        log_miss: f64,
    },
    Numeric(NumericKernels),
}

struct NumericKernels {
    distribution: Distribution,
    means: Vec<f64>,
    sigmas: Vec<f64>,
    inverse_sigmas: Vec<f64>,
    log_normalizers: Vec<f64>,
    log_coefficients: Vec<f64>,
    adapted_low: f64,
    adapted_high: f64,
    discrete: bool,
}

pub(crate) struct ProductMixture {
    params: SmallVec<[ParamId; 8]>,
    log_weights: Vec<f64>,
    cumulative_weights: Vec<f64>,
    kernels: Vec<KernelSet>,
    categorical_marginal: Option<CategoricalMarginal>,
}

struct CategoricalMarginal {
    cumulative: Vec<f64>,
    log_probabilities: Vec<f64>,
}

impl ProductMixture {
    pub(crate) fn build(
        params: &[ParamId],
        trials: &[TrialId],
        storage: &TrialStorage,
        space: &SearchSpace,
        prior_weight: f64,
        weights: WeightStrategy,
    ) -> Result<Self, ParzenError> {
        let component_count = trials.len() + 1;
        let mut component_weights: Vec<f64> = (0..trials.len())
            .map(|index| observation_weight(index, trials.len(), weights))
            .collect();
        component_weights.push(prior_weight);
        let total: f64 = component_weights.iter().sum();
        if !total.is_finite() || total <= 0.0 {
            return Err(ParzenError::InternalModel(
                "mixture weights are not finite and positive".into(),
            ));
        }
        for weight in &mut component_weights {
            *weight /= total;
        }
        let log_weights: Vec<f64> = component_weights.iter().map(|weight| weight.ln()).collect();
        let mut cumulative_weights = Vec::with_capacity(component_count);
        let mut cumulative = 0.0;
        for weight in component_weights {
            cumulative += weight;
            cumulative_weights.push(cumulative);
        }
        if let Some(last) = cumulative_weights.last_mut() {
            *last = 1.0;
        }

        let mut kernels = Vec::with_capacity(params.len());
        for param in params {
            let distribution = space.parameters[param.0 as usize].distribution.clone();
            let values: Vec<ParamValue> = trials
                .iter()
                .map(|trial| {
                    storage
                        .typed_value(*trial, *param, &distribution)
                        .ok_or_else(|| {
                            ParzenError::InternalModel(
                                "retained trial is missing an estimator parameter".into(),
                            )
                        })
                })
                .collect::<Result<_, _>>()?;
            kernels.push(match distribution {
                Distribution::Categorical(dist) => {
                    let observed = values
                        .into_iter()
                        .map(|value| {
                            value.as_categorical().ok_or_else(|| {
                                ParzenError::InternalModel("categorical value type mismatch".into())
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let base = prior_weight / component_count as f64;
                    let denominator = 1.0 + base * f64::from(dist.num_choices());
                    KernelSet::Categorical {
                        choices: dist.num_choices(),
                        observed,
                        log_hit: ((1.0 + base) / denominator).ln(),
                        log_miss: (base / denominator).ln(),
                    }
                }
                Distribution::Float(dist) => {
                    KernelSet::Numeric(NumericKernels::float(dist, values)?)
                }
                Distribution::Int(dist) => KernelSet::Numeric(NumericKernels::int(dist, values)?),
            });
        }
        let categorical_marginal = match kernels.as_slice() {
            [
                KernelSet::Categorical {
                    choices,
                    observed,
                    log_hit,
                    log_miss,
                },
            ] => {
                let mut probabilities = vec![0.0; *choices as usize];
                for component in 0..component_count {
                    let component_weight = log_weights[component].exp();
                    if component == observed.len() {
                        let probability = component_weight / f64::from(*choices);
                        for value in &mut probabilities {
                            *value += probability;
                        }
                    } else {
                        let miss = component_weight * log_miss.exp();
                        for value in &mut probabilities {
                            *value += miss;
                        }
                        probabilities[observed[component] as usize] +=
                            component_weight * (log_hit.exp() - log_miss.exp());
                    }
                }
                let mut cumulative = Vec::with_capacity(probabilities.len());
                let mut total = 0.0;
                for probability in &probabilities {
                    total += probability;
                    cumulative.push(total);
                }
                if let Some(last) = cumulative.last_mut() {
                    *last = 1.0;
                }
                Some(CategoricalMarginal {
                    cumulative,
                    log_probabilities: probabilities.into_iter().map(f64::ln).collect(),
                })
            }
            _ => None,
        };
        Ok(Self {
            params: params.iter().copied().collect(),
            log_weights,
            cumulative_weights,
            kernels,
            categorical_marginal,
        })
    }

    pub(crate) fn sample(
        &self,
        rng: &mut StdRng,
    ) -> Result<SmallVec<[(ParamId, ParamValue); 8]>, ParzenError> {
        if let Some(marginal) = &self.categorical_marginal {
            let draw = rng.random::<f64>();
            let value = marginal
                .cumulative
                .partition_point(|cumulative| *cumulative <= draw)
                .min(marginal.cumulative.len() - 1);
            return Ok(smallvec::smallvec![(
                self.params[0],
                ParamValue::Categorical(value as u32)
            )]);
        }
        let draw = rng.random::<f64>();
        let component = self
            .cumulative_weights
            .partition_point(|cumulative| *cumulative <= draw)
            .min(self.log_weights.len() - 1);
        self.params
            .iter()
            .copied()
            .zip(&self.kernels)
            .map(|(param, kernel)| Ok((param, kernel.sample(component, rng)?)))
            .collect()
    }

    pub(crate) fn log_pdf(
        &self,
        candidate: &[(ParamId, ParamValue)],
        scratch: &mut Vec<f64>,
    ) -> Result<f64, ParzenError> {
        if let Some(marginal) = &self.categorical_marginal {
            let value = candidate
                .first()
                .and_then(|(_, value)| value.as_categorical())
                .ok_or_else(|| {
                    ParzenError::InternalModel("categorical candidate is missing".into())
                })?;
            return Ok(marginal
                .log_probabilities
                .get(value as usize)
                .copied()
                .unwrap_or(f64::NEG_INFINITY));
        }
        scratch.clear();
        scratch.extend_from_slice(&self.log_weights);
        for (param, kernel) in self.params.iter().zip(&self.kernels) {
            let value = candidate
                .iter()
                .find(|(candidate, _)| candidate == param)
                .map(|(_, value)| *value)
                .ok_or_else(|| {
                    ParzenError::InternalModel("candidate dimension is missing".into())
                })?;
            kernel.add_log_probabilities(value, scratch)?;
        }
        Ok(math::logsumexp(scratch))
    }
}

impl KernelSet {
    fn sample(&self, component: usize, rng: &mut StdRng) -> Result<ParamValue, ParzenError> {
        match self {
            Self::Categorical {
                choices,
                observed,
                log_hit,
                ..
            } => {
                if component == observed.len() {
                    return Ok(ParamValue::Categorical(rng.random_range(0..*choices)));
                }
                let source = observed[component];
                if *choices == 1 || rng.random::<f64>() < log_hit.exp() {
                    return Ok(ParamValue::Categorical(source));
                }
                let other = rng.random_range(0..choices - 1);
                Ok(ParamValue::Categorical(if other >= source {
                    other + 1
                } else {
                    other
                }))
            }
            Self::Numeric(kernels) => kernels.sample(component, rng),
        }
    }

    fn add_log_probabilities(
        &self,
        value: ParamValue,
        scores: &mut [f64],
    ) -> Result<(), ParzenError> {
        match self {
            Self::Categorical {
                choices,
                observed,
                log_hit,
                log_miss,
            } => {
                let value = value.as_categorical().ok_or_else(|| {
                    ParzenError::InternalModel("categorical candidate type mismatch".into())
                })?;
                if value >= *choices {
                    scores.fill(f64::NEG_INFINITY);
                    return Ok(());
                }
                for (component, score) in scores.iter_mut().enumerate() {
                    *score += if component == observed.len() {
                        -f64::from(*choices).ln()
                    } else if value == observed[component] {
                        *log_hit
                    } else {
                        *log_miss
                    };
                }
                Ok(())
            }
            Self::Numeric(kernels) => kernels.add_log_probabilities(value, scores),
        }
    }
}

impl NumericKernels {
    fn float(dist: FloatDistribution, values: Vec<ParamValue>) -> Result<Self, ParzenError> {
        let values = values
            .into_iter()
            .map(|value| {
                value
                    .as_float()
                    .map(|value| dist.transform(value))
                    .ok_or_else(|| ParzenError::InternalModel("float value type mismatch".into()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let (low, high, discrete) = if let Some(step) = dist.step() {
            let highest = dist.grid_value(dist.max_step_index().unwrap_or(0));
            (dist.low() - step / 2.0, highest + step / 2.0, true)
        } else {
            (
                dist.transform(dist.low()),
                dist.transform(dist.high()),
                false,
            )
        };
        Self::build(Distribution::Float(dist), values, low, high, discrete)
    }

    fn int(dist: IntDistribution, values: Vec<ParamValue>) -> Result<Self, ParzenError> {
        let values = values
            .into_iter()
            .map(|value| {
                value
                    .as_int()
                    .map(|value| dist.transform(value))
                    .ok_or_else(|| ParzenError::InternalModel("integer value type mismatch".into()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let half_step = dist.step() as f64 / 2.0;
        let raw_low = dist.low() as f64 - half_step;
        let raw_high = dist.grid_value(dist.max_step_index()) as f64 + half_step;
        let (low, high) = match dist.scale() {
            crate::IntScale::Linear => (raw_low, raw_high),
            crate::IntScale::Log => (raw_low.ln(), raw_high.ln()),
        };
        Self::build(Distribution::Int(dist), values, low, high, true)
    }

    fn build(
        distribution: Distribution,
        mut means: Vec<f64>,
        adapted_low: f64,
        adapted_high: f64,
        discrete: bool,
    ) -> Result<Self, ParzenError> {
        let range = adapted_high - adapted_low;
        if !range.is_finite() || range <= 0.0 {
            return Err(ParzenError::InternalModel(
                "numeric kernel domain is not finite and positive".into(),
            ));
        }
        let mut sigmas = vec![range; means.len()];
        if !means.is_empty() {
            let mut order: Vec<usize> = (0..means.len()).collect();
            order.sort_unstable_by(|left, right| means[*left].total_cmp(&means[*right]));
            let minimum = range / (means.len() + 2).min(100) as f64;
            for (position, index) in order.iter().copied().enumerate() {
                let left = if position == 0 {
                    adapted_low
                } else {
                    means[order[position - 1]]
                };
                let right = if position + 1 == order.len() {
                    adapted_high
                } else {
                    means[order[position + 1]]
                };
                sigmas[index] = (means[index] - left)
                    .abs()
                    .max((right - means[index]).abs())
                    .clamp(minimum, range);
            }
        }
        means.push((adapted_low + adapted_high) * 0.5);
        sigmas.push(range);
        let inverse_sigmas: Vec<f64> = sigmas.iter().map(|sigma| sigma.recip()).collect();
        let log_normalizers = means
            .iter()
            .zip(&inverse_sigmas)
            .map(|(mean, inverse)| {
                math::log_gaussian_mass(
                    (adapted_low - mean) * inverse,
                    (adapted_high - mean) * inverse,
                )
            })
            .collect();
        let log_coefficients = sigmas
            .iter()
            .zip(&log_normalizers)
            .map(|(sigma, normalizer)| -sigma.ln() - 0.5 * TAU.ln() - normalizer)
            .collect();
        Ok(Self {
            distribution,
            means,
            sigmas,
            inverse_sigmas,
            log_normalizers,
            log_coefficients,
            adapted_low,
            adapted_high,
            discrete,
        })
    }

    fn sample(&self, component: usize, rng: &mut StdRng) -> Result<ParamValue, ParzenError> {
        let normal = Normal::new(self.means[component], self.sigmas[component]).map_err(|_| {
            ParzenError::InternalModel("normal kernel parameters are invalid".into())
        })?;
        let mut transformed = self.means[component];
        for _ in 0..64 {
            let candidate = normal.sample(rng);
            if candidate >= self.adapted_low && candidate <= self.adapted_high {
                transformed = candidate;
                break;
            }
        }
        Ok(match self.distribution {
            Distribution::Float(dist) => ParamValue::Float(dist.untransform(transformed)),
            Distribution::Int(dist) => ParamValue::Int(dist.untransform(transformed)),
            Distribution::Categorical(_) => {
                return Err(ParzenError::InternalModel(
                    "categorical distribution in numeric kernel".into(),
                ));
            }
        })
    }

    fn add_log_probabilities(
        &self,
        value: ParamValue,
        scores: &mut [f64],
    ) -> Result<(), ParzenError> {
        if self.discrete {
            let (center, left, right, log_width) = self.cell(value)?;
            for (component, score) in scores.iter_mut().enumerate() {
                let inverse = self.inverse_sigmas[component];
                let standardized_low = (left - self.means[component]) * inverse;
                let standardized_high = (right - self.means[component]) * inverse;
                let standardized_width = log_width + inverse.ln();
                let log_mass = if standardized_low < standardized_high
                    && standardized_width >= -11.512_925_464_970_229
                {
                    math::log_gaussian_mass(standardized_low, standardized_high)
                } else {
                    let z = (center - self.means[component]) * inverse;
                    -0.5 * z * z - 0.5 * TAU.ln() + standardized_width
                };
                *score += log_mass - self.log_normalizers[component];
            }
            return Ok(());
        }
        let transformed = match self.distribution {
            Distribution::Float(dist) => value.as_float().map(|value| dist.transform(value)),
            Distribution::Int(dist) => value.as_int().map(|value| dist.transform(value)),
            Distribution::Categorical(_) => None,
        }
        .ok_or_else(|| ParzenError::InternalModel("numeric candidate type mismatch".into()))?;
        for (component, score) in scores.iter_mut().enumerate() {
            let z = (transformed - self.means[component]) * self.inverse_sigmas[component];
            *score += self.log_coefficients[component] - 0.5 * z * z;
        }
        Ok(())
    }

    fn cell(&self, value: ParamValue) -> Result<(f64, f64, f64, f64), ParzenError> {
        match self.distribution {
            Distribution::Float(dist) => {
                let value = value.as_float().ok_or_else(|| {
                    ParzenError::InternalModel("float candidate type mismatch".into())
                })?;
                let half = dist.step().unwrap_or(0.0) / 2.0;
                Ok((value, value - half, value + half, (2.0 * half).ln()))
            }
            Distribution::Int(dist) => {
                let value = value.as_int().ok_or_else(|| {
                    ParzenError::InternalModel("integer candidate type mismatch".into())
                })? as f64;
                let half = dist.step() as f64 / 2.0;
                Ok(match dist.scale() {
                    crate::IntScale::Linear => {
                        (value, value - half, value + half, (2.0 * half).ln())
                    }
                    crate::IntScale::Log => {
                        let raw_low = value - half;
                        let transformed_width = (2.0 * half / raw_low).ln_1p();
                        (
                            value.ln(),
                            raw_low.ln(),
                            (value + half).ln(),
                            transformed_width.ln(),
                        )
                    }
                })
            }
            Distribution::Categorical(_) => Err(ParzenError::InternalModel(
                "categorical distribution in numeric cell".into(),
            )),
        }
    }
}

fn observation_weight(index: usize, count: usize, strategy: WeightStrategy) -> f64 {
    if strategy == WeightStrategy::Uniform || count < 25 {
        return 1.0;
    }
    let ramp = count - 25;
    if ramp <= 1 || index >= ramp {
        1.0
    } else {
        (1.0 / count as f64) + (index as f64 / (ramp - 1) as f64) * (1.0 - 1.0 / count as f64)
    }
}
