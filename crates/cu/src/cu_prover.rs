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

use std::path::PathBuf;

use ccp_randomx::cache::CACHE_LEN;
use ccp_randomx::Cache;
use ccp_randomx::DATASET_ITEM_SIZE;
use futures::future::BoxFuture;
use tokio_util::sync::CancellationToken;

use ccp_msr::MSRModeEnforcer;
use ccp_randomx::cache::CacheHandle;
use ccp_randomx::dataset::DatasetHandle;
use ccp_randomx::Dataset;
use ccp_randomx::RandomXFlags;
use ccp_shared::types::*;
use ccp_utils::run_utils::run_unordered;
use ccp_utils::run_utils::run_unordered_with_params;
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
use crate::memory_dumper::{dump, MemoryDumper};
use crate::proving_thread::sync::to_state_operating_message::ToStateOperatingInlet;
use crate::CUProverError;
use crate::CoreIdAllocation;

/// Intended to prove that a specific physical core was assigned to the Fluence network
/// by running PoW based on RandomX.
#[derive(Debug)]
pub struct CUProver {
    threads: nonempty::NonEmpty<ProvingThreadAsync>,
    core_id_allocation: CoreIdAllocation,
    randomx_flags: RandomXFlags,
    cpu_topology: HwlocCPUTopology,
    dataset: Dataset,
    status: CUStatus,
    dataset_chunk_size: u64,
    temp_dir: PathBuf,
}

impl CUProver {
    pub async fn create(
        config: CUProverConfig,
        to_state_operating: ToStateOperatingInlet,
        to_stats_collecting: ToStatsCollectingInlet,
        msr_enforcer: MSRModeEnforcer,
        core_id_allocation: CoreIdAllocation,
        temp_dir: PathBuf,
    ) -> CUResult<Self> {
        let topology = HwlocCPUTopology::new()?;
        let mut threads = ThreadAllocator::new(
            config.threads_per_core_policy,
            core_id_allocation.clone(),
            &topology,
        )?
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
            core_id_allocation,
            randomx_flags: config.randomx_flags,
            cpu_topology: topology,
            dataset,
            status: CUStatus::Idle,
            dataset_chunk_size: config.dataset_chunk_size,
            temp_dir,
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
        self.initialize_dataset(cache, dataset_handle.clone(), cancel_token, epoch)
            .await?;

        self.run_proving_jobs(epoch, dataset_handle, cu_id).await
    }

    pub async fn prepare_dataset(
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
        self.initialize_and_dump_dataset(cache, dataset_handle.clone(), cancel_token, epoch)
            .await?;

        Ok(())
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
        self.core_id_allocation = CoreIdAllocation::PhysicalCoreId(new_core_id);

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
        match self.core_id_allocation {
            CoreIdAllocation::SingleUnrestricted => None,
            CoreIdAllocation::LogicalCoreIds(_) => None,
            CoreIdAllocation::PhysicalCoreId(physical_core_id) => Some(physical_core_id),
        }
    }

    #[allow(clippy::needless_lifetimes)]
    async fn initialize_dataset<'threads>(
        &'threads mut self,
        cache: Cache,
        dataset: DatasetHandle,
        cancel_token: CancellationToken,
        epoch: EpochParameters,
    ) -> CUResult<()> {
        let dataset_size = dataset.items_count();

        let cache_len = CACHE_LEN;

        assert_eq!(sizes::MIB % DATASET_ITEM_SIZE, 0);
        // Add extra MIB for alignment, rounding etc
        let dataset_phase0_size = (cache_len + sizes::MIB) / DATASET_ITEM_SIZE;
        let dataset_phase1_size = dataset_size - dataset_phase0_size;

        let threads_number = self.threads.len() as u64;
        let chunk_size = self.dataset_chunk_size;

        let params_phase0 = InitializationState {
            range_start: 0,
            range_len: dataset_phase0_size,
            cache: cache.handle(),
            dataset: dataset.clone(),
            threads_number,
            chunk_size,
            cancel_token: cancel_token.clone(),
            epoch,
        };
        run_unordered_with_params(self.threads.iter_mut(), params_phase0, initialization_body)
            .await
            .unwrap();

        let dataset_phase0_len = (dataset_phase0_size * DATASET_ITEM_SIZE) as usize;

        // Safety: dataset is valid, initialized, mmap-allocated memory range
        let memory_dumper = unsafe {
            let dataset_base = dataset.memory() as usize;
            MemoryDumper::new(&self.temp_dir, dataset_base, dataset_phase0_len)
        }
        .await
        .map_err(CUProverError::DumperError)?;

        let params_phase1 = InitializationState {
            range_start: dataset_phase0_size,
            range_len: dataset_phase1_size,
            cache: cache.handle(),
            dataset: dataset.clone(),
            threads_number,
            chunk_size,
            cancel_token,
            epoch,
        };
        run_unordered_with_params(self.threads.iter_mut(), params_phase1, initialization_body)
            .await
            .unwrap();
        std::mem::drop(cache);

        // Safety: safe because memory_dumper refers to valid dataset memory range
        unsafe { memory_dumper.restore() }
            .await
            .map_err(CUProverError::DumperError)?;

        Ok(())
    }

    #[allow(clippy::needless_lifetimes)]
    pub async fn initialize_and_dump_dataset<'threads>(
        &'threads mut self,
        cache: Cache,
        dataset: DatasetHandle,
        cancel_token: CancellationToken,
        epoch: EpochParameters,
    ) -> CUResult<()> {
        let dataset_size = dataset.items_count();

        assert_eq!(sizes::MIB % DATASET_ITEM_SIZE, 0);
        // Add extra MIB for alignment, rounding etc

        let threads_number = self.threads.len() as u64;
        let chunk_size = self.dataset_chunk_size;

        let params = InitializationState {
            range_start: 0,
            range_len: dataset_size,
            cache: cache.handle(),
            dataset: dataset.clone(),
            threads_number,
            chunk_size,
            cancel_token: cancel_token.clone(),
            epoch,
        };
        run_unordered_with_params(self.threads.iter_mut(), params, initialization_body)
            .await
            .unwrap();

        let dataset_len = (dataset_size * DATASET_ITEM_SIZE) as usize;

        let dataset_file_path = self.temp_dir.join("dataset");
        let mut dataset_file = tokio::fs::File::create(&dataset_file_path).await.unwrap(); //TODO:

        // Safety: dataset is valid, initialized, mmap-allocated memory range
        unsafe {
            let dataset_base = dataset.memory() as usize;
            dump(&mut dataset_file, dataset_base, dataset_len)
        }
        .await
        .map_err(|err| CUProverError::DumperError(err.into()))?;

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

#[derive(Debug, Clone)]
struct InitializationState {
    range_start: u64,
    range_len: u64,
    cache: CacheHandle,
    dataset: DatasetHandle,
    threads_number: u64,
    chunk_size: u64,
    cancel_token: CancellationToken,
    epoch: EpochParameters,
}

fn initialization_body(
    thread_id: usize,
    state: InitializationState,
    thread: &mut ProvingThreadAsync,
) -> BoxFuture<'_, Result<(), <ProvingThreadAsync as ProvingThreadFacade>::Error>> {
    use futures::FutureExt;

    let InitializationState {
        range_start,
        range_len,
        cache,
        dataset,
        threads_number,
        chunk_size,
        cancel_token,
        epoch,
    } = state;

    let thread_id = thread_id as u64;

    let start_item = range_start + (range_len * thread_id) / threads_number;
    let next_start_item = range_start + (range_len * (thread_id + 1)) / threads_number;

    let chunk_generator = ChunkGenerator::new(start_item, next_start_item, chunk_size);

    async move {
        for chunk in chunk_generator {
            if cancel_token.is_cancelled() {
                break;
            }
            let start_item = chunk.start;
            let items_count = chunk.size;
            thread
                .initialize_dataset(
                    cache.clone(),
                    dataset.clone(),
                    start_item,
                    items_count,
                    epoch,
                )
                .await?;
        }
        Ok(())
    }
    .boxed()
}

impl ToCUStatus for CUProver {
    fn status(&self) -> CUStatus {
        self.status
    }
}
