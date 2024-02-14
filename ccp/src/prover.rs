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

use futures::future::MaybeDone::Future;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::collections::HashMap;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use tokio::sync::mpsc;

use ccp_config::CCPConfig;
use ccp_shared::types::*;

use super::cu::CUProver;
use super::cu::CUProverConfig;
use crate::cu::RawProof;
use crate::errors::CCProverError;

pub type CCResult<T> = Result<T, CCProverError>;

pub struct CCProver {
    allocated_provers: HashMap<PhysicalCoreId, CUProver>,
    config: CCPConfig,
    epoch_parameters: Option<GlobalEpochParameters>,
    proof_receiver_inlet: mpsc::Sender<RawProof>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlobalEpochParameters {
    pub(crate) global_nonce: GlobalNonce,
    pub(crate) difficulty: Difficulty,
}

impl CCProver {
    pub fn new(config: CCPConfig) -> (Self, mpsc::Receiver<RawProof>) {
        let (proof_receiver_inlet, proof_receiver_outlet) = mpsc::channel(100);
        let prover = Self {
            allocated_provers: HashMap::new(),
            config,
            epoch_parameters: None,
            proof_receiver_inlet,
        };

        (prover, proof_receiver_outlet)
    }

    pub async fn on_active_commitment(
        &mut self,
        global_nonce: GlobalNonce,
        difficulty: Difficulty,
        cu_allocation: CUAllocation,
    ) -> CCResult<()> {
        use futures::stream::FuturesUnordered;
        use futures::StreamExt;

        let cu_prover_config = CUProverConfig {
            randomx_flags: self.config.randomx_flags,
            threads_per_physical_core: self.config.threads_per_physical_core,
        };

        let allocated_provers = cu_allocation
            .iter()
            .map(|(&core_id, cu_id)| {
                let cu_prover = CUProver::new(
                    cu_prover_config.clone(),
                    self.proof_receiver_inlet.clone(),
                    core_id,
                );
                (core_id, cu_prover)
            })
            .collect::<HashMap<_, _>>();

        self.allocated_provers = allocated_provers;

        let results = self
            .allocated_provers
            .iter_mut()
            .map(|(&core_id, prover)| {
                let cu_id = cu_allocation.get(&core_id).unwrap();
                prover.new_epoch(global_nonce, *cu_id, difficulty, self.config.randomx_flags)
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;

        for result in results {
            result?;
        }

        Ok(())
    }

    pub async fn on_no_active_commitment(&mut self) -> CCResult<()> {
        use futures::stream::FuturesUnordered;
        use futures::StreamExt;

        let results = self
            .allocated_provers
            .iter_mut()
            .map(|(_, prover)| prover.stop())
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;

        let errors = results
            .into_iter()
            .filter_map(|result| match result {
                Ok(_) => None,
                Err(e) => Some(e),
            })
            .collect::<Vec<_>>();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(CCProverError::CUProverErrors(errors))
        }
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
