// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Immutable, validated search-space definitions.

use hashbrown::HashMap;

use crate::{Distribution, ParzenError};

/// Stable parameter identifier within one search space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParamId(pub(crate) u32);

/// Stable explicit multivariate-group identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GroupId(pub(crate) u32);

/// A condition controlling whether a parameter is active.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Condition {
    /// Active when the categorical parent has one of `choices`.
    CategoricalIn {
        parent: ParamId,
        choices: Box<[u32]>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ParameterDef {
    pub name: Box<str>,
    pub distribution: Distribution,
    pub conditions: Vec<Condition>,
    pub group: Option<GroupId>,
}

/// Read-only information about a registered parameter.
#[derive(Debug, Clone, Copy)]
pub struct ParameterRef<'a> {
    pub(crate) id: ParamId,
    pub(crate) def: &'a ParameterDef,
}

impl<'a> ParameterRef<'a> {
    #[must_use]
    pub const fn id(self) -> ParamId {
        self.id
    }
    #[must_use]
    pub fn name(self) -> &'a str {
        &self.def.name
    }
    #[must_use]
    pub const fn distribution(self) -> &'a Distribution {
        &self.def.distribution
    }
}

/// A validated collection of parameter distributions, conditions, and groups.
#[derive(Debug, Clone, Default)]
pub struct SearchSpace {
    pub(crate) parameters: Vec<ParameterDef>,
    names: HashMap<Box<str>, ParamId>,
    pub(crate) groups: Vec<Box<[ParamId]>>,
}

impl SearchSpace {
    /// Create an empty search space.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a named parameter.
    pub fn add(
        &mut self,
        name: impl Into<String>,
        distribution: Distribution,
    ) -> Result<ParamId, ParzenError> {
        let name = name.into();
        if name.is_empty() {
            return Err(ParzenError::InvalidDistribution(
                "parameter name must not be empty".into(),
            ));
        }
        if self.names.contains_key(name.as_str()) {
            return Err(ParzenError::DuplicateParameter(name));
        }
        let id = ParamId(
            u32::try_from(self.parameters.len()).map_err(|_| ParzenError::CapacityOverflow)?,
        );
        let boxed: Box<str> = name.into_boxed_str();
        self.names.insert(boxed.clone(), id);
        self.parameters.push(ParameterDef {
            name: boxed,
            distribution,
            conditions: Vec::new(),
            group: None,
        });
        Ok(id)
    }

    /// Add an AND-combined activation condition to a parameter.
    pub fn add_condition(
        &mut self,
        child: ParamId,
        condition: Condition,
    ) -> Result<(), ParzenError> {
        self.def(child)?;
        if self.def(child)?.group.is_some() {
            return Err(ParzenError::InvalidCondition(
                "add conditions before assigning the parameter to a group".into(),
            ));
        }
        let Condition::CategoricalIn { parent, choices } = &condition;
        let parent_def = self.def(*parent)?;
        let Distribution::Categorical(parent_dist) = parent_def.distribution else {
            return Err(ParzenError::InvalidCondition(
                "condition parent must be categorical".into(),
            ));
        };
        if child == *parent || self.depends_on(*parent, child) {
            return Err(ParzenError::InvalidCondition(
                "condition would create a cycle".into(),
            ));
        }
        if choices.is_empty()
            || choices
                .iter()
                .any(|choice| *choice >= parent_dist.num_choices())
        {
            return Err(ParzenError::InvalidCondition(
                "condition contains no choices or an out-of-range choice".into(),
            ));
        }
        if self.def(child)?.conditions.contains(&condition) {
            return Err(ParzenError::InvalidCondition("duplicate condition".into()));
        }
        self.parameters[child.0 as usize].conditions.push(condition);
        Ok(())
    }

    /// Declare an explicit multivariate group of two to eight parameters.
    pub fn add_group(
        &mut self,
        params: impl IntoIterator<Item = ParamId>,
    ) -> Result<GroupId, ParzenError> {
        let mut params: Vec<ParamId> = params.into_iter().collect();
        params.sort_unstable();
        params.dedup();
        if !(2..=8).contains(&params.len()) {
            return Err(ParzenError::InvalidGroup(
                "groups must contain two to eight distinct parameters".into(),
            ));
        }
        let first_conditions = self.def(params[0])?.conditions.clone();
        for id in &params {
            let def = self.def(*id)?;
            if def.group.is_some() {
                return Err(ParzenError::InvalidGroup(format!(
                    "`{}` already belongs to a group",
                    def.name
                )));
            }
            if def.conditions != first_conditions {
                return Err(ParzenError::InvalidGroup(
                    "group members must have identical activation conditions".into(),
                ));
            }
        }
        let id =
            GroupId(u32::try_from(self.groups.len()).map_err(|_| ParzenError::CapacityOverflow)?);
        for param in &params {
            self.parameters[param.0 as usize].group = Some(id);
        }
        self.groups.push(params.into_boxed_slice());
        Ok(id)
    }

    /// Look up a parameter by name.
    #[must_use]
    pub fn parameter(&self, name: &str) -> Option<ParameterRef<'_>> {
        self.names.get(name).map(|id| ParameterRef {
            id: *id,
            def: &self.parameters[id.0 as usize],
        })
    }

    /// Number of registered parameters.
    #[must_use]
    pub fn len(&self) -> usize {
        self.parameters.len()
    }
    /// Whether no parameters are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.parameters.is_empty()
    }

    pub(crate) fn def(&self, id: ParamId) -> Result<&ParameterDef, ParzenError> {
        self.parameters
            .get(id.0 as usize)
            .ok_or_else(|| ParzenError::UnknownParameter(format!("#{}", id.0)))
    }

    pub(crate) fn id(&self, name: &str) -> Result<ParamId, ParzenError> {
        self.names
            .get(name)
            .copied()
            .ok_or_else(|| ParzenError::UnknownParameter(name.into()))
    }

    fn depends_on(&self, start: ParamId, target: ParamId) -> bool {
        let Some(def) = self.parameters.get(start.0 as usize) else {
            return false;
        };
        def.conditions.iter().any(|condition| {
            let Condition::CategoricalIn { parent, .. } = condition;
            *parent == target || self.depends_on(*parent, target)
        })
    }
}
