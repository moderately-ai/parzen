// Copyright 2026 Thomas Santerre and Moderately AI Inc.
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{
    cmp::Ordering,
    collections::{BTreeSet, BinaryHeap, VecDeque},
};

use crate::{Direction, SearchSpace, TrialId, storage::TrialStorage};

use super::prepared::{PreparedParam, PreparedValue};

#[derive(Debug, Clone, Copy)]
pub(crate) struct RankKey {
    pub trial: TrialId,
    value: f64,
    direction: Direction,
    tie: u64,
}

impl RankKey {
    pub(crate) fn new(trial: TrialId, value: f64, direction: Direction, seed: u64) -> Self {
        Self {
            trial,
            value,
            direction,
            tie: stable_key(seed, trial.0),
        }
    }
}

impl PartialEq for RankKey {
    fn eq(&self, other: &Self) -> bool {
        self.trial == other.trial
    }
}
impl Eq for RankKey {}
impl PartialOrd for RankKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for RankKey {
    fn cmp(&self, other: &Self) -> Ordering {
        let objective = match self.direction {
            Direction::Minimize => self.value.total_cmp(&other.value),
            Direction::Maximize => other.value.total_cmp(&self.value),
        };
        objective
            .then_with(|| self.tie.cmp(&other.tie))
            .then_with(|| self.trial.cmp(&other.trial))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReservoirEntry {
    priority: u64,
    rank: RankKey,
}

impl PartialOrd for ReservoirEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ReservoirEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority
            .cmp(&other.priority)
            .then_with(|| self.rank.trial.cmp(&other.rank.trial))
    }
}

pub(crate) struct BoundedHistory {
    seen: usize,
    generation: u64,
    max_good: usize,
    max_bad: usize,
    recent_capacity: usize,
    reservoir_capacity: usize,
    seed: u64,
    top: Vec<RankKey>,
    recent: VecDeque<RankKey>,
    reservoir: BinaryHeap<ReservoirEntry>,
}

pub(crate) struct HistoryWorkspace {
    pub good_trials: Vec<TrialId>,
    bad_ranks: Vec<RankKey>,
    pub bad_trials: Vec<TrialId>,
}

impl HistoryWorkspace {
    pub(crate) fn new(max_good: usize, max_bad: usize) -> Self {
        Self {
            good_trials: Vec::with_capacity(max_good),
            bad_ranks: Vec::with_capacity(max_bad.saturating_add(1)),
            bad_trials: Vec::with_capacity(max_bad),
        }
    }
}

impl BoundedHistory {
    pub(crate) fn new(max_good: usize, max_bad: usize, recent: usize, seed: u64) -> Self {
        Self {
            seen: 0,
            generation: 0,
            max_good,
            max_bad,
            recent_capacity: recent,
            reservoir_capacity: max_bad - recent - max_good.saturating_sub(1),
            seed,
            top: Vec::with_capacity(max_good),
            recent: VecDeque::with_capacity(recent),
            reservoir: BinaryHeap::new(),
        }
    }

    pub(crate) fn insert(&mut self, rank: RankKey) {
        self.seen += 1;
        self.generation = self.generation.wrapping_add(1);
        let position = self.top.binary_search(&rank).unwrap_or_else(|index| index);
        if position < self.max_good {
            self.top.insert(position, rank);
            if self.top.len() > self.max_good
                && let Some(displaced) = self.top.pop()
            {
                self.offer_reservoir(displaced);
            }
        } else {
            self.push_recent(rank);
        }
    }

    fn push_recent(&mut self, rank: RankKey) {
        if self.recent_capacity == 0 {
            self.offer_reservoir(rank);
            return;
        }
        self.recent.push_back(rank);
        if self.recent.len() > self.recent_capacity
            && let Some(expired) = self.recent.pop_front()
        {
            self.offer_reservoir(expired);
        }
    }

    fn offer_reservoir(&mut self, rank: RankKey) {
        if self.reservoir_capacity == 0 {
            return;
        }
        let candidate = ReservoirEntry {
            priority: stable_key(self.seed ^ 0xa076_1d64_78bd_642f, rank.trial.0),
            rank,
        };
        if self.reservoir.len() < self.reservoir_capacity {
            self.reservoir.push(candidate);
        } else if self
            .reservoir
            .peek()
            .is_some_and(|largest| candidate < *largest)
        {
            self.reservoir.pop();
            self.reservoir.push(candidate);
        }
    }

    pub(crate) fn split_into(&self, good_count: usize, workspace: &mut HistoryWorkspace) {
        let good_count = good_count.min(self.top.len());
        workspace.good_trials.clear();
        workspace
            .good_trials
            .extend(self.top[..good_count].iter().map(|rank| rank.trial));

        workspace.bad_ranks.clear();
        workspace
            .bad_ranks
            .extend_from_slice(&self.top[good_count..]);
        workspace.bad_ranks.extend(self.recent.iter().copied());
        workspace
            .bad_ranks
            .extend(self.reservoir.iter().map(|entry| entry.rank));
        workspace.bad_ranks.sort_unstable();
        workspace.bad_ranks.truncate(self.max_bad);

        workspace.bad_trials.clear();
        workspace
            .bad_trials
            .extend(workspace.bad_ranks.iter().map(|rank| rank.trial));
    }

    pub(crate) const fn seen(&self) -> usize {
        self.seen
    }
    pub(crate) const fn generation(&self) -> u64 {
        self.generation
    }
    pub(crate) fn retained(&self) -> usize {
        self.top.len() + self.recent.len() + self.reservoir.len()
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.seen == 0
    }
    pub(crate) fn retained_trials(&self) -> impl Iterator<Item = TrialId> + '_ {
        self.top
            .iter()
            .map(|rank| rank.trial)
            .chain(self.recent.iter().map(|rank| rank.trial))
            .chain(self.reservoir.iter().map(|entry| entry.rank.trial))
    }
}

pub(crate) struct FullHistory {
    ranks: BTreeSet<RankKey>,
    params: Vec<PreparedParam>,
    columns: Vec<PreparedColumn>,
}

enum PreparedColumn {
    Continuous(Vec<Option<f64>>),
    Discrete(Vec<Option<PreparedDiscreteValue>>),
    Categorical(Vec<Option<u32>>),
}

#[derive(Clone, Copy)]
struct PreparedDiscreteValue {
    grid_index: u64,
    transformed: f64,
}

impl FullHistory {
    pub(crate) fn new(space: &SearchSpace) -> Result<Self, crate::ParzenError> {
        let params = (0..space.parameters.len())
            .map(|index| PreparedParam::new(crate::ParamId(index as u32), index, space))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            ranks: BTreeSet::new(),
            columns: params
                .iter()
                .map(|param| match param.kernel {
                    super::prepared::PreparedKernel::Continuous(_) => {
                        PreparedColumn::Continuous(Vec::new())
                    }
                    super::prepared::PreparedKernel::Discrete(_) => {
                        PreparedColumn::Discrete(Vec::new())
                    }
                    super::prepared::PreparedKernel::Categorical(_) => {
                        PreparedColumn::Categorical(Vec::new())
                    }
                })
                .collect(),
            params,
        })
    }

    pub(crate) fn insert(&mut self, rank: RankKey, storage: &TrialStorage) {
        let trial_index = rank.trial.0 as usize;
        for (param, column) in self.params.iter().zip(&mut self.columns) {
            let value = storage
                .typed_value(rank.trial, param.id, &param.distribution)
                .and_then(|value| param.prepare_canonical(value));
            column.set(trial_index, value);
        }
        self.ranks.insert(rank);
    }
    pub(crate) fn iter(&self) -> impl Iterator<Item = TrialId> + '_ {
        self.ranks.iter().map(|rank| rank.trial)
    }
    pub(crate) fn len(&self) -> usize {
        self.ranks.len()
    }
    pub(crate) fn typed_value(
        &self,
        trial: TrialId,
        param: crate::ParamId,
    ) -> Option<PreparedValue> {
        self.columns
            .get(param.0 as usize)
            .and_then(|column| column.get(trial.0 as usize))
    }
}

impl PreparedColumn {
    fn set(&mut self, index: usize, value: Option<PreparedValue>) {
        match self {
            Self::Continuous(column) => {
                set_column_value(
                    column,
                    index,
                    value.and_then(|value| match value {
                        PreparedValue::Continuous(value) => Some(value),
                        _ => None,
                    }),
                );
            }
            Self::Discrete(column) => {
                set_column_value(
                    column,
                    index,
                    value.and_then(|value| match value {
                        PreparedValue::Discrete {
                            grid_index,
                            transformed,
                        } => Some(PreparedDiscreteValue {
                            grid_index,
                            transformed,
                        }),
                        _ => None,
                    }),
                );
            }
            Self::Categorical(column) => {
                set_column_value(
                    column,
                    index,
                    value.and_then(|value| match value {
                        PreparedValue::Categorical(value) => Some(value),
                        _ => None,
                    }),
                );
            }
        }
    }

    fn get(&self, index: usize) -> Option<PreparedValue> {
        match self {
            Self::Continuous(column) => column
                .get(index)
                .copied()
                .flatten()
                .map(PreparedValue::Continuous),
            Self::Discrete(column) => {
                column
                    .get(index)
                    .copied()
                    .flatten()
                    .map(|value| PreparedValue::Discrete {
                        grid_index: value.grid_index,
                        transformed: value.transformed,
                    })
            }
            Self::Categorical(column) => column
                .get(index)
                .copied()
                .flatten()
                .map(PreparedValue::Categorical),
        }
    }
}

fn set_column_value<T>(column: &mut Vec<Option<T>>, index: usize, value: Option<T>) {
    if column.len() < index {
        column.resize_with(index, || None);
    }
    if column.len() == index {
        column.push(value);
    } else {
        column[index] = value;
    }
}

pub(crate) fn stable_key(seed: u64, value: u64) -> u64 {
    let mut value = value ^ seed.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Distribution, FloatDistribution, ParamValue};

    fn reference_split(
        history: &BoundedHistory,
        good_count: usize,
    ) -> (Vec<TrialId>, Vec<TrialId>) {
        let good_count = good_count.min(history.top.len());
        let good = history.top[..good_count]
            .iter()
            .map(|rank| rank.trial)
            .collect();
        let mut bad_ranks: Vec<RankKey> = history.top[good_count..].to_vec();
        bad_ranks.extend(history.recent.iter().copied());
        bad_ranks.extend(history.reservoir.iter().map(|entry| entry.rank));
        bad_ranks.sort_unstable();
        bad_ranks.truncate(history.max_bad);
        let bad = bad_ranks.into_iter().map(|rank| rank.trial).collect();
        (good, bad)
    }

    #[test]
    fn bounded_history_retains_exact_top_and_constant_capacity() {
        let mut history = BoundedHistory::new(3, 8, 2, 9);
        for id in 0..1_000_000 {
            history.insert(RankKey::new(TrialId(id), id as f64, Direction::Minimize, 9));
        }
        let mut workspace = HistoryWorkspace::new(3, 8);
        history.split_into(2, &mut workspace);
        assert_eq!(workspace.good_trials, vec![TrialId(0), TrialId(1)]);
        assert!(workspace.bad_trials.len() <= 8);
        assert!(history.retained() <= 11);
    }

    #[test]
    fn reusable_split_matches_allocating_reference() {
        let mut history = BoundedHistory::new(25, 512, 64, 23);
        let mut workspace = HistoryWorkspace::new(25, 512);
        for id in 0..10_000 {
            let objective = ((id * 7919) % 1009) as f64;
            history.insert(RankKey::new(
                TrialId(id),
                objective,
                Direction::Minimize,
                23,
            ));
            if matches!(id, 0 | 1 | 9 | 99 | 999 | 9_999) {
                for good_count in [0, 1, 5, 25, 50] {
                    let expected = reference_split(&history, good_count);
                    history.split_into(good_count, &mut workspace);
                    assert_eq!(workspace.good_trials, expected.0);
                    assert_eq!(workspace.bad_trials, expected.1);
                    assert!(workspace.good_trials.capacity() <= 25);
                    assert!(workspace.bad_ranks.capacity() <= 513);
                    assert!(workspace.bad_trials.capacity() <= 512);
                }
            }
        }
    }

    #[test]
    fn retained_trial_iterator_visits_each_retained_trial_once() {
        let mut history = BoundedHistory::new(3, 8, 2, 9);
        assert!(history.is_empty());
        for id in 0..100 {
            history.insert(RankKey::new(TrialId(id), id as f64, Direction::Minimize, 9));
        }
        let retained: BTreeSet<_> = history.retained_trials().collect();
        assert_eq!(retained.len(), history.retained());
    }

    #[test]
    fn full_history_matches_total_order_for_both_directions() {
        for direction in [Direction::Minimize, Direction::Maximize] {
            let objectives = [3.0, -1.0, 3.0, 0.0, 7.0, -1.0];
            let space = SearchSpace::new();
            let mut history = FullHistory::new(&space).unwrap();
            let mut reference = Vec::new();
            for (id, objective) in objectives.into_iter().enumerate() {
                let rank = RankKey::new(TrialId(id as u64), objective, direction, 19);
                history.ranks.insert(rank);
                reference.push(rank);
            }
            reference.sort_unstable();
            assert_eq!(
                history.iter().collect::<Vec<_>>(),
                reference.iter().map(|rank| rank.trial).collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn full_history_stores_transformed_values_in_typed_columns() {
        let mut space = SearchSpace::new();
        let linear = space
            .add(
                "linear",
                Distribution::Float(FloatDistribution::linear(-2.0, 2.0).unwrap()),
            )
            .unwrap();
        let log = space
            .add(
                "log",
                Distribution::Float(FloatDistribution::log(0.01, 100.0).unwrap()),
            )
            .unwrap();
        let mut storage = TrialStorage::default();
        let trial = storage
            .push(
                &[
                    (linear, ParamValue::Float(1.5)),
                    (log, ParamValue::Float(1.0)),
                ],
                3.0,
            )
            .unwrap();
        let mut history = FullHistory::new(&space).unwrap();
        history.insert(RankKey::new(trial, 3.0, Direction::Minimize, 17), &storage);

        assert_eq!(
            history.typed_value(trial, linear),
            Some(PreparedValue::Continuous(1.5))
        );
        assert_eq!(
            history.typed_value(trial, log),
            Some(PreparedValue::Continuous(0.0))
        );
        assert!(std::mem::size_of::<Option<f64>>() <= std::mem::size_of::<Option<ParamValue>>());
    }
}
