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

use std::collections::HashMap;

use ccp_config::CCPConfig;
use ccp_shared::types::*;

use super::cu::CUProver;
use super::cu::CUProverConfig;

pub struct CCProver {
    allocated_threads: HashMap<PhysicalCoreId, CUProver>,
    config: CCPConfig,
    epoch_parameters: Option<GlobalEpochParameters>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlobalEpochParameters {
    pub(crate) global_nonce: GlobalNonce,
    pub(crate) difficulty: Difficulty,
}

impl CCProver {
    pub fn new(config: CCPConfig) -> Self {
        Self {
            allocated_threads: HashMap::new(),
            config,
            epoch_parameters: None,
        }
    }

    pub async fn on_active_commitment(
        &mut self,
        global_nonce: GlobalNonce,
        difficulty: Difficulty,
        cu_allocation: CUAllocation,
    ) {
        /*
        let global_params = GlobalEpochParameters::new(global_nonce, difficulty);
        let cu_prover = CUProver::new();
         */
    }

    pub async fn on_no_active_commitment(&mut self) {
        /*
        self.allocated_threads
            .iter()
            .map(|(_, cu_prover)| cu_prover.stop())
         */
    }

    pub fn create_proof_watcher(&self) {
        unimplemented!()
    }
}

impl GlobalEpochParameters {
    pub(crate) fn new(global_nonce: GlobalNonce, difficulty: Difficulty) -> Self {
        Self {
            global_nonce,
            difficulty,
        }
    }
}
