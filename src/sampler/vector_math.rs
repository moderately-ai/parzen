// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Runtime-dispatched numeric kernels used by TPE acquisition.

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

pub(crate) use scalar::continuous_log_pdf_batch as continuous_log_pdf_batch_scalar;

#[cfg(test)]
mod tests {
    #[cfg(feature = "simd")]
    use super::ContinuousDimension;

    #[cfg(feature = "simd")]
    fn assert_batch_matches(
        component_count: usize,
        candidates: usize,
        dimension_count: usize,
    ) -> (Vec<f64>, Vec<f64>) {
        let values = (0..candidates * dimension_count)
            .map(|index| (index as f64 * 0.173).sin())
            .collect::<Vec<_>>();
        let means = (0..dimension_count)
            .map(|dimension| {
                (0..component_count)
                    .map(|component| (component as f64 * 0.31 + dimension as f64 * 0.07).cos())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let inverse_sigmas = (0..dimension_count)
            .map(|dimension| {
                (0..component_count)
                    .map(|component| 0.4 + (component + dimension) as f64 * 0.03)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let log_coefficients = (0..dimension_count)
            .map(|dimension| {
                (0..component_count)
                    .map(|component| -0.1 - (component + dimension) as f64 * 0.09)
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let dimensions = (0..dimension_count)
            .map(|dimension| ContinuousDimension {
                means: &means[dimension],
                inverse_sigmas: &inverse_sigmas[dimension],
                log_coefficients: &log_coefficients[dimension],
            })
            .collect::<Vec<_>>();
        let log_weights = (0..component_count)
            .map(|component| -0.2 - component as f64 * 0.11)
            .collect::<Vec<_>>();
        let mut scalar_output = vec![0.0; candidates];
        let mut simd_output = vec![0.0; candidates];
        let mut scalar_components = vec![0.0; component_count];
        let mut simd_components = vec![0.0; component_count];
        super::scalar::continuous_log_pdf_batch(
            &values,
            candidates,
            &dimensions,
            &log_weights,
            &mut scalar_output,
            &mut scalar_components,
        );
        super::simd::continuous_log_pdf_batch(
            &values,
            candidates,
            &dimensions,
            &log_weights,
            &mut simd_output,
            &mut simd_components,
        );
        for (candidate, (scalar, simd)) in scalar_output.iter().zip(&simd_output).enumerate() {
            let tolerance = 1e-12_f64.max(scalar.abs() * 1e-12);
            assert!(
                (scalar - simd).abs() <= tolerance,
                "components={component_count}, candidates={candidates}, dimensions={dimension_count}, candidate={candidate}, scalar={scalar}, simd={simd}"
            );
        }
        (scalar_output, simd_output)
    }

    #[cfg(feature = "simd")]
    #[test]
    fn continuous_batch_matches_scalar_across_vector_boundaries() {
        for component_count in [1, 3, 4, 5, 8, 9, 17] {
            for candidates in [1, 5, 24, 25] {
                for dimension_count in [1, 2, 4] {
                    let _ = assert_batch_matches(component_count, candidates, dimension_count);
                }
            }
        }
    }

    #[cfg(feature = "simd")]
    #[test]
    fn scalar_and_simd_choose_the_same_first_strict_winner() {
        fn winner(good: &[f64], bad: &[f64]) -> usize {
            good.iter()
                .zip(bad)
                .enumerate()
                .fold((0, f64::NEG_INFINITY), |best, (index, (good, bad))| {
                    let score = good - bad;
                    if score.is_finite() && score > best.1 {
                        (index, score)
                    } else {
                        best
                    }
                })
                .0
        }

        let (scalar_good, simd_good) = assert_batch_matches(9, 24, 4);
        let (scalar_bad, simd_bad) = assert_batch_matches(17, 24, 4);
        assert_eq!(
            winner(&scalar_good, &scalar_bad),
            winner(&simd_good, &simd_bad)
        );

        let tied_good = [2.0, 2.0, 1.0];
        let tied_bad = [0.5, 0.5, 0.0];
        assert_eq!(winner(&tied_good, &tied_bad), 0);
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

    #[cfg(all(feature = "simd", target_arch = "x86_64"))]
    #[test]
    #[ignore = "ten-million-value numerical audit"]
    fn x86_exponential_long_audit_stays_within_four_ulps() {
        const VALUES: usize = 10_000_000;
        let mut state = 0x243f_6a88_85a3_08d3_u64;
        let mut inputs = Vec::with_capacity(VALUES);
        for _ in 0..VALUES {
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15).rotate_left(17);
            let unit = (state >> 11) as f64 * (1.0 / ((1_u64 << 53) as f64));
            inputs.push(-700.0 * unit);
        }

        let outputs = super::simd::test_negative_exp(&inputs);
        let maximum_ulps = inputs
            .into_iter()
            .zip(outputs)
            .map(|(value, actual)| actual.to_bits().abs_diff(value.exp().to_bits()))
            .max()
            .unwrap_or(0);
        assert!(maximum_ulps <= 4, "maximum ULP difference: {maximum_ulps}");
    }
}
