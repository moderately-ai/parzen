// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Runtime-dispatched numeric kernels used by TPE acquisition.

#[cfg(any(test, not(feature = "simd")))]
mod scalar;

#[cfg(feature = "simd")]
mod simd;

pub(crate) struct ContinuousDimension<'a> {
    pub means: &'a [f64],
    pub inverse_sigmas: &'a [f64],
    pub log_coefficients: &'a [f64],
}

#[cfg(feature = "simd")]
pub(crate) use simd::continuous_log_pdf_batch;

#[cfg(not(feature = "simd"))]
pub(crate) use scalar::continuous_log_pdf_batch;

#[cfg(test)]
mod tests {
    #[cfg(feature = "simd")]
    use super::ContinuousDimension;
    #[cfg(feature = "simd")]
    #[test]
    fn continuous_batch_matches_scalar_oracle() {
        let values = [-0.75, 0.25, -0.5, 0.5, -0.25, 0.75, 0.0, 1.0, 0.25, 1.25];
        let first_means = [-0.4, 0.1, 0.8];
        let second_means = [0.2, 0.7, 1.1];
        let first_inverse = [1.3, 0.9, 0.6];
        let second_inverse = [0.8, 1.1, 0.7];
        let first_coefficients = [-0.2, -0.4, -0.7];
        let second_coefficients = [-0.3, -0.1, -0.6];
        let dimensions = [
            ContinuousDimension {
                means: &first_means,
                inverse_sigmas: &first_inverse,
                log_coefficients: &first_coefficients,
            },
            ContinuousDimension {
                means: &second_means,
                inverse_sigmas: &second_inverse,
                log_coefficients: &second_coefficients,
            },
        ];
        let log_weights = [-1.2, -0.8, -1.4];
        let mut scalar_output = [0.0; 5];
        let mut simd_output = [0.0; 5];
        let mut scalar_components = [0.0; 3];
        let mut simd_components = [0.0; 3];
        super::scalar::continuous_log_pdf_batch(
            &values,
            5,
            &dimensions,
            &log_weights,
            &mut scalar_output,
            &mut scalar_components,
        );
        super::simd::continuous_log_pdf_batch(
            &values,
            5,
            &dimensions,
            &log_weights,
            &mut simd_output,
            &mut simd_components,
        );
        for (scalar, simd) in scalar_output.into_iter().zip(simd_output) {
            let tolerance = 1e-12_f64.max(scalar.abs() * 1e-12);
            assert!(
                (scalar - simd).abs() <= tolerance,
                "scalar={scalar}, simd={simd}"
            );
        }
    }

    #[cfg(all(feature = "simd", target_arch = "x86_64"))]
    #[test]
    fn x86_exponential_stays_within_four_ulps() {
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut inputs = vec![
            0.0,
            -f64::EPSILON,
            -0.000_001,
            -0.1,
            -0.5,
            -1.0,
            -10.0,
            -100.0,
            -699.999_999,
            -700.0,
            -744.0,
            -745.0,
            f64::NEG_INFINITY,
            f64::NAN,
        ];
        inputs.reserve(100_000);
        for _ in 0..100_000 {
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15).rotate_left(17);
            let unit = (state >> 11) as f64 * (1.0 / ((1_u64 << 53) as f64));
            inputs.push(-745.0 * unit);
        }
        let outputs = super::simd::test_negative_exp(&inputs);
        for (value, actual) in inputs.into_iter().zip(outputs) {
            let expected = value.exp();
            assert_eq!(actual.classify(), expected.classify(), "x={value}");
            if expected != 0.0 && expected.is_finite() {
                assert!(
                    actual.to_bits().abs_diff(expected.to_bits()) <= 4,
                    "x={value}, actual={actual:e}, expected={expected:e}"
                );
            }
        }
    }
}
