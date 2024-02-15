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
use std::thread::JoinHandle;

use randomx::cache::CacheHandle;
use randomx::dataset::DatasetHandle;
use randomx::Cache;
use randomx::Dataset;
use randomx_rust_wrapper as randomx;
use randomx_rust_wrapper::RandomXFlags;
use tokio::sync::mpsc;

use super::api::ProvingThreadAPI;
use super::errors::ProvingThreadError;
use super::messages::*;
use super::state::RandomXJob;
use super::state::ThreadState;
use super::PTResult;
use crate::Difficulty;
use crate::GlobalNonce;
use crate::LogicalCoreId;
use crate::CUID;

const HASHES_PER_ROUND: usize = 1024;

const CHANNEL_DROPPED_MESSAGE: &str =
    "ThreadState::WaitForMessage async part of the ptt channel is dropped";

pub(crate) struct ProvingThread {
    inlet: mpsc::Sender<ProverToThreadMessage>,
    outlet: mpsc::Receiver<ThreadToProverMessage>,
    handle: JoinHandle<PTResult<()>>,
}

impl ProvingThread {
    pub(crate) fn new(
        core_id: LogicalCoreId,
        proof_receiver_inlet: mpsc::Sender<RawProof>,
    ) -> Self {
        let (ptt_inlet, ptt_outlet) = mpsc::channel::<ProverToThreadMessage>(1);
        let (ttp_inlet, ttp_outlet) = mpsc::channel::<ThreadToProverMessage>(1);

        let thread_closure =
            Self::create_thread_closure(ptt_outlet, ttp_inlet, proof_receiver_inlet);
        let handle = thread::spawn(thread_closure);

        Self {
            inlet: ptt_inlet,
            outlet: ttp_outlet,
            handle,
        }
    }

    fn create_thread_closure(
        mut ptt_outlet: mpsc::Receiver<ProverToThreadMessage>,
        ttp_inlet: mpsc::Sender<ThreadToProverMessage>,
        proof_receiver_inlet: mpsc::Sender<RawProof>,
    ) -> Box<dyn FnMut() -> PTResult<()> + Send + 'static> {
        Box::new(move || -> PTResult<()> {
            let ptt_message = ptt_outlet
                .blocking_recv()
                .ok_or(ProvingThreadError::channel_error(CHANNEL_DROPPED_MESSAGE))?;
            let mut thread_state = Self::handle_prover_message(ptt_message, &ttp_inlet)?;

            loop {
                log::debug!("proving_thread: new thread_state is {thread_state:?}");

                thread_state = match thread_state {
                    ThreadState::Stop => {
                        return Ok(());
                    }
                    ThreadState::WaitForMessage => {
                        // block on the channel till it returns a new message
                        let ptt_message = ptt_outlet
                            .blocking_recv()
                            .ok_or(ProvingThreadError::channel_error(CHANNEL_DROPPED_MESSAGE))?;
                        Self::handle_prover_message(ptt_message, &ttp_inlet)?
                    }
                    ThreadState::CCJob { parameters } => {
                        use tokio::sync::mpsc::error::TryRecvError;

                        let parameters = Self::cc_prove(parameters, proof_receiver_inlet.clone())?;
                        match ptt_outlet.try_recv() {
                            Ok(message) => Self::handle_prover_message(message, &ttp_inlet)?,
                            Err(TryRecvError::Empty) => ThreadState::CCJob { parameters },
                            Err(e) => Err(e)?,
                        }
                    }
                };
            }
        })
    }

    fn handle_prover_message<'vm>(
        message: ProverToThreadMessage,
        ttp_inlet: &mpsc::Sender<ThreadToProverMessage>,
    ) -> PTResult<ThreadState<'vm>> {
        log::debug!("proving_thread: handle message from CUProver: {message:?}");

        match message {
            ProverToThreadMessage::CreateCache(params) => {
                let global_nonce_cu =
                    ccp_utils::compute_global_nonce_cu(&params.global_nonce, &params.cu_id);
                let cache = Cache::new(&global_nonce_cu.into_bytes(), params.flags)?;

                let ttp_message = CacheCreated::new(cache);
                let ttp_message = ThreadToProverMessage::CacheCreated(ttp_message);
                ttp_inlet.blocking_send(ttp_message)?;

                Ok(ThreadState::WaitForMessage)
            }

            ProverToThreadMessage::AllocateDataset(params) => {
                let dataset = Dataset::allocate(params.flags.contains(RandomXFlags::LARGE_PAGES))?;

                let ttp_message = DatasetAllocated::new(dataset);
                let ttp_message = ThreadToProverMessage::DatasetAllocated(ttp_message);
                ttp_inlet.blocking_send(ttp_message)?;

                Ok(ThreadState::WaitForMessage)
            }

            ProverToThreadMessage::InitializeDataset(mut params) => {
                params
                    .dataset
                    .initialize(&params.cache, params.start_item, params.items_count);
                ttp_inlet.blocking_send(ThreadToProverMessage::DatasetInitialized)?;

                Ok(ThreadState::WaitForMessage)
            }

            ProverToThreadMessage::NewCCJob(cc_job) => {
                let parameters = RandomXJob::from_cc_job(cc_job)?;
                Ok(ThreadState::CCJob { parameters })
            }

            ProverToThreadMessage::Stop => Ok(ThreadState::Stop),
        }
    }

    fn cc_prove(
        mut job: RandomXJob,
        proof_receiver_inlet: mpsc::Sender<RawProof>,
    ) -> PTResult<RandomXJob> {
        job.hash_first();

        for hash_id in 0..HASHES_PER_ROUND {
            let result_hash = if hash_id == HASHES_PER_ROUND - 1 {
                job.hash_last()
            } else {
                job.hash_next()
            };

            if result_hash.as_ref() < &job.difficulty {
                log::info!("proving_thread:: found new golden result hash {result_hash:?}\nfor local_nonce {:?}", job.local_nonce);

                let proof = job.create_golden_proof();
                proof_receiver_inlet.blocking_send(proof)?;
            }
        }

        Ok(job)
    }
}

impl ProvingThreadAPI for ProvingThread {
    type Error = ProvingThreadError;

    async fn create_cache(
        &mut self,
        global_nonce: GlobalNonce,
        cu_id: CUID,
        flags: RandomXFlags,
    ) -> Result<Cache, Self::Error> {
        let message = CreateCache::new(global_nonce, cu_id, flags);
        let message = ProverToThreadMessage::CreateCache(message);
        self.inlet.send(message).await?;

        match self.outlet.recv().await {
            Some(ThreadToProverMessage::CacheCreated(params)) => Ok(params.cache),
            Some(message) => Err(ProvingThreadError::channel_error(format!(
                "expected the CacheCreated event, but {message:?} received"
            ))),
            None => Err(ProvingThreadError::channel_error(
                "sync to async channel is closed unexpectedly".to_string(),
            )),
        }
    }

    async fn allocate_dataset(&mut self, flags: RandomXFlags) -> Result<Dataset, Self::Error> {
        let message = AllocateDataset::new(flags);
        let message = ProverToThreadMessage::AllocateDataset(message);
        self.inlet.send(message).await?;

        match self.outlet.recv().await {
            Some(ThreadToProverMessage::DatasetAllocated(params)) => Ok(params.dataset),
            Some(message) => Err(ProvingThreadError::channel_error(format!(
                "expected the DatasetAllocated event, but {message:?} received"
            ))),
            None => Err(ProvingThreadError::channel_error(
                "sync to async channel is closed unexpectedly".to_string(),
            )),
        }
    }

    async fn initialize_dataset(
        &mut self,
        cache: CacheHandle,
        dataset: DatasetHandle,
        start_item: u64,
        items_count: u64,
    ) -> Result<(), Self::Error> {
        let message = InitializeDataset::new(cache, dataset, start_item, items_count);
        let message = ProverToThreadMessage::InitializeDataset(message);
        self.inlet.send(message).await?;

        match self.outlet.recv().await {
            Some(ThreadToProverMessage::DatasetInitialized) => Ok(()),
            Some(message) => Err(ProvingThreadError::channel_error(format!(
                "expected the DatasetInitialized event, but {message:?} received"
            ))),
            None => Err(ProvingThreadError::channel_error(
                "sync to async channel is closed unexpectedly".to_string(),
            )),
        }
    }

    async fn run_cc_job(
        &mut self,
        dataset: DatasetHandle,
        flags: RandomXFlags,
        global_nonce: GlobalNonce,
        difficulty: Difficulty,
        cu_id: CUID,
    ) -> Result<(), Self::Error> {
        let message = NewCCJob::new(dataset, flags, global_nonce, difficulty, cu_id);
        let message = ProverToThreadMessage::NewCCJob(message);
        self.inlet.send(message).await.map_err(Into::into)
    }

    async fn pin_thread(&self, logical_core_id: LogicalCoreId) -> Result<(), Self::Error> {
        todo!()
    }

    async fn stop(&self) -> Result<(), Self::Error> {
        let message = ProverToThreadMessage::Stop;
        self.inlet.send(message).await?;

        Ok(())
    }
}
