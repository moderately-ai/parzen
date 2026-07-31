// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

//! Tree-structured Parzen Estimator for Bayesian optimization.
//!
//! Implements the TPE algorithm (Bergstra et al., 2011) for categorical
//! parameter optimization. Used by prompt optimizers like `MIPROv2` to search
//! over instruction and demo set combinations.
//!
//! # Quick Start
//!
//! ```rust
//! use parzen::{
//!     Direction, GammaStrategy, Study, TpeSampler, TpeSamplerConfig, TpeSamplerDeps,
//! };
//!
//! let sampler = TpeSampler::new(
//!     TpeSamplerDeps { gamma_strategy: GammaStrategy::Default },
//!     TpeSamplerConfig {
//!         seed: 42,
//!         n_startup_trials: 5,
//!         prior_weight: TpeSamplerConfig::DEFAULT_PRIOR_WEIGHT,
//!     },
//! );
//! let mut study = Study::new(Direction::Maximize, sampler);
//!
//! for _ in 0..20 {
//!     let x = study.suggest_categorical("x", 5);
//!     let score = if x == 2 { 1.0 } else { 0.1 };
//!     study.complete_trial(score);
//! }
//!
//! let best = study.best_trial().unwrap();
//! assert!(best.value > 0.5);
//! ```

pub(crate) mod sampler;
pub(crate) mod study;
pub(crate) mod trial;

pub use sampler::{GammaStrategy, TpeSampler, TpeSamplerConfig, TpeSamplerDeps};
pub use study::Study;
pub use trial::{Direction, FrozenTrial, ParamValue};
