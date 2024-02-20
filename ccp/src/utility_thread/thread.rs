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

use futures::FutureExt;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::utility_thread::proof_storage::ProofStorage;
use crate::utility_thread::UTError;
use ccp_shared::proof::CCProof;
use ccp_shared::proof::CCProofId;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::GlobalNonce;
use cpu_utils::LogicalCoreId;

use super::message::*;
use super::UTResult;

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
        let handle = tokio::spawn(Self::utility_closure(
            core_id,
            from_utility,
            shutdown_out,
            proof_storage,
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
            .map_err(|_| UTError::ShutdownError)?;
        self.handle.await?
    }

    pub(crate) fn get_to_utility_channel(&self) -> ToUtilityInlet {
        self.to_utility.clone()
    }

    fn utility_closure(
        core_id: LogicalCoreId,
        mut to_utility: ToUtilityOutlet,
        mut shutdown_out: ThreadShutdownOutlet,
        proof_storage: ProofStorage,
    ) -> futures::future::BoxFuture<'static, UTResult<()>> {
        async move {
            if !cpu_utils::pinning::pin_current_thread_to(core_id) {
                log::error!("utility_thread: failed to pin to {core_id} core");
            }

            let mut proof_idx = ProofIdx::zero();
            let mut last_seen_global_nonce = GlobalNonce::new([0u8; 32]);

            loop {
                tokio::select! {
                    Some(message) = to_utility.recv() => {
                        match message {
                            ToUtilityMessage::ProofFound(proof) => {
                                log::debug!("utility_thread: new proof_received {proof:?}");

                                if proof.epoch.global_nonce != last_seen_global_nonce {
                                    last_seen_global_nonce = proof.epoch.global_nonce;
                                    proof_idx = ProofIdx::zero();
                                }
                                let cc_proof_id = CCProofId::new(proof.epoch.global_nonce, proof.epoch.difficulty, proof_idx);
                                let cc_proof = CCProof::new(cc_proof_id, proof.local_nonce, proof.cu_id, proof.result_hash);
                                proof_storage.store_new_proof(cc_proof).await.map_err(UTError::IOError)?;
                                proof_idx.increment();
                            },
                            ToUtilityMessage::ErrorHappened { thread_location, error} => {
                                log::error!("utility_thread: thread at {thread_location} encountered a error {error}");

                            }
                        }},
                    _ = &mut shutdown_out => {
                        log::info!("utility_thread: utility thread was shutdown");

                        return Ok(())
                    }
                }
            }
        }.boxed()
    }
}
