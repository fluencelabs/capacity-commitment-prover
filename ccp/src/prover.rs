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

use std::fmt::Display;
use std::sync::Arc;

use ccp_config::CCPConfig;
use eyre::Context;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use ccp_shared::nox_ccp_api::NoxCCPApi;
use ccp_shared::proof::CCProof;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::CUAllocation;
use ccp_shared::types::EpochParameters;
use tokio::try_join;
use tokio_util::sync::CancellationToken;

use self::cu_governor::CUGovernor;
use crate::cpuids_handle::CpuIdsHandle;
use crate::errors::CCProverError;

pub type CCResult<T> = Result<T, CCProverError>;

/// An façade that handles RPC calls in background.
pub struct CCProver {
    to_worker: mpsc::Sender<FacadeMessage>,
    cu_governor: Arc<RwLock<CUGovernor>>,
    worker: JoinHandle<()>,
    utility_core_ids_handle: CpuIdsHandle,
}

impl CCProver {
    pub async fn new(config: CCPConfig) -> CCResult<Self> {
        let facade_queue_size = config.rpc_endpoint.facade_queue_size;
        let utility_core_ids_handle = CpuIdsHandle::new(config.tokio.utility_cores_ids.clone());

        let cu_governor = CUGovernor::new(config, utility_core_ids_handle.clone()).await?;
        let cu_governor = Arc::new(RwLock::new(cu_governor));

        let prover = Self::create_prover(facade_queue_size, cu_governor, utility_core_ids_handle);
        Ok(prover)
    }

    pub async fn from_saved_state(
        config: CCPConfig,
        utility_core_ids_handle: CpuIdsHandle,
        cancellation_token: CancellationToken,
        facade_queue_size: usize,
    ) -> CCResult<Self> {
        let cu_governor = CUGovernor::from_saved_state(
            config,
            utility_core_ids_handle.clone(),
            cancellation_token,
        )
        .await?;
        let cu_governor = Arc::new(RwLock::new(cu_governor));

        Ok(Self::create_prover(
            facade_queue_size,
            cu_governor,
            utility_core_ids_handle,
        ))
    }

    fn create_prover(
        facade_queue_size: usize,
        cu_governor: Arc<RwLock<CUGovernor>>,
        utility_core_ids_handle: CpuIdsHandle,
    ) -> CCProver {
        let (to_worker, from_facade) = mpsc::channel(facade_queue_size);

        let worker = tokio::task::spawn(facade_loop(cu_governor.clone(), from_facade));

        Self {
            to_worker,
            cu_governor,
            worker,
            utility_core_ids_handle,
        }
    }

    pub async fn stop(self) -> CCResult<()> {
        // TODO check it once again
        std::mem::drop(self.to_worker);

        let mut guard = self.cu_governor.write().await;
        try_join!(
            async { self.worker.await.map_err(Into::into) },
            guard.stop()
        )?;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> CCResult<()> {
        self.cu_governor.write().await.shutdown().await?;
        Ok(())
    }
}

enum FacadeMessage {
    OnActiveCommitment(EpochParameters, CUAllocation),
    OnNoCommitment,
}

// implement for specific prover to implement granular state saving
impl NoxCCPApi for CCProver {
    type Error = eyre::Error;

    async fn on_active_commitment(
        &mut self,
        epoch_parameters: EpochParameters,
        cu_allocation: CUAllocation,
    ) -> Result<(), Self::Error> {
        // Save state early so that caller is sure it is saved.
        // Please note that the caller may be still stuck if dataset generation
        // is in progress and writer lock is held.
        {
            let guard = self.cu_governor.read().await;
            guard
                .save_state(epoch_parameters, cu_allocation.clone())
                .await
                // CCProverError is not Sync, so we convert it to a string in situ
                .map_err(|e| eyre::eyre!(e.to_string()))?;
        }
        self.to_worker
            .try_send(FacadeMessage::OnActiveCommitment(
                epoch_parameters,
                cu_allocation,
            ))
            .context("on_active_commitment")
    }

    async fn on_no_active_commitment(&mut self) -> Result<(), Self::Error> {
        // Save state early so that caller is sure it is saved.
        // Please note that the caller may be still stuck if dataset generation
        // is in progress and writer lock is held.
        {
            let guard = self.cu_governor.read().await;
            guard
                .save_no_state()
                .await
                // CCProverError is not Sync, so we convert it to a string in situ
                .map_err(|e| eyre::eyre!(e.to_string()))?;
        }
        self.to_worker
            .send(FacadeMessage::OnNoCommitment)
            .await
            .context("on_no_active_commitment")
    }

    async fn get_proofs_after(&self, proof_idx: ProofIdx) -> Result<Vec<CCProof>, Self::Error> {
        let guard = match self.cu_governor.try_read() {
            Ok(g) => g,
            Err(_) => {
                log::debug!(
                    "failed to get the prover lock: probably on_active_commitment in progress.\
 Return an empty list.",
                );
                return Ok(vec![]);
            }
        };
        guard
            .get_proofs_after(proof_idx)
            .await
            // CCProverError is not Sync, so we convert it to a string in situ
            .map_err(|e| eyre::eyre!(e.to_string()))
            .context("get_proofs_after")
    }

    async fn realloc_utility_cores(&self, utility_core_ids: Vec<cpu_utils::LogicalCoreId>) {
        self.utility_core_ids_handle.set_cores(utility_core_ids);
    }
}

async fn facade_loop<P>(prover: Arc<RwLock<P>>, mut from_facade: mpsc::Receiver<FacadeMessage>)
where
    P: NoxCCPApi,
    <P as NoxCCPApi>::Error: Display,
{
    use FacadeMessage::*;
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
