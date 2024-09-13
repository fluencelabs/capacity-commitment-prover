/*
 * Copyright 2024 Fluence DAO
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

use tokio_util::sync::CancellationToken;

use ccp_msr::MSRModeEnforcer;
use ccp_randomx::cache::CacheHandle;
use ccp_randomx::dataset::DatasetHandle;
use ccp_randomx::Dataset;
use ccp_randomx::RandomXFlags;
use ccp_shared::types::*;
use ccp_utils::run_utils::run_unordered;
use cpu_utils::CPUTopology;
use cpu_utils::HwlocCPUTopology;

use super::chunk::ChunkGenerator;
use super::config::CUProverConfig;
use super::proving_thread::sync::to_stats_collecting_message::ToStatsCollectingInlet;
use super::proving_thread::ProvingThreadAsync;
use super::proving_thread::ProvingThreadConfig;
use super::proving_thread::ProvingThreadFacade;
use super::proving_thread_utils::ThreadAllocator;
use super::status::CUStatus;
use super::status::ToCUStatus;
use super::CUResult;
use crate::proving_thread::sync::to_state_operating_message::ToStateOperatingInlet;

/// Intended to prove that a specific physical core was assigned to the Fluence network
/// by running PoW based on RandomX.
#[derive(Debug)]
pub struct CUProver {
    threads: nonempty::NonEmpty<ProvingThreadAsync>,
    pinned_core_id: Option<PhysicalCoreId>,
    randomx_flags: RandomXFlags,
    cpu_topology: HwlocCPUTopology,
    dataset: Dataset,
    status: CUStatus,
    dataset_chunk_size: u64,
}

impl CUProver {
    pub async fn create(
        config: CUProverConfig,
        to_state_operating: ToStateOperatingInlet,
        to_stats_collecting: ToStatsCollectingInlet,
        msr_enforcer: MSRModeEnforcer,
        core_id: Option<PhysicalCoreId>,
    ) -> CUResult<Self> {
        let topology = HwlocCPUTopology::new()?;
        let mut threads = ThreadAllocator::new(config.threads_per_core_policy, core_id, &topology)?
            .allocate(
                msr_enforcer,
                to_state_operating,
                to_stats_collecting,
                ProvingThreadConfig::from_cu_prover_config(&config),
            )?;

        let thread = &mut threads.head;
        let dataset = thread.allocate_dataset(config.randomx_flags).await?;

        let prover = Self {
            threads,
            pinned_core_id: core_id,
            randomx_flags: config.randomx_flags,
            cpu_topology: topology,
            dataset,
            status: CUStatus::Idle,
            dataset_chunk_size: config.dataset_chunk_size,
        };
        Ok(prover)
    }

    pub async fn new_epoch(
        &mut self,
        epoch: EpochParameters,
        cu_id: CUID,
        cancel_token: CancellationToken,
    ) -> CUResult<()> {
        log::info!("Generating dataset for new epoch: {epoch:?}");
        // pause provers to not produce proofs with a changing dataset at the moment
        self.pause().await?;

        self.status = CUStatus::Running { cu_id };

        let thread = &mut self.threads.head;
        let randomx_flags = self.randomx_flags;
        let cache = thread.create_cache(epoch, cu_id, randomx_flags).await?;

        let dataset_handle = self.dataset.handle();
        let cache_handle = cache.handle();
        self.initialize_dataset(cache_handle, dataset_handle.clone(), cancel_token)
            .await?;

        self.run_proving_jobs(epoch, dataset_handle, cu_id).await
    }

    #[allow(clippy::needless_lifetimes)]
    pub async fn pin<'threads>(&'threads mut self, new_core_id: PhysicalCoreId) -> CUResult<()> {
        use super::proving_thread_utils::RoundRobinDistributor;
        use super::proving_thread_utils::ThreadDistributionPolicy;

        use futures::FutureExt;

        let logical_cores = self.cpu_topology.logical_cores_for_physical(new_core_id)?;
        let distributor = RoundRobinDistributor {};

        let closure = |thread_id: usize, thread: &'threads mut ProvingThreadAsync| {
            let core_id = distributor.distribute(thread_id, &logical_cores);
            thread.pin(core_id).boxed()
        };
        run_unordered(self.threads.iter_mut(), closure).await?;
        self.pinned_core_id = Some(new_core_id);

        Ok(())
    }

    #[allow(clippy::needless_lifetimes)]
    pub async fn pause<'threads>(&'threads mut self) -> CUResult<()> {
        use futures::FutureExt;

        let closure = |_: usize, thread: &'threads mut ProvingThreadAsync| thread.pause().boxed();
        run_unordered(self.threads.iter_mut(), closure).await?;

        self.status = CUStatus::Idle;

        Ok(())
    }

    #[allow(clippy::needless_lifetimes)]
    pub async fn stop_nonblocking<'threads>(&'threads self) -> CUResult<()> {
        use futures::FutureExt;

        let closure =
            |_: usize, thread: &'threads ProvingThreadAsync| thread.stop_nonblocking().boxed();
        run_unordered(self.threads.iter(), closure).await?;
        Ok(())
    }

    pub async fn join(self) -> CUResult<()> {
        use futures::FutureExt;

        let closure = |_: usize, thread: ProvingThreadAsync| thread.join().boxed();
        run_unordered(self.threads.into_iter(), closure).await?;
        Ok(())
    }

    pub async fn stop_join(self) -> CUResult<()> {
        use futures::FutureExt;

        let closure = |_: usize, thread: ProvingThreadAsync| thread.stop_join().boxed();
        run_unordered(self.threads.into_iter(), closure).await?;
        Ok(())
    }

    pub fn pinned_core_id(&self) -> Option<PhysicalCoreId> {
        self.pinned_core_id
    }

    #[allow(clippy::needless_lifetimes)]
    async fn initialize_dataset<'threads>(
        &'threads mut self,
        cache: CacheHandle,
        dataset: DatasetHandle,
        cancel_token: CancellationToken,
    ) -> CUResult<()> {
        use futures::FutureExt;

        let dataset_size = dataset.items_count();
        let threads_number = self.threads.len() as u64;
        let chunk_size = self.dataset_chunk_size;

        let closure = |thread_id: usize, thread: &'threads mut ProvingThreadAsync| {
            let thread_id = thread_id as u64;

            let start_item = (dataset_size * thread_id) / threads_number;
            let next_start_item = (dataset_size * (thread_id + 1)) / threads_number;

            let chunk_generator = ChunkGenerator::new(start_item, next_start_item, chunk_size);
            async {
                for chunk in chunk_generator {
                    if cancel_token.is_cancelled() {
                        break;
                    }
                    let start_item = chunk.start;
                    let items_count = chunk.size;
                    thread
                        .initialize_dataset(cache.clone(), dataset.clone(), start_item, items_count)
                        .await?;
                }
                Ok(())
            }
            .boxed()
        };

        run_unordered(self.threads.iter_mut(), closure).await?;
        Ok(())
    }

    #[allow(clippy::needless_lifetimes)]
    async fn run_proving_jobs<'threads>(
        &'threads mut self,
        epoch: EpochParameters,
        dataset: DatasetHandle,
        cu_id: CUID,
    ) -> CUResult<()> {
        use futures::FutureExt;

        let randomx_flags = self.randomx_flags;
        let closure = |_: usize, thread: &'threads mut ProvingThreadAsync| {
            thread
                .run_cc_job(epoch, dataset.clone(), randomx_flags, cu_id)
                .boxed()
        };
        run_unordered(self.threads.iter_mut(), closure).await?;

        Ok(())
    }
}

impl ToCUStatus for CUProver {
    fn status(&self) -> CUStatus {
        self.status
    }
}
