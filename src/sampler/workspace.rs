// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::history::HistoryWorkspace;
use crate::ParamValue;

#[derive(Default)]
pub(crate) struct CandidateBatch {
    pub values: Vec<ParamValue>,
    pub good_scores: Vec<f64>,
    pub bad_scores: Vec<f64>,
    pub candidates: usize,
    pub dimensions: usize,
}

impl CandidateBatch {
    pub(crate) fn clear(&mut self, candidates: usize, dimensions: usize) {
        self.values.clear();
        self.good_scores.clear();
        self.bad_scores.clear();
        self.candidates = candidates;
        self.dimensions = dimensions;
        self.values.reserve(candidates.saturating_mul(dimensions));
        self.good_scores.reserve(candidates);
        self.bad_scores.reserve(candidates);
    }

    pub(crate) fn previous_duplicate(&self, index: usize) -> Option<usize> {
        if self.dimensions != 1 || index >= self.candidates {
            return None;
        }
        let value = self.values[index];
        self.values[..index]
            .iter()
            .position(|candidate| *candidate == value)
    }
}

pub(crate) struct AcquisitionWorkspace {
    pub good_components: Vec<f64>,
    pub bad_components: Vec<f64>,
    pub candidates: CandidateBatch,
    pub history: HistoryWorkspace,
}

impl AcquisitionWorkspace {
    pub(crate) fn new(max_good: usize, max_bad: usize) -> Self {
        Self {
            good_components: Vec::new(),
            bad_components: Vec::new(),
            candidates: CandidateBatch::default(),
            history: HistoryWorkspace::new(max_good, max_bad),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_dimensional_batch_finds_previous_duplicate() {
        let mut batch = CandidateBatch::default();
        batch.clear(4, 1);
        batch.values.extend([
            ParamValue::Int(3),
            ParamValue::Int(7),
            ParamValue::Int(3),
            ParamValue::Int(3),
        ]);
        assert_eq!(batch.previous_duplicate(0), None);
        assert_eq!(batch.previous_duplicate(1), None);
        assert_eq!(batch.previous_duplicate(2), Some(0));
        assert_eq!(batch.previous_duplicate(3), Some(0));
    }
}
