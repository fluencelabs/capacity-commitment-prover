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

use super::hashrate_record::HashrateCUEntryVariant;
use super::hashrate_record::HashrateCURecord;

#[derive(Clone, Debug)]
pub(crate) struct HashrateCollector {
    status: CollectorStatus,
    entries: HashMap<LogicalCoreId, CUHashrateRaw>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum CollectorStatus {
    Started {
        started_time: Instant,
        epoch: EpochParameters,
    },
    JustCreated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CUHashrateRaw {
    cache_creation: ParameterStatus<Duration>,
    dataset_initialization: Vec<Duration>,
    pow_duration: ParameterStatus<Duration>,
    hashes_checked: u64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum ParameterStatus<T> {
    Measured(T),
    NotMeasured,
}

pub(crate) type Hashrate = HashMap<LogicalCoreId, CUHashrate>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct CUHashrate {
    // hashes from epoch start, counts operations with cache and dataset
    pub(crate) effective_hashrate: f64,
    // pure hashrate, which doesn't count operations with cache and dataset
    pub(crate) hashrate: ParameterStatus<f64>,
    pub(crate) cache_creation: ParameterStatus<Duration>,
    pub(crate) dataset_initialization: ParameterStatus<Duration>,
}

impl HashrateCollector {
    pub(crate) fn new() -> Self {
        Self {
            status: CollectorStatus::JustCreated,
            entries: HashMap::new(),
        }
    }

    pub(crate) fn count_entry(&mut self, hashrate_record: HashrateCURecord) -> Option<Hashrate> {
        use std::collections::hash_map::Entry;

        let status_changed = match self.status {
            CollectorStatus::JustCreated => true,
            CollectorStatus::Started { epoch, .. } => epoch != hashrate_record.epoch,
        };

        let result = if status_changed {
            let statistics = self.collect();
            self.new_epoch(hashrate_record.epoch);

            Some(statistics)
        } else {
            None
        };

        match self.entries.entry(hashrate_record.core_id) {
            Entry::Vacant(entry) => {
                entry.insert(CUHashrateRaw::from_single_entry(hashrate_record));
            }
            Entry::Occupied(entry) => entry.into_mut().add_entry(hashrate_record),
        };

        result
    }

    pub(crate) fn collect(&self) -> HashMap<LogicalCoreId, CUHashrate> {
        let epoch_duration = match self.status {
            CollectorStatus::Started { started_time, .. } => started_time.elapsed(),
            CollectorStatus::JustCreated => return HashMap::new(),
        };

        let epoch_duration = epoch_duration.as_secs_f64();

        self.entries
            .iter()
            .map(|(&core_id, info)| {
                let hashrate = match info.pow_duration {
                    ParameterStatus::Measured(duration) => {
                        let hashrate = info.hashes_checked as f64 / duration.as_secs_f64();
                        ParameterStatus::Measured(hashrate)
                    }
                    ParameterStatus::NotMeasured => ParameterStatus::NotMeasured,
                };

                let dataset_initialization = match info.dataset_initialization.iter().max() {
                    Some(&duration) => ParameterStatus::Measured(duration),
                    None => ParameterStatus::NotMeasured,
                };

                let statistics = CUHashrate {
                    effective_hashrate: info.hashes_checked as f64 / epoch_duration,
                    hashrate,
                    cache_creation: info.cache_creation,
                    dataset_initialization,
                };

                (core_id, statistics)
            })
            .collect::<HashMap<_, _>>()
    }

    fn new_epoch(&mut self, epoch: EpochParameters) {
        self.status = CollectorStatus::Started {
            started_time: Instant::now(),
            epoch,
        };

        self.entries.clear();
    }
}

impl CUHashrateRaw {
    pub(self) fn from_single_entry(entry: HashrateCURecord) -> Self {
        match entry.variant {
            HashrateCUEntryVariant::CacheCreation => Self::from_cache_creation(entry.duration),
            HashrateCUEntryVariant::DatasetInitialization { .. } => {
                Self::from_dataset_initialization(entry.duration)
            }
            HashrateCUEntryVariant::HashesChecked { hashes_count } => {
                Self::from_cc_job(entry.duration, hashes_count as u64)
            }
        }
    }

    pub(self) fn add_entry(&mut self, entry: HashrateCURecord) {
        match entry.variant {
            HashrateCUEntryVariant::CacheCreation => {
                self.cache_creation = ParameterStatus::Measured(entry.duration)
            }
            HashrateCUEntryVariant::DatasetInitialization { .. } => {
                self.dataset_initialization.push(entry.duration)
            }
            HashrateCUEntryVariant::HashesChecked { hashes_count } => {
                self.hashes_checked += hashes_count as u64
            }
        }
    }

    fn from_cache_creation(duration: Duration) -> Self {
        Self {
            cache_creation: ParameterStatus::Measured(duration),
            dataset_initialization: vec![],
            pow_duration: ParameterStatus::NotMeasured,
            hashes_checked: 0,
        }
    }

    fn from_dataset_initialization(duration: Duration) -> Self {
        Self {
            cache_creation: ParameterStatus::NotMeasured,
            dataset_initialization: vec![duration],
            pow_duration: ParameterStatus::NotMeasured,
            hashes_checked: 0,
        }
    }

    fn from_cc_job(duration: Duration, hashes_checked: u64) -> Self {
        Self {
            cache_creation: ParameterStatus::NotMeasured,
            dataset_initialization: vec![duration],
            pow_duration: ParameterStatus::Measured(duration),
            hashes_checked,
        }
    }
}
