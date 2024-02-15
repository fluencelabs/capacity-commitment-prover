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

use ccp_shared::types::*;
use randomx::cache::CacheHandle;
use randomx::dataset::DatasetHandle;
use randomx::Dataset;
use randomx::RandomXFlags;
use randomx_rust_wrapper as randomx;

use super::errors::CUProverError;
use super::proving_thread::ProvingThread;
use super::proving_thread::ProvingThreadAPI;
use super::proving_thread::RawProof;
use super::CUResult;

/// Intended to prove that a specific physical core was assigned to the Fluence network
/// by running PoW based on RandomX.
pub struct CUProver {
    threads: nonempty::NonEmpty<ProvingThread>,
    config: CUProverConfig,
    dataset: Option<Dataset>,
    cu_id: Option<CUID>,
}

#[derive(Clone, Debug)]
pub struct CUProverConfig {
    pub randomx_flags: randomx::RandomXFlags,
    /// Defines how many threads will be assigned to a specific physical core,
    /// aims to utilize benefits of hyper-threading.
    pub threads_per_physical_core: std::num::NonZeroUsize,
}

impl CUProver {
    pub(crate) fn new(
        config: CUProverConfig,
        proof_receiver_inlet: mpsc::Sender<RawProof>,
        core_id: PhysicalCoreId,
    ) -> Self {
        let threads = (0..config.threads_per_physical_core.into())
            .map(|_| ProvingThread::new(core_id, proof_receiver_inlet.clone()))
            .collect::<Vec<_>>();
        let threads = nonempty::NonEmpty::from_vec(threads).unwrap();

        Self {
            threads,
            config,
            dataset: None,
            cu_id: None,
        }
    }

    pub(crate) async fn new_epoch(
        &mut self,
        global_nonce: GlobalNonce,
        cu_id: CUID,
        difficulty: Difficulty,
        flags: RandomXFlags,
    ) -> CUResult<()> {
        self.cu_id = Some(cu_id);

        let thread = &mut self.threads.head;
        let cache = thread.create_cache(global_nonce, cu_id, flags).await?;

        self.ensure_database_allocated(flags).await?;
        let dataset_handle = self.dataset.as_ref().unwrap().handle();
        let cache_handle = cache.handle();
        self.initialize_dataset(cache_handle, dataset_handle.clone())
            .await?;

        self.run_proving_jobs(dataset_handle, flags, global_nonce, difficulty)
            .await
    }

    pub(crate) async fn repin(core_id: PhysicalCoreId) -> Result<(), ()> {
        unimplemented!()
    }

    pub(crate) async fn stop<'threads>(&'threads mut self) -> CUResult<()> {
        use futures::FutureExt;

        let closure = |_: usize, thread: &'threads mut ProvingThread| thread.stop().boxed_local();
        self.run_on_all_threads(closure).await
    }

    async fn ensure_database_allocated(&mut self, flags: RandomXFlags) -> CUResult<()> {
        if let None = self.dataset {
            let thread = &mut self.threads.head;
            let dataset = thread.allocate_dataset(flags).await?;
            self.dataset = Some(dataset);
        }
        Ok(())
    }

    async fn initialize_dataset<'threads>(
        &'threads mut self,
        cache: CacheHandle,
        dataset: DatasetHandle,
    ) -> CUResult<()> {
        use futures::FutureExt;

        let thread_init_length = dataset.items_count() / (self.threads.len() as u64);
        let closure = |thread_id: usize, thread: &'threads mut ProvingThread| {
            thread
                .initialize_dataset(
                    cache.clone(),
                    dataset.clone(),
                    thread_id as u64 * thread_init_length,
                    thread_init_length,
                )
                .boxed_local()
        };

        self.run_on_all_threads(closure).await
    }

    async fn run_proving_jobs<'threads>(
        &'threads mut self,
        dataset: DatasetHandle,
        flags: RandomXFlags,
        global_nonce: GlobalNonce,
        difficulty: Difficulty,
    ) -> CUResult<()> {
        use futures::FutureExt;

        let cu_id = self.cu_id.unwrap().clone();
        let closure = |_: usize, thread: &'threads mut ProvingThread| {
            thread
                .run_cc_job(dataset.clone(), flags, global_nonce, difficulty, cu_id)
                .boxed_local()
        };
        self.run_on_all_threads(closure).await?;

        Ok(())
    }

    async fn run_on_all_threads<'thread, 'future: 'thread, T, E>(
        &'thread mut self,
        closure: impl Fn(
            usize,
            &'thread mut ProvingThread,
        ) -> futures::future::LocalBoxFuture<'future, Result<T, E>>,
    ) -> CUResult<()>
    where
        T: std::fmt::Debug,
        Vec<E>: Into<CUProverError>,
    {
        use futures::stream::FuturesUnordered;
        use futures::StreamExt;

        let (_, thread_errors): (Vec<_>, Vec<_>) = self
            .threads
            .iter_mut()
            .enumerate()
            .map(|(thread_id, thread)| closure(thread_id, thread))
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .partition(Result::is_ok);

        if thread_errors.is_empty() {
            return Ok(());
        }

        let thread_errors = thread_errors
            .into_iter()
            .map(Result::unwrap_err)
            .collect::<Vec<_>>();

        Err(thread_errors.into())
    }
}
