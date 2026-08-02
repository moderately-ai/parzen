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
    TrialId, search_space::ParamId,
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
    log_inverse_sigmas: Vec<f64>,
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

#[derive(Default)]
pub(crate) struct ModelBuildWorkspace {
    component_weights: Vec<f64>,
    values: Vec<ParamValue>,
    order: Vec<usize>,
}

struct CategoricalMarginal {
    cumulative: Vec<f64>,
    log_probabilities: Vec<f64>,
}

impl ProductMixture {
    pub(crate) fn params_len(&self) -> usize {
        self.params.len()
    }

    pub(crate) fn empty(params: &[ParamId], space: &SearchSpace) -> Result<Self, ParzenError> {
        let kernels = params
            .iter()
            .map(|param| {
                let distribution = space.parameters[param.0 as usize].distribution.clone();
                match distribution {
                    Distribution::Categorical(dist) => Ok(KernelSet::Categorical {
                        choices: dist.num_choices(),
                        observed: Vec::new(),
                        log_hit: 0.0,
                        log_miss: 0.0,
                    }),
                    Distribution::Float(dist) => {
                        Ok(KernelSet::Numeric(NumericKernels::empty_float(dist)?))
                    }
                    Distribution::Int(dist) => {
                        Ok(KernelSet::Numeric(NumericKernels::empty_int(dist)?))
                    }
                }
            })
            .collect::<Result<Vec<_>, ParzenError>>()?;
        let categorical_marginal = matches!(kernels.as_slice(), [KernelSet::Categorical { .. }])
            .then(|| CategoricalMarginal {
                cumulative: Vec::new(),
                log_probabilities: Vec::new(),
            });
        Ok(Self {
            params: params.iter().copied().collect(),
            log_weights: Vec::new(),
            cumulative_weights: Vec::new(),
            kernels,
            categorical_marginal,
        })
    }

    pub(crate) fn rebuild<F>(
        &mut self,
        trials: &[TrialId],
        space: &SearchSpace,
        prior_weight: f64,
        weights: WeightStrategy,
        workspace: &mut ModelBuildWorkspace,
        mut value_for: F,
    ) -> Result<(), ParzenError>
    where
        F: FnMut(TrialId, ParamId, &Distribution) -> Option<ParamValue>,
    {
        let component_count = trials.len() + 1;
        workspace.component_weights.clear();
        workspace.component_weights.extend(
            (0..trials.len()).map(|index| observation_weight(index, trials.len(), weights)),
        );
        workspace.component_weights.push(prior_weight);
        let total: f64 = workspace.component_weights.iter().sum();
        if !total.is_finite() || total <= 0.0 {
            return Err(ParzenError::InternalModel(
                "mixture weights are not finite and positive".into(),
            ));
        }
        for weight in &mut workspace.component_weights {
            *weight /= total;
        }
        self.log_weights.clear();
        self.log_weights
            .extend(workspace.component_weights.iter().map(|weight| weight.ln()));
        self.cumulative_weights.clear();
        let mut cumulative = 0.0;
        for weight in workspace.component_weights.iter().copied() {
            cumulative += weight;
            self.cumulative_weights.push(cumulative);
        }
        if let Some(last) = self.cumulative_weights.last_mut() {
            *last = 1.0;
        }

        for (param, kernel) in self.params.iter().zip(&mut self.kernels) {
            let definition = &space.parameters[param.0 as usize].distribution;
            workspace.values.clear();
            for trial in trials {
                workspace
                    .values
                    .push(value_for(*trial, *param, definition).ok_or_else(|| {
                        ParzenError::InternalModel(
                            "retained trial is missing an estimator parameter".into(),
                        )
                    })?);
            }
            match (kernel, definition) {
                (
                    KernelSet::Categorical {
                        choices,
                        observed,
                        log_hit,
                        log_miss,
                    },
                    Distribution::Categorical(dist),
                ) => {
                    observed.clear();
                    for value in &workspace.values {
                        observed.push(value.as_categorical().ok_or_else(|| {
                            ParzenError::InternalModel("categorical value type mismatch".into())
                        })?);
                    }
                    *choices = dist.num_choices();
                    let base = prior_weight / component_count as f64;
                    let denominator = 1.0 + base * f64::from(dist.num_choices());
                    *log_hit = ((1.0 + base) / denominator).ln();
                    *log_miss = (base / denominator).ln();
                }
                (KernelSet::Numeric(kernels), Distribution::Float(_))
                | (KernelSet::Numeric(kernels), Distribution::Int(_)) => {
                    kernels.rebuild(&workspace.values, &mut workspace.order)?;
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
                let component_weight = self.log_weights[component].exp();
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
        let component = self
            .cumulative_weights
            .partition_point(|cumulative| *cumulative <= draw)
            .min(self.log_weights.len() - 1);
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
        scratch.extend_from_slice(&self.log_weights);
        for (value, kernel) in candidate.iter().copied().zip(&self.kernels) {
            kernel.add_log_probabilities(value, scratch)?;
        }
        Ok(math::logsumexp(scratch))
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
            .copied()
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
    fn empty_float(dist: FloatDistribution) -> Result<Self, ParzenError> {
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
        Self::empty(Distribution::Float(dist), low, high, discrete)
    }

    fn empty_int(dist: IntDistribution) -> Result<Self, ParzenError> {
        let half_step = dist.step() as f64 / 2.0;
        let raw_low = dist.low() as f64 - half_step;
        let raw_high = dist.grid_value(dist.max_step_index()) as f64 + half_step;
        let (low, high) = match dist.scale() {
            crate::IntScale::Linear => (raw_low, raw_high),
            crate::IntScale::Log => (raw_low.ln(), raw_high.ln()),
        };
        Self::empty(Distribution::Int(dist), low, high, true)
    }

    fn empty(
        distribution: Distribution,
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
        Ok(Self {
            distribution,
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

    fn rebuild(
        &mut self,
        values: &[ParamValue],
        order: &mut Vec<usize>,
    ) -> Result<(), ParzenError> {
        self.means.clear();
        match self.distribution {
            Distribution::Float(dist) => {
                for value in values {
                    self.means.push(
                        value
                            .as_float()
                            .map(|value| dist.transform(value))
                            .ok_or_else(|| {
                                ParzenError::InternalModel("float value type mismatch".into())
                            })?,
                    );
                }
            }
            Distribution::Int(dist) => {
                for value in values {
                    self.means.push(
                        value
                            .as_int()
                            .map(|value| dist.transform(value))
                            .ok_or_else(|| {
                                ParzenError::InternalModel("integer value type mismatch".into())
                            })?,
                    );
                }
            }
            Distribution::Categorical(_) => {
                return Err(ParzenError::InternalModel(
                    "categorical distribution in numeric kernel".into(),
                ));
            }
        }
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
        self.means
            .push((self.adapted_low + self.adapted_high) * 0.5);
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
