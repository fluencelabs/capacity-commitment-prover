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

use futures::{future, FutureExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use ccp_config::CCPConfig;
use ccp_shared::nox_ccp_api::NoxCCPApi;
use ccp_shared::proof::CCProof;
use ccp_shared::proof::CCProofId;
use ccp_shared::types::*;

use crate::alignment_roadmap::*;
use crate::cu::CUProver;
use crate::cu::CUProverConfig;
use crate::cu::RawProof;
use crate::epoch::Epoch;
use crate::errors::CCProverError;
use crate::proof_storage_worker::ProofStorageWorker;
use crate::LogicalCoreId;

pub type CCResult<T> = Result<T, CCProverError>;

pub struct CCProver {
    active_provers: HashMap<PhysicalCoreId, CUProver>,
    cu_prover_config: CUProverConfig,
    epoch_parameters: Option<Epoch>,
    proof_receiver_inlet: mpsc::Sender<RawProof>,
    utility_thread_shutdown: oneshot::Sender<()>,
    proof_storage: Arc<ProofStorageWorker>,
}

impl NoxCCPApi for CCProver {
    type Error = CCProverError;

    async fn on_active_commitment(
        &mut self,
        global_nonce: GlobalNonce,
        difficulty: Difficulty,
        new_allocation: CUAllocation,
    ) -> Result<(), Self::Error> {
        let new_epoch = Epoch::new(global_nonce, difficulty);
        let roadmap = CCProverAlignmentRoadmap::create_roadmap(
            new_allocation,
            new_epoch,
            &self.active_provers,
            self.epoch_parameters.unwrap(),
        );
        self.align_with(roadmap).await
    }

    async fn on_no_active_commitment(&mut self) -> Result<(), Self::Error> {
        use futures::stream::FuturesUnordered;
        use futures::StreamExt;

        let results = self
            .active_provers
            .iter_mut()
            .map(|(_, prover)| prover.stop())
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;

        let errors = results
            .into_iter()
            .map(Result::unwrap_err)
            .collect::<Vec<_>>();

        if errors.is_empty() {
            Ok(())
        } else {
            Err(CCProverError::CUProverErrors(errors))
        }
    }

    async fn get_proofs_after(&self, proof_idx: u64) -> Result<Vec<CCProof>, Self::Error> {
        self.proof_storage
            .get_proofs_after(proof_idx)
            .await
            .map_err(Into::into)
    }
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

        let cu_prover_config = CUProverConfig {
            randomx_flags: config.randomx_flags,
            threads_per_physical_core: config.threads_per_physical_core,
        };

        Self {
            active_provers: HashMap::new(),
            cu_prover_config,
            epoch_parameters: None,
            proof_receiver_inlet,
            utility_thread_shutdown: shutdown_inlet,
            proof_storage,
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
                        log::debug!("cc_prover: new proof_received {proof:?}");

                        if proof.global_nonce != last_seen_global_nonce {
                            last_seen_global_nonce = proof.global_nonce;
                            proof_idx = 0;
                        }
                        let cc_proof_id = CCProofId::new(proof.global_nonce, proof.difficulty, proof_idx);
                        let cc_proof = CCProof::new(cc_proof_id, proof.local_nonce, proof.cu_id);
                        proof_storage.store_new_proof(cc_proof).await?;
                        proof_idx += 1;
                    },
                    _ = &mut shutdown_outlet => {
                        log::info!("cc_prover:: utility thread was shutdown");

                        return Ok::<_, std::io::Error>(())
                    }
                }
            }
        });
    }
}

impl RoadmapAlignable for CCProver {
    type Error = CCProverError;

    async fn align_with<'futures, 'prover: 'futures>(
        &'prover mut self,
        roadmap: CCProverAlignmentRoadmap,
    ) -> Result<(), Self::Error> {
        use crate::cu::CUResult;
        use futures::stream::FuturesUnordered;
        use futures::StreamExt;

        let CCProverAlignmentRoadmap { actions, epoch } = roadmap;
        let actions_as_future: FuturesUnordered<
            future::BoxFuture<'futures, CUResult<Option<CUProver>>>,
        > = FuturesUnordered::new();

        for action in actions {
            match action {
                CUProverAction::CreateCUProver {
                    new_core_id,
                    new_cu_id,
                } => {
                    let prover_config = self.cu_prover_config.clone();
                    let proof_receiver_inlet = self.proof_receiver_inlet.clone();

                    let creation_future = async move {
                        let mut prover =
                            CUProver::create(prover_config, proof_receiver_inlet, new_core_id)
                                .await?;
                        prover
                            .new_epoch(epoch.global_nonce, new_cu_id, epoch.difficulty)
                            .await?;

                        Ok(Some(prover))
                    }
                    .boxed();

                    actions_as_future.push(creation_future);
                }
                CUProverAction::RemoveCUProver { current_core_id } => {
                    let mut prover = self.active_provers.remove(&current_core_id).unwrap();
                    let removal_future = async move {
                        prover.stop().await?;
                        Ok(None)
                    }
                    .boxed();
                    actions_as_future.push(removal_future);
                }
                CUProverAction::NewCCJob {
                    current_core_id,
                    new_cu_id,
                } => {
                    let mut prover = self.active_provers.remove(&current_core_id).unwrap();
                    let cc_job_future = async move {
                        prover
                            .new_epoch(epoch.global_nonce, new_cu_id, epoch.difficulty)
                            .await?;
                        Ok(Some(prover))
                    }
                    .boxed();

                    actions_as_future.push(cc_job_future);
                }
                CUProverAction::NewCCJobWithRepining {
                    current_core_id,
                    new_core_id,
                    new_cu_id,
                } => {
                    let mut prover = self.active_provers.remove(&current_core_id).unwrap();
                    let cc_job_repin_future = async move {
                        prover.repin(new_core_id).await?;
                        prover
                            .new_epoch(epoch.global_nonce, new_cu_id, epoch.difficulty)
                            .await?;
                        Ok(Some(prover))
                    }
                    .boxed();
                    actions_as_future.push(cc_job_repin_future);
                }
                CUProverAction::CleanProofCache => {}
            }
        }

        let (provers, errors) = actions_as_future
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .partition::<Vec<_>, _>(Result::is_ok);

        let provers = provers
            .into_iter()
            .filter_map(Result::unwrap)
            .map(|prover| (prover.pinned_core_id(), prover))
            .collect::<Vec<_>>();
        self.active_provers.extend(provers);

        if errors.is_empty() {
            return Ok(());
        }

        let errors = errors
            .into_iter()
            .map(Result::unwrap_err)
            .collect::<Vec<_>>();

        Err(CCProverError::CUProverErrors(errors))
    }
}
