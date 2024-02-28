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

use std::sync::Arc;
use std::time::Duration;

use ccp_shared::nox_ccp_api::NoxCCPApi;
use ccp_shared::proof::CCProof;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::CUAllocation;
use ccp_shared::types::EpochParameters;

use eyre::Context;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::Mutex;
use tokio::time::timeout;

const WAIT_LOCK_DURATION: Duration = Duration::from_secs(2);

/// An async façade for RPC.
pub struct OfflineFacade<P> {
    sender_to_worker: mpsc::Sender<FacadeMessage>,
    prover: Arc<Mutex<P>>,
}

impl<P> OfflineFacade<P>
where
    P: NoxCCPApi + 'static,
    <P as NoxCCPApi>::Error: ToString,
{
    pub fn new(prover: P) -> Self {
        let prover = Arc::new(Mutex::new(prover));

        let (sender_to_worker, receiver) = mpsc::channel(100);

        let _worker = tokio::task::spawn(facade_loop(prover.clone(), receiver));

        Self {
            sender_to_worker,
            prover,
        }
    }
}

enum FacadeMessage {
    OnActiveCommitment(EpochParameters, CUAllocation),
    OnNoCommitment,
}

impl<P: NoxCCPApi> NoxCCPApi for OfflineFacade<P>
where
    <P as NoxCCPApi>::Error: ToString,
{
    type Error = eyre::Error;

    async fn on_active_commitment(
        &mut self,
        epoch_parameters: EpochParameters,
        cu_allocation: CUAllocation,
    ) -> Result<(), Self::Error> {
        self.sender_to_worker
            .try_send(FacadeMessage::OnActiveCommitment(
                epoch_parameters,
                cu_allocation,
            ))
            .context("on_active_commitment")
    }

    async fn on_no_active_commitment(&mut self) -> Result<(), Self::Error> {
        self.sender_to_worker
            .send(FacadeMessage::OnNoCommitment)
            .await
            .context("on_no_active_commitment")
    }

    async fn get_proofs_after(&self, proof_idx: ProofIdx) -> Result<Vec<CCProof>, Self::Error> {
        let guard = match timeout(WAIT_LOCK_DURATION, self.prover.lock()).await {
            Ok(g) => g,
            Err(e) => {
                return Err(e).context("failed to get lock in get_proofs_after: lock is still busy")
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

async fn facade_loop<P>(prover: Arc<Mutex<P>>, mut receiver: mpsc::Receiver<FacadeMessage>)
where
    P: NoxCCPApi,
    <P as NoxCCPApi>::Error: ToString,
{
    use FacadeMessage::*;
    while let Some(message) = receive_last(&mut receiver).await {
        let mut guard = prover.lock().await;
        match message {
            OnActiveCommitment(epoch_parameters, cu_allocation) => {
                let res = guard
                    .on_active_commitment(epoch_parameters, cu_allocation)
                    .await;
                if let Err(e) = res {
                    tracing::error!("nested prover on_active_commitment failed: {e:?}");
                }
            }
            OnNoCommitment => {
                let res = guard.on_no_active_commitment().await;
                if let Err(e) = res {
                    tracing::error!("nested prover on_no_active_commitment failed: {e:?}");
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
