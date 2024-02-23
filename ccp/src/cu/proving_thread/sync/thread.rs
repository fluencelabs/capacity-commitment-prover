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

use ccp_randomx::Cache;
use ccp_randomx::Dataset;
use ccp_randomx::RandomXFlags;
use cpu_utils::LogicalCoreId;

use super::errors::ProvingThreadSyncError;
use super::state::RandomXJob;
use super::state::ThreadState;
use super::to_utility_message::ToUtilityInlet;
use super::to_utility_message::ToUtilityMessage;
use super::STFResult;
use super::STResult;
use crate::cu::proving_thread::messages::*;
use crate::cu::proving_thread::sync::errors::ProvingThreadSyncFacadeError;

const HASHES_PER_ROUND: usize = 1024;

const CHANNEL_DROPPED_MESSAGE: &str =
    "ThreadState::WaitForMessage async part of the ptt channel is dropped";

#[derive(Debug)]
pub(crate) struct ProvingThreadSync {
    handle: thread::JoinHandle<STFResult<()>>,
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

    pub(crate) fn join(self) -> STFResult<()> {
        self.handle
            .join()
            .map_err(ProvingThreadSyncFacadeError::join_error)?
    }

    fn proving_closure(
        core_id: LogicalCoreId,
        mut from_async: AsyncToSyncOutlet,
        to_async: SyncToAsyncInlet,
        to_utility: ToUtilityInlet,
    ) -> Box<dyn FnMut() -> STFResult<()> + Send + 'static> {
        let to_utility_outer = to_utility.clone();

        let mut inner_closure = move || -> Result<(), ProvingThreadSyncError> {
            if !cpu_utils::pinning::pin_current_thread_to(core_id) {
                let error = ProvingThreadSyncError::ThreadPinFailed { core_id };
                to_utility.blocking_send(ToUtilityMessage::error_happened(core_id, error))?;
            }

            let mut thread_state = ThreadState::WaitForMessage;

            loop {
                log::trace!("proving_thread_sync: new thread_state is {thread_state:?}");

                thread_state = match thread_state {
                    ThreadState::WaitForMessage => {
                        println!("before reading message");
                        // block on the channel till it returns a new message
                        let message = from_async.blocking_recv().ok_or(
                            ProvingThreadSyncError::channel_error(CHANNEL_DROPPED_MESSAGE),
                        )?;
                        println!("after reading message: {message:?}");

                        ThreadState::NewMessage { message }
                    }
                    ThreadState::CCJob { mut job } => {
                        use tokio::sync::mpsc::error::TryRecvError;

                        job.cc_prove(&to_utility)?;

                        match from_async.try_recv() {
                            Ok(message) => ThreadState::NewMessage { message },
                            Err(TryRecvError::Empty) => ThreadState::CCJob { job },
                            Err(e) => Err(e)?,
                        }
                    }
                    ThreadState::NewMessage { message } => {
                        Self::handle_message(message, &to_async, &to_utility)?
                    }
                    ThreadState::Stop => {
                        return Ok(());
                    }
                };
            }
        };

        Box::new(move || match inner_closure() {
            Ok(_) => Ok(()),
            Err(error) => {
                let message = ToUtilityMessage::error_happened(core_id, error);
                to_utility_outer.blocking_send(message).map_err(Into::into)
            }
        })
    }

    fn handle_message(
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

                let to_async_message = CacheCreated::new(cache);
                let to_async_message = SyncToAsyncMessage::CacheCreated(to_async_message);
                to_async.blocking_send(to_async_message)?;

                Ok(ThreadState::WaitForMessage)
            }

            AsyncToSyncMessage::AllocateDataset(params) => {
                let dataset = Dataset::allocate(params.flags.contains(RandomXFlags::LARGE_PAGES))?;

                let to_async_message = DatasetAllocated::new(dataset);
                let to_async_message = SyncToAsyncMessage::DatasetAllocated(to_async_message);
                to_async.blocking_send(to_async_message)?;

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
                let parameters = RandomXJob::from_cc_job(cc_job, HASHES_PER_ROUND)?;
                Ok(ThreadState::CCJob { job: parameters })
            }

            AsyncToSyncMessage::PinThread(params) => {
                if !cpu_utils::pinning::pin_current_thread_to(params.core_id) {
                    let error = ProvingThreadSyncError::ThreadPinFailed {
                        core_id: params.core_id,
                    };
                    to_utility
                        .blocking_send(ToUtilityMessage::error_happened(params.core_id, error))?;
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
}
