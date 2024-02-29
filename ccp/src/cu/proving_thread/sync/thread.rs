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
use std::time::Instant;

use ccp_randomx::Cache;
use ccp_randomx::Dataset;
use ccp_randomx::RandomXFlags;
use cpu_utils::LogicalCoreId;

use super::channels_facade::ToAsync;
use super::channels_facade::ToUtility;
use super::errors::ProvingThreadSyncError;
use super::state::RandomXJob;
use super::state::ThreadState;
use super::to_utility_message::ToUtilityInlet;
use super::STFResult;
use super::STResult;
use crate::cu::proving_thread::messages::*;
use crate::cu::proving_thread::sync::errors::ProvingThreadSyncFacadeError;
use crate::hashrate::ThreadHashrateRecord;

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

        let to_async = ToAsync::from_channel(to_async);
        let to_utility = ToUtility::from_channel(to_utility);

        let mut inner_closure = move || -> Result<(), ProvingThreadSyncError> {
            if !cpu_utils::pinning::pin_current_thread_to(core_id) {
                to_utility.send_error(core_id, ProvingThreadSyncError::pinning_failed(core_id))?;
            }

            let mut thread_state = ThreadState::WaitForMessage;

            loop {
                log::trace!("proving_thread_sync: new thread_state is {thread_state:?}");

                thread_state = match thread_state {
                    ThreadState::WaitForMessage => {
                        // block on the channel till it returns a new message
                        let message = from_async.blocking_recv().ok_or(
                            ProvingThreadSyncError::channel_error(CHANNEL_DROPPED_MESSAGE),
                        )?;

                        ThreadState::NewMessage { message }
                    }
                    ThreadState::CCJob { mut job } => {
                        use tokio::sync::mpsc::error::TryRecvError;

                        job.cc_prove(core_id, &to_utility)?;

                        match from_async.try_recv() {
                            Ok(message) => ThreadState::NewMessage { message },
                            Err(TryRecvError::Empty) => ThreadState::CCJob { job },
                            Err(e) => Err(e)?,
                        }
                    }
                    ThreadState::NewMessage { message } => {
                        Self::handle_message(core_id, message, &to_async, &to_utility)?
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
                use crate::utility_thread::message::ToUtilityMessage;

                let message = ToUtilityMessage::error_happened(core_id, error);
                to_utility_outer.blocking_send(message).map_err(Into::into)
            }
        })
    }

    fn handle_message(
        core_id: LogicalCoreId,
        message: AsyncToSyncMessage,
        to_async: &ToAsync,
        to_utility: &ToUtility,
    ) -> STResult<ThreadState> {
        log::trace!("proving_thread_sync: handle message from CUProver: {message:?}");

        match message {
            AsyncToSyncMessage::CreateCache(params) => {
                let start = Instant::now();

                let global_nonce_cu = ccp_utils::hash::compute_global_nonce_cu(
                    &params.epoch.global_nonce,
                    &params.cu_id,
                );
                let cache = Cache::new(global_nonce_cu.as_slice(), params.flags)?;
                let duration = start.elapsed();

                to_async.send_cache(cache)?;
                let hashrate =
                    ThreadHashrateRecord::cache_creation(params.epoch, core_id, duration);
                to_utility.send_hashrate(hashrate)?;

                Ok(ThreadState::WaitForMessage)
            }

            AsyncToSyncMessage::AllocateDataset(params) => {
                let dataset = Dataset::allocate(params.flags.contains(RandomXFlags::LARGE_PAGES))?;
                to_async.send_dataset(dataset)?;

                Ok(ThreadState::WaitForMessage)
            }

            AsyncToSyncMessage::InitializeDataset(mut params) => {
                let start = Instant::now();
                params
                    .dataset
                    .initialize(&params.cache, params.start_item, params.items_count);
                let duration = start.elapsed();

                to_async.notify_dataset_initialized()?;

                let hasrate = ThreadHashrateRecord::dataset_initialization(
                    params.epoch,
                    core_id,
                    duration,
                    params.start_item,
                    params.items_count,
                );
                to_utility.send_hashrate(hasrate)?;

                Ok(ThreadState::WaitForMessage)
            }

            AsyncToSyncMessage::NewCCJob(job) => {
                let start = Instant::now();
                let job = RandomXJob::from_cc_job(job, HASHES_PER_ROUND)?;
                let duration = start.elapsed();

                let hashrate = ThreadHashrateRecord::checked_hashes(
                    job.epoch(),
                    core_id,
                    duration,
                    HASHES_PER_ROUND,
                );
                to_utility.send_hashrate(hashrate)?;

                Ok(ThreadState::CCJob { job })
            }

            AsyncToSyncMessage::PinThread(params) => {
                if !cpu_utils::pinning::pin_current_thread_to(params.core_id) {
                    to_utility.send_error(
                        core_id,
                        ProvingThreadSyncError::pinning_failed(params.core_id),
                    )?;
                }
                Ok(ThreadState::WaitForMessage)
            }

            AsyncToSyncMessage::Pause => {
                to_async.notify_paused()?;
                Ok(ThreadState::WaitForMessage)
            }

            AsyncToSyncMessage::Stop => Ok(ThreadState::Stop),
        }
    }
}
