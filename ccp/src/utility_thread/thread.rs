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
use crossterm::event::{EventStream, KeyCode};
use futures::StreamExt;
use futures::FutureExt;

use super::message::*;
use super::UTResult;
use crate::hashrate::HashrateCollector;
use crate::hashrate::HashrateCollectorByTimer;
use crate::hashrate::HashrateSaver;
use crate::utility_thread::proof_storage::ProofStorage;
use crate::utility_thread::UtilityThreadError;

type ThreadShutdownInlet = oneshot::Sender<()>;
type ThreadShutdownOutlet = oneshot::Receiver<()>;

const HASHRATE_UPDATE_INTERVAL: u64 = 60;

pub(crate) struct UtilityThread {
    to_utility: ToUtilityInlet,
    shutdown_in: ThreadShutdownInlet,
    handle: tokio::task::JoinHandle<UTResult<()>>,
}

impl UtilityThread {
    pub(crate) fn spawn(
        core_id: LogicalCoreId,
        proof_storage_dir: std::path::PathBuf,
        hashrate_dir: std::path::PathBuf,
    ) -> Self {
        let (to_utility, from_utility) = mpsc::channel(100);
        let (shutdown_in, shutdown_out) = oneshot::channel();

        let proof_storage = ProofStorage::new(proof_storage_dir);
        let new_proof_handler = NewProofHandler::new(proof_storage);
        let hashrate_collector = HashrateCollector::new();
        let hashrate_collector_by_timer = HashrateCollectorByTimer::new();
        let hashrate_saver = HashrateSaver::from_directory(hashrate_dir);

        let handle = tokio::spawn(Self::utility_closure(
            core_id,
            from_utility,
            shutdown_out,
            new_proof_handler,
            hashrate_collector,
            hashrate_collector_by_timer,
            hashrate_saver,
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
        mut hashrate_collector: HashrateCollector,
        mut hashrate_collector_by_timer: HashrateCollectorByTimer,
        hashrate_saver: HashrateSaver,
    ) -> UTResult<()> {
        if !cpu_utils::pinning::pin_current_thread_to(core_id) {
            log::error!("utility_thread: failed to pin to {core_id} core");
        }

        let mut hashrate_ticker = time::interval(Duration::from_secs(HASHRATE_UPDATE_INTERVAL));
        let mut event_stream = EventStream::new();

        loop {
            tokio::select! {
                Some(message) = to_utility.recv() => {
                    match message {
                        ToUtilityMessage::ProofFound { core_id, proof} => {
                            new_proof_handler.handle_found_proof(proof).await?;
                            hashrate_collector.proof_found(core_id);
                        }
                        ToUtilityMessage::ErrorHappened { thread_location, error} => {
                            log::error!("utility_thread: thread at {thread_location} core id encountered a error {error}");
                        }
                        ToUtilityMessage::Hashrate(record) => {
                            log::info!("utility_thread: new hashrate message {record}");
                            if let Some(prev_epoch_hashrate) = hashrate_collector.count_record(record) {
                                hashrate_saver.save_hashrate_previous(prev_epoch_hashrate)?;
                            }
                            hashrate_collector_by_timer.count_record(record);
                        }
                    }},
                _ = hashrate_ticker.tick() => {
                    let hashrate = hashrate_collector.collect();
                    hashrate_saver.save_hashrate_current(hashrate)?;
                }
                maybe_event = event_stream.next().fuse() => {
                    use crossterm::event::Event;

                    println!("crossterm event: {maybe_event:?}");
                    log::info!("event received");

                    if let Some(Ok(event)) = maybe_event {
                        if event == Event::Key(KeyCode::Enter.into()) {
                            log::info!("enter pressed");
                            //println!("{hashrate_collector_by_timer:?}");
                        }
                    }
                }
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
