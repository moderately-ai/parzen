// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Packed structure-of-arrays trial storage.

use std::ops::Range;

use crate::{Distribution, ParamValue, TrialId, search_space::ParamId};

#[derive(Debug, Clone, Copy)]
pub(crate) struct TrialHeader {
    pub value: f64,
    pub start: usize,
    pub len: u32,
}

#[derive(Debug, Default)]
pub(crate) struct TrialStorage {
    headers: Vec<TrialHeader>,
    pub(crate) param_ids: Vec<ParamId>,
    values: Vec<u64>,
}

impl TrialStorage {
    pub fn push(&mut self, params: &[(ParamId, ParamValue)], value: f64) -> Option<TrialId> {
        let id = TrialId(u64::try_from(self.headers.len()).ok()?);
        let len = u32::try_from(params.len()).ok()?;
        let start = self.param_ids.len();
        self.param_ids.reserve(params.len());
        self.values.reserve(params.len());
        for (param, value) in params {
            self.param_ids.push(*param);
            self.values.push(value.encode());
        }
        self.headers.push(TrialHeader { value, start, len });
        Some(id)
    }
    pub fn len(&self) -> usize {
        self.headers.len()
    }
    pub fn header(&self, id: TrialId) -> TrialHeader {
        self.headers[id.0 as usize]
    }
    pub fn range(&self, id: TrialId) -> Range<usize> {
        let header = self.header(id);
        header.start..header.start + header.len as usize
    }
    pub fn typed_value(
        &self,
        trial: TrialId,
        param: ParamId,
        distribution: &Distribution,
    ) -> Option<ParamValue> {
        let range = self.range(trial);
        let relative = self.param_ids[range.clone()].binary_search(&param).ok()?;
        Some(self.decode_at(range.start + relative, distribution))
    }
    pub fn decode_at(&self, index: usize, distribution: &Distribution) -> ParamValue {
        match distribution {
            Distribution::Categorical(_) => ParamValue::Categorical(self.values[index] as u32),
            Distribution::Float(_) => ParamValue::Float(f64::from_bits(self.values[index])),
            Distribution::Int(_) => ParamValue::Int(self.values[index] as i64),
        }
    }
    pub fn capacity_bytes(&self) -> usize {
        self.headers.capacity() * std::mem::size_of::<TrialHeader>()
            + self.param_ids.capacity() * std::mem::size_of::<ParamId>()
            + self.values.capacity() * std::mem::size_of::<u64>()
    }
}
