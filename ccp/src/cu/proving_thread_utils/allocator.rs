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

use nonempty::NonEmpty;
use tokio::sync::mpsc;

use ccp_config::ThreadsPerCoreAllocationPolicy;
use ccp_shared::types::LogicalCoreId;
use ccp_shared::types::PhysicalCoreId;
use cpu_utils::CPUTopology;

use super::RoundRobinDistributor;
use crate::cu::proving_thread::ProvingThreadAsync;
use crate::cu::CUResult;
use crate::cu::RawProof;
use crate::cu::ThreadAllocationError;

type ThreadAllocationStrategy = NonEmpty<LogicalCoreId>;

pub(crate) struct ThreadAllocator {
    allocation_strategy: ThreadAllocationStrategy,
}

impl ThreadAllocator {
    pub(crate) fn new(
        thread_policy: ThreadsPerCoreAllocationPolicy,
        core_id: PhysicalCoreId,
        topology: &CPUTopology,
    ) -> CUResult<ThreadAllocator> {
        let allocation_strategy = Self::create_allocate_strategy(thread_policy, core_id, topology)?;

        Ok(Self {
            allocation_strategy,
        })
    }

    pub(crate) fn allocate(
        &self,
        proof_receiver_inlet: mpsc::Sender<RawProof>,
    ) -> CUResult<nonempty::NonEmpty<ProvingThreadAsync>> {
        let threads = self
            .allocation_strategy
            .iter()
            .map(|logical_core| {
                ProvingThreadAsync::new(*logical_core, proof_receiver_inlet.clone())
            })
            .collect::<Vec<_>>();
        let threads = nonempty::NonEmpty::from_vec(threads).unwrap();

        Ok(threads)
    }

    pub(crate) fn create_allocate_strategy(
        thread_policy: ThreadsPerCoreAllocationPolicy,
        core_id: PhysicalCoreId,
        topology: &CPUTopology,
    ) -> CUResult<ThreadAllocationStrategy> {
        use super::ThreadDistributionPolicy;

        let logical_cores = topology
            .logical_cores_for_physical(core_id)
            .map_err(ThreadAllocationError::TopologyError)?;

        let threads_count = match thread_policy {
            ThreadsPerCoreAllocationPolicy::Exact {
                threads_per_physical_core,
            } => threads_per_physical_core,
            ThreadsPerCoreAllocationPolicy::Optimal => unsafe {
                std::num::NonZeroUsize::new_unchecked(logical_cores.len())
            },
        };

        let distributor = RoundRobinDistributor {};
        let strategy = (0..threads_count.get())
            .map(|thread_id| distributor.distribute(thread_id, &logical_cores))
            .collect::<Vec<_>>();
        Ok(NonEmpty::from_vec(strategy).unwrap())
    }
}
