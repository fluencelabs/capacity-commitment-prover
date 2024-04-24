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

use tokio::sync::mpsc;

use ccp_shared::proof::CCProof;
use ccp_shared::proof::CCProofId;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::GlobalNonce;
use tokio_util::sync::CancellationToken;

use super::message::*;
use super::SOTResult;
use crate::proof_storage::ProofStorageDrainer;
use crate::state_operating_thread::proof_storage::ProofStorage;
use crate::state_storage::StateStorage;

pub(crate) struct StateOperatingThread {
    to_state_operating: ToStateOperatingInlet,
    cancellation: CancellationToken,
    handle: tokio::task::JoinHandle<()>,
}

impl StateOperatingThread {
    pub(crate) fn spawn(
        prev_proof_idx: ProofIdx,
        proof_storage_dir: std::path::PathBuf,
        prev_global_nonce: Option<GlobalNonce>,
        state_operating_queue_size: usize,
        state_operating_control_outlet: StateOperatingControlOutlet,
        state_storage: StateStorage,
        proof_drainer: ProofStorageDrainer,
    ) -> Self {
        let (to_state_operating, from_state_operating) = mpsc::channel(state_operating_queue_size);

        let cancellation = CancellationToken::new();

        let proof_storage = ProofStorage::new(proof_storage_dir);
        let proofs_handler = NewProofHandler::new(proof_storage, prev_proof_idx, prev_global_nonce);
        let sot_impl = StateOperatingThreadImpl::new(
            from_state_operating,
            state_operating_control_outlet,
            cancellation.clone(),
            proofs_handler,
            state_storage,
            proof_drainer,
        );

        let handle = tokio::spawn(sot_impl.state_operating_closure());
        Self {
            to_state_operating,
            cancellation,
            handle,
        }
    }

    pub(crate) async fn shutdown(&mut self) -> SOTResult<()> {
        log::info!("Shutting down utility thread");
        self.cancellation.cancel();
        Ok((&mut self.handle).await?)
    }

    pub(crate) fn get_to_state_operating_channel(&self) -> ToStateOperatingInlet {
        self.to_state_operating.clone()
    }
}

struct StateOperatingThreadImpl {
    to_state_operating: ToStateOperatingOutlet,
    control: StateOperatingControlOutlet,
    cancellation: CancellationToken,
    proofs_handler: NewProofHandler,
    state_storage: StateStorage,
    proof_drainer: ProofStorageDrainer,
}

impl StateOperatingThreadImpl {
    pub(crate) fn new(
        to_state_operating: ToStateOperatingOutlet,
        control: StateOperatingControlOutlet,
        cancellation: CancellationToken,
        proofs_handler: NewProofHandler,
        state_storage: StateStorage,
        proof_drainer: ProofStorageDrainer,
    ) -> Self {
        Self {
            to_state_operating,
            control,
            cancellation,
            proofs_handler,
            state_storage,
            proof_drainer,
        }
    }

    pub(crate) async fn state_operating_closure(mut self) {
        loop {
            tokio::select! {
                Some(state_operating_message) = self.to_state_operating.recv() => self.handle_to_state_operating_message(state_operating_message).await,
                Some(control_message) = self.control.recv() => self.handle_control_message(control_message).await,
                _ = self.cancellation.cancelled() => {
                    log::info!("The utility thread was shutdown");
                    return;
                }
            }
        }
    }

    async fn handle_to_state_operating_message(&mut self, message: ToStateOperatingMessage) {
        use ToStateOperatingMessage::*;

        match message {
            ProofFound { proof } => {
                if let Err(error) = self.proofs_handler.handle_found_proof(&proof).await {
                    log::error!("failed to save proof: {error}\nfound proof {proof}");
                }
            }
            ErrorHappened { core_id, error } => {
                log::error!("{core_id}: {error}");
            }
        }
    }

    async fn handle_control_message(&mut self, message: StateOperatingControlMessage) {
        use StateOperatingControlMessage::*;

        match message {
            SaveConfig { notifier, state } => {
                let ret = self.state_storage.save_state(state.as_ref()).await;
                // if receiver is dead, there is nothing we can do
                let _ = notifier.send(ret);
            }
            CleanProofCache { notifier } => {
                let ret = self.proof_drainer.remove_proofs().await;
                // if receiver is dead, there is nothing we can do
                let _ = notifier.send(ret);
            }
            GetProofs {
                proof_idx,
                notifier,
            } => {
                let ret = self.proof_drainer.get_proofs_after(proof_idx).await;
                // if receiver is dead, there is nothing we can do
                let _ = notifier.send(ret);
            }
        }
    }
}

struct NewProofHandler {
    proof_idx: ProofIdx,
    last_seen_global_nonce: GlobalNonce,
    proof_storage: ProofStorage,
}

impl NewProofHandler {
    pub(self) fn new(
        proof_storage: ProofStorage,
        prev_proof_idx: ProofIdx,
        last_seen_global_nonce: Option<GlobalNonce>,
    ) -> Self {
        Self {
            proof_idx: prev_proof_idx,
            last_seen_global_nonce: last_seen_global_nonce.unwrap_or(GlobalNonce::new([0u8; 32])),
            proof_storage,
        }
    }

    async fn handle_found_proof(&mut self, proof: &RawProof) -> SOTResult<()> {
        log::debug!("state_operating_thread: new proof_received {proof:?}");

        self.maybe_new_epoch(proof);

        let cc_proof_id = CCProofId::new(
            proof.epoch.global_nonce,
            proof.epoch.difficulty,
            self.proof_idx,
        );
        let cc_proof = CCProof::new(
            cc_proof_id,
            proof.local_nonce,
            proof.cu_id,
            proof.result_hash,
        );
        self.proof_storage.store_new_proof(cc_proof).await?;
        self.proof_idx.increment();

        Ok(())
    }

    fn maybe_new_epoch(&mut self, proof: &RawProof) {
        if self.is_new_epoch(proof) {
            self.last_seen_global_nonce = proof.epoch.global_nonce;
            self.proof_idx = ProofIdx::zero();
        }
    }

    fn is_new_epoch(&self, proof: &RawProof) -> bool {
        self.last_seen_global_nonce != proof.epoch.global_nonce
    }
}
