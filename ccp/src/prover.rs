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

use std::sync::Arc;
use std::sync::Mutex as StdMutex;

use ccp_config::CCPConfig;
use ccp_msr::state::MSRState;
use ccp_msr::MSRModeEnforcer;
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
use crate::cu::proving_thread::sync::to_utility_message::UtilityControlInlet;
use crate::cu::proving_thread::sync::to_utility_message::UtilityControlMessage;
use crate::errors::CCProverError;
use crate::hashrate::prometheus::PrometheusEndpoint;
use crate::hashrate::HashrateCollector;
use crate::hashrate::HashrateHandler;
use crate::proof_storage::ProofStorageDrainer;
use crate::state_storage::CCPState;
use crate::state_storage::Commitment;
use crate::state_storage::StateStorage;
use crate::utility_thread::UtilityThread;

const PROOF_DIR: &str = "cc_proofs";

pub type CCResult<T> = Result<T, CCProverError>;

pub struct CCProver {
    governor_task: JoinHandle<()>,
    governor_outlet: mpsc::Sender<GovernorMessage>,
    cu_governor: Arc<RwLock<CUGovernor>>,
    utility_thread: UtilityThread,
    utility_control_inlet: UtilityControlInlet,
    prometheus_endpoint: Option<PrometheusEndpoint>,
    utility_core_ids_handle: CpuIdsHandle,

    initial_msr_state: MSRState,
    target_commitment: Option<Commitment>,
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

        utility_core_ids_handle.set_cores(prev_state.utility_cores.clone());

        if let Some(prev_commitment) = &prev_state.commitment {
            if !config.optimizations.msr_enabled {
                cease_prev_msr_policy(prev_commitment, &prev_state.msr_state);
            }
        }

        let msr_enforcer = if prev_state.msr_state.msr_preset.is_empty() {
            // empty present can be in the following situations:
            // 1. no previous config was used
            //    then loading MSR from OS is must
            // 2. there is a previous config, but it was run with msr_enabled being false
            //    then loading MSR is either absolutely cheap (when msr_enabled is still false) or must
            MSRModeEnforcer::from_os(config.optimizations.msr_enabled)
        } else {
            MSRModeEnforcer::from_preset(
                config.optimizations.msr_enabled,
                prev_state.msr_state.msr_preset,
            )
        };

        let prover = Self::create_prover(
            config,
            prev_state.commitment.clone(),
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
        target_commitment: Option<Commitment>,
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
            .map(|comm| comm.epoch_params.global_nonce.clone());
        let (utility_control_inlet, utility_control_outlet) =
            tokio::sync::mpsc::channel(config.rpc_endpoint.control_queue_size);
        let utility_thread = UtilityThread::spawn(
            start_proof_idx,
            proof_dir,
            prev_global_nonce,
            hashrate_handler,
            config.rpc_endpoint.utility_queue_size,
            utility_control_outlet,
            state_storage,
            proof_drainer,
        );

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
            msr_enforcer,
            cancellation_token,
            utility_thread.get_to_utility_channel(),
            utility_control_inlet.clone(),
        );
        let cu_governor = Arc::new(RwLock::new(cu_governor));

        // `facace_queue_size` is historically used for this queue
        let governor_queue_size = config.rpc_endpoint.facade_queue_size;
        let (governor_outlet, governor_inlet) = mpsc::channel(governor_queue_size);
        let governor_task = tokio::task::spawn(facade_loop(cu_governor.clone(), governor_inlet));

        let prover = Self {
            governor_task,
            governor_outlet,
            cu_governor,
            utility_core_ids_handle,
            utility_thread,
            utility_control_inlet,
            prometheus_endpoint,

            initial_msr_state,
            target_commitment,
        };

        if let Some(commitment) = &prover.target_commitment {
            prover
                .governor_outlet
                .send(GovernorMessage::OnActiveCommitment(
                    commitment.epoch_params,
                    commitment.cu_allocation.clone(),
                ))
                .await
                .expect("the queue cannot be full on startup");
        }

        Ok(prover)
    }

    async fn call_utility_get_proofs_after(&self, proof_idx: ProofIdx) -> CCResult<Vec<CCProof>> {
        let (notifier, wait_for) = oneshot::channel();
        let message = UtilityControlMessage::GetProofs {
            proof_idx,
            notifier,
        };
        self.execute_utility_call(message, wait_for).await
    }

    async fn execute_utility_call<T>(
        &self,
        message: UtilityControlMessage,
        wait_for: oneshot::Receiver<Result<T, std::io::Error>>,
    ) -> Result<T, CCProverError> {
        use tokio::sync::mpsc::error::TrySendError;

        self.utility_control_inlet
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

        let state = CCPState {
            commitment: self.target_commitment.clone(),
            utility_cores,
            msr_state: self.initial_msr_state.clone(),
        };

        let (notifier, wait_for) = oneshot::channel();
        let message = UtilityControlMessage::SaveConfig { notifier, state };
        self.execute_utility_call(message, wait_for).await
    }

    pub async fn stop(self) -> CCResult<()> {
        use futures::FutureExt;

        // TODO check it once again
        std::mem::drop(self.governor_outlet);

        let mut guard = self.cu_governor.write().await;
        try_join! {
            self.governor_task.map(|res| res.map_err(Into::into)),
            guard.stop()
        }?;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> CCResult<()> {
        self.cu_governor.write().await.shutdown();

        // TODO does it return some interesting error? log, if so
        let _ = self.utility_thread.shutdown().await;

        if let Some(prometheus_endpoint) = &mut self.prometheus_endpoint {
            prometheus_endpoint.shutdown().await?;
        }
        Ok(())
    }
}

enum GovernorMessage {
    OnActiveCommitment(EpochParameters, CUAllocation),
    OnNoCommitment,
}

// implement for specific prover to implement granular state saving
impl NoxCCPApi for CCProver {
    type Error = eyre::Error;

    async fn on_active_commitment(
        &mut self,
        epoch_params: EpochParameters,
        cu_allocation: CUAllocation,
    ) -> Result<(), Self::Error> {
        self.target_commitment = Some(Commitment {
            epoch_params,
            cu_allocation: cu_allocation.clone(),
        });
        self.save_state()
            .await
            // CCProverError is not Sync, so we convert it to a string in situ
            .map_err(|e| eyre::eyre!(e.to_string()))?;

        self.governor_outlet
            .try_send(GovernorMessage::OnActiveCommitment(
                epoch_params,
                cu_allocation,
            ))
            .context("on_active_commitment")
    }

    async fn on_no_active_commitment(&mut self) -> Result<(), Self::Error> {
        self.target_commitment = None;
        self.save_state()
            .await
            // CCProverError is not Sync, so we convert it to a string in situ
            .map_err(|e| eyre::eyre!(e.to_string()))?;

        self.governor_outlet
            .try_send(GovernorMessage::OnNoCommitment)
            .context("on_no_active_commitment")?;
        Ok(())
    }

    #[inline]
    async fn get_proofs_after(&self, proof_idx: ProofIdx) -> Result<Vec<CCProof>, Self::Error> {
        self.call_utility_get_proofs_after(proof_idx)
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

async fn facade_loop(
    prover: Arc<RwLock<CUGovernor>>,
    mut from_facade: mpsc::Receiver<GovernorMessage>,
) {
    use GovernorMessage::*;
    while let Some(message) = receive_last(&mut from_facade).await {
        let mut guard = prover.write().await;
        match message {
            OnActiveCommitment(epoch_parameters, cu_allocation) => {
                let res = guard
                    .on_active_commitment(epoch_parameters, cu_allocation)
                    .await;
                if let Err(e) = res {
                    log::error!("nested prover on_active_commitment failed: {e}");
                }
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

fn cease_prev_msr_policy(prev_commitment: &Commitment, msr_state: &MSRState) {
    use ccp_msr::MSREnforce as _;

    let msr_enforcer = MSRModeEnforcer::from_preset(true, msr_state.msr_preset.clone());

    for &physical_core_id in prev_commitment.cu_allocation.keys() {
        let core_id: u32 = physical_core_id.into();
        let logical_core_id = core_id.into();
        if let Err(error) = msr_enforcer.cease(logical_core_id) {
            log::error!(
                "{logical_core_id}: failed to cease MSR policy from previous state with {error}"
            );
        }
    }
}
