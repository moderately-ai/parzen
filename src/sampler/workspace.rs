// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::history::HistoryWorkspace;

pub(crate) struct AcquisitionWorkspace {
    pub good_scores: Vec<f64>,
    pub bad_scores: Vec<f64>,
    pub history: HistoryWorkspace,
}

impl AcquisitionWorkspace {
    pub(crate) fn new(max_good: usize, max_bad: usize) -> Self {
        Self {
            good_scores: Vec::new(),
            bad_scores: Vec::new(),
            history: HistoryWorkspace::new(max_good, max_bad),
        }
    }
}
