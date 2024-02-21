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
use std::time::Instant;

use cpu_utils::LogicalCoreId;

pub(crate) enum HashrateCUEntry {
    CacheCreation {
        core_id: LogicalCoreId,
        time: Duration
    },

    DatasetAllocation {
        core_id: LogicalCoreId,
        time: Instant,
    },

    DatasetInitialization {
        core_id: LogicalCoreId,
        time: Instant,
        item_start: u64,
        items_count: u64,
    },

    HashesChecked {
        core_id: LogicalCoreId,
        hashes_count: usize,
        duration: Duration,
    }
}
