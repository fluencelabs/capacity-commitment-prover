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

use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::time;

use ccp_shared::proof::CCProof;
use ccp_shared::proof::CCProofId;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::GlobalNonce;
use cpu_utils::LogicalCoreId;

use super::message::*;
use super::UTResult;
use crate::hashrate::HashrateHandler;
use crate::utility_thread::proof_storage::ProofStorage;
use crate::utility_thread::UtilityThreadError;

type ThreadShutdownInlet = oneshot::Sender<()>;
type ThreadShutdownOutlet = oneshot::Receiver<()>;

const CUMULATIVE_HASHRATE_UPDATE_INTERVAL: u64 = 60;

pub(crate) struct UtilityThread {
    to_utility: ToUtilityInlet,
    shutdown_in: ThreadShutdownInlet,
    handle: tokio::task::JoinHandle<()>,
}

impl UtilityThread {
    pub(crate) fn spawn(
        core_id: LogicalCoreId,
        prev_proof_idx: ProofIdx,
        proof_storage_dir: std::path::PathBuf,
        prev_global_nonce: Option<GlobalNonce>,
        hashrate_handler: HashrateHandler,
    ) -> Self {
        let (to_utility, from_utility) = mpsc::channel(100);
        let (shutdown_in, shutdown_out) = oneshot::channel();

        let proof_storage = ProofStorage::new(proof_storage_dir);
        let proofs_handler = NewProofHandler::new(proof_storage, prev_proof_idx, prev_global_nonce);
        let ut_impl =
            UtilityThreadImpl::new(from_utility, shutdown_out, proofs_handler, hashrate_handler);

        let handle = tokio::spawn(ut_impl.utility_closure(core_id));
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
        Ok(self.handle.await?)
    }

    pub(crate) fn get_to_utility_channel(&self) -> ToUtilityInlet {
        self.to_utility.clone()
    }
}

struct UtilityThreadImpl {
    to_utility: ToUtilityOutlet,
    shutdown_out: ThreadShutdownOutlet,
    proofs_handler: NewProofHandler,
    hashrate_handler: HashrateHandler,
}

impl UtilityThreadImpl {
    pub(crate) fn new(
        to_utility: ToUtilityOutlet,
        shutdown_out: ThreadShutdownOutlet,
        proofs_handler: NewProofHandler,
        hashrate_handler: HashrateHandler,
    ) -> Self {
        Self {
            to_utility,
            shutdown_out,
            proofs_handler,
            hashrate_handler,
        }
    }

    pub(crate) async fn utility_closure(mut self, core_id: LogicalCoreId) {
        use crossterm::event::EventStream;
        use futures::FutureExt;
        use futures::StreamExt;

        if !cpu_utils::pinning::pin_current_thread_to(core_id) {
            log::error!("utility_thread: failed to pin to {core_id} core");
        }

        let mut cum_hashrate_ticker =
            time::interval(Duration::from_secs(CUMULATIVE_HASHRATE_UPDATE_INTERVAL));

        let mut terminal_event_reader = EventStream::new();

        loop {
            tokio::select! {
                Some(message) = self.to_utility.recv() => self.handle_to_utility_message(message).await,
                _ = cum_hashrate_ticker.tick() => self.handle_cum_hashrate_tick().await,
                maybe_event = terminal_event_reader.next().fuse() => self.handle_terminal_event(maybe_event).await,
                _ = &mut self.shutdown_out => {
                    log::info!("utility_thread: utility thread was shutdown");
                    return;
                }
            }
        }
    }

    async fn handle_to_utility_message(&mut self, message: ToUtilityMessage) {
        match message {
            ToUtilityMessage::ProofFound { core_id, proof } => {
                if let Err(error) = self.proofs_handler.handle_found_proof(&proof).await {
                    log::error!("failed to save proof: {error}\nfound proof {proof}");
                }
                self.hashrate_handler.proof_found(core_id);
            }
            ToUtilityMessage::ErrorHappened {
                thread_location,
                error,
            } => {
                log::error!("utility_thread: thread at {thread_location} core id encountered a error {error}");
            }
            ToUtilityMessage::Hashrate(record) => {
                log::info!("utility_thread: hashrate {record}");
                if let Err(error) = self.hashrate_handler.account_record(record) {
                    log::error!("account hashrate error faield: {error}");
                }
            }
        }
    }

    async fn handle_cum_hashrate_tick(&mut self) {
        if let Err(error) = self.hashrate_handler.handle_cum_tick() {
            log::error!("cumulative hashrate tick failed: {error}");
        }
    }

    async fn handle_terminal_event(
        &mut self,
        maybe_event: Option<Result<crossterm::event::Event, std::io::Error>>,
    ) {
        use crossterm::event::Event;
        use crossterm::event::KeyCode;

        if let Some(Ok(event)) = maybe_event {
            if event == Event::Key(KeyCode::Enter.into()) {
                let sliding_hashrate = self.hashrate_handler.sliding_hashrate();

                println!(
                    "{0: <10} | {1: <10} | {2: <10} | {3: <10}",
                    "core id", "10 secs", "60 secs", "900 secs"
                );

                for (core_id, thread_hashrate) in sliding_hashrate {
                    println!(
                        "{0: <10} | {1: <10} | {2: <10} | {3: <10}",
                        core_id,
                        thread_hashrate
                            .windows_10
                            .compute_hashrate()
                            .unwrap_or(0f64),
                        thread_hashrate
                            .windows_60
                            .compute_hashrate()
                            .unwrap_or(0f64),
                        thread_hashrate
                            .windows_900
                            .compute_hashrate()
                            .unwrap_or(0f64),
                    );
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

    async fn handle_found_proof(&mut self, proof: &RawProof) -> UTResult<()> {
        log::debug!("utility_thread: new proof_received {proof:?}");

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
