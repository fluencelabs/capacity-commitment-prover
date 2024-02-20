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

use cpu_utils::LogicalCoreId;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::GlobalNonce;

use crate::cu::proving_thread::sync::to_utility_message::*;

type ThreadShutdownInlet = oneshot::Sender<()>;
type ThreadShutdownOutlet = oneshot::Receiver<()>;

pub(crate) struct UtilityThread {
    thread_shutdown: ThreadShutdownInlet,
    handle: std::thread::JoinHandle<()>,
}

impl UtilityThread {
    fn spawn(core_id: LogicalCoreId, thread_shutdown: ThreadShutdownInlet, to_utility: ToUtilityOutlet) {
        let handle = tokio::spawn(Self::proving_closure(core_id, to_utility));

        Self {
            handle,
            thread_shutdown
        }
    }

    fn proving_closure(
        core_id: LogicalCoreId,
        to_utility: ToUtilityOutlet,
    ) -> Box<dyn FnMut() -> PTResult<()> + Send + 'static> {
        Box::new(move || {
            cpu_utils::pinning::pin_current_thread_to(core_id);

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
        })
    }
}
