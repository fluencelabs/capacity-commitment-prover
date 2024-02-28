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

use ccp_shared::types::EpochParameters;
use ccp_shared::types::LogicalCoreId;

#[derive(Copy, Clone, Debug)]
pub(crate) struct ThreadHashrateRecord {
    pub(crate) epoch: EpochParameters,
    pub(crate) core_id: LogicalCoreId,
    pub(crate) duration: Duration,
    pub(crate) variant: HashrateCUEntryVariant,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum HashrateCUEntryVariant {
    CacheCreation,

    DatasetInitialization { start_item: u64, items_count: u64 },

    HashesChecked { hashes_count: usize },
}

impl ThreadHashrateRecord {
    pub(crate) fn cache_creation(
        epoch: EpochParameters,
        core_id: LogicalCoreId,
        duration: Duration,
    ) -> Self {
        Self {
            epoch,
            core_id,
            duration,
            variant: HashrateCUEntryVariant::CacheCreation,
        }
    }

    pub(crate) fn dataset_initialization(
        epoch: EpochParameters,
        core_id: LogicalCoreId,
        duration: Duration,
        start_item: u64,
        items_count: u64,
    ) -> Self {
        Self {
            epoch,
            core_id,
            duration,
            variant: HashrateCUEntryVariant::DatasetInitialization {
                start_item,
                items_count,
            },
        }
    }

    pub(crate) fn hashes_checked(
        epoch: EpochParameters,
        core_id: LogicalCoreId,
        duration: Duration,
        hashes_count: usize,
    ) -> Self {
        Self {
            epoch,
            core_id,
            duration,
            variant: HashrateCUEntryVariant::HashesChecked { hashes_count },
        }
    }
}

impl std::fmt::Display for ThreadHashrateRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.variant {
            HashrateCUEntryVariant::CacheCreation => write!(
                f,
                "logical core id {}: spent {:?} for cache creation",
                self.core_id, self.duration
            ),
            HashrateCUEntryVariant::DatasetInitialization {
                start_item,
                items_count,
            } => write!(
                f,
                "logical core id {}: spent {:?} for dataset init in ({start_item}, {items_count})",
                self.core_id, self.duration
            ),
            HashrateCUEntryVariant::HashesChecked { hashes_count } => write!(
                f,
                "logical core id {}: spent {:?} to check {hashes_count} hashes",
                self.core_id, self.duration
            ),
        }
    }
}
