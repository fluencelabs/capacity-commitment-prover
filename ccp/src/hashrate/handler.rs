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
use std::sync::Arc;
use std::sync::Mutex;

use super::HResult;
use super::HashrateCollector;
use super::HashrateSaver;
use super::SlidingHashrateCollector;
use super::ThreadHashrateRecord;
use crate::hashrate::collector::EpochObservation;
use crate::hashrate::sliding_collector::SlidingHashrate;

pub(crate) struct HashrateHandler {
    collector: Arc<Mutex<HashrateCollector>>,
    instant_hashrate_enabled: bool,
    sliding_collector: SlidingHashrateCollector,
    saver: HashrateSaver,
}

impl HashrateHandler {
    pub(crate) fn new(
        collector: Arc<Mutex<HashrateCollector>>,
        state_dir_path: PathBuf,
        instant_hashrate_enabled: bool,
    ) -> HResult<Self> {
        let sliding_collector = SlidingHashrateCollector::new();
        let saver = HashrateSaver::from_directory(state_dir_path)?;

        let handler = Self {
            collector,
            sliding_collector,
            instant_hashrate_enabled,
            saver,
        };

        Ok(handler)
    }

    pub(crate) fn account_record(&mut self, record: ThreadHashrateRecord) -> HResult<()> {
        let mut guard = self.collector.lock().unwrap();

        if let EpochObservation::EpochChanged {
            prev_epoch_hashrate,
        } = guard.account_record(record)
        {
            self.saver.save_hashrate_previous(prev_epoch_hashrate)?;
            self.saver.cleanup_sliding_hashrate()?;
        }

        self.sliding_collector.account_record(record);
        if self.instant_hashrate_enabled {
            self.saver.save_hashrate_entry(&record)?;
        }

        Ok(())
    }

    pub(crate) fn proof_found(&mut self, core_id: LogicalCoreId) {
        let mut guard = self.collector.lock().unwrap();
        guard.proof_found(core_id)
    }

    pub(crate) fn handle_cum_tick(&self) -> HResult<()> {
        let guard = self.collector.lock().unwrap();
        let hashrate = guard.collect();
        self.saver.save_hashrate_current(hashrate)
    }

    #[allow(dead_code)]
    pub(crate) fn sliding_hashrate(&self) -> &SlidingHashrate {
        self.sliding_collector.hashrate()
    }
}
