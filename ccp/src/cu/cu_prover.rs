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

use ccp_config::ThreadsPerCoreAllocationPolicy;
use ccp_shared::types::*;
use cpu_topology::CPUTopology;
use randomx::cache::CacheHandle;
use randomx::dataset::DatasetHandle;
use randomx::Dataset;
use randomx::RandomXFlags;
use randomx_rust_wrapper as randomx;

use super::proving_thread::ProvingThreadAsync;
use super::proving_thread::ProvingThreadFacade;
use super::proving_thread::RawProof;
use super::proving_thread_utils::ThreadAllocator;
use super::status::CUStatus;
use super::status::ToCUStatus;
use super::CUResult;

/// Intended to prove that a specific physical core was assigned to the Fluence network
/// by running PoW based on RandomX.
#[derive(Debug)]
pub struct CUProver {
    threads: nonempty::NonEmpty<ProvingThreadAsync>,
    pinned_core_id: PhysicalCoreId,
    randomx_flags: RandomXFlags,
    topology: CPUTopology,
    dataset: Dataset,
    status: CUStatus,
}

#[derive(Clone, Debug)]
pub struct CUProverConfig {
    pub randomx_flags: randomx::RandomXFlags,
    /// Defines how many threads will be assigned to a specific physical core,
    /// aims to utilize benefits of hyper-threading.
    pub thread_allocation_policy: ThreadsPerCoreAllocationPolicy,
}

impl CUProver {
    pub(crate) async fn create(
        config: CUProverConfig,
        proof_receiver_inlet: mpsc::Sender<RawProof>,
        core_id: PhysicalCoreId,
    ) -> CUResult<Self> {
        let topology = CPUTopology::new()?;
        let mut threads =
            ThreadAllocator::new(config.thread_allocation_policy, core_id, &topology)?
                .allocate(proof_receiver_inlet)?;

        let thread = &mut threads.head;
        let dataset = thread.allocate_dataset(config.randomx_flags).await?;

        let prover = Self {
            threads,
            pinned_core_id: core_id,
            randomx_flags: config.randomx_flags,
            topology,
            dataset,
            status: CUStatus::Idle,
        };
        Ok(prover)
    }

    pub(crate) async fn new_epoch(
        &mut self,
        global_nonce: GlobalNonce,
        cu_id: CUID,
        difficulty: Difficulty,
    ) -> CUResult<()> {
        self.status = CUStatus::Running { cu_id };

        let thread = &mut self.threads.head;
        let randomx_flags = self.randomx_flags;
        let cache = thread
            .create_cache(global_nonce, cu_id, randomx_flags)
            .await?;

        let dataset_handle = self.dataset.handle();
        let cache_handle = cache.handle();
        self.initialize_dataset(cache_handle, dataset_handle.clone())
            .await?;

        self.run_proving_jobs(dataset_handle, global_nonce, difficulty, cu_id)
            .await
    }

    #[allow(clippy::needless_lifetimes)]
    pub(crate) async fn pin<'threads>(&'threads mut self, core_id: PhysicalCoreId) -> CUResult<()> {
        use super::proving_thread_utils::RoundRobinDistributor;
        use super::proving_thread_utils::ThreadDistributionPolicy;

        use futures::FutureExt;

        let logical_cores = self.topology.logical_cores_for_physical(core_id)?;
        let distributor = RoundRobinDistributor {};

        let closure = |thread_id: usize, thread: &'threads mut ProvingThreadAsync| {
            let core_id = distributor.distribute(thread_id, &logical_cores);
            thread.pin(core_id).boxed()
        };
        run_on_all_threads(self.threads.iter_mut(), closure).await?;

        Ok(())
    }

    pub(crate) async fn stop(self) -> CUResult<()> {
        use futures::FutureExt;

        let closure = |_: usize, thread: ProvingThreadAsync| thread.stop().boxed();
        run_on_all_threads(self.threads.into_iter(), closure).await?;

        Ok(())
    }

    pub(crate) fn pinned_core_id(&self) -> PhysicalCoreId {
        self.pinned_core_id
    }

    #[allow(clippy::needless_lifetimes)]
    async fn initialize_dataset<'threads>(
        &'threads mut self,
        cache: CacheHandle,
        dataset: DatasetHandle,
    ) -> CUResult<()> {
        use futures::FutureExt;

        let threads_number = self.threads.len() as u64;
        let dataset_size = dataset.items_count();
        let thread_workload_size = dataset_size / threads_number;

        let closure = |thread_id: usize, thread: &'threads mut ProvingThreadAsync| {
            let workload_size = if thread_id as u64 == threads_number - 1 {
                thread_workload_size + (dataset_size % threads_number)
            } else {
                thread_workload_size
            };

            thread
                .initialize_dataset(
                    cache.clone(),
                    dataset.clone(),
                    thread_id as u64 * thread_workload_size,
                    workload_size,
                )
                .boxed()
        };

        run_on_all_threads(self.threads.iter_mut(), closure)
            .await
            .map_err(Into::into)
    }

    #[allow(clippy::needless_lifetimes)]
    async fn run_proving_jobs<'threads>(
        &'threads mut self,
        dataset: DatasetHandle,
        global_nonce: GlobalNonce,
        difficulty: Difficulty,
        cu_id: CUID,
    ) -> CUResult<()> {
        use futures::FutureExt;

        let randomx_flags = self.randomx_flags;
        let closure = |_: usize, thread: &'threads mut ProvingThreadAsync| {
            thread
                .run_cc_job(
                    dataset.clone(),
                    randomx_flags,
                    global_nonce,
                    difficulty,
                    cu_id,
                )
                .boxed()
        };
        run_on_all_threads(self.threads.iter_mut(), closure).await?;

        Ok(())
    }
}

impl ToCUStatus for CUProver {
    fn status(&self) -> CUStatus {
        self.status
    }
}

async fn run_on_all_threads<'future, Thread, T, E>(
    threads: impl Iterator<Item = Thread>,
    closure: impl Fn(usize, Thread) -> futures::future::BoxFuture<'future, Result<T, E>>,
) -> Result<(), Vec<E>>
where
    T: Send + std::fmt::Debug,
{
    use futures::stream::FuturesUnordered;
    use futures::StreamExt;

    let (_, thread_errors): (Vec<_>, Vec<_>) = threads
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

    Err(thread_errors)
}
