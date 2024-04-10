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

use crate::cu::CUProverConfig;

#[derive(Debug, Clone)]
pub struct ProvingThreadConfig {
    pub hashes_per_round: usize,
    pub async_to_sync_queue_size: usize,
    pub sync_to_async_queue_size: usize,
}

impl ProvingThreadConfig {
    pub fn from_cu_prover_config(cu_config: &CUProverConfig) -> Self {
        Self {
            hashes_per_round: cu_config.hashes_per_round,
            async_to_sync_queue_size: cu_config.async_to_sync_queue_size,
            sync_to_async_queue_size: cu_config.sync_to_async_queue_size,
        }
    }
}
