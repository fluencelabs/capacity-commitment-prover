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

use std::collections::HashMap;
use std::collections::VecDeque;
use std::time::Duration;
use std::time::Instant;

use ccp_shared::types::LogicalCoreId;

use super::record::ThreadHashrateRecord;
use crate::hashrate::record::HashrateRecordType;

pub(crate) type SlidingHashrate<const SECS: u64> = HashMap<LogicalCoreId, SlidingWindow<SECS>>;

#[derive(Clone, Debug, Default)]
pub(crate) struct SlidingHashrateCollector<const SECS: u64> {
    sliding_hashrate: SlidingHashrate<SECS>,
}

#[derive(Clone, Debug)]
pub(crate) struct SlidingWindow<const SECS: u64> {
    records: VecDeque<SlidingWindowRecord>,
    window_size: Duration,
}

#[derive(Copy, Clone, Debug)]
struct SlidingWindowRecord {
    pub(self) time: Instant,
    pub(self) checked_hashes_count: u64,
    pub(self) duration: Duration,
}

impl<const SECS: u64> SlidingHashrateCollector<SECS> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn account_record(&mut self, record: ThreadHashrateRecord) {
        let hashes_count = match record.variant {
            HashrateRecordType::CheckedHashes {
                count: hashes_count,
            } => hashes_count as u64,
            _ => {
                return;
            }
        };

        add_record_to_hashmap(
            record.core_id,
            hashes_count,
            record.duration,
            &mut self.sliding_hashrate,
        );
    }

    pub(crate) fn sliding_hashrate(&self) -> &SlidingHashrate<SECS> {
        &self.sliding_hashrate
    }
}

impl<const SECS: u64> SlidingWindow<SECS> {
    pub(self) fn new() -> Self {
        Self {
            records: VecDeque::new(),
            window_size: Duration::from_secs(SECS),
        }
    }

    pub(self) fn account_record(&mut self, hashes_count: u64, duration: Duration) {
        let current_time = Instant::now();
        self.prune_old(current_time);

        let record = SlidingWindowRecord::new(current_time, hashes_count, duration);
        self.records.push_front(record);
    }

    pub(crate) fn compute_hashrate(&self) -> f64 {
        let mut overall_hashes_found = 0;
        let mut overall_duration = Duration::default();

        for record in self.records.iter() {
            overall_hashes_found += record.checked_hashes_count;
            overall_duration += record.duration;
        }

        overall_hashes_found as f64 / overall_duration.as_secs_f64()
    }

    fn prune_old(&mut self, current_time: Instant) {
        let last_account_time = match current_time.checked_sub(self.window_size) {
            Some(time) => time,
            None => return,
        };

        while let Some(record) = self.records.back() {
            if record.time < last_account_time {
                self.records.pop_back();
            } else {
                break;
            }
        }
    }
}

impl SlidingWindowRecord {
    pub(self) fn new(time: Instant, hashes_count: u64, duration: Duration) -> Self {
        Self {
            time,
            checked_hashes_count: hashes_count,
            duration,
        }
    }
}

fn add_record_to_hashmap<const SECS: u64>(
    core_id: LogicalCoreId,
    hashes_count: u64,
    duration: Duration,
    running_average: &mut HashMap<LogicalCoreId, SlidingWindow<SECS>>,
) {
    use std::collections::hash_map::Entry;

    match running_average.entry(core_id) {
        Entry::Vacant(entry) => {
            let mut new_average = SlidingWindow::new();
            new_average.account_record(hashes_count, duration);
            entry.insert(new_average);
        }
        Entry::Occupied(entry) => entry.into_mut().account_record(hashes_count, duration),
    }
}

use std::fmt;
use std::fmt::Formatter;

impl<const SECS: u64> fmt::Display for SlidingHashrateCollector<SECS> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (core, average) in self.sliding_hashrate.iter() {
            writeln!(f, "core {} - {}", core, average.compute_hashrate())?;
        }

        Ok(())
    }
}
