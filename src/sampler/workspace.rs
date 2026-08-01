// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

#[derive(Default)]
pub(crate) struct AcquisitionWorkspace {
    pub good_scores: Vec<f64>,
    pub bad_scores: Vec<f64>,
}
