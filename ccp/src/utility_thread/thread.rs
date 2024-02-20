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
use tokio::sync::oneshot;

use ccp_shared::proof::CCProof;
use ccp_shared::proof::CCProofId;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::GlobalNonce;
use cpu_utils::LogicalCoreId;

use super::message::*;
use super::UTResult;
use crate::utility_thread::proof_storage::ProofStorage;
use crate::utility_thread::UtilityThreadError;

type ThreadShutdownInlet = oneshot::Sender<()>;
type ThreadShutdownOutlet = oneshot::Receiver<()>;

pub(crate) struct UtilityThread {
    to_utility: ToUtilityInlet,
    shutdown_in: ThreadShutdownInlet,
    handle: tokio::task::JoinHandle<UTResult<()>>,
}

impl UtilityThread {
    pub(crate) fn spawn(core_id: LogicalCoreId, proof_storage_dir: std::path::PathBuf) -> Self {
        let (to_utility, from_utility) = mpsc::channel(100);
        let (shutdown_in, shutdown_out) = oneshot::channel();

        let proof_storage = ProofStorage::new(proof_storage_dir);
        let new_proof_handler = NewProofHandler::new(proof_storage);

        let handle = tokio::spawn(Self::utility_closure(
            core_id,
            from_utility,
            shutdown_out,
            new_proof_handler,
        ));

        Self {
            to_utility,
            shutdown_in,
            handle,
        }
    }

    pub(crate) async fn stop(self) -> UTResult<()> {
        self.shutdown_in
            .send(())
            .map_err(|_| UtilityThreadError::ShutdownError)?;
        self.handle.await?
    }

    pub(crate) fn get_to_utility_channel(&self) -> ToUtilityInlet {
        self.to_utility.clone()
    }

    async fn utility_closure(
        core_id: LogicalCoreId,
        mut to_utility: ToUtilityOutlet,
        mut shutdown_out: ThreadShutdownOutlet,
        mut new_proof_handler: NewProofHandler,
    ) -> UTResult<()> {
        if !cpu_utils::pinning::pin_current_thread_to(core_id) {
            log::error!("utility_thread: failed to pin to {core_id} core");
        }

        loop {
            tokio::select! {
                Some(message) = to_utility.recv() => {
                    match message {
                        ToUtilityMessage::ProofFound(proof) => new_proof_handler.handle_found_proof(proof).await?,
                        ToUtilityMessage::ErrorHappened { thread_location, error} => {
                            log::error!("utility_thread: thread at {thread_location} core id encountered a error {error}");
                        }
                    }},
                _ = &mut shutdown_out => {
                    log::info!("utility_thread: utility thread was shutdown");

                    return Ok(())
                }
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
    pub(self) fn new(proof_storage: ProofStorage) -> Self {
        Self {
            proof_idx: ProofIdx::zero(),
            last_seen_global_nonce: GlobalNonce::new([0u8; 32]),
            proof_storage,
        }
    }

    async fn handle_found_proof(&mut self, proof: RawProof) -> UTResult<()> {
        log::debug!("utility_thread: new proof_received {proof:?}");

        self.maybe_new_epoch(&proof);

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
