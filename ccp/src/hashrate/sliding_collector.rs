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

pub(crate) type SlidingHashrate = HashMap<LogicalCoreId, SlidingThreadHashrate>;

#[derive(Clone, Debug, Default)]
pub(crate) struct SlidingThreadHashrate {
    pub(crate) window_10: SlidingWindow<10>,
    pub(crate) window_60: SlidingWindow<60>,
    pub(crate) window_900: SlidingWindow<900>,
}

/// Intended for debugging purpose, collect hashrate if run with the --report-hashrate flag.
#[derive(Clone, Debug, Default)]
pub(crate) struct SlidingHashrateCollector {
    hashrate: SlidingHashrate,
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

impl SlidingHashrateCollector {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn account_record(&mut self, record: ThreadHashrateRecord) {
        let hashes_count = match record.variant {
            HashrateRecordType::CheckedHashes { count } => count as u64,
            _ => return,
        };

        self.hashrate
            .entry(record.core_id)
            .or_default()
            .account_record(hashes_count, record.duration);
    }

    pub(crate) fn hashrate(&self) -> &SlidingHashrate {
        &self.hashrate
    }
}

impl<const SECS: u64> SlidingWindow<SECS> {
    pub(self) fn new() -> Self {
        let window_size = Duration::from_secs(SECS);
        Self {
            records: VecDeque::new(),
            window_size,
        }
    }

    pub(self) fn account_record(&mut self, hashes_count: u64, duration: Duration) {
        let current_time = Instant::now();
        self.prune_old(current_time);

        let record = SlidingWindowRecord::new(current_time, hashes_count, duration);
        self.records.push_front(record);
    }

    pub(crate) fn compute_hashrate(&self) -> f64 {
        use super::hashratable::Hashratable;

        let mut overall_hashes_found = 0;
        let mut overall_duration = Duration::default();

        for record in self.records.iter() {
            overall_hashes_found += record.checked_hashes_count;
            overall_duration += record.duration;
        }

        super::hashratable::HashrateCalculator::hashrate(overall_hashes_found, overall_duration)
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

impl<const SECS: u64> Default for SlidingWindow<SECS> {
    fn default() -> Self {
        Self::new()
    }
}

impl SlidingWindowRecord {
    pub(self) fn new(time: Instant, checked_hashes_count: u64, duration: Duration) -> Self {
        Self {
            time,
            checked_hashes_count,
            duration,
        }
    }
}

impl SlidingThreadHashrate {
    pub(crate) fn account_record(&mut self, hashes_count: u64, duration: Duration) {
        self.window_10.account_record(hashes_count, duration);
        self.window_60.account_record(hashes_count, duration);
        self.window_900.account_record(hashes_count, duration);
    }
}
