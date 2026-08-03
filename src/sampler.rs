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
mod prepared;
mod vector_math;
mod workspace;

use std::{num::NonZeroUsize, sync::Arc};

use rand::{Rng, SeedableRng, distr::Uniform, rngs::StdRng};
use smallvec::SmallVec;

use self::{
    history::{BoundedHistory, FullHistory, RankKey},
    mixture::{ModelBuildWorkspace, ProductMixture},
    prepared::{PreparedEstimator, PreparedEstimatorKind},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EstimatorKey {
    Param(ParamId),
    Group(GroupId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EstimatorId(usize);

enum EstimatorHistory {
    Bounded(BoundedHistory),
    Full { seen: usize, generation: u64 },
}

struct EstimatorState {
    prepared: PreparedEstimator,
    history: EstimatorHistory,
    cache: Option<ModelCache>,
}

struct EstimatorRegistry {
    states: Vec<EstimatorState>,
    param_to_estimator: Vec<EstimatorId>,
    group_to_estimator: Vec<EstimatorId>,
}

struct ModelCache {
    generation: u64,
    good: ProductMixture,
    bad: ProductMixture,
    workspace: ModelBuildWorkspace,
    discrete_scores: Vec<(i64, f64, f64)>,
}

/// Seeded Tree-structured Parzen Estimator sampler.
pub struct TpeSampler {
    rng: StdRng,
    config: TpeSamplerConfig,
    registry: Option<EstimatorRegistry>,
    full_history: Option<FullHistory>,
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
            registry: None,
            full_history: None,
            workspace: AcquisitionWorkspace::new(max_good, max_bad),
        })
    }

    pub(crate) const fn model_strategy(&self) -> ModelStrategy {
        self.config.model
    }

    pub(crate) fn initialize(&mut self, space: &SearchSpace) -> Result<(), ParzenError> {
        let definitions = estimator_definitions(self.config.model, space);
        let mut param_to_estimator = vec![EstimatorId(usize::MAX); space.parameters.len()];
        let mut group_to_estimator = vec![EstimatorId(usize::MAX); space.groups.len()];
        let mut states = Vec::with_capacity(definitions.len());
        for (index, (key, params)) in definitions.into_iter().enumerate() {
            let id = EstimatorId(index);
            match key {
                EstimatorKey::Param(param) => param_to_estimator[param.0 as usize] = id,
                EstimatorKey::Group(group) => {
                    group_to_estimator[group.0 as usize] = id;
                    for param in &params {
                        param_to_estimator[param.0 as usize] = id;
                    }
                }
            }
            let prepared = PreparedEstimator::new(key, &params, space)?;
            debug_assert_eq!(prepared.key, key);
            let history = match self.config.history {
                HistoryPolicy::Full => EstimatorHistory::Full {
                    seen: 0,
                    generation: 0,
                },
                HistoryPolicy::Bounded {
                    max_good_trials,
                    max_bad_trials,
                    recent_bad_trials,
                } => EstimatorHistory::Bounded(BoundedHistory::new(
                    max_good_trials.get(),
                    max_bad_trials.get(),
                    recent_bad_trials,
                    self.config.seed ^ index as u64,
                )),
            };
            states.push(EstimatorState {
                prepared,
                history,
                cache: None,
            });
        }
        if param_to_estimator
            .iter()
            .any(|estimator| estimator.0 == usize::MAX)
            || group_to_estimator
                .iter()
                .any(|estimator| estimator.0 == usize::MAX)
        {
            return Err(ParzenError::InternalModel(
                "estimator registry mapping is incomplete".into(),
            ));
        }
        self.full_history = if matches!(self.config.history, HistoryPolicy::Full) {
            Some(FullHistory::new(space)?)
        } else {
            None
        };
        self.registry = Some(EstimatorRegistry {
            states,
            param_to_estimator,
            group_to_estimator,
        });
        Ok(())
    }

    pub(crate) fn on_trial_added(
        &mut self,
        id: TrialId,
        storage: &TrialStorage,
        direction: Direction,
    ) {
        let rank = RankKey::new(id, storage.header(id).value, direction, self.config.seed);
        if let Some(history) = &mut self.full_history {
            history.insert(rank, storage);
        }
        let Some(registry) = &mut self.registry else {
            return;
        };
        for state in &mut registry.states {
            let applicable = state.prepared.params.iter().all(|param| {
                self.full_history.as_ref().map_or_else(
                    || {
                        storage
                            .typed_value(id, param.id, &param.distribution)
                            .is_some()
                    },
                    |history| history.typed_value(id, param.id).is_some(),
                )
            });
            if !applicable {
                continue;
            }
            match &mut state.history {
                EstimatorHistory::Bounded(history) => history.insert(rank),
                EstimatorHistory::Full { seen, generation } => {
                    *seen += 1;
                    *generation = generation.wrapping_add(1);
                }
            }
        }
    }

    pub(crate) fn sample_param(
        &mut self,
        param: ParamId,
        space: &SearchSpace,
        storage: &TrialStorage,
    ) -> Result<ParamValue, ParzenError> {
        let estimator = self
            .registry
            .as_ref()
            .and_then(|registry| registry.param_to_estimator.get(param.0 as usize))
            .copied()
            .ok_or_else(|| ParzenError::InternalModel("parameter estimator is missing".into()))?;
        let values = self.sample_estimator(estimator, space, storage)?;
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
        let estimator = self
            .registry
            .as_ref()
            .and_then(|registry| registry.group_to_estimator.get(group.0 as usize))
            .copied()
            .ok_or_else(|| ParzenError::InternalModel("group estimator is missing".into()))?;
        self.sample_estimator(estimator, space, storage)
    }

    fn sample_estimator(
        &mut self,
        estimator: EstimatorId,
        space: &SearchSpace,
        storage: &TrialStorage,
    ) -> Result<SmallVec<[(ParamId, ParamValue); 8]>, ParzenError> {
        let state = self.state(estimator)?;
        let generation = state.history.generation();
        if state
            .cache
            .as_ref()
            .is_some_and(|cache| cache.generation == generation)
        {
            return self.acquire(estimator);
        }
        let all_categorical = state.prepared.all_categorical();
        let params = state
            .prepared
            .params
            .iter()
            .map(|param| param.id)
            .collect::<SmallVec<[ParamId; 8]>>();
        let (seen, generation, applicable) = self.applicable_history(estimator, all_categorical)?;
        let flat = all_categorical
            && applicable.first().is_some_and(|first| {
                applicable
                    .iter()
                    .all(|trial| storage.header(*trial).value == storage.header(*first).value)
            });
        if seen < self.config.startup_trials || seen == 0 || (all_categorical && flat) {
            if all_categorical {
                return self.sample_unseen_categorical(&params, &applicable, space, storage);
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
        let bounded = matches!(self.state(estimator)?.history, EstimatorHistory::Bounded(_));
        let (good_trials, bad_trials) = if bounded {
            let registry = self.registry.as_ref().ok_or_else(|| {
                ParzenError::InternalModel("sampler registry is not initialized".into())
            })?;
            match &registry.states[estimator.0].history {
                EstimatorHistory::Bounded(history) => {
                    history.split_into(good_count, &mut self.workspace.history);
                    (
                        self.workspace.history.good_trials.as_slice(),
                        self.workspace.history.bad_trials.as_slice(),
                    )
                }
                EstimatorHistory::Full { .. } => unreachable!(),
            }
        } else {
            let count = good_count.min(applicable.len().saturating_sub(1)).max(1);
            (&applicable[..count], &applicable[count..])
        };

        let registry = self.registry.as_mut().ok_or_else(|| {
            ParzenError::InternalModel("sampler registry is not initialized".into())
        })?;
        let state = &mut registry.states[estimator.0];
        if state.cache.is_none() {
            state.cache = Some(ModelCache {
                generation: u64::MAX,
                good: ProductMixture::empty(&state.prepared)?,
                bad: ProductMixture::empty(&state.prepared)?,
                workspace: ModelBuildWorkspace::default(),
                discrete_scores: Vec::new(),
            });
        }
        let cache = state.cache.as_mut().ok_or_else(|| {
            ParzenError::InternalModel("estimator cache initialization failed".into())
        })?;
        if let Some(history) = &self.full_history {
            cache.good.rebuild(
                good_trials,
                self.config.prior_weight,
                self.config.weights,
                &mut cache.workspace,
                |trial, param| history.typed_value(trial, param.id),
            )?;
            cache.bad.rebuild(
                bad_trials,
                self.config.prior_weight,
                self.config.weights,
                &mut cache.workspace,
                |trial, param| history.typed_value(trial, param.id),
            )?;
        } else {
            cache.good.rebuild(
                good_trials,
                self.config.prior_weight,
                self.config.weights,
                &mut cache.workspace,
                |trial, param| {
                    storage
                        .typed_value(trial, param.id, &param.distribution)
                        .and_then(|value| param.prepare_canonical(value))
                },
            )?;
            cache.bad.rebuild(
                bad_trials,
                self.config.prior_weight,
                self.config.weights,
                &mut cache.workspace,
                |trial, param| {
                    storage
                        .typed_value(trial, param.id, &param.distribution)
                        .and_then(|value| param.prepare_canonical(value))
                },
            )?;
        }
        cache.generation = generation;
        cache.discrete_scores.clear();
        self.acquire(estimator)
    }

    fn acquire(
        &mut self,
        estimator: EstimatorId,
    ) -> Result<SmallVec<[(ParamId, ParamValue); 8]>, ParzenError> {
        let registry = self
            .registry
            .as_mut()
            .ok_or_else(|| ParzenError::InternalModel("sampler is not initialized".into()))?;
        let continuous = matches!(
            registry.states[estimator.0].prepared.kind,
            PreparedEstimatorKind::Continuous1 | PreparedEstimatorKind::ContinuousGroup
        );
        let vectorized = continuous
            && should_vectorize_continuous_acquisition(
                registry.states.iter().map(|state| &state.prepared.kind),
            );
        let cache = &mut registry.states[estimator.0]
            .cache
            .as_mut()
            .ok_or_else(|| ParzenError::InternalModel("estimator cache is missing".into()))?;
        let candidates = self.config.ei_candidates.get();
        let dimensions = cache.good.params_len();
        self.workspace.candidates.clear(candidates, dimensions);
        for _ in 0..candidates {
            cache
                .good
                .sample_values(&mut self.rng, &mut self.workspace.candidates.values)?;
        }
        if continuous {
            debug_assert!(cache.good.is_all_continuous() && cache.bad.is_all_continuous());
            for candidate in self.workspace.candidates.values.chunks_exact(dimensions) {
                cache.good.append_continuous_values(
                    candidate,
                    &mut self.workspace.candidates.transformed_values,
                )?;
            }
            cache.good.log_pdf_continuous_batch(
                &self.workspace.candidates.transformed_values,
                candidates,
                vectorized,
                &mut self.workspace.candidates.good_scores,
                &mut self.workspace.good_components,
                &mut self.workspace.candidates.component_scores,
            )?;
            cache.bad.log_pdf_continuous_batch(
                &self.workspace.candidates.transformed_values,
                candidates,
                vectorized,
                &mut self.workspace.candidates.bad_scores,
                &mut self.workspace.bad_components,
                &mut self.workspace.candidates.component_scores,
            )?;
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
            if continuous {
                let score = self.workspace.candidates.good_scores[index]
                    - self.workspace.candidates.bad_scores[index];
                if score.is_finite() && score > best_score {
                    best_score = score;
                    best = Some(index);
                }
                continue;
            }
            let integer = match candidate {
                [ParamValue::Int(value)] => Some(*value),
                _ => None,
            };
            let persisted = integer.and_then(|value| {
                cache
                    .discrete_scores
                    .iter()
                    .find(|(cached, _, _)| *cached == value)
                    .map(|(_, good, bad)| (*good, *bad))
            });
            let duplicate =
                integer.and_then(|_| self.workspace.candidates.previous_duplicate(index));
            let (good, bad) = if let Some(scores) = persisted {
                scores
            } else if let Some(previous) = duplicate {
                (
                    self.workspace.candidates.good_scores[previous],
                    self.workspace.candidates.bad_scores[previous],
                )
            } else {
                (
                    cache
                        .good
                        .log_pdf_positional(candidate, &mut self.workspace.good_components)?,
                    cache
                        .bad
                        .log_pdf_positional(candidate, &mut self.workspace.bad_components)?,
                )
            };
            if let Some(value) = integer
                && persisted.is_none()
                && duplicate.is_none()
                && cache.discrete_scores.len() < 4_096
            {
                cache.discrete_scores.push((value, good, bad));
            }
            self.workspace.candidates.good_scores[index] = good;
            self.workspace.candidates.bad_scores[index] = bad;
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

    fn state(&self, estimator: EstimatorId) -> Result<&EstimatorState, ParzenError> {
        self.registry
            .as_ref()
            .and_then(|registry| registry.states.get(estimator.0))
            .ok_or_else(|| ParzenError::InternalModel("estimator is not initialized".into()))
    }

    fn applicable_history(
        &self,
        estimator: EstimatorId,
        materialize_bounded: bool,
    ) -> Result<(usize, u64, Vec<TrialId>), ParzenError> {
        let state = self.state(estimator)?;
        match &state.history {
            EstimatorHistory::Bounded(history) => {
                let retained = if materialize_bounded {
                    history.retained_trials().collect()
                } else {
                    Vec::new()
                };
                let seen = if history.is_empty() {
                    0
                } else {
                    history.seen()
                };
                Ok((seen, history.generation(), retained))
            }
            EstimatorHistory::Full { seen, generation } => {
                let history = self.full_history.as_ref().ok_or_else(|| {
                    ParzenError::InternalModel("full history is not initialized".into())
                })?;
                let applicable: Vec<TrialId> = history
                    .iter()
                    .filter(|trial| {
                        state
                            .prepared
                            .params
                            .iter()
                            .all(|param| history.typed_value(*trial, param.id).is_some())
                    })
                    .collect();
                debug_assert_eq!(applicable.len(), *seen);
                Ok((*seen, *generation, applicable))
            }
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
        if let Some(history) = &self.full_history {
            return history.len();
        }
        self.registry.as_ref().map_or(0, |registry| {
            registry
                .states
                .iter()
                .map(|state| state.history.retained())
                .sum()
        })
    }
}

fn should_vectorize_continuous_acquisition<'a>(
    kinds: impl ExactSizeIterator<Item = &'a PreparedEstimatorKind>,
) -> bool {
    kinds.len() >= 4
        && kinds
            .into_iter()
            .all(|kind| *kind == PreparedEstimatorKind::Continuous1)
}

impl EstimatorHistory {
    fn generation(&self) -> u64 {
        match self {
            Self::Bounded(history) => history.generation(),
            Self::Full { generation, .. } => *generation,
        }
    }

    fn retained(&self) -> usize {
        match self {
            Self::Bounded(history) => history.retained(),
            Self::Full { seen, .. } => *seen,
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
    use crate::{CategoricalDistribution, Condition, FloatDistribution, IntDistribution};

    #[test]
    fn gamma_strategies_match_named_formulas() {
        let sampler = TpeSampler::new(TpeSamplerConfig::performance(1)).unwrap();
        assert_eq!(sampler.good_count(100).unwrap(), 10);
        assert_eq!(((100_f64.sqrt() * 0.25).ceil() as usize).min(25), 3);
    }

    #[test]
    fn estimator_registry_uses_parameter_then_group_order() {
        let mut space = SearchSpace::new();
        let independent = space
            .add(
                "independent",
                Distribution::Float(FloatDistribution::linear(-1.0, 1.0).unwrap()),
            )
            .unwrap();
        let grouped_left = space
            .add(
                "grouped-left",
                Distribution::Float(FloatDistribution::linear(-1.0, 1.0).unwrap()),
            )
            .unwrap();
        let grouped_right = space
            .add(
                "grouped-right",
                Distribution::Int(IntDistribution::linear(0, 8).unwrap()),
            )
            .unwrap();
        let group = space.add_group([grouped_right, grouped_left]).unwrap();
        let mut sampler = TpeSampler::new(
            TpeSamplerConfig::performance(7).model(ModelStrategy::Grouped { max_group_size: 8 }),
        )
        .unwrap();
        sampler.initialize(&space).unwrap();

        let registry = sampler.registry.as_ref().unwrap();
        assert_eq!(registry.states.len(), 2);
        assert_eq!(
            registry.states[0].prepared.key,
            EstimatorKey::Param(independent)
        );
        assert_eq!(registry.states[1].prepared.key, EstimatorKey::Group(group));
        assert_eq!(
            registry.param_to_estimator[independent.0 as usize],
            EstimatorId(0)
        );
        assert_eq!(
            registry.param_to_estimator[grouped_left.0 as usize],
            EstimatorId(1)
        );
        assert_eq!(
            registry.param_to_estimator[grouped_right.0 as usize],
            EstimatorId(1)
        );
        assert_eq!(
            registry.group_to_estimator[group.0 as usize],
            EstimatorId(1)
        );
        assert_eq!(registry.states[1].prepared.params[0].id, grouped_left);
        assert_eq!(registry.states[1].prepared.params[1].id, grouped_right);
    }

    #[test]
    fn inactive_estimator_generation_is_unchanged() {
        let mut space = SearchSpace::new();
        let parent = space
            .add(
                "kind",
                Distribution::Categorical(CategoricalDistribution::new(2).unwrap()),
            )
            .unwrap();
        let child = space
            .add(
                "depth",
                Distribution::Int(IntDistribution::linear(1, 5).unwrap()),
            )
            .unwrap();
        space
            .add_condition(
                child,
                Condition::CategoricalIn {
                    parent,
                    choices: vec![1].into_boxed_slice(),
                },
            )
            .unwrap();
        let mut sampler = TpeSampler::new(TpeSamplerConfig::performance(11)).unwrap();
        sampler.initialize(&space).unwrap();
        let mut storage = TrialStorage::default();
        let trial = storage
            .push(&[(parent, ParamValue::Categorical(0))], 1.0)
            .unwrap();
        sampler.on_trial_added(trial, &storage, Direction::Minimize);

        let registry = sampler.registry.as_ref().unwrap();
        assert_eq!(registry.states[0].history.generation(), 1);
        assert_eq!(registry.states[1].history.generation(), 0);
        assert_eq!(registry.states[0].history.retained(), 1);
        assert_eq!(registry.states[1].history.retained(), 0);
    }

    #[test]
    fn vectorized_acquisition_is_limited_to_proven_independent_envelope() {
        use PreparedEstimatorKind::{Categorical1, Continuous1, ContinuousGroup};

        assert!(should_vectorize_continuous_acquisition(
            [Continuous1; 4].iter()
        ));
        assert!(should_vectorize_continuous_acquisition(
            [Continuous1; 16].iter()
        ));
        assert!(!should_vectorize_continuous_acquisition(
            [Continuous1].iter()
        ));
        assert!(!should_vectorize_continuous_acquisition(
            [ContinuousGroup].iter()
        ));
        assert!(!should_vectorize_continuous_acquisition(
            [Continuous1, Categorical1, Continuous1, Continuous1].iter()
        ));
    }
}
