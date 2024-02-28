/*
 * Copyright 2024 Fluence Labs Limited
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#![allow(dead_code)]

use std::collections::HashMap;
use std::collections::VecDeque;
use std::time::Duration;
use std::time::Instant;

use ccp_shared::types::LogicalCoreId;

use super::record::ThreadHashrateRecord;
use crate::hashrate::record::HashrateCUEntryVariant;

#[derive(Clone, Debug, Default)]
pub(crate) struct HashrateCollectorByTimer {
    running_average_10: HashMap<LogicalCoreId, RunningAverage<10>>,
    running_average_60: HashMap<LogicalCoreId, RunningAverage<60>>,
    running_average_900: HashMap<LogicalCoreId, RunningAverage<900>>,
}

#[derive(Clone, Debug)]
struct RunningAverage<const SECS: u64> {
    records: VecDeque<RunningAverageRecord>,
    window_size: Duration,
}

#[derive(Copy, Clone, Debug)]
struct RunningAverageRecord {
    pub(self) time: Instant,
    pub(self) hashes_count: u64,
    pub(self) duration: Duration,
}

impl HashrateCollectorByTimer {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn count_record(&mut self, record: ThreadHashrateRecord) {
        let hashes_count = match record.variant {
            HashrateCUEntryVariant::HashesChecked { hashes_count } => hashes_count as u64,
            _ => {
                return;
            }
        };

        add_record_to_hashmap(record.core_id, hashes_count, record.duration, &mut self.running_average_10);
        add_record_to_hashmap(record.core_id, hashes_count, record.duration, &mut self.running_average_60);
        add_record_to_hashmap(record.core_id, hashes_count, record.duration, &mut self.running_average_900);
    }
}

impl<const SECS: u64> RunningAverage<SECS> {
    pub(self) fn new() -> Self {
        Self {
            records: VecDeque::new(),
            window_size: Duration::from_secs(SECS),
        }
    }

    pub(self) fn count_record(&mut self, hashes_count: u64, duration: Duration) {
        let current_time = Instant::now();
        self.prune_old(current_time);

        let record = RunningAverageRecord::new(current_time, hashes_count, duration);
        self.records.push_front(record);
    }

    pub(self) fn compute_hashrate(&mut self) -> f64 {
        self.prune_old(Instant::now());

        let mut overall_hashes_found = 0;
        let mut overall_duration = Duration::default();

        for record in self.records.iter() {
            overall_hashes_found += record.hashes_count;
            overall_duration += record.duration;
        }

        overall_hashes_found as f64 / overall_duration.as_secs_f64()
    }

    fn prune_old(&mut self, current_time: Instant) {
        let last_count_time = match current_time.checked_sub(self.window_size) {
            Some(time) => time,
            None => return,
        };

        while let Some(last_record) = self.records.back() {
            if last_record.time < last_count_time {
                self.records.pop_back();
            }
        }
    }
}

impl RunningAverageRecord {
    pub(self) fn new(time: Instant, hashes_count: u64, duration: Duration) -> Self {
        Self {
            time,
            hashes_count,
            duration,
        }
    }
}

fn add_record_to_hashmap<const SECS: u64>(
    core_id: LogicalCoreId,
    hashes_count: u64,
    duration: Duration,
    running_average: &mut HashMap<LogicalCoreId, RunningAverage<SECS>>,
) {
    use std::collections::hash_map::Entry;

    match running_average.entry(core_id) {
        Entry::Vacant(entry) => {
            let mut new_average = RunningAverage::new();
            new_average.count_record(hashes_count, duration);
            entry.insert(new_average);
        }
        Entry::Occupied(entry) => entry.into_mut().count_record(hashes_count, duration),
    }
}
