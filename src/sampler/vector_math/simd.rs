// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use pulp::{Arch, Simd, WithSimd};

use super::ContinuousDimension;

#[allow(clippy::too_many_arguments)]
pub(crate) fn continuous_log_pdf_batch(
    values: &[f64],
    candidates: usize,
    dimensions: &[ContinuousDimension<'_>],
    log_weights: &[f64],
    output: &mut [f64],
    component_scores: &mut [f64],
) {
    Arch::new().dispatch(ContinuousLogPdfBatch {
        values,
        candidates,
        dimensions,
        log_weights,
        output,
        component_scores,
    });
}

struct ContinuousLogPdfBatch<'a> {
    values: &'a [f64],
    candidates: usize,
    dimensions: &'a [ContinuousDimension<'a>],
    log_weights: &'a [f64],
    output: &'a mut [f64],
    component_scores: &'a mut [f64],
}

impl WithSimd for ContinuousLogPdfBatch<'_> {
    type Output = ();

    #[inline(always)]
    fn with_simd<S: Simd>(self, simd: S) {
        let Self {
            values,
            candidates,
            dimensions,
            log_weights,
            output,
            component_scores,
        } = self;
        let lanes = S::F64_LANES;
        debug_assert!(lanes <= 8);
        let components = log_weights.len();
        let vector_components = components / lanes * lanes;

        for candidate in 0..candidates {
            component_scores.copy_from_slice(log_weights);
            for (dimension, kernels) in dimensions.iter().enumerate() {
                let value = simd.splat_f64s(values[candidate * dimensions.len() + dimension]);
                for start in (0..vector_components).step_by(lanes) {
                    let means = simd.partial_load_f64s(&kernels.means[start..start + lanes]);
                    let inverse =
                        simd.partial_load_f64s(&kernels.inverse_sigmas[start..start + lanes]);
                    let coefficients =
                        simd.partial_load_f64s(&kernels.log_coefficients[start..start + lanes]);
                    let scores = simd.partial_load_f64s(&component_scores[start..start + lanes]);
                    let z = simd.mul_f64s(simd.sub_f64s(value, means), inverse);
                    let density = simd.negate_mul_add_f64s(
                        simd.splat_f64s(0.5),
                        simd.mul_f64s(z, z),
                        coefficients,
                    );
                    let scores = simd.add_f64s(scores, density);
                    simd.partial_store_f64s(&mut component_scores[start..start + lanes], scores);
                }
                for (component, score) in component_scores
                    .iter_mut()
                    .enumerate()
                    .skip(vector_components)
                {
                    let z = (values[candidate * dimensions.len() + dimension]
                        - kernels.means[component])
                        * kernels.inverse_sigmas[component];
                    *score += kernels.log_coefficients[component] - 0.5 * z * z;
                }
            }

            let maximum = component_scores
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            #[cfg(target_arch = "x86_64")]
            let total = {
                let maximum_vector = simd.splat_f64s(maximum);
                let mut sum = simd.splat_f64s(0.0);
                for start in (0..vector_components).step_by(lanes) {
                    let scores = simd.partial_load_f64s(&component_scores[start..start + lanes]);
                    let delta = simd.sub_f64s(scores, maximum_vector);
                    sum = simd.add_f64s(sum, exp_non_positive(simd, delta));
                }
                simd.reduce_sum_f64s(sum)
                    + component_scores[vector_components..]
                        .iter()
                        .map(|score| (score - maximum).exp())
                        .sum::<f64>()
            };
            #[cfg(not(target_arch = "x86_64"))]
            let total = component_scores
                .iter()
                .map(|score| (score - maximum).exp())
                .sum::<f64>();
            output[candidate] = maximum + total.ln();
        }
    }
}

/// Vector exponential specialized for the log-sum-exp domain `x <= 0`.
///
/// Range reduction follows `exp(x) = 2^k exp(r)` with a degree-13 Taylor
/// polynomial on `|r| <= ln(2)/2`. Values outside the normal-result range are
/// exceptional in Parzen scoring and use the platform scalar implementation.
/// Range-reduction structure and split-ln(2) constants follow the freely
/// licensed musl `exp` implementation; the polynomial and SIMD formulation here
/// are Parzen-specific. See <https://git.musl-libc.org/cgit/musl/tree/src/math/exp.c>.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
fn exp_non_positive<S: Simd>(simd: S, x: S::f64s) -> S::f64s {
    const LOG2_E: f64 = std::f64::consts::LOG2_E;
    const LN2_HI: f64 = 6.931_471_803_691_238e-1;
    const LN2_LO: f64 = 1.908_214_929_270_587_7e-10;
    const ROUNDING_MAGIC: f64 = 6_755_399_441_055_744.0;
    const COEFFICIENTS: [f64; 14] = [
        1.0,
        1.0,
        0.5,
        1.0 / 6.0,
        1.0 / 24.0,
        1.0 / 120.0,
        1.0 / 720.0,
        1.0 / 5_040.0,
        1.0 / 40_320.0,
        1.0 / 362_880.0,
        1.0 / 3_628_800.0,
        1.0 / 39_916_800.0,
        1.0 / 479_001_600.0,
        1.0 / 6_227_020_800.0,
    ];

    let biased = simd.mul_add_f64s(x, simd.splat_f64s(LOG2_E), simd.splat_f64s(ROUNDING_MAGIC));
    let exponent = simd.sub_f64s(biased, simd.splat_f64s(ROUNDING_MAGIC));
    let reduced = simd.sub_f64s(
        simd.sub_f64s(x, simd.mul_f64s(exponent, simd.splat_f64s(LN2_HI))),
        simd.mul_f64s(exponent, simd.splat_f64s(LN2_LO)),
    );
    let exponent_bits = simd.sub_u64s(
        simd.transmute_u64s_f64s(biased),
        simd.splat_u64s(ROUNDING_MAGIC.to_bits()),
    );
    let power_bits = simd.mul_u64s(
        simd.add_u64s(exponent_bits, simd.splat_u64s(1_023)),
        simd.splat_u64s(1_u64 << 52),
    );
    let mut polynomial = simd.splat_f64s(COEFFICIENTS[13]);
    for coefficient in COEFFICIENTS[..13].iter().rev() {
        polynomial = simd.mul_add_f64s(polynomial, reduced, simd.splat_f64s(*coefficient));
    }
    let result = simd.mul_f64s(polynomial, simd.transmute_f64s_u64s(power_bits));

    let lanes = S::F64_LANES;
    let mut input = [0.0; 8];
    let mut output = [0.0; 8];
    debug_assert!(lanes <= input.len());
    simd.partial_store_f64s(&mut input[..lanes], x);
    simd.partial_store_f64s(&mut output[..lanes], result);
    for lane in 0..lanes {
        if !input[lane].is_finite() || !(-700.0..=0.0).contains(&input[lane]) {
            output[lane] = input[lane].exp();
        }
    }
    simd.partial_load_f64s(&output[..lanes])
}

#[cfg(all(test, target_arch = "x86_64"))]
pub(super) fn test_negative_exp(values: &[f64]) -> Vec<f64> {
    struct Exp<'a>(&'a [f64]);
    impl WithSimd for Exp<'_> {
        type Output = Vec<f64>;

        fn with_simd<S: Simd>(self, simd: S) -> Vec<f64> {
            let lanes = S::F64_LANES;
            let vector_values = self.0.len() / lanes * lanes;
            let mut output = vec![0.0; self.0.len()];
            for start in (0..vector_values).step_by(lanes) {
                let values = simd.partial_load_f64s(&self.0[start..start + lanes]);
                simd.partial_store_f64s(
                    &mut output[start..start + lanes],
                    exp_non_positive(simd, values),
                );
            }
            for (result, value) in output[vector_values..]
                .iter_mut()
                .zip(&self.0[vector_values..])
            {
                *result = value.exp();
            }
            output
        }
    }
    Arch::new().dispatch(Exp(values))
}
