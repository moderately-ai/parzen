// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Study orchestration and typed suggestion API.

use smallvec::SmallVec;

use crate::{
    Condition, Direction, Distribution, ModelStrategy, ParamValue, ParzenError, SearchSpace,
    TpeSampler, TrialId, TrialInput, TrialRef, Trials, search_space::ParamId,
    storage::TrialStorage,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Activation {
    Active,
    Inactive,
    Unresolved,
}

/// A sequential Bayesian optimization study.
pub struct Study {
    direction: Direction,
    sampler: TpeSampler,
    space: SearchSpace,
    storage: TrialStorage,
    pending: SmallVec<[(ParamId, ParamValue); 8]>,
    best: Option<TrialId>,
}

impl Study {
    /// Create a study over a non-empty immutable search space.
    pub fn new(
        direction: Direction,
        mut sampler: TpeSampler,
        space: SearchSpace,
    ) -> Result<Self, ParzenError> {
        if space.is_empty() {
            return Err(ParzenError::InvalidConfig(
                "search space must not be empty".into(),
            ));
        }
        if let ModelStrategy::Grouped { max_group_size } = sampler.model_strategy() {
            if space
                .groups
                .iter()
                .any(|group| group.len() > max_group_size)
            {
                return Err(ParzenError::InvalidConfig(
                    "search-space group exceeds sampler group limit".into(),
                ));
            }
        }
        sampler.initialize(&space);
        Ok(Self {
            direction,
            sampler,
            space,
            storage: TrialStorage::default(),
            pending: SmallVec::new(),
            best: None,
        })
    }

    /// Suggest a categorical choice index.
    pub fn suggest_categorical(&mut self, name: &str) -> Result<u32, ParzenError> {
        match self.suggest(name, "categorical", |d| {
            matches!(d, Distribution::Categorical(_))
        })? {
            ParamValue::Categorical(value) => Ok(value),
            _ => Err(ParzenError::ParameterTypeMismatch {
                name: name.into(),
                expected: "categorical",
            }),
        }
    }

    /// Suggest a floating-point value.
    pub fn suggest_float(&mut self, name: &str) -> Result<f64, ParzenError> {
        match self.suggest(name, "a float", |d| matches!(d, Distribution::Float(_)))? {
            ParamValue::Float(value) => Ok(value),
            _ => Err(ParzenError::ParameterTypeMismatch {
                name: name.into(),
                expected: "a float",
            }),
        }
    }

    /// Suggest an integer value.
    pub fn suggest_int(&mut self, name: &str) -> Result<i64, ParzenError> {
        match self.suggest(name, "an integer", |d| matches!(d, Distribution::Int(_)))? {
            ParamValue::Int(value) => Ok(value),
            _ => Err(ParzenError::ParameterTypeMismatch {
                name: name.into(),
                expected: "an integer",
            }),
        }
    }

    fn suggest(
        &mut self,
        name: &str,
        expected: &'static str,
        matches: impl FnOnce(&Distribution) -> bool,
    ) -> Result<ParamValue, ParzenError> {
        let id = self.space.id(name)?;
        let distribution = &self.space.parameters[id.0 as usize].distribution;
        if !matches(distribution) {
            return Err(ParzenError::ParameterTypeMismatch {
                name: name.into(),
                expected,
            });
        }
        if let Some(value) = self.pending_value(id) {
            return Ok(value);
        }
        match self.activation(id) {
            Activation::Inactive => return Err(ParzenError::InactiveParameter(name.into())),
            Activation::Unresolved => return Err(ParzenError::UnresolvedCondition(name.into())),
            Activation::Active => {}
        }
        let group = self.space.parameters[id.0 as usize].group;
        if let (ModelStrategy::Grouped { .. }, Some(group)) = (self.sampler.model_strategy(), group)
        {
            let values = self
                .sampler
                .sample_group(group, &self.space, &self.storage)?;
            for (param, value) in values {
                if self.pending_value(param).is_none() {
                    self.pending.push((param, value));
                }
            }
        } else {
            let value = self.sampler.sample_param(id, &self.space, &self.storage)?;
            self.pending.push((id, value));
        }
        self.pending_value(id)
            .ok_or_else(|| ParzenError::MissingParameter(name.into()))
    }

    /// Complete the pending trial with a finite objective.
    pub fn complete_trial(&mut self, value: f64) -> Result<TrialId, ParzenError> {
        if !value.is_finite() {
            return Err(ParzenError::NonFiniteObjective);
        }
        if self.pending.is_empty() {
            return Err(ParzenError::NoPendingTrial);
        }
        self.validate_complete(&self.pending)?;
        self.pending.sort_unstable_by_key(|(id, _)| *id);
        let id = self
            .storage
            .push(&self.pending, value)
            .ok_or(ParzenError::CapacityOverflow)?;
        self.pending.clear();
        self.update_best(id);
        self.sampler
            .on_trial_added(id, &self.storage, &self.space, self.direction);
        Ok(id)
    }

    /// Clear a pending trial without adding it to history.
    pub fn abort_trial(&mut self) -> bool {
        let existed = !self.pending.is_empty();
        self.pending.clear();
        existed
    }

    /// Validate and inject an externally evaluated completed trial.
    pub fn add_trial(&mut self, input: TrialInput) -> Result<TrialId, ParzenError> {
        if !self.pending.is_empty() {
            return Err(ParzenError::PendingTrial);
        }
        if !input.value.is_finite() {
            return Err(ParzenError::NonFiniteObjective);
        }
        let mut values: SmallVec<[(ParamId, ParamValue); 8]> =
            SmallVec::with_capacity(input.params.len());
        for (name, value) in input.params {
            let id = self.space.id(&name)?;
            if values.iter().any(|(existing, _)| *existing == id) {
                return Err(ParzenError::DuplicateParameter(name));
            }
            let Some(value) = self.space.parameters[id.0 as usize]
                .distribution
                .canonicalize(value)
            else {
                return Err(ParzenError::ValueOutsideDistribution(name));
            };
            values.push((id, value));
        }
        self.validate_complete(&values)?;
        values.sort_unstable_by_key(|(id, _)| *id);
        let id = self
            .storage
            .push(&values, input.value)
            .ok_or(ParzenError::CapacityOverflow)?;
        self.update_best(id);
        self.sampler
            .on_trial_added(id, &self.storage, &self.space, self.direction);
        Ok(id)
    }

    fn validate_complete(&self, values: &[(ParamId, ParamValue)]) -> Result<(), ParzenError> {
        for (index, def) in self.space.parameters.iter().enumerate() {
            let id = ParamId(index as u32);
            let actual = values
                .iter()
                .find(|(candidate, _)| *candidate == id)
                .map(|(_, value)| *value);
            match activation_for(&self.space, id, values) {
                Activation::Active => {
                    let Some(value) = actual else {
                        return Err(ParzenError::MissingParameter(def.name.to_string()));
                    };
                    if def.distribution.canonicalize(value) != Some(value) {
                        return Err(ParzenError::ValueOutsideDistribution(def.name.to_string()));
                    }
                }
                Activation::Inactive => {
                    if actual.is_some() {
                        return Err(ParzenError::InactiveParameter(def.name.to_string()));
                    }
                }
                Activation::Unresolved => {
                    return Err(ParzenError::MissingParameter(def.name.to_string()));
                }
            }
        }
        Ok(())
    }

    fn activation(&self, id: ParamId) -> Activation {
        activation_for(&self.space, id, &self.pending)
    }
    fn pending_value(&self, id: ParamId) -> Option<ParamValue> {
        self.pending
            .iter()
            .find(|(candidate, _)| *candidate == id)
            .map(|(_, value)| *value)
    }
    fn update_best(&mut self, candidate: TrialId) {
        let better = self.best.is_none_or(|best| {
            let candidate_value = self.storage.header(candidate).value;
            let best_value = self.storage.header(best).value;
            match self.direction {
                Direction::Maximize => candidate_value > best_value,
                Direction::Minimize => candidate_value < best_value,
            }
        });
        if better {
            self.best = Some(candidate);
        }
    }

    /// Best trial in constant time.
    #[must_use]
    pub fn best_trial(&self) -> Option<TrialRef<'_>> {
        self.best.map(|id| TrialRef {
            id,
            storage: &self.storage,
            space: &self.space,
        })
    }
    #[must_use]
    pub fn best_value(&self) -> Option<f64> {
        self.best_trial().map(TrialRef::value)
    }
    #[must_use]
    pub fn trial(&self, id: TrialId) -> Option<TrialRef<'_>> {
        (id.0 < self.storage.len() as u64).then_some(TrialRef {
            id,
            storage: &self.storage,
            space: &self.space,
        })
    }
    #[must_use]
    pub fn trials(&self) -> Trials<'_> {
        Trials {
            storage: &self.storage,
            space: &self.space,
            front: 0,
            back: self.storage.len(),
        }
    }
    #[must_use]
    pub fn num_trials(&self) -> usize {
        self.storage.len()
    }
    #[must_use]
    pub const fn direction(&self) -> Direction {
        self.direction
    }
    #[must_use]
    pub const fn search_space(&self) -> &SearchSpace {
        &self.space
    }

    /// Heap capacity occupied by packed raw trial history.
    ///
    /// This excludes the immutable search space, sampler model caches, and allocator metadata.
    #[must_use]
    pub fn history_capacity_bytes(&self) -> usize {
        self.storage.capacity_bytes()
    }

    /// Number of trial references retained by sampler estimators.
    ///
    /// This is bounded independently of completed-trial storage under
    /// [`HistoryPolicy::Bounded`](crate::HistoryPolicy::Bounded).
    #[must_use]
    pub fn estimator_history_len(&self) -> usize {
        self.sampler.retained_history_len()
    }
}

fn activation_for(
    space: &SearchSpace,
    id: ParamId,
    values: &[(ParamId, ParamValue)],
) -> Activation {
    let def = &space.parameters[id.0 as usize];
    for condition in &def.conditions {
        let Condition::CategoricalIn { parent, choices } = condition;
        let parent_value = values
            .iter()
            .find(|(candidate, _)| candidate == parent)
            .map(|(_, value)| *value);
        match parent_value {
            Some(ParamValue::Categorical(choice)) if choices.contains(&choice) => {}
            Some(ParamValue::Categorical(_)) => return Activation::Inactive,
            Some(_) => return Activation::Inactive,
            None => match activation_for(space, *parent, values) {
                Activation::Inactive => return Activation::Inactive,
                Activation::Active | Activation::Unresolved => return Activation::Unresolved,
            },
        }
    }
    Activation::Active
}
