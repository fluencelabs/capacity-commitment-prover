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

use ccp_shared::types::LogicalCoreId;
use std::path::PathBuf;

use super::HResult;
use super::HashrateCollector;
use super::HashrateSaver;
use super::SlidingHashrateCollector;
use super::ThreadHashrateRecord;
use crate::hashrate::collector::EpochObservation;

pub(crate) struct HashrateHandler<const SECS: u64> {
    collector: HashrateCollector,
    sliding_enabled: bool,
    sliding_collector: SlidingHashrateCollector<SECS>,
    saver: HashrateSaver,
}

impl<const SECS: u64> HashrateHandler<SECS> {
    pub(crate) fn new(state_dir_path: PathBuf, sliding_enabled: bool) -> HResult<Self> {
        let collector = HashrateCollector::new();
        let sliding_collector = SlidingHashrateCollector::new();
        let saver = HashrateSaver::from_directory(state_dir_path)?;

        let handler = Self {
            collector,
            sliding_collector,
            sliding_enabled,
            saver,
        };

        Ok(handler)
    }

    pub(crate) fn account_record(&mut self, record: ThreadHashrateRecord) -> HResult<()> {
        if let EpochObservation::EpochChanged { prev_epoch_hashrate} = self.collector.account_record(record) {
            self.saver.save_hashrate_previous(prev_epoch_hashrate)?;
            self.saver.cleanup_sliding_hashrate()?;
        }

        self.sliding_collector.account_record(record);
        Ok(())
    }

    pub(crate) fn proof_found(&mut self, core_id: LogicalCoreId) {
        self.collector.proof_found(core_id)
    }

    pub(crate) fn handle_cum_tick(&self) -> HResult<()> {
        let hashrate = self.collector.collect();
        self.saver.save_hashrate_current(hashrate)
    }

    pub(crate) fn handle_instant_tick(&self) -> HResult<()> {
        if self.sliding_enabled {
            let sliding_hashrate = self.sliding_collector.sliding_hashrate();
            self.saver.save_sliding_hashrate(sliding_hashrate)?;
        }

        Ok(())
    }
}
