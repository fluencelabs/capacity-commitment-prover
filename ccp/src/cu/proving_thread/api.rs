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

use ccp_shared::types::*;
use randomx_rust_wrapper::cache::CacheHandle;
use randomx_rust_wrapper::dataset::DatasetHandle;
use randomx_rust_wrapper::Cache;
use randomx_rust_wrapper::Dataset;
use randomx_rust_wrapper::RandomXFlags;

use crate::LogicalCoreId;

pub trait ProvingThreadAPI {
    type Error;

    async fn create_cache(
        &mut self,
        global_nonce: GlobalNonce,
        cu_id: CUID,
        flags: RandomXFlags,
    ) -> Result<Cache, Self::Error>;

    async fn allocate_dataset(&mut self, flags: RandomXFlags) -> Result<Dataset, Self::Error>;

    async fn initialize_dataset(
        &mut self,
        cache: CacheHandle,
        dataset: DatasetHandle,
        start_item: u64,
        items_count: u64,
    ) -> Result<(), Self::Error>;

    async fn run_cc_job(
        &mut self,
        dataset: DatasetHandle,
        flags: RandomXFlags,
        global_nonce: GlobalNonce,
        difficulty: Difficulty,
        cu_id: CUID,
    ) -> Result<(), Self::Error>;

    async fn pin_thread(&self, logical_core_id: LogicalCoreId) -> Result<(), Self::Error>;

    async fn stop(&self) -> Result<(), Self::Error>;
}
