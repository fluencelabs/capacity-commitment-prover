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
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use ccp_config::CCPConfig;
use ccp_shared::proof::{CCProof, CCProofId};
use ccp_shared::types::*;

use super::cu::CUProver;
use super::cu::CUProverConfig;
use crate::cu::RawProof;
use crate::errors::CCProverError;
use crate::proof_storage_worker::ProofStorageWorker;
use crate::LogicalCoreId;

pub type CCResult<T> = Result<T, CCProverError>;

pub struct CCProver {
    allocated_provers: HashMap<PhysicalCoreId, CUProver>,
    config: CCPConfig,
    epoch_parameters: Option<GlobalEpochParameters>,
    proof_receiver_inlet: mpsc::Sender<RawProof>,
    utility_thread_shutdown: oneshot::Sender<()>,
    proof_storage: Arc<ProofStorageWorker>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlobalEpochParameters {
    pub(crate) global_nonce: GlobalNonce,
    pub(crate) difficulty: Difficulty,
}

impl CCProver {
    pub fn new(utility_core_id: LogicalCoreId, config: CCPConfig) -> Self {
        let (proof_receiver_inlet, proof_receiver_outlet) = mpsc::channel(100);
        let (shutdown_inlet, shutdown_outlet) = oneshot::channel();

        let proof_storage = ProofStorageWorker::new(config.dir_to_store_proofs.clone());
        let proof_storage = Arc::new(proof_storage);
        Self::spawn_utility_thread(
            proof_storage.clone(),
            proof_receiver_outlet,
            shutdown_outlet,
            utility_core_id,
        );

        Self {
            allocated_provers: HashMap::new(),
            config,
            epoch_parameters: None,
            proof_receiver_inlet,
            utility_thread_shutdown: shutdown_inlet,
            proof_storage,
        }
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

    pub async fn stop(mut self) -> CCResult<()> {
        // stop all active provers
        self.on_no_active_commitment().await?;
        // stop background thread
        self.utility_thread_shutdown
            .send(())
            .map_err(|_| CCProverError::UtilityThreadShutdownFailed)
    }

    fn spawn_utility_thread(
        proof_storage: Arc<ProofStorageWorker>,
        mut proof_receiver_outlet: mpsc::Receiver<RawProof>,
        mut shutdown_outlet: oneshot::Receiver<()>,
        utility_core_id: LogicalCoreId,
    ) {
        tokio::spawn(async move {
            let mut proof_idx = 0;
            let mut last_seen_global_nonce = [0u8; 32];

            loop {
                tokio::select! {
                    Some(proof) = proof_receiver_outlet.recv() => {
                        if proof.global_nonce != last_seen_global_nonce {
                            last_seen_global_nonce = proof.global_nonce;
                            proof_idx = 0;
                        }
                        let cc_proof_id = CCProofId::new(proof.global_nonce, proof.difficulty, proof_idx);
                        let cc_proof = CCProof::new(cc_proof_id, proof.local_nonce, proof.cu_id);
                        proof_storage.store_new_proof(cc_proof).await?;
                    },
                    _ = &mut shutdown_outlet => {
                        return Ok::<_, std::io::Error>(())
                    }
                }
            }
        });
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
