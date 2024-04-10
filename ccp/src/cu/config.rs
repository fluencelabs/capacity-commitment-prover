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

use ccp_config::Optimizations;
use ccp_config::RandomXFlags;
use ccp_config::ThreadsPerCoreAllocationPolicy;
use ccp_config::Workers;

#[derive(Clone, Debug)]
pub struct CUProverConfig {
    pub randomx_flags: RandomXFlags,
    /// Defines how many threads will be assigned to a specific physical core,
    /// aims to utilize benefits of hyper-threading.
    pub threads_per_core_policy: ThreadsPerCoreAllocationPolicy,

    pub hashes_per_round: usize,
    pub async_to_sync_queue_size: usize,
    pub sync_to_async_queue_size: usize,
}

impl CUProverConfig {
    pub fn new(ccp_optimizations: Optimizations, workers: Workers) -> Self {
        Self {
            randomx_flags: ccp_optimizations.randomx_flags,
            threads_per_core_policy: ccp_optimizations.threads_per_core_policy,

            hashes_per_round: workers.hashes_per_round,
            async_to_sync_queue_size: workers.async_to_sync_queue_size,
            sync_to_async_queue_size: workers.sync_to_async_queue_size,
        }
    }
}
