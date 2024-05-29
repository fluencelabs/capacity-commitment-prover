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

mod cu_governor;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;

use ccp_config::CCPConfig;
use ccp_msr::state::MSRState;
use ccp_msr::MSRModeEnforcer;
use ccp_shared::types::CUID;
use eyre::Context;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::oneshot;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::try_join;
use tokio_util::sync::CancellationToken;

use ccp_shared::nox_ccp_api::NoxCCPApi;
use ccp_shared::proof::CCProof;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::CUAllocation;
use ccp_shared::types::EpochParameters;

use self::cu_governor::CUGovernor;
use crate::cpuids_handle::CpuIdsHandle;
use crate::cu::proving_thread::sync::to_state_operating_message::GovernorOperatingInlet;
use crate::cu::proving_thread::sync::to_state_operating_message::GovernorOperatingOutlet;
use crate::cu::proving_thread::sync::to_state_operating_message::StateOperatingControlInlet;
use crate::cu::proving_thread::sync::to_state_operating_message::StateOperatingControlMessage;
use crate::errors::CCProverError;
use crate::hashrate::prometheus::PrometheusEndpoint;
use crate::hashrate::HashrateCollector;
use crate::hashrate::HashrateHandler;
use crate::proof_storage::ProofStorageDrainer;
use crate::state_operating_thread::StateOperatingThread;
use crate::state_storage::CCPState;
use crate::state_storage::EpochAndAllocation;
use crate::state_storage::StateStorage;
use crate::stats_collecting_thread::StatsCollectingThread;

const PROOF_DIR: &str = "cc_proofs";

pub type CCResult<T> = Result<T, CCProverError>;

pub struct CCProver {
    governor_task: JoinHandle<()>,
    governor_inlet: GovernorOperatingInlet,
    cu_governor: Arc<RwLock<CUGovernor>>,
    state_operating_thread: StateOperatingThread,
    state_operating_control_inlet: StateOperatingControlInlet,
    stats_collecting_thread: StatsCollectingThread,
    prometheus_endpoint: Option<PrometheusEndpoint>,
    utility_core_ids_handle: CpuIdsHandle,
    initial_msr_state: MSRState,
    target_allocation: Option<EpochAndAllocation>,
}

impl CCProver {
    pub async fn new(config: CCPConfig) -> CCResult<Self> {
        let utility_core_ids_handle = CpuIdsHandle::new(config.tokio.utility_cores_ids.clone());

        let msr_enforcer = MSRModeEnforcer::from_os(config.optimizations.msr_enabled);
        let state_storage = StateStorage::new(config.state_dir.clone());
        let cancellation_token = CancellationToken::new();

        let prover = Self::create_prover(
            config,
            None,
            msr_enforcer,
            state_storage,
            utility_core_ids_handle,
            cancellation_token,
        )
        .await?;
        Ok(prover)
    }

    pub async fn from_saved_state(
        config: CCPConfig,
        utility_core_ids_handle: CpuIdsHandle,
        cancellation_token: CancellationToken,
    ) -> CCResult<Self> {
        let state_storage = StateStorage::new(config.state_dir.clone());
        let prev_state = state_storage.try_to_load_data().await?;

        let commitment;
        let msr_enforcer;

        match prev_state {
            Some(prev_state) => {
                utility_core_ids_handle.set_cores(prev_state.utility_cores.clone());

                // if there is a state, then it means that CCP crashed without setting back
                // possibly changed original MSR state, so, let's set it back
                if !config.optimizations.msr_enabled {
                    cease_prev_msr_policy(&prev_state);
                }

                commitment = Some(EpochAndAllocation {
                    epoch_params: prev_state.epoch_params,
                    cu_allocation: prev_state.cu_allocation,
                });

                msr_enforcer = MSRModeEnforcer::from_preset(
                    config.optimizations.msr_enabled,
                    prev_state.msr_state.msr_preset.clone(),
                );
            }
            None => {
                commitment = None;
                msr_enforcer = MSRModeEnforcer::from_os(config.optimizations.msr_enabled);
            }
        }

        let prover = Self::create_prover(
            config,
            commitment,
            msr_enforcer,
            state_storage,
            utility_core_ids_handle,
            cancellation_token,
        )
        .await?;

        Ok(prover)
    }

    async fn create_prover(
        config: CCPConfig,
        target_commitment: Option<EpochAndAllocation>,
        msr_enforcer: MSRModeEnforcer,
        state_storage: StateStorage,
        utility_core_ids_handle: CpuIdsHandle,
        cancellation_token: CancellationToken,
    ) -> CCResult<Self> {
        let proof_dir = config.state_dir.join(PROOF_DIR);
        let mut proof_drainer = ProofStorageDrainer::new(proof_dir.clone());
        let start_proof_idx = proof_drainer
            .validate_proofs(target_commitment.as_ref().map(|comm| &comm.epoch_params))
            .await?;
        log::info!("continuing from proof index {start_proof_idx}");

        let hashrate_collector = Arc::new(StdMutex::new(HashrateCollector::new()));
        let hashrate_handler = HashrateHandler::new(
            hashrate_collector.clone(),
            config.state_dir.clone(),
            config.logs.report_hashrate,
        )?;

        let prev_global_nonce = target_commitment
            .as_ref()
            .map(|comm| comm.epoch_params.global_nonce);
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

        let stats_collecting_thread =
            StatsCollectingThread::spawn(hashrate_handler, config.rpc_endpoint.utility_queue_size);

        let prometheus_endpoint = config.prometheus_endpoint.as_ref().map(|endpoint_cfg| {
            PrometheusEndpoint::new(
                (endpoint_cfg.host.clone(), endpoint_cfg.port),
                hashrate_collector,
            )
        });

        let initial_msr_state = MSRState::new(msr_enforcer.original_preset().clone());

        let cu_governor = CUGovernor::new(
            config.optimizations,
            config.workers,
            stats_collecting_thread.get_to_stats_collecting_channel(),
            state_operating_thread.get_to_state_operating_channel(),
            state_operating_control_inlet.clone(),
            msr_enforcer,
            cancellation_token,
        );
        let cu_governor = Arc::new(RwLock::new(cu_governor));

        // `facace_queue_size` is historically used for this queue
        let governor_queue_size = config.rpc_endpoint.facade_queue_size;
        let (governor_outlet, governor_inlet) = mpsc::channel(governor_queue_size);
        let governor_task = tokio::task::spawn(facade_loop(cu_governor.clone(), governor_inlet));

        let prover = Self {
            governor_task,
            governor_inlet: governor_outlet,
            cu_governor,
            state_operating_thread,
            state_operating_control_inlet,
            stats_collecting_thread,
            prometheus_endpoint,
            utility_core_ids_handle,

            initial_msr_state,
            target_allocation: target_commitment,
        };

        if let Some(commitment) = &prover.target_allocation {
            let (dummy_sender, _receiver) = oneshot::channel();
            prover
                .governor_inlet
                .send(GovernorMessage::OnActiveCommitment(
                    commitment.epoch_params,
                    commitment.cu_allocation.clone(),
                    dummy_sender,
                ))
                .await
                .expect("the queue cannot be full on startup");
        }
        Ok(prover)
    }

    // TODO rename
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

    #[inline]
    pub async fn save_state(&self) -> CCResult<()> {
        self.call_utility_save_state().await
    }

    async fn call_utility_save_state(&self) -> CCResult<()> {
        let utility_cores = self.utility_core_ids_handle.get_cores();

        let state = self.target_allocation.as_ref().map(|commitment| CCPState {
            epoch_params: commitment.epoch_params,
            cu_allocation: commitment.cu_allocation.clone(),
            msr_state: self.initial_msr_state.clone(),
            utility_cores,
        });
        let (notifier, wait_for) = oneshot::channel();
        let message = StateOperatingControlMessage::SaveConfig { notifier, state };
        self.execute_utility_call(message, wait_for).await
    }

    async fn call_utility_get_batch_proofs_after(
        &self,
        batch_length: usize,
        batch_count: usize,
        last_known_proofs: HashMap<CUID, ProofIdx>,
    ) -> CCResult<Option<HashMap<CUID, Vec<Vec<CCProof>>>>> {
        let (notifier, wait_for) = oneshot::channel();
        let message = StateOperatingControlMessage::GetBatchProofs {
            batch_length,
            batch_count,
            last_known_proofs,
            notifier,
        };
        self.execute_utility_call(message, wait_for).await
    }

    async fn call_utility_get_all_proofs_after(
        &self,
        last_known_proofs: HashMap<CUID, ProofIdx>,
    ) -> CCResult<HashMap<CUID, Vec<CCProof>>> {
        let (notifier, wait_for) = oneshot::channel();
        let message = StateOperatingControlMessage::GetAllProofs {
            last_known_proofs,
            notifier,
        };
        self.execute_utility_call(message, wait_for).await
    }

    pub async fn stop(self) -> CCResult<()> {
        use futures::FutureExt;

        // TODO check it once again
        std::mem::drop(self.governor_inlet);

        let mut guard = self.cu_governor.write().await;
        try_join! {
            self.governor_task.map(|res| res.map_err(Into::into)),
            guard.stop()
        }?;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> CCResult<()> {
        self.cu_governor.write().await.shutdown();

        let state_op_shutdown_res =
            self.state_operating_thread
                .shutdown()
                .await
                .inspect_err(|shutdown_err| {
                    log::error!("state_operationg shutdown: {shutdown_err}");
                });

        let stats_coll_shutdown_res =
            self.stats_collecting_thread
                .shutdown()
                .await
                .inspect_err(|shutdown_err| {
                    log::error!("stats_collecting shutdown: {shutdown_err}");
                });

        if let Some(prometheus_endpoint) = &mut self.prometheus_endpoint {
            prometheus_endpoint
                .shutdown()
                .await
                .inspect_err(|shutdown_err| {
                    log::error!("prometheus_endpoint shutdown: {shutdown_err}")
                })?;
        }

        state_op_shutdown_res?;
        stats_coll_shutdown_res?;

        Ok(())
    }
}

pub(crate) enum GovernorMessage {
    OnActiveCommitment(EpochParameters, CUAllocation, oneshot::Sender<()>),
    OnNoCommitment,
}

// implement for specific prover to implement granular state saving
impl NoxCCPApi for CCProver {
    type Error = eyre::Error;
    type CommitmentError = oneshot::error::RecvError;
    type CommitmentReadiness = oneshot::Receiver<()>;

    async fn on_active_commitment(
        &mut self,
        epoch_params: EpochParameters,
        cu_allocation: CUAllocation,
    ) -> Result<oneshot::Receiver<()>, Self::Error> {
        self.target_allocation = Some(EpochAndAllocation {
            epoch_params,
            cu_allocation: cu_allocation.clone(),
        });
        self.save_state()
            .await
            // CCProverError is not Sync, so we convert it to a string in situ
            .map_err(|e| eyre::eyre!(e.to_string()))?;

        let (sender, receiver) = oneshot::channel();

        self.governor_inlet
            .try_send(GovernorMessage::OnActiveCommitment(
                epoch_params,
                cu_allocation,
                sender,
            ))
            .context("on_active_commitment")?;
        Ok(receiver)
    }

    async fn on_no_active_commitment(&mut self) -> Result<(), Self::Error> {
        self.target_allocation = None;
        self.save_state()
            .await
            // CCProverError is not Sync, so we convert it to a string in situ
            .map_err(|e| eyre::eyre!(e.to_string()))?;

        self.governor_inlet
            .try_send(GovernorMessage::OnNoCommitment)
            .context("on_no_active_commitment")?;
        Ok(())
    }

    #[inline]
    async fn get_proofs_after(
        &self,
        last_known_proofs: HashMap<CUID, ProofIdx>,
    ) -> Result<HashMap<CUID, Vec<CCProof>>, Self::Error> {
        self.call_utility_get_all_proofs_after(last_known_proofs)
            .await
            // CCProverError is not Sync, so we convert it to a string in situ
            .map_err(|e| eyre::eyre!(e.to_string()))
    }

    #[inline]
    async fn get_batch_proofs_after(
        &self,
        batch_length: usize,
        batch_count: usize,
        last_known_proofs: HashMap<CUID, ProofIdx>,
    ) -> Result<Option<HashMap<CUID, Vec<Vec<CCProof>>>>, Self::Error> {
        self.call_utility_get_batch_proofs_after(batch_length, batch_count, last_known_proofs)
            .await
            // CCProverError is not Sync, so we convert it to a string in situ
            .map_err(|e| eyre::eyre!(e.to_string()))
    }

    async fn realloc_utility_cores(&self, utility_core_ids: Vec<cpu_utils::LogicalCoreId>) {
        self.utility_core_ids_handle.set_cores(utility_core_ids);
        // realloc doesn't return result, ignore any error
        let _ = self
            .save_state()
            .await
            // CCProverError is not Sync, so we convert it to a string in situ
            .map_err(|e| eyre::eyre!(e.to_string()));
    }
}

async fn facade_loop(prover: Arc<RwLock<CUGovernor>>, mut from_facade: GovernorOperatingOutlet) {
    use GovernorMessage::*;
    while let Some(message) = receive_last(&mut from_facade).await {
        let mut guard = prover.write().await;
        match message {
            OnActiveCommitment(epoch_parameters, cu_allocation, sender) => {
                let res = guard
                    .on_active_commitment(epoch_parameters, cu_allocation)
                    .await;
                if let Err(e) = res {
                    log::error!("nested prover on_active_commitment failed: {e}");
                }
                // if receiver is dead, it's ok
                let _ = sender.send(());
            }
            OnNoCommitment => {
                let res = guard.on_no_active_commitment().await;
                if let Err(e) = res {
                    log::error!("nested prover on_no_active_commitment failed: {e}");
                }
            }
        }
    }
}

async fn receive_last<T>(receiver: &mut mpsc::Receiver<T>) -> Option<T> {
    // wating for a new value
    let mut val = receiver.recv().await?;
    //  non-wating getting of the last available value
    loop {
        match receiver.try_recv() {
            Ok(v) => {
                val = v;
            }
            Err(TryRecvError::Empty) => {
                return Some(val);
            }
            Err(TryRecvError::Disconnected) => {
                return None;
            }
        }
    }
}

fn cease_prev_msr_policy(prev_state: &CCPState) {
    use ccp_msr::MSREnforce as _;

    let msr_enforcer = MSRModeEnforcer::from_preset(true, prev_state.msr_state.msr_preset.clone());

    for &physical_core_id in prev_state.cu_allocation.keys() {
        let core_id: u32 = physical_core_id.into();
        let logical_core_id = core_id.into();
        if let Err(error) = msr_enforcer.cease(logical_core_id) {
            log::error!(
                "{logical_core_id}: failed to cease MSR policy from previous state with {error}"
            );
        }
    }
}
