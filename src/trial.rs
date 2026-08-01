// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Trial values and zero-copy history views.

use std::iter::FusedIterator;

use crate::{SearchSpace, storage::TrialStorage};

/// Optimization direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    Maximize,
    Minimize,
}

/// A typed parameter value.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParamValue {
    Categorical(u32),
    Float(f64),
    Int(i64),
}

impl ParamValue {
    #[must_use]
    pub const fn as_categorical(self) -> Option<u32> {
        if let Self::Categorical(v) = self {
            Some(v)
        } else {
            None
        }
    }
    #[must_use]
    pub const fn as_float(self) -> Option<f64> {
        if let Self::Float(v) = self {
            Some(v)
        } else {
            None
        }
    }
    #[must_use]
    pub const fn as_int(self) -> Option<i64> {
        if let Self::Int(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub(crate) const fn encode(self) -> u64 {
        match self {
            Self::Categorical(v) => v as u64,
            Self::Float(v) => v.to_bits(),
            Self::Int(v) => v as u64,
        }
    }
}

/// Stable identifier assigned to a completed trial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrialId(pub(crate) u64);

impl TrialId {
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Owned input for injecting a completed trial.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrialInput {
    pub params: Vec<(String, ParamValue)>,
    pub value: f64,
}

/// Owned, serializable representation of a completed trial.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrialRecord {
    pub id: TrialId,
    pub params: Vec<(String, ParamValue)>,
    pub value: f64,
}

/// Borrowed, allocation-free view of a completed trial.
#[derive(Clone, Copy)]
pub struct TrialRef<'a> {
    pub(crate) id: TrialId,
    pub(crate) storage: &'a TrialStorage,
    pub(crate) space: &'a SearchSpace,
}

impl std::fmt::Debug for TrialRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrialRef")
            .field("id", &self.id)
            .field("value", &self.value())
            .finish_non_exhaustive()
    }
}

impl<'a> TrialRef<'a> {
    #[must_use]
    pub const fn id(self) -> TrialId {
        self.id
    }
    #[must_use]
    pub fn value(self) -> f64 {
        self.storage.header(self.id).value
    }
    #[must_use]
    pub fn params(self) -> Params<'a> {
        let range = self.storage.range(self.id);
        Params {
            storage: self.storage,
            space: self.space,
            index: range.start,
            end: range.end,
        }
    }
    #[must_use]
    pub fn get(self, name: &str) -> Option<ParamValue> {
        let id = self.space.parameter(name)?.id();
        self.storage.typed_value(
            self.id,
            id,
            &self.space.parameters[id.0 as usize].distribution,
        )
    }
    #[must_use]
    pub fn to_record(self) -> TrialRecord {
        TrialRecord {
            id: self.id,
            params: self
                .params()
                .map(|(name, value)| (name.to_owned(), value))
                .collect(),
            value: self.value(),
        }
    }
}

/// Iterator over one trial's parameter names and values.
pub struct Params<'a> {
    storage: &'a TrialStorage,
    space: &'a SearchSpace,
    index: usize,
    end: usize,
}

impl<'a> Iterator for Params<'a> {
    type Item = (&'a str, ParamValue);
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.end {
            return None;
        }
        let id = self.storage.param_ids[self.index];
        let value = self.storage.decode_at(
            self.index,
            &self.space.parameters[id.0 as usize].distribution,
        );
        self.index += 1;
        Some((&self.space.parameters[id.0 as usize].name, value))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.end - self.index;
        (n, Some(n))
    }
}
impl ExactSizeIterator for Params<'_> {}
impl FusedIterator for Params<'_> {}

/// Exact-size, double-ended iterator over completed trials.
pub struct Trials<'a> {
    pub(crate) storage: &'a TrialStorage,
    pub(crate) space: &'a SearchSpace,
    pub(crate) front: usize,
    pub(crate) back: usize,
}

impl<'a> Iterator for Trials<'a> {
    type Item = TrialRef<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        let id = TrialId(self.front as u64);
        self.front += 1;
        Some(TrialRef {
            id,
            storage: self.storage,
            space: self.space,
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.back - self.front;
        (n, Some(n))
    }
}
impl DoubleEndedIterator for Trials<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front == self.back {
            return None;
        }
        self.back -= 1;
        Some(TrialRef {
            id: TrialId(self.back as u64),
            storage: self.storage,
            space: self.space,
        })
    }
}
impl ExactSizeIterator for Trials<'_> {}
impl FusedIterator for Trials<'_> {}
