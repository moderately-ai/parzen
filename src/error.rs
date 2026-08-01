// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Public error type.

use std::{error::Error, fmt};

/// An invalid configuration, search space, trial, or study operation.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ParzenError {
    /// A sampler option is invalid.
    InvalidConfig(String),
    /// A distribution is invalid.
    InvalidDistribution(String),
    /// A parameter name is already registered.
    DuplicateParameter(String),
    /// A parameter is not registered.
    UnknownParameter(String),
    /// A parameter was requested through the wrong typed method.
    ParameterTypeMismatch {
        name: String,
        expected: &'static str,
    },
    /// A value does not belong to its registered distribution.
    ValueOutsideDistribution(String),
    /// A condition is malformed or cyclic.
    InvalidCondition(String),
    /// A multivariate group is malformed.
    InvalidGroup(String),
    /// A conditional parameter was requested before its parents.
    UnresolvedCondition(String),
    /// A conditional parameter is inactive in the current trial.
    InactiveParameter(String),
    /// A required parameter was not suggested.
    MissingParameter(String),
    /// The objective is NaN or infinite.
    NonFiniteObjective,
    /// No trial is currently pending.
    NoPendingTrial,
    /// An injected trial cannot be added while a suggested trial is pending.
    PendingTrial,
    /// Internal packed storage would overflow its public ID representation.
    CapacityOverflow,
    /// A bounded history cannot retain the good set requested by gamma.
    GammaExceedsHistoryLimit { requested: usize, limit: usize },
    /// A sampler invariant was violated without panicking.
    InternalModel(String),
}

impl fmt::Display for ParzenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(message) => write!(f, "invalid sampler configuration: {message}"),
            Self::InvalidDistribution(message) => write!(f, "invalid distribution: {message}"),
            Self::DuplicateParameter(name) => write!(f, "parameter `{name}` is already registered"),
            Self::UnknownParameter(name) => write!(f, "unknown parameter `{name}`"),
            Self::ParameterTypeMismatch { name, expected } => {
                write!(f, "parameter `{name}` is not {expected}")
            }
            Self::ValueOutsideDistribution(name) => {
                write!(f, "value is outside the distribution for `{name}`")
            }
            Self::InvalidCondition(message) => write!(f, "invalid condition: {message}"),
            Self::InvalidGroup(message) => write!(f, "invalid parameter group: {message}"),
            Self::UnresolvedCondition(name) => {
                write!(f, "conditions for `{name}` have not been resolved")
            }
            Self::InactiveParameter(name) => {
                write!(f, "parameter `{name}` is inactive in this trial")
            }
            Self::MissingParameter(name) => write!(f, "active parameter `{name}` is missing"),
            Self::NonFiniteObjective => f.write_str("objective value must be finite"),
            Self::NoPendingTrial => f.write_str("no parameter has been suggested for this trial"),
            Self::PendingTrial => {
                f.write_str("abort or complete the pending trial before injecting another trial")
            }
            Self::CapacityOverflow => f.write_str("study capacity exceeded"),
            Self::GammaExceedsHistoryLimit { requested, limit } => write!(
                f,
                "gamma requested {requested} good trials, exceeding the bounded-history limit of {limit}"
            ),
            Self::InternalModel(message) => write!(f, "invalid internal sampler model: {message}"),
        }
    }
}

impl Error for ParzenError {}
