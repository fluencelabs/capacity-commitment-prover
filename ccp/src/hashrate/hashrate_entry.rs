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

use std::time::Duration;

use cpu_utils::LogicalCoreId;

#[derive(Debug)]
pub(crate) enum HashrateCUEntry {
    CacheCreation {
        core_id: LogicalCoreId,
        duration: Duration,
    },

    DatasetAllocation {
        core_id: LogicalCoreId,
        duration: Duration,
    },

    DatasetInitialization {
        core_id: LogicalCoreId,
        duration: Duration,
        start_item: u64,
        items_count: u64,
    },

    HashesChecked {
        core_id: LogicalCoreId,
        hashes_count: usize,
        duration: Duration,
    },
}

impl HashrateCUEntry {
    pub(crate) fn cache_creation(core_id: LogicalCoreId, duration: Duration) -> Self {
        Self::CacheCreation { core_id, duration }
    }

    pub(crate) fn dataset_allocation(core_id: LogicalCoreId, duration: Duration) -> Self {
        Self::DatasetAllocation { core_id, duration }
    }

    pub(crate) fn dataset_initialization(
        core_id: LogicalCoreId,
        duration: Duration,
        start_item: u64,
        items_count: u64,
    ) -> Self {
        Self::DatasetInitialization {
            core_id,
            duration,
            start_item,
            items_count,
        }
    }

    pub(crate) fn hashes_checked(
        core_id: LogicalCoreId,
        hashes_count: usize,
        duration: Duration,
    ) -> Self {
        Self::HashesChecked {
            core_id,
            hashes_count,
            duration,
        }
    }
}

impl std::fmt::Display for HashrateCUEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CacheCreation {core_id, duration} => write!(f, "logical core id {core_id}: spent {duration:?} for cache creation"),
            Self::DatasetAllocation {core_id, duration} => write!(f, "logical core id {core_id}: spent {duration:?} for dataset allocation"),
            Self::DatasetInitialization {core_id, duration, start_item, items_count} => write!(f, "logical core id {core_id}: spent {duration:?} for dataset init in ({start_item}, {items_count})"),
            Self::HashesChecked {core_id, duration, hashes_count} => write!(f, "logical core id {core_id}: spent {duration:?} to check {hashes_count} hashes"),
        }
    }
}
