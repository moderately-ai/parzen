// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Study — manages trials, tracks best results, orchestrates the sampler.

use std::collections::BTreeMap;

use crate::{
    sampler::TpeSampler,
    trial::{Direction, FrozenTrial, ParamValue},
};

/// A Bayesian optimization study that manages trials and tracks the best result.
///
/// # Usage
///
/// ```rust
/// use parzen::{
///     Direction, GammaStrategy, Study, TpeSampler, TpeSamplerConfig, TpeSamplerDeps,
/// };
///
/// let sampler = TpeSampler::new(
///     TpeSamplerDeps { gamma_strategy: GammaStrategy::Default },
///     TpeSamplerConfig {
///         seed: 42,
///         n_startup_trials: 5,
///         prior_weight: TpeSamplerConfig::DEFAULT_PRIOR_WEIGHT,
///     },
/// );
/// let mut study = Study::new(Direction::Maximize, sampler);
///
/// for _ in 0..20 {
///     let x = study.suggest_categorical("x", 5);
///     let y = study.suggest_categorical("y", 3);
///     let score = if x == 2 && y == 1 { 1.0 } else { 0.1 };
///     study.complete_trial(score);
/// }
///
/// let best = study.best_trial().unwrap();
/// println!("Best trial #{}: score {}", best.number, best.value);
/// ```
pub struct Study {
    direction: Direction,
    sampler: TpeSampler,
    trials: Vec<FrozenTrial>,
    pending_params: BTreeMap<String, ParamValue>,
}

impl Study {
    /// Create a new study with the given optimization direction and sampler.
    #[must_use]
    pub const fn new(direction: Direction, sampler: TpeSampler) -> Self {
        Self {
            direction,
            sampler,
            trials: Vec::new(),
            pending_params: BTreeMap::new(),
        }
    }

    /// Suggest a categorical parameter value for the current trial.
    ///
    /// Accumulates parameters internally. Call [`complete_trial`](Self::complete_trial)
    /// after evaluating to freeze the trial.
    pub fn suggest_categorical(&mut self, name: &str, num_choices: usize) -> usize {
        let value =
            self.sampler
                .sample_categorical(name, num_choices, &self.trials, self.direction);
        self.pending_params.insert(
            name.to_owned(),
            ParamValue::Categorical(u32::try_from(value).unwrap_or(u32::MAX)),
        );
        value
    }

    /// Complete the current trial with the given objective value.
    ///
    /// Freezes accumulated parameters into a [`FrozenTrial`] and returns
    /// the trial number.
    pub fn complete_trial(&mut self, value: f64) -> usize {
        let number = self.trials.len();
        let params = std::mem::take(&mut self.pending_params);
        self.trials.push(FrozenTrial {
            number,
            params,
            value,
        });
        number
    }

    /// Manually inject a completed trial (e.g., for baseline seeding).
    ///
    /// The trial is appended as-is. Its `number` field is not modified.
    pub fn add_trial(&mut self, trial: FrozenTrial) {
        self.trials.push(trial);
    }

    /// The best trial by objective value, according to the study's direction.
    ///
    /// Returns `None` if no trials have been completed.
    #[must_use]
    pub fn best_trial(&self) -> Option<&FrozenTrial> {
        use std::cmp::Ordering;
        // NaN values compare as Equal so they can't win over a real number.
        match self.direction {
            Direction::Maximize => self
                .trials
                .iter()
                .max_by(|a, b| a.value.partial_cmp(&b.value).unwrap_or(Ordering::Equal)),
            Direction::Minimize => self
                .trials
                .iter()
                .min_by(|a, b| a.value.partial_cmp(&b.value).unwrap_or(Ordering::Equal)),
        }
    }

    /// The best objective value, according to the study's direction.
    #[must_use]
    pub fn best_value(&self) -> Option<f64> {
        self.best_trial().map(|t| t.value)
    }

    /// All completed trials in insertion order.
    #[must_use]
    pub fn trials(&self) -> &[FrozenTrial] {
        &self.trials
    }

    /// Number of completed trials.
    #[must_use]
    pub fn num_trials(&self) -> usize {
        self.trials.len()
    }

    /// The optimization direction.
    #[must_use]
    pub const fn direction(&self) -> Direction {
        self.direction
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sampler::{GammaStrategy, TpeSamplerConfig, TpeSamplerDeps};

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

    #[test]
    fn suggest_and_complete_round_trip() {
        let mut study = Study::new(Direction::Maximize, sampler_with_seed(42));

        let x = study.suggest_categorical("x", 5);
        assert!(x < 5);

        let trial_num = study.complete_trial(0.8);
        assert_eq!(trial_num, 0);
        assert_eq!(study.num_trials(), 1);
        assert_eq!(
            study.trials()[0].params["x"],
            ParamValue::Categorical(u32::try_from(x).unwrap())
        );
        assert!((study.trials()[0].value - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn best_trial_maximize() {
        let mut study = Study::new(Direction::Maximize, sampler_with_seed(42));

        study.suggest_categorical("x", 3);
        study.complete_trial(0.5);

        study.suggest_categorical("x", 3);
        study.complete_trial(0.9);

        study.suggest_categorical("x", 3);
        study.complete_trial(0.3);

        let best = study.best_trial().unwrap();
        assert!((best.value - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn best_trial_minimize() {
        let mut study = Study::new(Direction::Minimize, sampler_with_seed(42));

        study.suggest_categorical("x", 3);
        study.complete_trial(0.5);

        study.suggest_categorical("x", 3);
        study.complete_trial(0.1);

        study.suggest_categorical("x", 3);
        study.complete_trial(0.9);

        let best = study.best_trial().unwrap();
        assert!((best.value - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn empty_study() {
        let study = Study::new(Direction::Maximize, sampler_with_seed(42));
        assert!(study.best_trial().is_none());
        assert!(study.best_value().is_none());
        assert_eq!(study.num_trials(), 0);
        assert!(study.trials().is_empty());
    }

    #[test]
    fn add_trial_injected_baseline() {
        let mut study = Study::new(Direction::Maximize, sampler_with_seed(42));

        // Manually inject a baseline trial (like MIPROv2 does)
        let baseline = FrozenTrial {
            number: 0,
            params: BTreeMap::from([
                ("instruction".into(), ParamValue::Categorical(0)),
                ("demos".into(), ParamValue::Categorical(0)),
            ]),
            value: 0.6,
        };
        study.add_trial(baseline);

        assert_eq!(study.num_trials(), 1);
        assert!((study.best_value().unwrap() - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn multiple_params_per_trial() {
        let mut study = Study::new(Direction::Maximize, sampler_with_seed(42));

        let x = study.suggest_categorical("instruction", 6);
        let y = study.suggest_categorical("demos", 4);
        study.complete_trial(0.75);

        let trial = &study.trials()[0];
        assert_eq!(
            trial.params["instruction"],
            ParamValue::Categorical(u32::try_from(x).unwrap())
        );
        assert_eq!(
            trial.params["demos"],
            ParamValue::Categorical(u32::try_from(y).unwrap())
        );
    }

    #[test]
    fn multiple_trials_all_stored() {
        let mut study = Study::new(Direction::Maximize, sampler_with_seed(42));

        for i in 0..10 {
            study.suggest_categorical("x", 5);
            study.complete_trial(f64::from(i) * 0.1);
        }

        assert_eq!(study.num_trials(), 10);
        assert!((study.best_value().unwrap() - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn integration_miprov2_pattern() {
        let sampler = sampler_with_startup(42, 5);
        let mut study = Study::new(Direction::Maximize, sampler);

        // Inject baseline
        study.add_trial(FrozenTrial {
            number: 0,
            params: BTreeMap::from([
                ("instruction".into(), ParamValue::Categorical(0)),
                ("demos".into(), ParamValue::Categorical(0)),
            ]),
            value: 0.5,
        });

        // Run 25 trials with known optimal: instruction=3, demos=2
        for _ in 0..25 {
            let inst = study.suggest_categorical("instruction", 6);
            let demos = study.suggest_categorical("demos", 4);
            let score = if inst == 3 && demos == 2 {
                1.0
            } else if inst == 3 || demos == 2 {
                0.6
            } else {
                0.2
            };
            study.complete_trial(score);
        }

        let best = study.best_trial().unwrap();
        // Best trial should have found the optimal or near-optimal combination
        assert!(
            best.value >= 0.6,
            "best value should be at least 0.6, got {}",
            best.value
        );
    }
}
