// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

//! High-performance Tree-structured Parzen Estimator optimization.
//!
//! ```rust
//! use parzen::{
//!     CategoricalDistribution, Direction, Distribution, SearchSpace, Study,
//!     TpeSampler, TpeSamplerConfig,
//! };
//!
//! # fn main() -> Result<(), parzen::ParzenError> {
//! let mut space = SearchSpace::new();
//! space.add("x", Distribution::Categorical(CategoricalDistribution::new(5)?))?;
//! let sampler = TpeSampler::new(TpeSamplerConfig::performance(42).startup_trials(5))?;
//! let mut study = Study::new(Direction::Maximize, sampler, space)?;
//!
//! for _ in 0..20 {
//!     let x = study.suggest_categorical("x")?;
//!     study.complete_trial(if x == 2 { 1.0 } else { 0.1 })?;
//! }
//! assert!(study.best_value().is_some_and(|value| value > 0.5));
//! # Ok(()) }
//! ```
//!
//! Explicit parameter groups use one trial-aligned mixture component for the
//! entire vector. Their joint likelihood is
//! `logsumexp(log(weight[k]) + sum_d log(kernel[d][k](x[d])))`, preserving
//! correlations that independent marginal models discard. Integer and stepped
//! distributions integrate each Gaussian kernel over the selected grid cell
//! instead of treating a discrete value as a continuous point.
//!
//! [`HistoryPolicy::Bounded`] keeps a fixed-size exact best set, recent bad
//! observations, and a deterministic reservoir, so estimator state and
//! incremental update work do not grow with completed-trial count. Raw trial
//! records remain complete. [`HistoryPolicy::Full`] retains exact full-history
//! ranking and therefore has linear storage and model-construction costs.

mod distribution;
mod error;
mod sampler;
mod search_space;
mod storage;
mod study;
mod trial;

pub use distribution::{
    CategoricalDistribution, Distribution, FloatDistribution, FloatScale, IntDistribution, IntScale,
};
pub use error::ParzenError;
pub use sampler::{
    GammaStrategy, HistoryPolicy, ModelStrategy, TpeSampler, TpeSamplerConfig, WeightStrategy,
};
pub use search_space::{Condition, GroupId, ParamId, ParameterRef, SearchSpace};
pub use study::Study;
pub use trial::{
    Direction, ParamValue, Params, TrialId, TrialInput, TrialRecord, TrialRef, Trials,
};
