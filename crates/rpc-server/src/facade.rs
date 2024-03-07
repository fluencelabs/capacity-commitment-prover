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

use std::fmt::Display;
use std::sync::Arc;

use eyre::Context;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

use capacity_commitment_prover::CCProver;
use ccp_shared::nox_ccp_api::NoxCCPApi;
use ccp_shared::proof::CCProof;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::CUAllocation;
use ccp_shared::types::EpochParameters;

/// An façade that handles RPC calls in background.
pub struct BackgroundFacade<P> {
    to_worker: mpsc::Sender<FacadeMessage>,
    prover: Arc<RwLock<P>>,
    worker: JoinHandle<()>,
}

impl<P> BackgroundFacade<P>
where
    P: NoxCCPApi + Sync + 'static,
    <P as NoxCCPApi>::Error: Display,
{
    pub fn new(prover: P) -> Self {
        let prover = Arc::new(RwLock::new(prover));

        let (to_worker, from_facade) = mpsc::channel(100);

        let worker = tokio::task::spawn(facade_loop(prover.clone(), from_facade));

        Self {
            to_worker,
            prover,
            worker,
        }
    }

    pub async fn stop(self) -> Result<(), tokio::task::JoinError> {
        std::mem::drop(self.to_worker);
        self.worker.await
    }
}

enum FacadeMessage {
    OnActiveCommitment(EpochParameters, CUAllocation),
    OnNoCommitment,
}

// implement for specific prover to implement granular state saving
impl NoxCCPApi for BackgroundFacade<CCProver> {
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
            let guard = self.prover.read().await;
            guard
                .save_state(epoch_parameters, cu_allocation.clone())
                .await?;
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
            let guard = self.prover.read().await;
            guard.save_no_state().await?;
        }
        self.to_worker
            .send(FacadeMessage::OnNoCommitment)
            .await
            .context("on_no_active_commitment")
    }

    async fn get_proofs_after(&self, proof_idx: ProofIdx) -> Result<Vec<CCProof>, Self::Error> {
        let guard = match self.prover.try_read() {
            Ok(g) => g,
            Err(_) => {
                tracing::debug!(
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
}

#[tracing::instrument(skip_all)]
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
                    tracing::error!("nested prover on_active_commitment failed: {e}");
                }
            }
            OnNoCommitment => {
                let res = guard.on_no_active_commitment().await;
                if let Err(e) = res {
                    tracing::error!("nested prover on_no_active_commitment failed: {e}");
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
