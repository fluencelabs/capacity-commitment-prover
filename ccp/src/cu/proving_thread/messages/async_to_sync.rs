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

use cpu_utils::LogicalCoreId;
use randomx_rust_wrapper::cache::CacheHandle;
use randomx_rust_wrapper::dataset::DatasetHandle;
use randomx_rust_wrapper::RandomXFlags;

use crate::Difficulty;
use crate::GlobalNonce;
use crate::CUID;

#[derive(Debug)]
pub(crate) struct CreateCache {
    pub(crate) global_nonce: GlobalNonce,
    pub(crate) cu_id: CUID,
    pub(crate) flags: RandomXFlags,
}

#[derive(Debug)]
pub(crate) struct AllocateDataset {
    pub(crate) flags: RandomXFlags,
}

#[derive(Debug)]
pub(crate) struct InitializeDataset {
    pub(crate) cache: CacheHandle,
    pub(crate) dataset: DatasetHandle,
    pub(crate) start_item: u64,
    pub(crate) items_count: u64,
}

#[derive(Debug)]
pub(crate) struct NewCCJob {
    pub(crate) dataset: DatasetHandle,
    pub(crate) flags: RandomXFlags,
    pub(crate) global_nonce: GlobalNonce,
    pub(crate) difficulty: Difficulty,
    pub(crate) cu_id: CUID,
}

#[derive(Debug)]
pub(crate) struct PinThread {
    pub(crate) core_id: LogicalCoreId,
}

impl CreateCache {
    pub(crate) fn new(global_nonce: GlobalNonce, cu_id: CUID, flags: RandomXFlags) -> Self {
        Self {
            global_nonce,
            cu_id,
            flags,
        }
    }
}

impl AllocateDataset {
    pub(crate) fn new(flags: RandomXFlags) -> Self {
        Self { flags }
    }
}

impl InitializeDataset {
    pub fn new(
        cache: CacheHandle,
        dataset: DatasetHandle,
        start_item: u64,
        items_count: u64,
    ) -> Self {
        Self {
            cache,
            dataset,
            start_item,
            items_count,
        }
    }
}

impl NewCCJob {
    pub fn new(
        dataset: DatasetHandle,
        flags: RandomXFlags,
        global_nonce: GlobalNonce,
        difficulty: Difficulty,
        cu_id: CUID,
    ) -> Self {
        Self {
            dataset,
            flags,
            global_nonce,
            difficulty,
            cu_id,
        }
    }
}
