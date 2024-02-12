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
use tokio::sync::mpsc::error::TryRecvError;

use super::api::ProvingThreadAPI;
use super::errors::ProvingThreadError;
use super::messages::*;
use super::state::RandomXJobParams;
use super::state::ThreadState;
use super::PTResult;
use crate::Difficulty;
use crate::GlobalNonce;
use crate::LogicalCoreId;
use crate::CUID;

const HASH_PER_ROUND: usize = 1024;

pub(crate) struct ProvingThread {
    inlet: mpsc::Sender<ProverToThreadMessage>,
    outlet: mpsc::Receiver<ThreadToProverMessage>,
    handle: JoinHandle<PTResult<()>>,
}

impl ProvingThread {
    pub(crate) fn new(core_id: LogicalCoreId) -> Self {
        let (ptt_inlet, ptt_outlet) = mpsc::channel::<ProverToThreadMessage>(1);
        let (ttp_inlet, ttp_outlet) = mpsc::channel::<ThreadToProverMessage>(1);

        let thread_closure = Self::create_thread_closure(ptt_outlet, ttp_inlet);
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
    ) -> Box<dyn FnMut() -> PTResult<()> + Send + 'static> {
        Box::new(move || -> PTResult<()> {
            let ptt_message =
                ptt_outlet
                    .blocking_recv()
                    .ok_or(ProvingThreadError::channel_error(
                        "async part of the ptt channel is dropped",
                    ))?;
            let mut thread_state = Self::handle_prover_message(ptt_message, &ttp_inlet)?;

            loop {
                println!("loop state: {thread_state:?}");
                thread_state = match thread_state {
                    ThreadState::Stop => {
                        return Ok(());
                    }
                    ThreadState::WaitForMessage => {
                        // block on the channel till it returns a new message
                        let ptt_message = ptt_outlet
                            .blocking_recv()
                            .ok_or(ProvingThreadError::channel_error(
                            "ThreadState::WaitForMessage: async part of the ptt channel is dropped",
                        ))?;
                        Self::handle_prover_message(ptt_message, &ttp_inlet)?
                    }
                    ThreadState::CCJob { parameters } => {
                        use tokio::sync::mpsc::error::TryRecvError;

                        let parameters = Self::cc_prove(parameters)?;
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
        println!("handle prover message: {message:?}");
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

            ProverToThreadMessage::NewCCJob(params) => {
                let parameters = RandomXJobParams::new(
                    params.dataset,
                    params.flags,
                    params.difficulty,
                    params.proof_receiver_inlet,
                )?;

                Ok(ThreadState::CCJob { parameters })
            }

            ProverToThreadMessage::Stop => Ok(ThreadState::Stop),
        }
    }

    fn cc_prove(job_parameters: RandomXJobParams) -> PTResult<RandomXJobParams> {
        let RandomXJobParams {
            vm,
            mut local_nonce,
            difficulty,
            proof_receiver_inlet,
        } = job_parameters;

        vm.hash_first(local_nonce.get());

        for hash_id in 0..HASH_PER_ROUND {
            local_nonce.next();

            let result_hash = if hash_id == HASH_PER_ROUND - 1 {
                vm.hash_next(local_nonce.get())
            } else {
                vm.hash_last()
            };

            if result_hash.as_ref() < &difficulty {
                local_nonce.prev();
                println!("golden result hash {result_hash:?}");
                let proof = RawProof::new(*local_nonce.get());
                proof_receiver_inlet.blocking_send(proof)?;
            }
        }

        let job_parameters =
            RandomXJobParams::from_vm(vm, local_nonce, difficulty, proof_receiver_inlet);
        Ok(job_parameters)
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
            None => Err(ProvingThreadError::channel_error(format!(
                "sync to async channel is closed unexpectedly"
            ))),
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
            None => Err(ProvingThreadError::channel_error(format!(
                "sync to async channel is closed unexpectedly"
            ))),
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
            None => Err(ProvingThreadError::channel_error(format!(
                "sync to async channel is closed unexpectedly"
            ))),
        }
    }

    async fn run_cc_job(
        &mut self,
        dataset: DatasetHandle,
        flags: RandomXFlags,
        difficulty: Difficulty,
        proof_receiver_inlet: mpsc::Sender<RawProof>,
    ) -> Result<(), Self::Error> {
        let message = NewCCJob::new(dataset, flags, difficulty, proof_receiver_inlet);
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
