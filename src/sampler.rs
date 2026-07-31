#![expect(
// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "TPE is a statistical sampler in f64 space. usize <-> f64 conversions are the \
              core arithmetic (trial counts and category counts feed densities, weights, and \
              expected-improvement ratios); precision loss above 2^53 would require more \
              completed trials than anyone will ever run. Covers `.ceil() as usize` in \
              `gamma()` where the result is clamped to [1, 25] immediately."
)]

//! TPE (Tree-structured Parzen Estimator) sampler.
//!
//! Models `p(x|y)` by splitting trials into "good" and "bad" groups,
//! estimating per-category densities, and sampling proportional to the
//! Expected Improvement ratio `l(x) / g(x)`.

use std::cmp::Ordering;

use rand::{
    SeedableRng,
    distr::{Distribution, weighted::WeightedIndex},
    rngs::StdRng,
};

use crate::trial::{Direction, FrozenTrial, ParamValue};

/// Default gamma function matching Optuna: `min(ceil(0.25 * sqrt(n)), 25)`.
///
/// Returns the number of "good" trials given `n` completed trials.
fn default_gamma(n: usize) -> usize {
    let g = (0.25 * (n as f64).sqrt()).ceil() as usize;
    g.clamp(1, 25)
}

/// Pure-value configuration for [`TpeSampler::new`].
///
/// Every field is named at the construction site; `DEFAULT_*` constants
/// on the impl block carry the Optuna defaults so callers that only
/// want to override one or two fields can use struct-update syntax.
pub struct TpeSamplerConfig {
    /// PRNG seed for reproducible runs.
    pub seed: u64,
    /// Random startup trials before TPE kicks in. Use
    /// [`TpeSamplerConfig::DEFAULT_N_STARTUP_TRIALS`] for the Optuna default.
    pub n_startup_trials: usize,
    /// Laplace-smoothing prior weight per category. Use
    /// [`TpeSamplerConfig::DEFAULT_PRIOR_WEIGHT`] for the Optuna default.
    pub prior_weight: f64,
}

impl TpeSamplerConfig {
    /// Optuna default: 10 random startup trials before TPE kicks in.
    pub const DEFAULT_N_STARTUP_TRIALS: usize = 10;
    /// Optuna default: prior weight of `1.0`.
    pub const DEFAULT_PRIOR_WEIGHT: f64 = 1.0;
}

/// Gamma function strategy: maps `n_completed_trials -> n_good`.
///
/// The `Default` variant matches Optuna's
/// `min(ceil(0.25 * sqrt(n)), 25)`. The `Custom` variant lets callers
/// inject an alternate function. Modeled as an enum (rather than
/// `Option<Box<dyn Fn>>` on the config) so the choice is explicit at
/// the construction site.
pub enum GammaStrategy {
    /// Optuna default `min(ceil(0.25 * sqrt(n)), 25)`.
    Default,
    /// Caller-supplied function.
    Custom(Box<dyn Fn(usize) -> usize + Send + Sync>),
}

impl GammaStrategy {
    fn into_boxed_fn(self) -> Box<dyn Fn(usize) -> usize + Send + Sync> {
        match self {
            Self::Default => Box::new(default_gamma),
            Self::Custom(f) => f,
        }
    }
}

/// Injected dependencies for [`TpeSampler`]. Carries the
/// behaviour-bearing gamma strategy.
pub struct TpeSamplerDeps {
    pub gamma_strategy: GammaStrategy,
}

/// TPE sampler for categorical parameters.
///
/// During startup (`n < n_startup_trials`), uses uniform random sampling.
/// After startup, splits completed trials into "good" and "bad" groups
/// based on the gamma function, estimates per-category densities with
/// Laplace smoothing, and samples proportional to the EI ratio `l(x)/g(x)`.
pub struct TpeSampler {
    rng: StdRng,
    n_startup_trials: usize,
    prior_weight: f64,
    gamma_fn: Box<dyn Fn(usize) -> usize + Send + Sync>,
}

impl TpeSampler {
    /// Create a new TPE sampler from the given deps and config.
    #[must_use]
    pub fn new(deps: TpeSamplerDeps, config: TpeSamplerConfig) -> Self {
        let TpeSamplerConfig {
            seed,
            n_startup_trials,
            prior_weight,
        } = config;
        Self {
            rng: StdRng::seed_from_u64(seed),
            n_startup_trials,
            prior_weight,
            gamma_fn: deps.gamma_strategy.into_boxed_fn(),
        }
    }

    /// Suggest a categorical parameter value.
    ///
    /// During startup: uniform random from `0..num_choices`.
    /// After startup: TPE-based EI sampling.
    pub(crate) fn sample_categorical(
        &mut self,
        param_name: &str,
        num_choices: usize,
        completed_trials: &[FrozenTrial],
        direction: Direction,
    ) -> usize {
        assert!(num_choices > 0, "num_choices must be > 0");

        if num_choices == 1 {
            return 0;
        }

        if completed_trials.len() < self.n_startup_trials {
            return self.sample_uniform(num_choices);
        }

        self.sample_tpe(param_name, num_choices, completed_trials, direction)
    }

    /// Uniform random sampling for startup phase.
    fn sample_uniform(&mut self, num_choices: usize) -> usize {
        rand::Rng::random_range(&mut self.rng, 0..num_choices)
    }

    /// TPE-based sampling after startup.
    fn sample_tpe(
        &mut self,
        param_name: &str,
        num_choices: usize,
        completed_trials: &[FrozenTrial],
        direction: Direction,
    ) -> usize {
        // Sort trials by value. NaN trial values sort as Equal so they
        // don't bubble to the top (can't win over a real value).
        let mut sorted: Vec<&FrozenTrial> = completed_trials.iter().collect();
        match direction {
            Direction::Maximize => {
                sorted.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap_or(Ordering::Equal));
            }
            Direction::Minimize => {
                sorted.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap_or(Ordering::Equal));
            }
        }

        // Split into good and bad
        let n = sorted.len();
        let n_good = (self.gamma_fn)(n).clamp(1, n.saturating_sub(1).max(1));
        let good = &sorted[..n_good];
        let bad = &sorted[n_good..];
        let n_bad = bad.len();

        // Compute EI weights for each choice
        let prior = self.prior_weight / num_choices as f64;
        let mut weights = vec![0.0_f64; num_choices];

        for (c, weight) in weights.iter_mut().enumerate() {
            let count_good = count_categorical(good, param_name, c);
            let count_bad = count_categorical(bad, param_name, c);

            let l_c = (count_good as f64 + prior) / (n_good as f64 + self.prior_weight);
            let g_c = (count_bad as f64 + prior) / (n_bad as f64 + self.prior_weight);

            // EI weight = l(c) / g(c)
            *weight = if g_c > 0.0 { l_c / g_c } else { l_c };
        }

        // Sample from weighted distribution. Weights are constructed above
        // as `(count + prior) / (total + prior_weight)` with prior > 0, so
        // every weight is strictly positive. If WeightedIndex::new ever
        // fails here, the weight construction invariant has been broken —
        // fall back to uniform sampling rather than crashing the study.
        match WeightedIndex::new(&weights) {
            Ok(dist) => dist.sample(&mut self.rng),
            Err(_) => self.sample_uniform(num_choices),
        }
    }
}

/// Count how many trials have `param_name == Categorical(choice)`.
///
/// Trials that don't contain `param_name` are skipped (not counted).
fn count_categorical(trials: &[&FrozenTrial], param_name: &str, choice: usize) -> usize {
    let needle = ParamValue::Categorical(u32::try_from(choice).unwrap_or(u32::MAX));
    trials
        .iter()
        .filter(|t| t.params.get(param_name) == Some(&needle))
        .count()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    /// Helper: TpeSampler with the given seed and Optuna defaults.
    fn sampler_with_seed(seed: u64) -> TpeSampler {
        TpeSampler::new(
            TpeSamplerDeps {
                gamma_strategy: GammaStrategy::Default,
            },
            TpeSamplerConfig {
                seed,
                n_startup_trials: TpeSamplerConfig::DEFAULT_N_STARTUP_TRIALS,
                prior_weight: TpeSamplerConfig::DEFAULT_PRIOR_WEIGHT,
            },
        )
    }

    /// Helper: TpeSampler with an override for `n_startup_trials`.
    fn sampler_with_startup(seed: u64, n_startup_trials: usize) -> TpeSampler {
        TpeSampler::new(
            TpeSamplerDeps {
                gamma_strategy: GammaStrategy::Default,
            },
            TpeSamplerConfig {
                seed,
                n_startup_trials,
                prior_weight: TpeSamplerConfig::DEFAULT_PRIOR_WEIGHT,
            },
        )
    }

    fn make_trial(number: usize, params: &[(&str, usize)], value: f64) -> FrozenTrial {
        FrozenTrial {
            number,
            params: params
                .iter()
                .map(|(k, v)| {
                    (
                        (*k).to_string(),
                        ParamValue::Categorical(u32::try_from(*v).unwrap()),
                    )
                })
                .collect(),
            value,
        }
    }

    #[test]
    fn single_choice_always_returns_zero() {
        let mut sampler = sampler_with_seed(42);
        for _ in 0..10 {
            assert_eq!(
                sampler.sample_categorical("x", 1, &[], Direction::Maximize),
                0
            );
        }
    }

    #[test]
    fn startup_uses_random_sampling() {
        let mut seen = [false; 5];

        // Run many startup samples (no completed trials)
        for seed in 0..100 {
            let mut s = sampler_with_startup(seed, 5);
            let choice = s.sample_categorical("x", 5, &[], Direction::Maximize);
            assert!(choice < 5);
            seen[choice] = true;
        }

        // With 100 different seeds, all 5 choices should appear
        assert!(
            seen.iter().all(|&s| s),
            "not all choices appeared during startup"
        );
    }

    #[test]
    fn deterministic_with_seed() {
        let trials: Vec<FrozenTrial> = (0..15)
            .map(|i| make_trial(i, &[("x", i % 5)], if i % 5 == 2 { 1.0 } else { 0.1 }))
            .collect();

        let mut s1 = sampler_with_startup(99, 5);
        let mut s2 = sampler_with_startup(99, 5);

        for _ in 0..10 {
            let a = s1.sample_categorical("x", 5, &trials, Direction::Maximize);
            let b = s2.sample_categorical("x", 5, &trials, Direction::Maximize);
            assert_eq!(a, b, "same seed must produce same sequence");
        }
    }

    #[test]
    fn converges_to_best_choice_maximize() {
        // Choice 2 always scores 1.0, others score 0.1
        let mut trials: Vec<FrozenTrial> = (0..15)
            .map(|i| make_trial(i, &[("x", i % 5)], if i % 5 == 2 { 1.0 } else { 0.1 }))
            .collect();

        let mut sampler = sampler_with_startup(42, 5);
        let mut counts = [0usize; 5];

        // Run 50 more trials, tracking which choices TPE suggests
        for i in 0..50 {
            let choice = sampler.sample_categorical("x", 5, &trials, Direction::Maximize);
            counts[choice] += 1;
            let value = if choice == 2 { 1.0 } else { 0.1 };
            trials.push(make_trial(15 + i, &[("x", choice)], value));
        }

        // Choice 2 should be sampled most often
        let max_idx = counts
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| **c)
            .unwrap()
            .0;
        assert_eq!(
            max_idx, 2,
            "TPE should converge to choice 2, counts: {counts:?}"
        );
    }

    #[test]
    fn converges_to_best_choice_minimize() {
        // Choice 1 always scores 0.0, others score 1.0
        let mut trials: Vec<FrozenTrial> = (0..15)
            .map(|i| make_trial(i, &[("x", i % 5)], if i % 5 == 1 { 0.0 } else { 1.0 }))
            .collect();

        let mut sampler = sampler_with_startup(42, 5);
        let mut counts = [0usize; 5];

        for i in 0..50 {
            let choice = sampler.sample_categorical("x", 5, &trials, Direction::Minimize);
            counts[choice] += 1;
            let value = if choice == 1 { 0.0 } else { 1.0 };
            trials.push(make_trial(15 + i, &[("x", choice)], value));
        }

        let max_idx = counts
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| **c)
            .unwrap()
            .0;
        assert_eq!(
            max_idx, 1,
            "TPE should converge to choice 1 for minimize, counts: {counts:?}"
        );
    }

    #[test]
    fn two_parameters_converge_independently() {
        // Best: x=3, y=1
        let mut trials: Vec<FrozenTrial> = (0..20)
            .map(|i| {
                let x = i % 5;
                let y = i % 3;
                let value = if x == 3 { 0.5 } else { 0.0 } + if y == 1 { 0.5 } else { 0.0 };
                make_trial(i, &[("x", x), ("y", y)], value)
            })
            .collect();

        let mut sampler = sampler_with_startup(42, 10);
        let mut x_counts = [0usize; 5];
        let mut y_counts = [0usize; 3];

        for i in 0..50 {
            let x = sampler.sample_categorical("x", 5, &trials, Direction::Maximize);
            let y = sampler.sample_categorical("y", 3, &trials, Direction::Maximize);
            x_counts[x] += 1;
            y_counts[y] += 1;
            let value = if x == 3 { 0.5 } else { 0.0 } + if y == 1 { 0.5 } else { 0.0 };
            trials.push(make_trial(20 + i, &[("x", x), ("y", y)], value));
        }

        let best_x = x_counts
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| **c)
            .unwrap()
            .0;
        let best_y = y_counts
            .iter()
            .enumerate()
            .max_by_key(|(_, c)| **c)
            .unwrap()
            .0;
        assert_eq!(best_x, 3, "x should converge to 3, counts: {x_counts:?}");
        assert_eq!(best_y, 1, "y should converge to 1, counts: {y_counts:?}");
    }

    #[test]
    fn prior_smoothing_gives_unseen_choices_nonzero_probability() {
        // All 15 trials chose x=0, scoring 1.0
        let trials: Vec<FrozenTrial> = (0..15).map(|i| make_trial(i, &[("x", 0)], 1.0)).collect();

        let mut sampler = sampler_with_startup(42, 5);
        let mut saw_nonzero = false;

        // Over many samples, unseen choices (1, 2, 3, 4) should appear sometimes
        for _ in 0..200 {
            let choice = sampler.sample_categorical("x", 5, &trials, Direction::Maximize);
            if choice != 0 {
                saw_nonzero = true;
                break;
            }
        }

        assert!(
            saw_nonzero,
            "prior smoothing should allow unseen choices to be sampled"
        );
    }

    #[test]
    fn trials_missing_param_are_skipped() {
        // Some trials have "x", some don't
        let trials = vec![
            FrozenTrial {
                number: 0,
                params: BTreeMap::from([("x".into(), ParamValue::Categorical(2))]),
                value: 1.0,
            },
            FrozenTrial {
                number: 1,
                params: BTreeMap::new(), // no "x" param
                value: 0.5,
            },
        ];

        // Should not panic
        let mut sampler = sampler_with_startup(42, 0);
        let _choice = sampler.sample_categorical("x", 5, &trials, Direction::Maximize);
    }

    #[test]
    fn default_gamma_values() {
        assert_eq!(default_gamma(1), 1);
        assert_eq!(default_gamma(4), 1);
        assert_eq!(default_gamma(6), 1); // MIPROv2 light
        assert_eq!(default_gamma(12), 1); // MIPROv2 medium
        assert_eq!(default_gamma(18), 2); // MIPROv2 heavy
        assert_eq!(default_gamma(100), 3);
        assert_eq!(default_gamma(10000), 25);
    }
}
