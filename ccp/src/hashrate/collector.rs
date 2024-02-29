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

use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::Instant;

use ccp_shared::types::EpochParameters;
use ccp_shared::types::LogicalCoreId;

use super::record::HashrateRecordType;
use super::record::ThreadHashrateRecord;

#[derive(Clone, Debug, Default)]
pub(crate) struct HashrateCollector {
    status: CollectorStatus,
    entries: HashMap<LogicalCoreId, ThreadHashrateRaw>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum CollectorStatus {
    Started {
        started_time: Instant,
        epoch: EpochParameters,
    },
    #[default]
    JustCreated,
}

pub(crate) type Hashrate = HashMap<LogicalCoreId, ThreadHashrate>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ThreadHashrateRaw {
    cache_creation: ParameterStatus<Duration>,
    dataset_initialization: ParameterStatus<Duration>,
    cc_job_duration: ParameterStatus<Duration>,
    hashes_checked_count: u64,
    proofs_found_count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ThreadHashrate {
    // hashes from epoch start, counts operations with cache and dataset
    pub(crate) effective_hashrate: f64,
    // pure hashrate, which doesn't count operations with cache and dataset
    pub(crate) hashrate: ParameterStatus<f64>,
    pub(crate) proofs_found: u64,
    pub(crate) cache_creation: ParameterStatus<Duration>,
    pub(crate) dataset_initialization: ParameterStatus<Duration>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ParameterStatus<T> {
    Measured(T),
    #[default]
    NotMeasured,
}

#[derive(Clone, Debug, Default)]
pub(crate) enum EpochObservation {
    EpochChanged {
        prev_epoch_hashrate: Hashrate,
    },
    #[default]
    EpochNotChanged,
    StartedWorking,
}

impl HashrateCollector {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn account_record(
        &mut self,
        hashrate_record: ThreadHashrateRecord,
    ) -> EpochObservation {
        use std::collections::hash_map::Entry;

        let result = self.observe_epoch(hashrate_record.epoch);

        match self.entries.entry(hashrate_record.core_id) {
            Entry::Vacant(entry) => {
                let mut raw_hashrate = ThreadHashrateRaw::default();
                raw_hashrate.account_record(hashrate_record);
                entry.insert(raw_hashrate);
            }
            Entry::Occupied(entry) => entry.into_mut().account_record(hashrate_record),
        };

        result
    }

    pub(crate) fn collect(&self) -> HashMap<LogicalCoreId, ThreadHashrate> {
        let epoch_duration = match self.status {
            CollectorStatus::Started { started_time, .. } => started_time.elapsed(),
            CollectorStatus::JustCreated => return HashMap::new(),
        };

        let epoch_duration = epoch_duration.as_secs_f64();

        self.entries
            .iter()
            .map(|(&core_id, info)| {
                let hashrate = info
                    .cc_job_duration
                    .map(|duration| info.hashes_checked_count as f64 / duration.as_secs_f64());

                let statistics = ThreadHashrate {
                    effective_hashrate: info.hashes_checked_count as f64 / epoch_duration,
                    hashrate,
                    proofs_found: info.proofs_found_count,
                    cache_creation: info.cache_creation,
                    dataset_initialization: info.dataset_initialization,
                };

                (core_id, statistics)
            })
            .collect::<HashMap<_, _>>()
    }

    pub(crate) fn proof_found(&mut self, core_id: LogicalCoreId) {
        use std::collections::hash_map::Entry;

        match self.entries.entry(core_id) {
            Entry::Vacant(entry) => {
                let mut raw_hashrate = ThreadHashrateRaw::default();
                raw_hashrate.account_proof_found();
                entry.insert(raw_hashrate);
            }
            Entry::Occupied(entry) => entry.into_mut().account_proof_found(),
        };
    }

    fn observe_epoch(&mut self, new_epoch: EpochParameters) -> EpochObservation {
        match self.status {
            CollectorStatus::JustCreated => {
                self.handler_new_epoch(new_epoch);
                EpochObservation::StartedWorking
            }
            CollectorStatus::Started { epoch, .. } => {
                if epoch == new_epoch {
                    return EpochObservation::EpochNotChanged;
                }

                let prev_epoch_hashrate = self.collect();
                self.handler_new_epoch(new_epoch);
                EpochObservation::EpochChanged {
                    prev_epoch_hashrate,
                }
            }
        }
    }

    fn handler_new_epoch(&mut self, epoch: EpochParameters) {
        self.status = CollectorStatus::Started {
            started_time: Instant::now(),
            epoch,
        };

        self.entries.clear();
    }
}

impl ThreadHashrateRaw {
    pub(self) fn account_record(&mut self, new_entry: ThreadHashrateRecord) {
        match new_entry.variant {
            HashrateRecordType::CacheCreation => {
                self.cache_creation = ParameterStatus::Measured(new_entry.duration)
            }
            HashrateRecordType::DatasetInitialization { .. } => {
                self.dataset_initialization = ParameterStatus::Measured(new_entry.duration)
            }
            HashrateRecordType::HashesChecked { hashes_count } => {
                self.cc_job_duration
                    .map(|duration| duration + new_entry.duration);
                self.hashes_checked_count += hashes_count as u64
            }
        }
    }

    pub(crate) fn account_proof_found(&mut self) {
        self.proofs_found_count += 1;
    }
}

impl<T> ParameterStatus<T> {
    pub fn map<U, F>(self, f: F) -> ParameterStatus<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            ParameterStatus::Measured(value) => ParameterStatus::Measured(f(value)),
            ParameterStatus::NotMeasured => ParameterStatus::NotMeasured,
        }
    }
}
