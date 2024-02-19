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

use std::thread;
use tokio::sync::mpsc;

use randomx::Cache;
use randomx::Dataset;
use randomx_rust_wrapper as randomx;
use randomx_rust_wrapper::RandomXFlags;

use super::errors::ProvingThreadError;
use super::messages::*;
use super::state::RandomXJob;
use super::state::ThreadState;
use super::PTResult;
use crate::LogicalCoreId;

const HASHES_PER_ROUND: usize = 1024;

const CHANNEL_DROPPED_MESSAGE: &str =
    "ThreadState::WaitForMessage async part of the ptt channel is dropped";

#[derive(Debug)]
pub(crate) struct ProvingThreadSync {
    handle: thread::JoinHandle<PTResult<()>>,
}

impl ProvingThreadSync {
    pub(crate) fn spawn(
        core_id: LogicalCoreId,
        ats_outlet: AsyncToSyncOutlet,
        sta_inlet: SyncToAsyncInlet,
        proof_receiver_inlet: mpsc::Sender<RawProof>,
    ) -> Self {
        let thread_closure =
            Self::proving_closure(core_id, ats_outlet, sta_inlet, proof_receiver_inlet);
        let handle = thread::spawn(thread_closure);

        Self { handle }
    }

    pub(crate) fn join(self) -> PTResult<()> {
        self.handle.join().map_err(ProvingThreadError::join_error)?
    }

    fn proving_closure(
        core_id: LogicalCoreId,
        mut ats_outlet: AsyncToSyncOutlet,
        sta_inlet: SyncToAsyncInlet,
        proof_receiver_inlet: mpsc::Sender<RawProof>,
    ) -> Box<dyn FnMut() -> PTResult<()> + Send + 'static> {
        Box::new(move || {
            let _ = cpu_utils::pinning::pin_current_thread_to(core_id);

            let ptt_message = ats_outlet
                .blocking_recv()
                .ok_or(ProvingThreadError::channel_error(CHANNEL_DROPPED_MESSAGE))?;
            let mut thread_state = Self::handle_prover_message(ptt_message, &sta_inlet)?;

            loop {
                log::trace!("proving_thread_sync: new thread_state is {thread_state:?}");

                thread_state = match thread_state {
                    ThreadState::Stop => {
                        return Ok(());
                    }
                    ThreadState::WaitForMessage => {
                        // block on the channel till it returns a new message
                        let ptt_message = ats_outlet
                            .blocking_recv()
                            .ok_or(ProvingThreadError::channel_error(CHANNEL_DROPPED_MESSAGE))?;
                        Self::handle_prover_message(ptt_message, &sta_inlet)?
                    }
                    ThreadState::CCJob { parameters } => {
                        use tokio::sync::mpsc::error::TryRecvError;

                        let parameters = Self::cc_prove(parameters, proof_receiver_inlet.clone())?;
                        match ats_outlet.try_recv() {
                            Ok(message) => Self::handle_prover_message(message, &sta_inlet)?,
                            Err(TryRecvError::Empty) => ThreadState::CCJob { parameters },
                            Err(e) => Err(e)?,
                        }
                    }
                };
            }
        })
    }

    fn handle_prover_message(
        message: AsyncToSyncMessage,
        sta_inlet: &SyncToAsyncInlet,
    ) -> PTResult<ThreadState> {
        log::trace!("proving_thread_sync: handle message from CUProver: {message:?}");

        match message {
            AsyncToSyncMessage::CreateCache(params) => {
                let global_nonce_cu =
                    ccp_utils::compute_global_nonce_cu(&params.global_nonce, &params.cu_id);
                let cache = Cache::new(global_nonce_cu.as_slice(), params.flags)?;

                let ttp_message = CacheCreated::new(cache);
                let ttp_message = SyncToAsyncMessage::CacheCreated(ttp_message);
                sta_inlet.blocking_send(ttp_message)?;

                Ok(ThreadState::WaitForMessage)
            }

            AsyncToSyncMessage::AllocateDataset(params) => {
                let dataset = Dataset::allocate(params.flags.contains(RandomXFlags::LARGE_PAGES))?;

                let ttp_message = DatasetAllocated::new(dataset);
                let ttp_message = SyncToAsyncMessage::DatasetAllocated(ttp_message);
                sta_inlet.blocking_send(ttp_message)?;

                Ok(ThreadState::WaitForMessage)
            }

            AsyncToSyncMessage::InitializeDataset(mut params) => {
                params
                    .dataset
                    .initialize(&params.cache, params.start_item, params.items_count);
                sta_inlet.blocking_send(SyncToAsyncMessage::DatasetInitialized)?;

                Ok(ThreadState::WaitForMessage)
            }

            AsyncToSyncMessage::NewCCJob(cc_job) => {
                let parameters = RandomXJob::from_cc_job(cc_job)?;
                Ok(ThreadState::CCJob { parameters })
            }

            AsyncToSyncMessage::PinThread(params) => {
                // TODO: propagate error
                cpu_utils::pinning::pin_current_thread_to(params.core_id);
                Ok(ThreadState::WaitForMessage)
            }

            AsyncToSyncMessage::Stop => Ok(ThreadState::Stop),
        }
    }

    fn cc_prove(
        mut job: RandomXJob,
        proof_receiver_inlet: mpsc::Sender<RawProof>,
    ) -> PTResult<RandomXJob> {
        use ccp_shared::meet_difficulty::MeetDifficulty;

        let is_last_iteration = |hash_id: usize| -> bool { hash_id == HASHES_PER_ROUND - 1 };

        job.hash_first();

        for hash_id in 0..HASHES_PER_ROUND {
            let result_hash = if is_last_iteration(hash_id) {
                job.hash_last()
            } else {
                job.hash_next()
            };

            if result_hash.meet_difficulty(&job.difficulty) {
                log::info!("proving_thread_sync: found new golden result hash {result_hash:?}\nfor local_nonce {:?}", job.local_nonce);

                let proof = job.create_golden_proof(result_hash);
                proof_receiver_inlet.blocking_send(proof)?;
            }
        }

        Ok(job)
    }
}
