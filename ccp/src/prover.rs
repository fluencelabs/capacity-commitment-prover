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
use std::sync::Mutex;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use ccp_config::CCPConfig;
use ccp_msr::state::MSRState;
use ccp_msr::{MSREnforce, MSRModeEnforcer};
use ccp_shared::nox_ccp_api::NoxCCPApi;
use ccp_shared::proof::CCProof;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::*;
use ccp_utils::run_utils::run_unordered;

use crate::alignment_roadmap::*;
use crate::cpuids_handle::CpuIdsHandle;
use crate::cu::proving_thread::sync::to_state_operating_message::StateOperatingControlInlet;
use crate::cu::proving_thread::sync::to_state_operating_message::StateOperatingControlMessage;
use crate::cu::CUProver;
use crate::cu::CUProverConfig;
use crate::cu::CUResult;
use crate::errors::CCProverError;
use crate::hashrate::prometheus::PrometheusEndpoint;
use crate::hashrate::HashrateCollector;
use crate::hashrate::HashrateHandler;
use crate::proof_storage::ProofStorageDrainer;
use crate::state_operating_thread::StateOperatingThread;
use crate::state_storage::CCPState;
use crate::state_storage::StateStorage;
use crate::stats_collecting_thread::StatsCollectingThread;
use crate::status::CCStatus;
use crate::status::ToCCStatus;

const PROOF_DIR: &str = "cc_proofs";

pub type CCResult<T> = Result<T, CCProverError>;

pub struct CCProver {
    cu_provers: HashMap<PhysicalCoreId, CUProver>,
    cu_prover_config: CUProverConfig,
    status: CCStatus,
    state_operating_thread: StateOperatingThread,
    stats_collecting_thread: StatsCollectingThread,
    prometheus_endpoint: Option<PrometheusEndpoint>,
    msr_enforcer: MSRModeEnforcer,
    utility_core_ids_handle: CpuIdsHandle,
    state_operating_control_inlet: StateOperatingControlInlet,
    cancellation_token: CancellationToken,
}

impl NoxCCPApi for CCProver {
    type Error = CCProverError;

    async fn on_active_commitment(
        &mut self,
        new_epoch: EpochParameters,
        new_allocation: CUAllocation,
    ) -> Result<(), Self::Error> {
        let apply_resut = self
            .apply_cc_parameters(new_epoch, &new_allocation)
            .await
            .inspect_err(|e| {
                log::error!("Failed to apply parameters: {e}.  Still trying to save state.");
            });
        self.save_state(new_epoch, new_allocation.clone())
            .await
            .inspect_err(|e| {
                log::error!("Failed to save state: {e}");
            })?;

        apply_resut
    }

    async fn on_no_active_commitment(&mut self) -> Result<(), Self::Error> {
        self.stop_provers_nonblocking().await?;
        self.join_provers().await?;

        self.status = CCStatus::Idle;

        self.save_no_state().await?;

        Ok(())
    }

    #[inline]
    async fn get_proofs_after(&self, proof_idx: ProofIdx) -> Result<Vec<CCProof>, Self::Error> {
        self.call_utility_get_proofs_after(proof_idx).await
    }

    async fn realloc_utility_cores(&self, utility_core_ids: Vec<LogicalCoreId>) {
        self.utility_core_ids_handle.set_cores(utility_core_ids);
    }
}

impl ToCCStatus for CCProver {
    fn status(&self) -> CCStatus {
        self.status
    }
}

impl CCProver {
    // this method should be used in simple tests only
    pub async fn new(config: CCPConfig) -> CCResult<Self> {
        let msr_enforcer = MSRModeEnforcer::from_os(config.optimizations.msr_enabled);
        let state_storage = StateStorage::new(config.state_dir.clone());
        let utility_core_ids_handle = CpuIdsHandle::new(config.tokio.utility_cores_ids.clone());
        let cancellation_token = CancellationToken::new();

        Self::create_prover(
            config,
            None,
            msr_enforcer,
            state_storage,
            utility_core_ids_handle,
            cancellation_token,
        )
        .await
    }

    pub async fn from_saved_state(
        config: CCPConfig,
        utility_core_ids_handle: CpuIdsHandle,
        cancellation_token: CancellationToken,
    ) -> CCResult<Self> {
        let state_storage = StateStorage::new(config.state_dir.clone());

        let prev_state = state_storage.try_to_load_data().await?;

        let epoch;
        let msr_enforcer;

        match &prev_state {
            Some(prev_state) => {
                utility_core_ids_handle.set_cores(prev_state.utility_cores.clone());

                // if there is a state, then it means that CCP crashed without setting back
                // possibly changed original MSR state, so, let's set it back
                if !config.optimizations.msr_enabled {
                    cease_prev_msr_policy(prev_state);
                }

                epoch = Some(prev_state.epoch_params);

                msr_enforcer = MSRModeEnforcer::from_preset(
                    config.optimizations.msr_enabled,
                    prev_state.msr_state.msr_preset.clone(),
                );
            }
            None => {
                epoch = None;
                msr_enforcer = MSRModeEnforcer::from_os(config.optimizations.msr_enabled);
            }
        }

        let mut prover = Self::create_prover(
            config,
            epoch,
            msr_enforcer,
            state_storage,
            utility_core_ids_handle,
            cancellation_token,
        )
        .await?;

        if let Some(prev_state) = &prev_state {
            prover
                .apply_cc_parameters(prev_state.epoch_params, &prev_state.cu_allocation)
                .await?;
        }

        Ok(prover)
    }

    async fn create_prover(
        config: CCPConfig,
        epoch: Option<EpochParameters>,
        msr_enforcer: MSRModeEnforcer,
        state_storage: StateStorage,
        utility_core_ids_handle: CpuIdsHandle,
        cancellation_token: CancellationToken,
    ) -> CCResult<Self> {
        let proof_dir = config.state_dir.join(PROOF_DIR);
        let mut proof_drainer = ProofStorageDrainer::new(proof_dir.clone());
        let start_proof_idx = proof_drainer.validate_proofs(&epoch).await?;

        log::info!("continuing from proof index {start_proof_idx}");

        let prev_global_nonce = epoch.map(|epoch| epoch.global_nonce);
        let (state_operating_control_inlet, state_operating_control_outlet) =
            tokio::sync::mpsc::channel(config.rpc_endpoint.control_queue_size);

        let state_operating_thread = StateOperatingThread::spawn(
            start_proof_idx,
            proof_dir,
            prev_global_nonce,
            config.rpc_endpoint.utility_queue_size,
            state_operating_control_outlet,
            state_storage,
            proof_drainer,
        );

        let hashrate_collector = Arc::new(Mutex::new(HashrateCollector::new()));
        let hashrate_handler = HashrateHandler::new(
            hashrate_collector.clone(),
            config.state_dir.clone(),
            config.logs.report_hashrate,
        )?;
        // let (stats_collecting_control_inlet, stats_collecting_control_outlet) =
        //     tokio::sync::mpsc::channel(config.rpc_endpoint.control_queue_size);
        let stats_collecting_thread =
            StatsCollectingThread::spawn(hashrate_handler, config.rpc_endpoint.utility_queue_size);

        let prometheus_endpoint = config.prometheus_endpoint.as_ref().map(|endpoint_cfg| {
            PrometheusEndpoint::new(
                (endpoint_cfg.host.clone(), endpoint_cfg.port),
                hashrate_collector,
            )
        });

        let cu_prover_config = CUProverConfig::new(config.optimizations, config.workers);
        let prover = Self {
            cu_provers: HashMap::new(),
            cu_prover_config,
            status: CCStatus::Idle,
            state_operating_thread,
            stats_collecting_thread,
            prometheus_endpoint,
            msr_enforcer,
            utility_core_ids_handle,
            state_operating_control_inlet,
            cancellation_token,
        };

        Ok(prover)
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
        self.shutdown().await
    }

    #[allow(clippy::needless_lifetimes)]
    async fn stop_provers_nonblocking<'provers>(&'provers self) -> CCResult<()> {
        let nonblocking_closure =
            move |_: usize, (_, prover): (&PhysicalCoreId, &'provers CUProver)| {
                prover.stop_nonblocking().boxed()
            };

        run_unordered(self.cu_provers.iter(), nonblocking_closure).await?;
        Ok(())
    }

    async fn join_provers(&mut self) -> CCResult<()> {
        let join_closure =
            move |_: usize, (_, prover): (PhysicalCoreId, CUProver)| prover.join().boxed();

        run_unordered(self.cu_provers.drain(), join_closure).await?;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> CCResult<()> {
        log::info!("Shutting down prover...");
        // stop service background threads
        // TODO
        self.state_operating_thread.shutdown().await?;
        self.stats_collecting_thread.shutdown().await?;

        if let Some(prometheus_endpoint) = &mut self.prometheus_endpoint {
            prometheus_endpoint.shutdown().await?;
        }

        log::info!("Shutting down prover done");
        Ok(())
    }

    pub async fn save_state(
        &self,
        epoch_state: EpochParameters,
        cu_allocation: CUAllocation,
    ) -> CCResult<()> {
        let original_msr_preset = self.msr_enforcer.original_preset();
        let msr_state = MSRState::new(original_msr_preset.clone());

        let utility_cores = self.utility_core_ids_handle.get_cores();

        let state = CCPState {
            epoch_params: epoch_state,
            cu_allocation,
            utility_cores,
            msr_state,
        };
        self.call_utility_save_state(Some(state)).await
    }

    #[inline]
    pub async fn save_no_state(&self) -> CCResult<()> {
        self.call_utility_save_state(None).await
    }

    async fn apply_cc_parameters(
        &mut self,
        new_epoch: EpochParameters,
        new_allocation: &HashMap<PhysicalCoreId, CUID>,
    ) -> Result<(), <CCProver as NoxCCPApi>::Error> {
        let roadmap = CCProverAlignmentRoadmap::make(
            new_allocation.clone(),
            new_epoch,
            &self.cu_provers,
            self.status,
        );
        self.align_with(roadmap).await?;
        self.status = CCStatus::Running { epoch: new_epoch };

        Ok(())
    }

    async fn call_utility_save_state(&self, state: Option<CCPState>) -> CCResult<()> {
        let (notifier, wait_for) = oneshot::channel();
        let message = StateOperatingControlMessage::SaveConfig { notifier, state };
        self.execute_utility_call(message, wait_for).await
    }

    async fn call_utility_cleanup_proof_cache(&self) -> CCResult<()> {
        let (notifier, wait_for) = oneshot::channel();
        let message = StateOperatingControlMessage::CleanProofCache { notifier };
        // if the queue is full, it a sign of serious problem
        self.execute_utility_call(message, wait_for).await
    }

    async fn call_utility_get_proofs_after(&self, proof_idx: ProofIdx) -> CCResult<Vec<CCProof>> {
        let (notifier, wait_for) = oneshot::channel();
        let message = StateOperatingControlMessage::GetProofs {
            proof_idx,
            notifier,
        };
        self.execute_utility_call(message, wait_for).await
    }

    async fn execute_utility_call<T>(
        &self,
        message: StateOperatingControlMessage,
        wait_for: oneshot::Receiver<Result<T, std::io::Error>>,
    ) -> Result<T, CCProverError> {
        use tokio::sync::mpsc::error::TrySendError;

        self.state_operating_control_inlet
            .try_send(message)
            .map_err(|e| match e {
                // if the queue is full, it a sign of serious problem
                TrySendError::Full(_) => CCProverError::UtilityFullError,
                // if the queue is closed, the problem is even more serios
                TrySendError::Closed(_) => CCProverError::UtilityClosedError,
            })?;
        Ok(wait_for
            .await
            .map_err(|_| CCProverError::UtilityClosedError)??)
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
                self.call_utility_cleanup_proof_cache().await?;
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
    pub(self) fn cu_creation(
        &mut self,
        state: actions_state::CreateCUProverState,
        epoch: EpochParameters,
    ) -> future::BoxFuture<'static, CUResult<AlignmentPostAction>> {
        let prover_config = self.cu_prover_config.clone();
        let to_state_operating = self.state_operating_thread.get_to_state_operating_channel();
        let to_stats_collecting = self
            .stats_collecting_thread
            .get_to_stats_collecting_channel();
        let msr_enforcer = self.msr_enforcer.clone();
        let child_cancel_token = self.cancellation_token.clone();

        async move {
            let mut prover = CUProver::create(
                prover_config,
                to_state_operating,
                to_stats_collecting,
                msr_enforcer,
                state.new_core_id,
            )
            .await?;
            prover
                .new_epoch(epoch, state.new_cu_id, child_cancel_token)
                .await?;

            Ok(AlignmentPostAction::KeepProver(prover))
        }
        .boxed()
    }

    pub(self) fn cu_removal(
        &mut self,
        state: actions_state::RemoveCUProverState,
    ) -> future::BoxFuture<'static, CUResult<AlignmentPostAction>> {
        let prover = self.cu_provers.remove(&state.current_core_id).unwrap();
        async move {
            prover.stop_join().await?;
            Ok(AlignmentPostAction::Nothing)
        }
        .boxed()
    }

    pub(self) fn new_cc_job(
        &mut self,
        state: actions_state::NewCCJobState,
        epoch: EpochParameters,
    ) -> future::BoxFuture<'static, CUResult<AlignmentPostAction>> {
        let mut prover = self.cu_provers.remove(&state.current_core_id).unwrap();
        let cancel_token = self.cancellation_token.clone();
        async move {
            prover
                .new_epoch(epoch, state.new_cu_id, cancel_token)
                .await?;
            Ok(AlignmentPostAction::KeepProver(prover))
        }
        .boxed()
    }

    pub(self) fn new_cc_job_repin(
        &mut self,
        state: actions_state::NewCCJobWithRepiningState,
        epoch: EpochParameters,
    ) -> future::BoxFuture<'static, CUResult<AlignmentPostAction>> {
        let mut prover = self.cu_provers.remove(&state.current_core_id).unwrap();
        let cancel_token = self.cancellation_token.clone();
        async move {
            prover.pin(state.new_core_id).await?;
            prover
                .new_epoch(epoch, state.new_cu_id, cancel_token)
                .await?;
            Ok(AlignmentPostAction::KeepProver(prover))
        }
        .boxed()
    }
}

fn cease_prev_msr_policy(prev_state: &CCPState) {
    let msr_enforcer = MSRModeEnforcer::from_preset(true, prev_state.msr_state.msr_preset.clone());

    for (&physical_core_id, _) in prev_state.cu_allocation.iter() {
        let core_id: u32 = physical_core_id.into();
        let logical_core_id = core_id.into();
        if let Err(error) = msr_enforcer.cease(logical_core_id) {
            log::error!(
                "{logical_core_id}: failed to cease MSR policy from previous state with {error}"
            );
        }
    }
}
