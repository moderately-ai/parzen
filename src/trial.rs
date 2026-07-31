// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Trial types for Bayesian optimization.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The direction to optimize.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// Higher objective values are better.
    Maximize,
    /// Lower objective values are better.
    Minimize,
}

/// A parameter value in a trial.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParamValue {
    /// An index into a choices array.
    Categorical(u32),
}

impl ParamValue {
    /// Extract the categorical index, if this is a categorical value.
    #[must_use]
    pub const fn as_categorical(&self) -> Option<u32> {
        match self {
            Self::Categorical(idx) => Some(*idx),
        }
    }
}

/// A completed trial with frozen parameters and objective value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrozenTrial {
    /// The trial number (0-indexed). `usize` mirrors Vec indexing in the
    /// study's `trials` storage; the previous `u32` field forced an
    /// `unwrap_or(u32::MAX)` cast at every push that could silently
    /// stamp the sentinel onto an out-of-range trial number.
    pub number: usize,
    /// The parameter values used in this trial.
    pub params: BTreeMap<String, ParamValue>,
    /// The objective value achieved.
    pub value: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn param_value_as_categorical() {
        let v = ParamValue::Categorical(3);
        assert_eq!(v.as_categorical(), Some(3));
    }

    #[test]
    fn frozen_trial_serde_round_trip() {
        let trial = FrozenTrial {
            number: 0,
            params: BTreeMap::from([
                ("x".into(), ParamValue::Categorical(2)),
                ("y".into(), ParamValue::Categorical(1)),
            ]),
            value: 0.85,
        };
        let json = serde_json::to_string(&trial).unwrap();
        let restored: FrozenTrial = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.number, 0);
        assert_eq!(restored.params["x"], ParamValue::Categorical(2));
        assert!((restored.value - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn direction_serde() {
        let json = serde_json::to_string(&Direction::Maximize).unwrap();
        let restored: Direction = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, Direction::Maximize);
    }
}
