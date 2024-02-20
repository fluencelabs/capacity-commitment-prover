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

#[cfg(test)]
mod tests;

use futures::future;
use futures::FutureExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use ccp_config::CCPConfig;
use ccp_shared::nox_ccp_api::NoxCCPApi;
use ccp_shared::proof::CCProof;
use ccp_shared::proof::CCProofId;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::*;
use ccp_utils::run_utils::run_unordered;

use crate::alignment_roadmap::*;
use crate::cu::CUProver;
use crate::cu::CUProverConfig;
use crate::cu::CUResult;
use crate::cu::RawProof;
use crate::errors::CCProverError;
use crate::proof_storage_worker::ProofStorageWorker;
use crate::status::CCStatus;
use crate::status::ToCCStatus;
use crate::LogicalCoreId;

pub type CCResult<T> = Result<T, CCProverError>;

pub struct CCProver {
    cu_provers: HashMap<PhysicalCoreId, CUProver>,
    cu_prover_config: CUProverConfig,
    status: CCStatus,
    proof_receiver_inlet: mpsc::Sender<RawProof>,
    utility_thread_shutdown: oneshot::Sender<()>,
    proof_storage: Arc<ProofStorageWorker>,
}

impl NoxCCPApi for CCProver {
    type Error = CCProverError;

    async fn on_active_commitment(
        &mut self,
        new_epoch: EpochParameters,
        new_allocation: CUAllocation,
    ) -> Result<(), Self::Error> {
        let roadmap = CCProverAlignmentRoadmap::create_roadmap(
            new_allocation,
            new_epoch,
            &self.cu_provers,
            self.status,
        );
        self.status = CCStatus::Running { epoch: new_epoch };
        self.align_with(roadmap).await
    }

    async fn on_no_active_commitment(&mut self) -> Result<(), Self::Error> {
        let closure =
            move |_: usize, (_, prover): (PhysicalCoreId, CUProver)| prover.stop().boxed();

        run_unordered(self.cu_provers.drain(), closure).await?;
        self.status = CCStatus::Idle;

        Ok(())
    }

    async fn get_proofs_after(&self, proof_idx: ProofIdx) -> Result<Vec<CCProof>, Self::Error> {
        self.proof_storage
            .get_proofs_after(proof_idx)
            .await
            .map_err(Into::into)
    }
}

impl ToCCStatus for CCProver {
    fn status(&self) -> CCStatus {
        self.status
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
            thread_allocation_policy: config.thread_allocation_policy,
        };

        Self {
            cu_provers: HashMap::new(),
            cu_prover_config,
            status: CCStatus::Idle,
            proof_receiver_inlet,
            utility_thread_shutdown: shutdown_inlet,
            proof_storage,
        }
    }

    #[allow(clippy::needless_lifetimes)]
    pub async fn pause<'provers>(&'provers mut self) -> CCResult<()> {
        let closure = move |_: usize, (_, prover): (&PhysicalCoreId, &'provers mut CUProver)| {
            prover.pause().boxed()
        };
        run_unordered(self.cu_provers.iter_mut(), closure).await?;

        self.status = CCStatus::Idle;

        Ok(())
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
            cpu_utils::pinning::pin_current_thread_to(utility_core_id);

            let mut proof_idx = ProofIdx::zero();
            let mut last_seen_global_nonce = GlobalNonce::new([0u8; 32]);

            loop {
                tokio::select! {
                    Some(proof) = proof_receiver_outlet.recv() => {
                        log::debug!("cc_prover: new proof_received {proof:?}");

                        if proof.epoch.global_nonce != last_seen_global_nonce {
                            last_seen_global_nonce = proof.epoch.global_nonce;
                            proof_idx = ProofIdx::zero();
                        }
                        let cc_proof_id = CCProofId::new(proof.epoch.global_nonce, proof.epoch.difficulty, proof_idx);
                        let cc_proof = CCProof::new(cc_proof_id, proof.local_nonce, proof.cu_id, proof.result_hash);
                        proof_storage.store_new_proof(cc_proof).await?;
                        proof_idx.increment();
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

#[derive(Debug)]
enum AlignmentPostAction {
    KeepProver(CUProver),
    Nothing,
}

impl RoadmapAlignable for CCProver {
    type Error = CCProverError;

    async fn align_with(&mut self, roadmap: CCProverAlignmentRoadmap) -> Result<(), Self::Error> {
        use futures::stream::FuturesUnordered;
        use futures::StreamExt;

        let CCProverAlignmentRoadmap {
            pre_action,
            actions,
            epoch,
        } = roadmap;

        match pre_action {
            CUProverPreAction::NoAction => {}
            CUProverPreAction::CleanupProofCache => {
                self.pause().await?;
                self.proof_storage.remove_proofs().await?;
            }
        }

        let actions_as_futures = actions
            .into_iter()
            .map(|action| match action {
                CUProverAction::CreateCUProver(state) => self.cu_creation(state, epoch),
                CUProverAction::RemoveCUProver(state) => self.cu_removal(state),
                CUProverAction::NewCCJob(state) => self.new_cc_job(state, epoch),
                CUProverAction::NewCCJobWithRepining(state) => self.new_cc_job_repin(state, epoch),
            })
            .collect::<FuturesUnordered<_>>();

        let (provers, errors) = actions_as_futures
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .partition::<Vec<_>, _>(Result::is_ok);

        let provers_to_keep = provers
            .into_iter()
            .map(Result::unwrap)
            .flat_map(|prover_post_action| match prover_post_action {
                AlignmentPostAction::KeepProver(prover) => Some((prover.pinned_core_id(), prover)),
                AlignmentPostAction::Nothing => None,
            })
            .collect::<Vec<_>>();
        self.cu_provers.extend(provers_to_keep);

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

impl CCProver {
    pub(self) fn cu_creation<'prover, 'futures: 'prover>(
        &'prover mut self,
        state: actions_state::CreateCUProverState,
        epoch: EpochParameters,
    ) -> future::BoxFuture<'futures, CUResult<AlignmentPostAction>> {
        let prover_config = self.cu_prover_config.clone();
        let proof_receiver_inlet = self.proof_receiver_inlet.clone();

        async move {
            let mut prover =
                CUProver::create(prover_config, proof_receiver_inlet, state.new_core_id).await?;
            prover.new_epoch(epoch, state.new_cu_id).await?;

            Ok(AlignmentPostAction::KeepProver(prover))
        }
        .boxed()
    }

    pub(self) fn cu_removal<'prover, 'futures: 'prover>(
        &'prover mut self,
        state: actions_state::RemoveCUProverState,
    ) -> future::BoxFuture<'futures, CUResult<AlignmentPostAction>> {
        let prover = self.cu_provers.remove(&state.current_core_id).unwrap();
        async move {
            prover.stop().await?;
            Ok(AlignmentPostAction::Nothing)
        }
        .boxed()
    }

    pub(self) fn new_cc_job<'prover, 'futures: 'prover>(
        &'prover mut self,
        state: actions_state::NewCCJobState,
        epoch: EpochParameters,
    ) -> future::BoxFuture<'futures, CUResult<AlignmentPostAction>> {
        let mut prover = self.cu_provers.remove(&state.current_core_id).unwrap();
        async move {
            prover.new_epoch(epoch, state.new_cu_id).await?;
            Ok(AlignmentPostAction::KeepProver(prover))
        }
        .boxed()
    }

    pub(self) fn new_cc_job_repin<'prover, 'futures: 'prover>(
        &'prover mut self,
        state: actions_state::NewCCJobWithRepiningState,
        epoch: EpochParameters,
    ) -> future::BoxFuture<'futures, CUResult<AlignmentPostAction>> {
        let mut prover = self.cu_provers.remove(&state.current_core_id).unwrap();
        async move {
            prover.pin(state.new_core_id).await?;
            prover.new_epoch(epoch, state.new_cu_id).await?;
            Ok(AlignmentPostAction::KeepProver(prover))
        }
        .boxed()
    }
}
