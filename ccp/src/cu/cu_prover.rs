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

pub struct CUProverConfig {
    pub randomx_flags: randomx::RandomXFlags,
    /// Defines how many threads will be assigned to a specific physical core,
    /// aims to utilize benefits of hyper-threading.
    pub threads_per_physical_core: std::num::NonZeroUsize,
}

impl CUProver {
    pub(crate) fn new(config: CUProverConfig, core_id: PhysicalCoreId) -> Self {
        let threads = (0..config.threads_per_physical_core.into())
            .map(|_| ProvingThread::new(core_id))
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
    ) -> CUResult<mpsc::Receiver<RawProof>> {
        self.cu_id = Some(cu_id);

        let thread = &mut self.threads.head;
        let cache = thread.create_cache(global_nonce, cu_id, flags).await?;

        self.ensure_database_allocated(flags).await?;
        let dataset_handle = self.dataset.as_ref().unwrap().handle();
        let cache_handle = cache.handle();
        self.initialize_dataset(cache_handle, dataset_handle.clone())
            .await?;

        self.run_proving_jobs(dataset_handle, flags, difficulty)
            .await
    }

    pub(crate) async fn repin(core_id: PhysicalCoreId) -> Result<(), ()> {
        unimplemented!()
    }

    pub(crate) async fn stop<'tasks>(&'tasks mut self) -> CUResult<()> {
        use futures::FutureExt;

        let closure = |_: usize, task: &'tasks mut ProvingThread| task.stop().boxed_local();
        self.run_on_all_threads(closure).await
    }

    async fn ensure_database_allocated(&mut self, flags: RandomXFlags) -> CUResult<()> {
        if let None = self.dataset {
            let task = &mut self.threads.head;
            let dataset = task.allocate_dataset(flags).await?;
            self.dataset = Some(dataset);
        }
        Ok(())
    }

    async fn initialize_dataset<'tasks>(
        &'tasks mut self,
        cache: CacheHandle,
        dataset: DatasetHandle,
    ) -> CUResult<()> {
        use futures::FutureExt;

        let task_init_length = dataset.items_count() / (self.threads.len() as u64);
        let closure = |task_id: usize, task: &'tasks mut ProvingThread| {
            task.initialize_dataset(
                cache.clone(),
                dataset.clone(),
                task_id as u64 * task_init_length,
                task_init_length,
            )
            .boxed_local()
        };

        self.run_on_all_threads(closure).await
    }

    async fn run_proving_jobs<'tasks>(
        &'tasks mut self,
        dataset: DatasetHandle,
        flags: RandomXFlags,
        difficulty: Difficulty,
    ) -> CUResult<mpsc::Receiver<RawProof>> {
        use futures::FutureExt;

        let (inlet, outlet) = mpsc::channel(100);
        let closure = |_: usize, task: &'tasks mut ProvingThread| {
            task.run_cc_job(dataset.clone(), flags, difficulty, inlet.clone())
                .boxed_local()
        };
        self.run_on_all_threads(closure).await?;

        Ok(outlet)
    }

    async fn run_on_all_threads<'tasks, 'future: 'tasks, T, E>(
        &'tasks mut self,
        closure: impl Fn(
            usize,
            &'tasks mut ProvingThread,
        ) -> futures::future::LocalBoxFuture<'future, Result<T, E>>,
    ) -> CUResult<()>
    where
        T: std::fmt::Debug,
        Vec<E>: Into<CUProverError>,
    {
        use futures::stream::FuturesUnordered;
        use futures::StreamExt;

        let (_, task_errors): (Vec<_>, Vec<_>) = self
            .threads
            .iter_mut()
            .enumerate()
            .map(|(task_id, task)| closure(task_id, task))
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .partition(Result::is_ok);

        if task_errors.is_empty() {
            return Ok(());
        }

        let task_errors = task_errors
            .into_iter()
            .map(Result::unwrap_err)
            .collect::<Vec<_>>();
        Err(task_errors.into())
    }
}
