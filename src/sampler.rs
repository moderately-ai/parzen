// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

#![expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

//! TPE sampler configuration and estimators.

mod history;
mod math;
mod mixture;
mod workspace;

use std::{num::NonZeroUsize, sync::Arc};

use hashbrown::HashMap;
use rand::{Rng, SeedableRng, distr::Uniform, rngs::StdRng};
use smallvec::SmallVec;

use self::{
    history::{BoundedHistory, FullHistory, RankKey},
    mixture::{ModelBuildWorkspace, ProductMixture},
    workspace::AcquisitionWorkspace,
};
use crate::{
    Direction, Distribution, ParamValue, ParzenError, SearchSpace, TrialId,
    search_space::{GroupId, ParamId},
    storage::TrialStorage,
};

/// Strategy mapping applicable observation count to good-trial count.
pub enum GammaStrategy {
    /// `min(ceil(0.1 * n), 25)`.
    Optuna,
    /// `min(ceil(0.25 * sqrt(n)), 25)`.
    Hyperopt,
    /// Caller-provided strategy.
    Custom(Arc<dyn Fn(usize) -> usize + Send + Sync>),
}

impl std::fmt::Debug for GammaStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Optuna => f.write_str("Optuna"),
            Self::Hyperopt => f.write_str("Hyperopt"),
            Self::Custom(_) => f.write_str("Custom(..)"),
        }
    }
}

/// Observation weighting strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightStrategy {
    Uniform,
    Optuna,
}

/// Independent or explicit-group multivariate modeling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelStrategy {
    Independent,
    Grouped { max_group_size: usize },
}

/// Amount of estimator history retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryPolicy {
    /// Retain exact full history. Model construction is linear in applicable trials.
    Full,
    /// Retain exact best trials and a bounded representative bad set.
    Bounded {
        max_good_trials: NonZeroUsize,
        max_bad_trials: NonZeroUsize,
        recent_bad_trials: usize,
    },
}

/// Validated sampler configuration.
#[derive(Debug)]
pub struct TpeSamplerConfig {
    seed: u64,
    startup_trials: usize,
    ei_candidates: NonZeroUsize,
    prior_weight: f64,
    gamma: GammaStrategy,
    weights: WeightStrategy,
    model: ModelStrategy,
    history: HistoryPolicy,
}

impl TpeSamplerConfig {
    /// Fast defaults with strictly bounded estimator state.
    #[must_use]
    pub fn performance(seed: u64) -> Self {
        Self {
            seed,
            startup_trials: 10,
            ei_candidates: NonZeroUsize::new(24).unwrap_or(NonZeroUsize::MIN),
            prior_weight: 1.0,
            gamma: GammaStrategy::Optuna,
            weights: WeightStrategy::Uniform,
            model: ModelStrategy::Independent,
            history: HistoryPolicy::Bounded {
                max_good_trials: NonZeroUsize::new(25).unwrap_or(NonZeroUsize::MIN),
                max_bad_trials: NonZeroUsize::new(512).unwrap_or(NonZeroUsize::MIN),
                recent_bad_trials: 64,
            },
        }
    }

    /// Full-history Optuna-style gamma, weights, and mixture formulation.
    ///
    /// This does not promise suggestion-sequence identity with Optuna.
    #[must_use]
    pub fn optuna_compatible(seed: u64) -> Self {
        Self {
            weights: WeightStrategy::Optuna,
            history: HistoryPolicy::Full,
            ..Self::performance(seed)
        }
    }
    #[must_use]
    pub const fn startup_trials(mut self, value: usize) -> Self {
        self.startup_trials = value;
        self
    }
    #[must_use]
    pub const fn ei_candidates(mut self, value: NonZeroUsize) -> Self {
        self.ei_candidates = value;
        self
    }
    #[must_use]
    pub const fn prior_weight(mut self, value: f64) -> Self {
        self.prior_weight = value;
        self
    }
    #[must_use]
    pub fn gamma(mut self, value: GammaStrategy) -> Self {
        self.gamma = value;
        self
    }
    #[must_use]
    pub const fn weights(mut self, value: WeightStrategy) -> Self {
        self.weights = value;
        self
    }
    #[must_use]
    pub const fn model(mut self, value: ModelStrategy) -> Self {
        self.model = value;
        self
    }
    #[must_use]
    pub const fn history(mut self, value: HistoryPolicy) -> Self {
        self.history = value;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum EstimatorKey {
    Param(ParamId),
    Group(GroupId),
}

struct Tracker {
    key: EstimatorKey,
    params: SmallVec<[ParamId; 8]>,
    history: BoundedHistory,
}

enum Histories {
    Uninitialized,
    Bounded(Vec<Tracker>),
    Full(FullHistory),
}

struct ModelCache {
    generation: u64,
    good: ProductMixture,
    bad: ProductMixture,
    workspace: ModelBuildWorkspace,
}

/// Seeded Tree-structured Parzen Estimator sampler.
pub struct TpeSampler {
    rng: StdRng,
    config: TpeSamplerConfig,
    histories: Histories,
    caches: HashMap<EstimatorKey, ModelCache>,
    workspace: AcquisitionWorkspace,
}

impl TpeSampler {
    /// Validate configuration and create a sampler.
    pub fn new(config: TpeSamplerConfig) -> Result<Self, ParzenError> {
        if !config.prior_weight.is_finite() || config.prior_weight <= 0.0 {
            return Err(ParzenError::InvalidConfig(
                "prior weight must be finite and positive".into(),
            ));
        }
        if let ModelStrategy::Grouped { max_group_size } = config.model
            && !(2..=8).contains(&max_group_size)
        {
            return Err(ParzenError::InvalidConfig(
                "maximum group size must be between two and eight".into(),
            ));
        }
        if let HistoryPolicy::Bounded {
            max_good_trials,
            max_bad_trials,
            recent_bad_trials,
        } = config.history
        {
            let required = recent_bad_trials + max_good_trials.get().saturating_sub(1);
            if recent_bad_trials > max_bad_trials.get() || max_bad_trials.get() < required {
                return Err(ParzenError::InvalidConfig(
                    "bounded bad history must fit recent trials and non-good top entries".into(),
                ));
            }
        }
        let (max_good, max_bad) = match config.history {
            HistoryPolicy::Full => (0, 0),
            HistoryPolicy::Bounded {
                max_good_trials,
                max_bad_trials,
                ..
            } => (max_good_trials.get(), max_bad_trials.get()),
        };
        Ok(Self {
            rng: StdRng::seed_from_u64(config.seed),
            config,
            histories: Histories::Uninitialized,
            caches: HashMap::new(),
            workspace: AcquisitionWorkspace::new(max_good, max_bad),
        })
    }

    pub(crate) const fn model_strategy(&self) -> ModelStrategy {
        self.config.model
    }

    pub(crate) fn initialize(&mut self, space: &SearchSpace) {
        self.histories = match self.config.history {
            HistoryPolicy::Full => Histories::Full(FullHistory::default()),
            HistoryPolicy::Bounded {
                max_good_trials,
                max_bad_trials,
                recent_bad_trials,
            } => {
                let definitions = estimator_definitions(self.config.model, space);
                Histories::Bounded(
                    definitions
                        .into_iter()
                        .enumerate()
                        .map(|(index, (key, params))| Tracker {
                            key,
                            params,
                            history: BoundedHistory::new(
                                max_good_trials.get(),
                                max_bad_trials.get(),
                                recent_bad_trials,
                                self.config.seed ^ index as u64,
                            ),
                        })
                        .collect(),
                )
            }
        };
    }

    pub(crate) fn on_trial_added(
        &mut self,
        id: TrialId,
        storage: &TrialStorage,
        space: &SearchSpace,
        direction: Direction,
    ) {
        let rank = RankKey::new(id, storage.header(id).value, direction, self.config.seed);
        match &mut self.histories {
            Histories::Full(history) => history.insert(rank),
            Histories::Bounded(trackers) => {
                for tracker in trackers {
                    if tracker.params.iter().all(|param| {
                        storage
                            .typed_value(
                                id,
                                *param,
                                &space.parameters[param.0 as usize].distribution,
                            )
                            .is_some()
                    }) {
                        tracker.history.insert(rank);
                    }
                }
            }
            Histories::Uninitialized => {}
        }
    }

    pub(crate) fn sample_param(
        &mut self,
        param: ParamId,
        space: &SearchSpace,
        storage: &TrialStorage,
    ) -> Result<ParamValue, ParzenError> {
        let values = self.sample_estimator(EstimatorKey::Param(param), &[param], space, storage)?;
        values.first().map(|(_, value)| *value).ok_or_else(|| {
            ParzenError::InternalModel("parameter estimator returned no value".into())
        })
    }

    pub(crate) fn sample_group(
        &mut self,
        group: GroupId,
        space: &SearchSpace,
        storage: &TrialStorage,
    ) -> Result<SmallVec<[(ParamId, ParamValue); 8]>, ParzenError> {
        self.sample_estimator(
            EstimatorKey::Group(group),
            &space.groups[group.0 as usize],
            space,
            storage,
        )
    }

    fn sample_estimator(
        &mut self,
        key: EstimatorKey,
        params: &[ParamId],
        space: &SearchSpace,
        storage: &TrialStorage,
    ) -> Result<SmallVec<[(ParamId, ParamValue); 8]>, ParzenError> {
        let generation = self.generation_for(key)?;
        if self
            .caches
            .get(&key)
            .is_some_and(|cache| cache.generation == generation)
        {
            return self.acquire(key);
        }
        let all_categorical = params.iter().all(|param| {
            matches!(
                space.parameters[param.0 as usize].distribution,
                Distribution::Categorical(_)
            )
        });
        let (seen, generation, applicable) =
            self.applicable_history(key, params, space, storage, all_categorical)?;
        let flat = all_categorical
            && applicable.first().is_some_and(|first| {
                applicable
                    .iter()
                    .all(|trial| storage.header(*trial).value == storage.header(*first).value)
            });
        if seen < self.config.startup_trials || seen == 0 || (all_categorical && flat) {
            if all_categorical {
                return self.sample_unseen_categorical(params, &applicable, space, storage);
            }
            return params
                .iter()
                .map(|param| {
                    Ok((
                        *param,
                        sample_prior(
                            &mut self.rng,
                            &space.parameters[param.0 as usize].distribution,
                        )?,
                    ))
                })
                .collect();
        }

        let good_count = self.good_count(seen)?;
        let (good_trials, bad_trials) = match &self.histories {
            Histories::Bounded(trackers) => {
                let history = &trackers
                    .iter()
                    .find(|tracker| tracker.key == key)
                    .ok_or_else(|| ParzenError::InternalModel("bounded tracker is missing".into()))?
                    .history;
                history.split_into(good_count, &mut self.workspace.history);
                (
                    self.workspace.history.good_trials.as_slice(),
                    self.workspace.history.bad_trials.as_slice(),
                )
            }
            Histories::Full(_) => {
                let count = good_count.min(applicable.len().saturating_sub(1)).max(1);
                (&applicable[..count], &applicable[count..])
            }
            Histories::Uninitialized => {
                return Err(ParzenError::InternalModel(
                    "sampler is not initialized".into(),
                ));
            }
        };

        if !self.caches.contains_key(&key) {
            self.caches.insert(
                key,
                ModelCache {
                    generation: u64::MAX,
                    good: ProductMixture::empty(params, space)?,
                    bad: ProductMixture::empty(params, space)?,
                    workspace: ModelBuildWorkspace::default(),
                },
            );
        }
        let cache = self
            .caches
            .get_mut(&key)
            .ok_or_else(|| ParzenError::InternalModel("model cache insertion failed".into()))?;
        cache.good.rebuild(
            good_trials,
            storage,
            space,
            self.config.prior_weight,
            self.config.weights,
            &mut cache.workspace,
        )?;
        cache.bad.rebuild(
            bad_trials,
            storage,
            space,
            self.config.prior_weight,
            self.config.weights,
            &mut cache.workspace,
        )?;
        cache.generation = generation;
        self.acquire(key)
    }

    fn acquire(
        &mut self,
        key: EstimatorKey,
    ) -> Result<SmallVec<[(ParamId, ParamValue); 8]>, ParzenError> {
        let cache = self
            .caches
            .get(&key)
            .ok_or_else(|| ParzenError::InternalModel("model cache insertion failed".into()))?;
        let candidates = self.config.ei_candidates.get();
        let dimensions = cache.good.params_len();
        self.workspace.candidates.clear(candidates, dimensions);
        for _ in 0..candidates {
            cache
                .good
                .sample_values(&mut self.rng, &mut self.workspace.candidates.values)?;
        }
        let mut best_score = f64::NEG_INFINITY;
        let mut best = None;
        for (index, candidate) in self
            .workspace
            .candidates
            .values
            .chunks_exact(dimensions)
            .enumerate()
        {
            let good = cache
                .good
                .log_pdf_positional(candidate, &mut self.workspace.good_components)?;
            let bad = cache
                .bad
                .log_pdf_positional(candidate, &mut self.workspace.bad_components)?;
            self.workspace.candidates.good_scores.push(good);
            self.workspace.candidates.bad_scores.push(bad);
            let score = good - bad;
            if score.is_finite() && score > best_score {
                best_score = score;
                best = Some(index);
            }
        }
        let best = best.ok_or_else(|| {
            ParzenError::InternalModel("acquisition produced no finite candidate".into())
        })?;
        let start = best * dimensions;
        cache
            .good
            .candidate_from_values(&self.workspace.candidates.values[start..start + dimensions])
    }

    fn generation_for(&self, key: EstimatorKey) -> Result<u64, ParzenError> {
        match &self.histories {
            Histories::Bounded(trackers) => trackers
                .iter()
                .find(|tracker| tracker.key == key)
                .map(|tracker| tracker.history.generation())
                .ok_or_else(|| ParzenError::InternalModel("bounded tracker is missing".into())),
            Histories::Full(history) => Ok(history.generation()),
            Histories::Uninitialized => Err(ParzenError::InternalModel(
                "sampler is not initialized".into(),
            )),
        }
    }

    fn applicable_history(
        &self,
        key: EstimatorKey,
        params: &[ParamId],
        space: &SearchSpace,
        storage: &TrialStorage,
        materialize_bounded: bool,
    ) -> Result<(usize, u64, Vec<TrialId>), ParzenError> {
        match &self.histories {
            Histories::Bounded(trackers) => {
                let tracker = trackers
                    .iter()
                    .find(|tracker| tracker.key == key)
                    .ok_or_else(|| {
                        ParzenError::InternalModel("bounded tracker is missing".into())
                    })?;
                let retained = if materialize_bounded {
                    tracker.history.retained_trials().collect()
                } else {
                    Vec::new()
                };
                let seen = if tracker.history.is_empty() {
                    0
                } else {
                    tracker.history.seen()
                };
                Ok((seen, tracker.history.generation(), retained))
            }
            Histories::Full(history) => {
                let applicable: Vec<TrialId> = history
                    .iter()
                    .filter(|trial| {
                        params.iter().all(|param| {
                            storage
                                .typed_value(
                                    *trial,
                                    *param,
                                    &space.parameters[param.0 as usize].distribution,
                                )
                                .is_some()
                        })
                    })
                    .collect();
                Ok((applicable.len(), history.generation(), applicable))
            }
            Histories::Uninitialized => Err(ParzenError::InternalModel(
                "sampler is not initialized".into(),
            )),
        }
    }

    fn good_count(&self, seen: usize) -> Result<usize, ParzenError> {
        if seen <= 1 {
            return Ok(1);
        }
        let requested = match &self.config.gamma {
            GammaStrategy::Optuna => ((seen as f64 * 0.1).ceil() as usize).min(25),
            GammaStrategy::Hyperopt => {
                ((seen as f64).sqrt().mul_add(0.25, 0.0).ceil() as usize).min(25)
            }
            GammaStrategy::Custom(function) => function(seen),
        };
        if let HistoryPolicy::Bounded {
            max_good_trials, ..
        } = self.config.history
            && requested > max_good_trials.get()
        {
            return Err(ParzenError::GammaExceedsHistoryLimit {
                requested,
                limit: max_good_trials.get(),
            });
        }
        Ok(requested.clamp(1, seen - 1))
    }

    fn sample_unseen_categorical(
        &mut self,
        params: &[ParamId],
        trials: &[TrialId],
        space: &SearchSpace,
        storage: &TrialStorage,
    ) -> Result<SmallVec<[(ParamId, ParamValue); 8]>, ParzenError> {
        let counts: SmallVec<[u32; 8]> = params
            .iter()
            .map(
                |param| match space.parameters[param.0 as usize].distribution {
                    Distribution::Categorical(dist) => Ok(dist.num_choices()),
                    _ => Err(ParzenError::InternalModel(
                        "non-categorical parameter in categorical startup".into(),
                    )),
                },
            )
            .collect::<Result<_, _>>()?;
        let product = counts
            .iter()
            .try_fold(1_u64, |total, count| total.checked_mul(u64::from(*count)));
        let Some(product) = product.filter(|product| *product <= 1_000_000) else {
            return params
                .iter()
                .map(|param| {
                    Ok((
                        *param,
                        sample_prior(
                            &mut self.rng,
                            &space.parameters[param.0 as usize].distribution,
                        )?,
                    ))
                })
                .collect();
        };
        let mut seen = hashbrown::HashSet::with_capacity(trials.len());
        for trial in trials {
            let mut code = 0_u64;
            for (param, count) in params.iter().zip(&counts) {
                let value = storage
                    .typed_value(
                        *trial,
                        *param,
                        &space.parameters[param.0 as usize].distribution,
                    )
                    .and_then(ParamValue::as_categorical)
                    .ok_or_else(|| {
                        ParzenError::InternalModel("categorical history is incomplete".into())
                    })?;
                code = code * u64::from(*count) + u64::from(value);
            }
            seen.insert(code);
        }
        let start = if product == 1 {
            0
        } else {
            self.rng.random_range(0..product)
        };
        let code = (0..product)
            .map(|offset| (start + offset) % product)
            .find(|code| !seen.contains(code))
            .unwrap_or(start);
        Ok(decode_categorical(code, params, &counts))
    }

    pub(crate) fn retained_history_len(&self) -> usize {
        match &self.histories {
            Histories::Bounded(trackers) => trackers.iter().map(|t| t.history.retained()).sum(),
            Histories::Full(history) => history.len(),
            Histories::Uninitialized => 0,
        }
    }
}

fn estimator_definitions(
    strategy: ModelStrategy,
    space: &SearchSpace,
) -> Vec<(EstimatorKey, SmallVec<[ParamId; 8]>)> {
    let mut definitions = Vec::new();
    for (index, def) in space.parameters.iter().enumerate() {
        let param = ParamId(index as u32);
        if matches!(strategy, ModelStrategy::Grouped { .. }) && def.group.is_some() {
            continue;
        }
        definitions.push((EstimatorKey::Param(param), smallvec::smallvec![param]));
    }
    if matches!(strategy, ModelStrategy::Grouped { .. }) {
        definitions.extend(space.groups.iter().enumerate().map(|(index, params)| {
            (
                EstimatorKey::Group(GroupId(index as u32)),
                params.iter().copied().collect(),
            )
        }));
    }
    definitions
}

fn sample_prior(rng: &mut StdRng, distribution: &Distribution) -> Result<ParamValue, ParzenError> {
    match distribution {
        Distribution::Categorical(dist) => Ok(ParamValue::Categorical(
            rng.random_range(0..dist.num_choices()),
        )),
        Distribution::Float(dist) => {
            if let Some(max_index) = dist.max_step_index() {
                let index = rng.random_range(0..=max_index);
                return Ok(ParamValue::Float(dist.grid_value(index)));
            }
            let uniform =
                Uniform::new_inclusive(dist.transform(dist.low()), dist.transform(dist.high()))
                    .map_err(|_| {
                        ParzenError::InternalModel("float prior range is invalid".into())
                    })?;
            Ok(ParamValue::Float(dist.untransform(rng.sample(uniform))))
        }
        Distribution::Int(dist) => {
            if dist.low() == dist.high() {
                return Ok(ParamValue::Int(dist.low()));
            }
            if dist.scale() == crate::IntScale::Linear {
                let max_index = dist.max_step_index();
                let index = if max_index == u64::MAX {
                    rng.random::<u64>()
                } else {
                    rng.random_range(0..=max_index)
                };
                return Ok(ParamValue::Int(dist.grid_value(index)));
            }
            let low = (dist.low() as f64 - 0.5).ln();
            let high = (dist.high() as f64 + 0.5).ln();
            let uniform = Uniform::new_inclusive(low, high).map_err(|_| {
                ParzenError::InternalModel("log-integer prior range is invalid".into())
            })?;
            Ok(ParamValue::Int(dist.untransform(rng.sample(uniform))))
        }
    }
}

fn decode_categorical(
    mut code: u64,
    params: &[ParamId],
    counts: &[u32],
) -> SmallVec<[(ParamId, ParamValue); 8]> {
    let mut values = smallvec::smallvec![(ParamId(0), ParamValue::Categorical(0)); params.len()];
    for index in (0..params.len()).rev() {
        let choices = u64::from(counts[index]);
        values[index] = (
            params[index],
            ParamValue::Categorical((code % choices) as u32),
        );
        code /= choices;
    }
    values
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamma_strategies_match_named_formulas() {
        let sampler = TpeSampler::new(TpeSamplerConfig::performance(1)).unwrap();
        assert_eq!(sampler.good_count(100).unwrap(), 10);
        assert_eq!(((100_f64.sqrt() * 0.25).ceil() as usize).min(25), 3);
    }
}
