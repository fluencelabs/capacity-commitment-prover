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

use crate::errors::CPUTopologyError;
use crate::CTResult;
use crate::LogicalCoreId;
use crate::PhysicalCoreId;

pub struct CPUTopology {
    topology: hwloc2::Topology,
}

impl CPUTopology {
    pub fn new() -> CTResult<Self> {
        let topology = hwloc2::Topology::new().ok_or(CPUTopologyError::TopologyAllocationFailed)?;
        Ok(Self { topology })
    }

    pub fn physical_cores_count(&self) -> CTResult<usize> {
        let physical_cores = self.topology.objects_with_type(&hwloc2::ObjectType::Core)?;
        Ok(physical_cores.len())
    }

    pub fn logical_cores_for_physical(
        &self,
        core_id: PhysicalCoreId,
    ) -> CTResult<Vec<LogicalCoreId>> {
        let physical_cores = self.topology.objects_with_type(&hwloc2::ObjectType::Core)?;
        let physical_core = physical_cores
            .get(<PhysicalCoreId as Into<usize>>::into(core_id))
            .ok_or(CPUTopologyError::physical_core_not_found(core_id))?;

        let physical_core_cpuset = physical_core
            .cpuset()
            .ok_or(CPUTopologyError::cpuset_not_found(core_id))?;

        let logical_core_ids = physical_core_cpuset
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();

        Ok(logical_core_ids)
    }

    pub fn pin_current_thread_to_cpuset(
        &mut self,
        allowed_core_ids: impl Iterator<Item = LogicalCoreId>,
    ) -> CTResult<()> {
        let cpu_set: hwloc2::CpuSet = allowed_core_ids.into_iter().map(Into::into).collect();
        self.topology
            .set_cpubind(cpu_set, hwloc2::CpuBindFlags::CPUBIND_STRICT)
            .map_err(Into::into)
    }
}
