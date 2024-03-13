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
use tokio::time;

use ccp_shared::proof::CCProof;
use ccp_shared::proof::CCProofId;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::GlobalNonce;
use cpu_utils::LogicalCoreId;
use tokio_util::sync::CancellationToken;

use super::message::*;
use super::UTResult;
use crate::hashrate::HashrateHandler;
use crate::utility_thread::proof_storage::ProofStorage;

const CUMULATIVE_HASHRATE_UPDATE_INTERVAL: u64 = 60;

pub(crate) struct UtilityThread {
    to_utility: ToUtilityInlet,
    cancellation: CancellationToken,
    handle: tokio::task::JoinHandle<()>,
}

impl UtilityThread {
    pub(crate) fn spawn(
        core_ids: Vec<LogicalCoreId>,
        prev_proof_idx: ProofIdx,
        proof_storage_dir: std::path::PathBuf,
        prev_global_nonce: Option<GlobalNonce>,
        hashrate_handler: HashrateHandler,
    ) -> Self {
        let (to_utility, from_utility) = mpsc::channel(100);

        let cancellation = CancellationToken::new();

        let proof_storage = ProofStorage::new(proof_storage_dir);
        let proofs_handler = NewProofHandler::new(proof_storage, prev_proof_idx, prev_global_nonce);
        let ut_impl = UtilityThreadImpl::new(
            from_utility,
            cancellation.clone(),
            proofs_handler,
            hashrate_handler,
        );

        let handle = tokio::spawn(ut_impl.utility_closure(core_ids));
        Self {
            to_utility,
            cancellation,
            handle,
        }
    }

    pub(crate) async fn shutdown(&mut self) -> UTResult<()> {
        log::info!("Shutting down utility thread");
        self.cancellation.cancel();
        Ok((&mut self.handle).await?)
    }

    pub(crate) fn get_to_utility_channel(&self) -> ToUtilityInlet {
        self.to_utility.clone()
    }
}

struct UtilityThreadImpl {
    to_utility: ToUtilityOutlet,
    cancellation: CancellationToken,
    proofs_handler: NewProofHandler,
    hashrate_handler: HashrateHandler,
}

impl UtilityThreadImpl {
    pub(crate) fn new(
        to_utility: ToUtilityOutlet,
        cancellation: CancellationToken,
        proofs_handler: NewProofHandler,
        hashrate_handler: HashrateHandler,
    ) -> Self {
        Self {
            to_utility,
            cancellation,
            proofs_handler,
            hashrate_handler,
        }
    }

    pub(crate) async fn utility_closure(mut self, core_ids: Vec<LogicalCoreId>) {
        use futures::FutureExt;
        use futures::StreamExt;

        if !cpu_utils::pinning::pin_current_thread_to_cpuset(core_ids.iter().cloned()) {
            log::error!("failed to pin to {core_ids:?} cores");
        }

        let mut cum_hashrate_ticker =
            time::interval(Duration::from_secs(CUMULATIVE_HASHRATE_UPDATE_INTERVAL));

        #[cfg(feature = "crossterm")]
        let mut terminal_event_reader = crossterm::event::EventStream::new();

        #[cfg(not(feature = "crossterm"))]
        let mut terminal_event_reader = futures::stream::pending::<()>();

        loop {
            tokio::select! {
                Some(message) = self.to_utility.recv() => self.handle_to_utility_message(message).await,
                _ = cum_hashrate_ticker.tick() => self.handle_cum_hashrate_tick().await,
                maybe_event = terminal_event_reader.next().fuse() => self.handle_terminal_event(maybe_event).await,
                _ = self.cancellation.cancelled() => {
                    log::info!("The utility thread was shutdown");
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
            ToUtilityMessage::ErrorHappened { core_id, error } => {
                log::error!("{core_id}: {error}");
            }
            ToUtilityMessage::Hashrate(record) => {
                log::info!("{record}");

                if let Err(error) = self.hashrate_handler.account_record(record) {
                    log::error!("hashrate accounting failed: {error}");
                }
            }
        }
    }

    async fn handle_cum_hashrate_tick(&mut self) {
        if let Err(error) = self.hashrate_handler.handle_cum_tick() {
            log::error!("cumulative hashrate tick failed: {error}");
        }
    }

    #[cfg(feature = "crossterm")]
    async fn handle_terminal_event(
        &mut self,
        maybe_event: Option<Result<crossterm::event::Event, std::io::Error>>,
    ) {
        use crossterm::event::Event;
        use crossterm::event::KeyCode;
        use itertools::Itertools;

        if let Some(Ok(event)) = maybe_event {
            if event == Event::Key(KeyCode::Enter.into()) {
                let sliding_hashrate = self.hashrate_handler.sliding_hashrate();
                if sliding_hashrate.is_empty() {
                    println!("no hashrate for the last 900 secs,\nCCP is either busy with initialization or idle");
                    return;
                }

                println!(
                    "{0: <10} | {1: <10} | {2: <10} | {3: <10}",
                    "core id", "10 secs", "60 secs", "900 secs"
                );

                for (core_id, thread_hashrate) in sliding_hashrate
                    .iter()
                    .sorted_by_key(|(&core_id, _)| core_id)
                {
                    println!(
                        "{0: <10} | {1: <10.2} | {2: <10.2} | {3: <10.2}",
                        core_id,
                        thread_hashrate.window_10.compute_hashrate(),
                        thread_hashrate.window_60.compute_hashrate(),
                        thread_hashrate.window_900.compute_hashrate()
                    );
                }
            }
        }
    }

    #[cfg(not(feature = "crossterm"))]
    async fn handle_terminal_event(&mut self, _maybe_event: Option<()>) {}
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
