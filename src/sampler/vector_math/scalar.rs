// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::ContinuousDimension;

pub(crate) fn continuous_log_pdf_batch(
    values: &[f64],
    candidates: usize,
    dimensions: &[ContinuousDimension<'_>],
    log_weights: &[f64],
    output: &mut [f64],
    component_scores: &mut [f64],
) {
    for candidate in 0..candidates {
        component_scores.copy_from_slice(log_weights);
        for (dimension, kernels) in dimensions.iter().enumerate() {
            let value = values[candidate * dimensions.len() + dimension];
            for (component, score) in component_scores.iter_mut().enumerate() {
                let z = (value - kernels.means[component]) * kernels.inverse_sigmas[component];
                *score += kernels.log_coefficients[component] - 0.5 * z * z;
            }
        }
        let maximum = component_scores
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let sum = component_scores
            .iter()
            .map(|score| (score - maximum).exp())
            .sum::<f64>();
        output[candidate] = maximum + sum.ln();
    }
}
