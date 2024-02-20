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

use cpu_utils::LogicalCoreId;
use randomx::Cache;
use randomx::Dataset;
use randomx_rust_wrapper as randomx;
use randomx_rust_wrapper::RandomXFlags;

use super::errors::SyncThreadError;
use super::state::RandomXJob;
use super::state::ThreadState;
use super::to_utility_message::ToUtilityInlet;
use super::to_utility_message::ToUtilityMessage;
use super::STResult;
use crate::cu::proving_thread::messages::*;

const HASHES_PER_ROUND: usize = 1024;

const CHANNEL_DROPPED_MESSAGE: &str =
    "ThreadState::WaitForMessage async part of the ptt channel is dropped";

#[derive(Debug)]
pub(crate) struct ProvingThreadSync {
    handle: thread::JoinHandle<STResult<()>>,
}

impl ProvingThreadSync {
    pub(crate) fn spawn(
        core_id: LogicalCoreId,
        from_async: AsyncToSyncOutlet,
        to_async: SyncToAsyncInlet,
        to_utility: ToUtilityInlet,
    ) -> Self {
        let thread_closure = Self::proving_closure(core_id, from_async, to_async, to_utility);
        let handle = thread::spawn(thread_closure);

        Self { handle }
    }

    pub(crate) fn join(self) -> STResult<()> {
        self.handle.join().map_err(SyncThreadError::join_error)?
    }

    fn proving_closure(
        core_id: LogicalCoreId,
        mut from_async: AsyncToSyncOutlet,
        to_async: SyncToAsyncInlet,
        to_utility: ToUtilityInlet,
    ) -> Box<dyn FnMut() -> STResult<()> + Send + 'static> {
        let inner_closure = move || {
            if cpu_utils::pinning::pin_current_thread_to(core_id) {
                to_utility.blocking_send(ToUtilityMessage::ErrorHappened(
                    SyncThreadError::ThreadPinFailed { core_id },
                ))?;
            }

            let message = from_async
                .blocking_recv()
                .ok_or(SyncThreadError::channel_error(CHANNEL_DROPPED_MESSAGE))?;
            let mut thread_state = Self::handle_message_from_async(message, &to_async, &to_utility)?;

            loop {
                log::trace!("proving_thread_sync: new thread_state is {thread_state:?}");

                thread_state = match thread_state {
                    ThreadState::Stop => {
                        return Ok(());
                    }
                    ThreadState::WaitForMessage => {
                        // block on the channel till it returns a new message
                        let message = from_async
                            .blocking_recv()
                            .ok_or(SyncThreadError::channel_error(CHANNEL_DROPPED_MESSAGE))?;
                        Self::handle_message_from_async(message, &to_async, &to_utility)?
                    }
                    ThreadState::CCJob { parameters } => {
                        use tokio::sync::mpsc::error::TryRecvError;

                        let parameters = Self::cc_prove(parameters, &to_utility)?;
                        match from_async.try_recv() {
                            Ok(message) => {
                                Self::handle_message_from_async(message, &to_async, &to_utility)?
                            }
                            Err(TryRecvError::Empty) => ThreadState::CCJob { parameters },
                            Err(e) => Err(e)?,
                        }
                    }
                };
            }
        };

        Box::new(move || match inner_closure() {
            Ok(_) => Ok(()),
            Err(e) => {
                let message = ToUtilityMessage::error_happened(e.clone());
                to_utility.blocking_send(message)?;
                Err(e)
            }
        })
    }

    fn handle_message_from_async(
        message: AsyncToSyncMessage,
        to_async: &SyncToAsyncInlet,
        to_utility: &ToUtilityInlet,
    ) -> STResult<ThreadState> {
        log::trace!("proving_thread_sync: handle message from CUProver: {message:?}");

        match message {
            AsyncToSyncMessage::CreateCache(params) => {
                let global_nonce_cu =
                    ccp_utils::hash::compute_global_nonce_cu(&params.global_nonce, &params.cu_id);
                let cache = Cache::new(global_nonce_cu.as_slice(), params.flags)?;

                let ttp_message = CacheCreated::new(cache);
                let ttp_message = SyncToAsyncMessage::CacheCreated(ttp_message);
                to_async.blocking_send(ttp_message)?;

                Ok(ThreadState::WaitForMessage)
            }

            AsyncToSyncMessage::AllocateDataset(params) => {
                let dataset = Dataset::allocate(params.flags.contains(RandomXFlags::LARGE_PAGES))?;

                let ttp_message = DatasetAllocated::new(dataset);
                let ttp_message = SyncToAsyncMessage::DatasetAllocated(ttp_message);
                to_async.blocking_send(ttp_message)?;

                Ok(ThreadState::WaitForMessage)
            }

            AsyncToSyncMessage::InitializeDataset(mut params) => {
                params
                    .dataset
                    .initialize(&params.cache, params.start_item, params.items_count);
                to_async.blocking_send(SyncToAsyncMessage::DatasetInitialized)?;

                Ok(ThreadState::WaitForMessage)
            }

            AsyncToSyncMessage::NewCCJob(cc_job) => {
                let parameters = RandomXJob::from_cc_job(cc_job)?;
                Ok(ThreadState::CCJob { parameters })
            }

            AsyncToSyncMessage::PinThread(params) => {
                if cpu_utils::pinning::pin_current_thread_to(params.core_id) {
                    to_utility.blocking_send(ToUtilityMessage::ErrorHappened(
                        SyncThreadError::ThreadPinFailed { core_id },
                    ))?;
                }
                Ok(ThreadState::WaitForMessage)
            }

            AsyncToSyncMessage::Pause => {
                to_async.blocking_send(SyncToAsyncMessage::Paused)?;
                Ok(ThreadState::WaitForMessage)
            }

            AsyncToSyncMessage::Stop => Ok(ThreadState::Stop),
        }
    }

    fn cc_prove(mut job: RandomXJob, to_utility: &ToUtilityInlet) -> STResult<RandomXJob> {
        use ccp_shared::meet_difficulty::MeetDifficulty;

        let is_last_iteration = |hash_id: usize| -> bool { hash_id == HASHES_PER_ROUND - 1 };

        job.hash_first();

        for hash_id in 0..HASHES_PER_ROUND {
            let result_hash = if is_last_iteration(hash_id) {
                job.hash_last()
            } else {
                job.hash_next()
            };

            if result_hash.meet_difficulty(&job.epoch.difficulty) {
                log::info!("proving_thread_sync: found new golden result hash {result_hash:?}\nfor local_nonce {:?}", job.local_nonce);

                let proof = job.create_golden_proof(result_hash);
                let message = ToUtilityMessage::proof_found(proof);
                to_utility.blocking_send(message)?;
            }
        }

        Ok(job)
    }
}
