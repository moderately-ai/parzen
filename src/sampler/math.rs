// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::f64::consts::{SQRT_2, TAU};

pub(crate) fn normal_log_cdf(value: f64) -> f64 {
    if value < -10.0 {
        let inverse_square = 1.0 / (value * value);
        let correction =
            1.0 - inverse_square + 3.0 * inverse_square.powi(2) - 15.0 * inverse_square.powi(3);
        return -0.5 * value * value - (-value).ln() - 0.5 * TAU.ln()
            + correction.max(f64::MIN_POSITIVE).ln();
    }
    (0.5 * libm::erfc(-value / SQRT_2))
        .max(f64::MIN_POSITIVE)
        .ln()
}

pub(crate) fn normal_log_survival(value: f64) -> f64 {
    normal_log_cdf(-value)
}

pub(crate) fn log_diff_exp(larger: f64, smaller: f64) -> f64 {
    if smaller == f64::NEG_INFINITY {
        return larger;
    }
    if !larger.is_finite() || larger <= smaller {
        return f64::NEG_INFINITY;
    }
    larger + (-((smaller - larger).exp())).ln_1p()
}

pub(crate) fn log_gaussian_mass(low: f64, high: f64) -> f64 {
    if low >= high {
        return f64::NEG_INFINITY;
    }
    let mass = if high <= 0.0 {
        log_diff_exp(normal_log_cdf(high), normal_log_cdf(low))
    } else if low >= 0.0 {
        log_diff_exp(normal_log_survival(low), normal_log_survival(high))
    } else {
        log_diff_exp(normal_log_cdf(high), normal_log_cdf(low))
    };
    mass.max(f64::MIN_POSITIVE.ln())
}

pub(crate) fn logsumexp(values: &[f64]) -> f64 {
    let maximum = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if maximum == f64::NEG_INFINITY {
        return maximum;
    }
    maximum
        + values
            .iter()
            .map(|value| (value - maximum).exp())
            .sum::<f64>()
            .ln()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_mass_is_stable_in_both_tails() {
        assert!(log_gaussian_mass(-40.0, -39.9).is_finite());
        assert!(log_gaussian_mass(39.9, 40.0).is_finite());
        assert!((normal_log_cdf(0.0) - 0.5_f64.ln()).abs() < 1e-14);
    }

    #[test]
    fn logsumexp_handles_empty_support() {
        assert_eq!(logsumexp(&[f64::NEG_INFINITY]), f64::NEG_INFINITY);
        assert!((logsumexp(&[0.0, 0.0]) - 2.0_f64.ln()).abs() < 1e-14);
    }
}
