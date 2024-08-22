/*
 * Copyright 2024 Fluence DAO
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

use std::time::Duration;

use ccp_shared::types::LogicalCoreId;

#[derive(Copy, Clone, Debug)]
pub struct ThreadHashrateRecord {
    pub core_id: LogicalCoreId,
    pub duration: Duration,
    pub variant: HashrateRecordType,
}

#[derive(Copy, Clone, Debug)]
pub enum HashrateRecordType {
    CacheCreation,

    DatasetInitialization { start_item: u64, items_count: u64 },

    CheckedHashes { count: usize },
}

impl ThreadHashrateRecord {
    pub fn cache_creation(core_id: LogicalCoreId, duration: Duration) -> Self {
        Self {
            core_id,
            duration,
            variant: HashrateRecordType::CacheCreation,
        }
    }

    pub fn dataset_initialization(
        core_id: LogicalCoreId,
        duration: Duration,
        start_item: u64,
        items_count: u64,
    ) -> Self {
        Self {
            core_id,
            duration,
            variant: HashrateRecordType::DatasetInitialization {
                start_item,
                items_count,
            },
        }
    }

    pub fn checked_hashes(
        core_id: LogicalCoreId,
        duration: Duration,
        hashes_count: usize,
    ) -> Self {
        Self {
            core_id,
            duration,
            variant: HashrateRecordType::CheckedHashes {
                count: hashes_count,
            },
        }
    }
}

impl std::fmt::Display for ThreadHashrateRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.variant {
            HashrateRecordType::CacheCreation => write!(
                f,
                "{}: spent {:?} for cache creation",
                self.core_id, self.duration
            ),
            HashrateRecordType::DatasetInitialization {
                start_item,
                items_count,
            } => write!(
                f,
                "{}: spent {:?} for dataset init in ({start_item}, {items_count})",
                self.core_id, self.duration
            ),
            HashrateRecordType::CheckedHashes {
                count: hashes_count,
            } => {
                let hashrate = hashes_count as f64 / self.duration.as_secs_f64();
                write!(f, "{}: hashrate {hashrate}", self.core_id)
            }
        }
    }
}
