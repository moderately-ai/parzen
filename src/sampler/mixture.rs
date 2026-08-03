// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::f64::consts::TAU;

use rand::{Rng, distr::Distribution as _, rngs::StdRng};
use rand_distr::Normal;
use smallvec::SmallVec;

use super::{
    WeightStrategy, math,
    prepared::{PreparedEstimator, PreparedKernel, PreparedParam, PreparedValue},
};
use crate::{Distribution, ParamId, ParamValue, ParzenError, TrialId};

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
    prior_center: f64,
    means: Vec<f64>,
    sigmas: Vec<f64>,
    inverse_sigmas: Vec<f64>,
    log_inverse_sigmas: Vec<f64>,
    log_normalizers: Vec<f64>,
    log_coefficients: Vec<f64>,
    adapted_low: f64,
    adapted_high: f64,
    discrete: bool,
}

pub(crate) struct ProductMixture {
    params: Box<[PreparedParam]>,
    weights: MixtureWeights,
    kernels: Vec<KernelSet>,
    categorical_marginal: Option<CategoricalMarginal>,
}

enum MixtureWeights {
    Uniform {
        observations: usize,
        observation_probability: f64,
        observation_log_weight: f64,
        prior_log_weight: f64,
        prior_probability: f64,
    },
    General {
        log_weights: Vec<f64>,
        cumulative_weights: Vec<f64>,
    },
}

#[derive(Default)]
pub(crate) struct ModelBuildWorkspace {
    component_weights: Vec<f64>,
    numeric_values: Vec<f64>,
    order: Vec<usize>,
}

struct CategoricalMarginal {
    cumulative: Vec<f64>,
    log_probabilities: Vec<f64>,
}

impl MixtureWeights {
    fn rebuild(
        &mut self,
        observations: usize,
        prior_weight: f64,
        strategy: WeightStrategy,
        component_weights: &mut Vec<f64>,
    ) -> Result<(), ParzenError> {
        if strategy == WeightStrategy::Uniform {
            let total = observations as f64 + prior_weight;
            let observation_probability = total.recip();
            let prior_probability = prior_weight / total;
            *self = Self::Uniform {
                observations,
                observation_probability,
                observation_log_weight: observation_probability.ln(),
                prior_log_weight: prior_probability.ln(),
                prior_probability,
            };
            return Ok(());
        }
        component_weights.clear();
        component_weights.extend(
            (0..observations).map(|index| observation_weight(index, observations, strategy)),
        );
        component_weights.push(prior_weight);
        let total: f64 = component_weights.iter().sum();
        if !total.is_finite() || total <= 0.0 {
            return Err(ParzenError::InternalModel(
                "mixture weights are not finite and positive".into(),
            ));
        }
        for weight in component_weights.iter_mut() {
            *weight /= total;
        }
        if !matches!(self, Self::General { .. }) {
            *self = Self::General {
                log_weights: Vec::new(),
                cumulative_weights: Vec::new(),
            };
        }
        let Self::General {
            log_weights,
            cumulative_weights,
        } = self
        else {
            unreachable!();
        };
        log_weights.clear();
        log_weights.extend(component_weights.iter().map(|weight| weight.ln()));
        cumulative_weights.clear();
        let mut cumulative = 0.0;
        for weight in component_weights.iter().copied() {
            cumulative += weight;
            cumulative_weights.push(cumulative);
        }
        if let Some(last) = cumulative_weights.last_mut() {
            *last = 1.0;
        }
        Ok(())
    }

    fn probability(&self, component: usize) -> f64 {
        match self {
            Self::Uniform {
                observations,
                observation_log_weight,
                prior_log_weight,
                ..
            } => {
                if component == *observations {
                    prior_log_weight.exp()
                } else {
                    observation_log_weight.exp()
                }
            }
            Self::General { log_weights, .. } => log_weights[component].exp(),
        }
    }

    fn sample_component(&self, draw: f64) -> usize {
        match self {
            Self::Uniform {
                observations,
                observation_probability,
                prior_probability,
                ..
            } => {
                if *observations == 0 || draw >= 1.0 - prior_probability {
                    return *observations;
                }
                (draw / observation_probability).floor() as usize
            }
            Self::General {
                log_weights,
                cumulative_weights,
            } => cumulative_weights
                .partition_point(|cumulative| *cumulative <= draw)
                .min(log_weights.len() - 1),
        }
    }

    fn fill_log_weights(&self, output: &mut Vec<f64>) {
        match self {
            Self::Uniform {
                observations,
                observation_log_weight,
                prior_log_weight,
                ..
            } => {
                output.resize(*observations, *observation_log_weight);
                output.push(*prior_log_weight);
            }
            Self::General { log_weights, .. } => output.extend_from_slice(log_weights),
        }
    }
}

impl ProductMixture {
    pub(crate) fn params_len(&self) -> usize {
        self.params.len()
    }

    pub(crate) fn empty(prepared: &PreparedEstimator) -> Result<Self, ParzenError> {
        let kernels = prepared
            .params
            .iter()
            .map(|param| match param.kernel {
                PreparedKernel::Categorical(categorical) => Ok(KernelSet::Categorical {
                    choices: categorical.choices,
                    observed: Vec::new(),
                    log_hit: 0.0,
                    log_miss: 0.0,
                }),
                PreparedKernel::Continuous(continuous) => {
                    Ok(KernelSet::Numeric(NumericKernels::empty(
                        param.distribution.clone(),
                        continuous.transformed_low,
                        continuous.transformed_high,
                        continuous.range,
                        continuous.prior_center,
                        false,
                    )?))
                }
                PreparedKernel::Discrete(discrete) => {
                    Ok(KernelSet::Numeric(NumericKernels::empty(
                        param.distribution.clone(),
                        discrete.adapted_low,
                        discrete.adapted_high,
                        discrete.range,
                        discrete.prior_center,
                        true,
                    )?))
                }
            })
            .collect::<Result<Vec<_>, ParzenError>>()?;
        let categorical_marginal = matches!(kernels.as_slice(), [KernelSet::Categorical { .. }])
            .then(|| CategoricalMarginal {
                cumulative: Vec::new(),
                log_probabilities: Vec::new(),
            });
        Ok(Self {
            params: prepared.params.iter().cloned().collect(),
            weights: MixtureWeights::Uniform {
                observations: 0,
                observation_probability: 0.0,
                observation_log_weight: f64::NEG_INFINITY,
                prior_log_weight: 0.0,
                prior_probability: 1.0,
            },
            kernels,
            categorical_marginal,
        })
    }

    pub(crate) fn rebuild<F>(
        &mut self,
        trials: &[TrialId],
        prior_weight: f64,
        weights: WeightStrategy,
        workspace: &mut ModelBuildWorkspace,
        mut value_for: F,
    ) -> Result<(), ParzenError>
    where
        F: FnMut(TrialId, &PreparedParam) -> Option<PreparedValue>,
    {
        let component_count = trials.len() + 1;
        self.weights.rebuild(
            trials.len(),
            prior_weight,
            weights,
            &mut workspace.component_weights,
        )?;

        for (position, (param, kernel)) in self.params.iter().zip(&mut self.kernels).enumerate() {
            debug_assert_eq!(param.position, position);
            match (kernel, param.kernel) {
                (
                    KernelSet::Categorical {
                        choices,
                        observed,
                        log_hit,
                        log_miss,
                    },
                    PreparedKernel::Categorical(categorical),
                ) => {
                    observed.clear();
                    for trial in trials {
                        let Some(PreparedValue::Categorical(value)) = value_for(*trial, param)
                        else {
                            return Err(ParzenError::InternalModel(
                                "retained categorical trial value is missing or invalid".into(),
                            ));
                        };
                        observed.push(value);
                    }
                    *choices = categorical.choices;
                    let base = prior_weight / component_count as f64;
                    let denominator = 1.0 + base * f64::from(categorical.choices);
                    *log_hit = ((1.0 + base) / denominator).ln();
                    *log_miss = (base / denominator).ln();
                }
                (KernelSet::Numeric(kernels), PreparedKernel::Continuous(_))
                | (KernelSet::Numeric(kernels), PreparedKernel::Discrete(_)) => {
                    workspace.numeric_values.clear();
                    for trial in trials {
                        let transformed = match value_for(*trial, param) {
                            Some(PreparedValue::Continuous(value))
                            | Some(PreparedValue::Discrete {
                                transformed: value, ..
                            }) => value,
                            _ => {
                                return Err(ParzenError::InternalModel(
                                    "retained numeric trial value is missing or invalid".into(),
                                ));
                            }
                        };
                        workspace.numeric_values.push(transformed);
                    }
                    kernels.rebuild(&workspace.numeric_values, &mut workspace.order)?;
                }
                _ => {
                    return Err(ParzenError::InternalModel(
                        "prepared estimator distribution mismatch".into(),
                    ));
                }
            }
        }
        if let (
            [
                KernelSet::Categorical {
                    choices,
                    observed,
                    log_hit,
                    log_miss,
                },
            ],
            Some(marginal),
        ) = (self.kernels.as_slice(), &mut self.categorical_marginal)
        {
            marginal.log_probabilities.clear();
            marginal.log_probabilities.resize(*choices as usize, 0.0);
            for component in 0..component_count {
                let component_weight = self.weights.probability(component);
                if component == observed.len() {
                    let probability = component_weight / f64::from(*choices);
                    for value in &mut marginal.log_probabilities {
                        *value += probability;
                    }
                } else {
                    let miss = component_weight * log_miss.exp();
                    for value in &mut marginal.log_probabilities {
                        *value += miss;
                    }
                    marginal.log_probabilities[observed[component] as usize] +=
                        component_weight * (log_hit.exp() - log_miss.exp());
                }
            }
            marginal.cumulative.clear();
            let mut total = 0.0;
            for probability in &marginal.log_probabilities {
                total += probability;
                marginal.cumulative.push(total);
            }
            if let Some(last) = marginal.cumulative.last_mut() {
                *last = 1.0;
            }
            for probability in &mut marginal.log_probabilities {
                *probability = probability.ln();
            }
        }
        Ok(())
    }

    pub(crate) fn sample_values(
        &self,
        rng: &mut StdRng,
        values: &mut Vec<ParamValue>,
    ) -> Result<(), ParzenError> {
        if let Some(marginal) = &self.categorical_marginal {
            let draw = rng.random::<f64>();
            let value = marginal
                .cumulative
                .partition_point(|cumulative| *cumulative <= draw)
                .min(marginal.cumulative.len() - 1);
            values.push(ParamValue::Categorical(value as u32));
            return Ok(());
        }
        let draw = rng.random::<f64>();
        let component = self.weights.sample_component(draw);
        for kernel in &self.kernels {
            values.push(kernel.sample(component, rng)?);
        }
        Ok(())
    }

    pub(crate) fn log_pdf_positional(
        &self,
        candidate: &[ParamValue],
        scratch: &mut Vec<f64>,
    ) -> Result<f64, ParzenError> {
        if let Some(marginal) = &self.categorical_marginal {
            let value = candidate
                .first()
                .and_then(|value| value.as_categorical())
                .ok_or_else(|| {
                    ParzenError::InternalModel("categorical candidate is missing".into())
                })?;
            return Ok(marginal
                .log_probabilities
                .get(value as usize)
                .copied()
                .unwrap_or(f64::NEG_INFINITY));
        }
        if candidate.len() != self.kernels.len() {
            return Err(ParzenError::InternalModel(
                "candidate dimension count does not match estimator".into(),
            ));
        }
        scratch.clear();
        self.weights.fill_log_weights(scratch);
        for (value, kernel) in candidate.iter().copied().zip(&self.kernels) {
            kernel.add_log_probabilities(value, scratch)?;
        }
        Ok(math::logsumexp(scratch))
    }

    pub(crate) fn is_all_continuous(&self) -> bool {
        self.kernels
            .iter()
            .all(|kernel| matches!(kernel, KernelSet::Numeric(kernels) if !kernels.discrete))
    }

    pub(crate) fn append_continuous_values(
        &self,
        candidate: &[ParamValue],
        output: &mut Vec<f64>,
    ) -> Result<(), ParzenError> {
        if candidate.len() != self.kernels.len() {
            return Err(ParzenError::InternalModel(
                "candidate dimension count does not match estimator".into(),
            ));
        }
        for (value, kernel) in candidate.iter().copied().zip(&self.kernels) {
            let KernelSet::Numeric(kernels) = kernel else {
                return Err(ParzenError::InternalModel(
                    "continuous batch contains a categorical kernel".into(),
                ));
            };
            output.push(kernels.transform_continuous(value)?);
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn log_pdf_continuous_batch(
        &self,
        values: &[f64],
        candidates: usize,
        output: &mut [f64],
        log_weights: &mut Vec<f64>,
        component_scores: &mut Vec<f64>,
    ) -> Result<(), ParzenError> {
        let mut dimensions = SmallVec::<[super::vector_math::ContinuousDimension<'_>; 8]>::new();
        for kernel in &self.kernels {
            let KernelSet::Numeric(kernels) = kernel else {
                return Err(ParzenError::InternalModel(
                    "continuous batch contains a categorical kernel".into(),
                ));
            };
            if kernels.discrete {
                return Err(ParzenError::InternalModel(
                    "continuous batch contains a discrete kernel".into(),
                ));
            }
            dimensions.push(super::vector_math::ContinuousDimension {
                means: &kernels.means,
                inverse_sigmas: &kernels.inverse_sigmas,
                log_coefficients: &kernels.log_coefficients,
            });
        }
        log_weights.clear();
        self.weights.fill_log_weights(log_weights);
        component_scores.resize(log_weights.len(), 0.0);
        super::vector_math::continuous_log_pdf_batch(
            values,
            candidates,
            &dimensions,
            log_weights,
            output,
            component_scores,
        );
        Ok(())
    }

    pub(crate) fn candidate_from_values(
        &self,
        values: &[ParamValue],
    ) -> Result<SmallVec<[(ParamId, ParamValue); 8]>, ParzenError> {
        if values.len() != self.params.len() {
            return Err(ParzenError::InternalModel(
                "selected candidate dimension count does not match estimator".into(),
            ));
        }
        Ok(self
            .params
            .iter()
            .map(|param| param.id)
            .zip(values.iter().copied())
            .collect())
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
    fn empty(
        distribution: Distribution,
        adapted_low: f64,
        adapted_high: f64,
        range: f64,
        prior_center: f64,
        discrete: bool,
    ) -> Result<Self, ParzenError> {
        if !range.is_finite()
            || range <= 0.0
            || (adapted_high - adapted_low - range).abs() > f64::EPSILON * range.abs()
            || !prior_center.is_finite()
        {
            return Err(ParzenError::InternalModel(
                "numeric kernel domain is not finite and positive".into(),
            ));
        }
        Ok(Self {
            distribution,
            prior_center,
            means: Vec::new(),
            sigmas: Vec::new(),
            inverse_sigmas: Vec::new(),
            log_inverse_sigmas: Vec::new(),
            log_normalizers: Vec::new(),
            log_coefficients: Vec::new(),
            adapted_low,
            adapted_high,
            discrete,
        })
    }

    fn rebuild(&mut self, values: &[f64], order: &mut Vec<usize>) -> Result<(), ParzenError> {
        self.means.clear();
        self.means.extend_from_slice(values);
        let range = self.adapted_high - self.adapted_low;
        self.sigmas.clear();
        self.sigmas.resize(self.means.len(), range);
        if !self.means.is_empty() {
            order.clear();
            order.extend(0..self.means.len());
            order.sort_unstable_by(|left, right| self.means[*left].total_cmp(&self.means[*right]));
            let minimum = range / (self.means.len() + 2).min(100) as f64;
            for (position, index) in order.iter().copied().enumerate() {
                let left = if position == 0 {
                    self.adapted_low
                } else {
                    self.means[order[position - 1]]
                };
                let right = if position + 1 == order.len() {
                    self.adapted_high
                } else {
                    self.means[order[position + 1]]
                };
                self.sigmas[index] = (self.means[index] - left)
                    .abs()
                    .max((right - self.means[index]).abs())
                    .clamp(minimum, range);
            }
        }
        self.means.push(self.prior_center);
        self.sigmas.push(range);
        self.inverse_sigmas.clear();
        self.inverse_sigmas
            .extend(self.sigmas.iter().map(|sigma| sigma.recip()));
        self.log_inverse_sigmas.clear();
        self.log_inverse_sigmas
            .extend(self.inverse_sigmas.iter().map(|inverse| inverse.ln()));
        self.log_normalizers.clear();
        self.log_normalizers
            .extend(
                self.means
                    .iter()
                    .zip(&self.inverse_sigmas)
                    .map(|(mean, inverse)| {
                        math::log_gaussian_mass(
                            (self.adapted_low - mean) * inverse,
                            (self.adapted_high - mean) * inverse,
                        )
                    }),
            );
        self.log_coefficients.clear();
        self.log_coefficients.extend(
            self.sigmas
                .iter()
                .zip(&self.log_normalizers)
                .map(|(sigma, normalizer)| -sigma.ln() - 0.5 * TAU.ln() - normalizer),
        );
        Ok(())
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
                let standardized_width = log_width + self.log_inverse_sigmas[component];
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
        let transformed = match (&self.distribution, value) {
            (Distribution::Float(dist), ParamValue::Float(value)) => dist.transform(value),
            (Distribution::Int(dist), ParamValue::Int(value)) => dist.transform(value),
            _ => {
                return Err(ParzenError::InternalModel(
                    "numeric candidate type mismatch".into(),
                ));
            }
        };
        for (component, score) in scores.iter_mut().enumerate() {
            let z = (transformed - self.means[component]) * self.inverse_sigmas[component];
            *score += self.log_coefficients[component] - 0.5 * z * z;
        }
        Ok(())
    }

    fn transform_continuous(&self, value: ParamValue) -> Result<f64, ParzenError> {
        if self.discrete {
            return Err(ParzenError::InternalModel(
                "discrete candidate used in continuous batch".into(),
            ));
        }
        match (&self.distribution, value) {
            (Distribution::Float(dist), ParamValue::Float(value)) => Ok(dist.transform(value)),
            (Distribution::Int(dist), ParamValue::Int(value)) => Ok(dist.transform(value)),
            _ => Err(ParzenError::InternalModel(
                "numeric candidate type mismatch".into(),
            )),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        let tolerance = 1e-14_f64.max(expected.abs() * 1e-14);
        assert!(
            (actual - expected).abs() <= tolerance,
            "actual={actual}, expected={expected}"
        );
    }

    #[test]
    fn uniform_weights_match_explicit_normalization() {
        let mut weights = MixtureWeights::Uniform {
            observations: 0,
            observation_probability: 0.0,
            observation_log_weight: f64::NEG_INFINITY,
            prior_log_weight: 0.0,
            prior_probability: 1.0,
        };
        let mut workspace = Vec::new();
        weights
            .rebuild(4, 2.0, WeightStrategy::Uniform, &mut workspace)
            .unwrap();

        for component in 0..4 {
            assert_close(weights.probability(component), 1.0 / 6.0);
        }
        assert_close(weights.probability(4), 2.0 / 6.0);
        assert_eq!(weights.sample_component(0.0), 0);
        assert_eq!(weights.sample_component(1.0 / 6.0), 1);
        assert_eq!(weights.sample_component(2.0 / 3.0), 4);

        let mut logs = Vec::new();
        weights.fill_log_weights(&mut logs);
        assert_eq!(logs.len(), 5);
        assert_close(logsumexp_probabilities(&logs), 1.0);
    }

    #[test]
    fn uniform_component_selection_matches_explicit_cumulative_weights() {
        for (observations, prior_weight) in [(1, 0.25), (4, 1.0), (25, 2.5), (512, 1.0)] {
            let mut weights = MixtureWeights::Uniform {
                observations: 0,
                observation_probability: 0.0,
                observation_log_weight: f64::NEG_INFINITY,
                prior_log_weight: 0.0,
                prior_probability: 1.0,
            };
            let mut workspace = Vec::new();
            weights
                .rebuild(
                    observations,
                    prior_weight,
                    WeightStrategy::Uniform,
                    &mut workspace,
                )
                .unwrap();
            let total = observations as f64 + prior_weight;
            let mut cumulative = Vec::with_capacity(observations + 1);
            let mut sum = 0.0;
            for raw in std::iter::repeat_n(1.0, observations).chain([prior_weight]) {
                sum += raw / total;
                cumulative.push(sum);
            }
            *cumulative.last_mut().unwrap() = 1.0;

            let mut state = 0x9e37_79b9_7f4a_7c15_u64;
            for _ in 0..10_000 {
                state = state.wrapping_add(0x9e37_79b9_7f4a_7c15).rotate_left(17);
                let draw = (state >> 11) as f64 * (1.0 / ((1_u64 << 53) as f64));
                let expected = cumulative
                    .partition_point(|boundary| *boundary <= draw)
                    .min(cumulative.len() - 1);
                assert_eq!(weights.sample_component(draw), expected);
            }
        }
    }

    #[test]
    fn general_weights_reuse_vectors_and_match_optuna_formula() {
        let mut weights = MixtureWeights::Uniform {
            observations: 0,
            observation_probability: 0.0,
            observation_log_weight: f64::NEG_INFINITY,
            prior_log_weight: 0.0,
            prior_probability: 1.0,
        };
        let mut workspace = Vec::new();
        weights
            .rebuild(40, 1.0, WeightStrategy::Optuna, &mut workspace)
            .unwrap();

        let mut logs = Vec::new();
        weights.fill_log_weights(&mut logs);
        assert_eq!(logs.len(), 41);
        assert_close(logsumexp_probabilities(&logs), 1.0);
        assert!(weights.probability(0) < weights.probability(20));
        assert_close(weights.probability(39), weights.probability(40));

        let previous_capacity = match &weights {
            MixtureWeights::General { log_weights, .. } => log_weights.capacity(),
            MixtureWeights::Uniform { .. } => unreachable!(),
        };
        weights
            .rebuild(30, 1.0, WeightStrategy::Optuna, &mut workspace)
            .unwrap();
        let current_capacity = match &weights {
            MixtureWeights::General { log_weights, .. } => log_weights.capacity(),
            MixtureWeights::Uniform { .. } => unreachable!(),
        };
        assert_eq!(current_capacity, previous_capacity);
    }

    fn logsumexp_probabilities(log_weights: &[f64]) -> f64 {
        log_weights.iter().map(|value| value.exp()).sum()
    }
}
