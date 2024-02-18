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
use cpu_topology::CPUTopology;

use super::proving_thread::ProvingThread;
use super::CUResult;
use super::ThreadAllocationError;
use crate::cu::CUProverError;
use crate::cu::RawProof;

pub(crate) struct ThreadAllocator {
    topology: CPUTopology,
    allocation_strategy: ThreadAllocationStrategy,
}

struct ThreadAllocationStrategy(nonempty::NonEmpty<LogicalCoreId>);

impl ThreadAllocator {
    pub(crate) fn new(
        thread_policy: ThreadsPerCoreAllocationPolicy,
        core_id: PhysicalCoreId,
    ) -> CUResult<ThreadAllocator> {
        let topology = CPUTopology::new().map_err(ThreadAllocationError::TopologyError)?;
        let allocation_strategy = ThreadAllocationStrategy::new(&topology, thread_policy, core_id)?;

        Ok(Self {
            topology,
            allocation_strategy,
        })
    }

    pub(crate) fn allocate_threads(
        &self,
        proof_receiver_inlet: mpsc::Sender<RawProof>,
    ) -> CUResult<nonempty::NonEmpty<ProvingThread>> {
        let threads = self
            .allocation_strategy
            .0
            .iter()
            .map(|logical_core| ProvingThread::new(*logical_core, proof_receiver_inlet.clone()))
            .collect::<Vec<_>>();
        let threads = nonempty::NonEmpty::from_vec(threads).unwrap();
        Ok(threads)
    }
}

impl ThreadAllocationStrategy {
    pub(crate) fn new(
        topology: &CPUTopology,
        thread_policy: ThreadsPerCoreAllocationPolicy,
        core_id: PhysicalCoreId,
    ) -> CUResult<Self> {
        let logical_threads = topology
            .logical_cores_for_physical(core_id)
            .map_err(ThreadAllocationError::TopologyError)?;
        if logical_threads.is_empty() {
            return Err(CUProverError::logical_cpus_not_found(core_id));
        }

        let threads_count = match thread_policy {
            ThreadsPerCoreAllocationPolicy::Exact {
                threads_per_physical_core,
            } => threads_per_physical_core,
            ThreadsPerCoreAllocationPolicy::Optimal => unsafe {
                std::num::NonZeroUsize::new_unchecked(logical_threads.len())
            },
        };

        let strategy = (0..threads_count.get())
            .map(|thread_id| logical_threads[thread_id % logical_threads.len()])
            .collect::<Vec<_>>();
        let strategy = Self(NonEmpty::from_vec(strategy).unwrap());

        Ok(strategy)
    }
}
