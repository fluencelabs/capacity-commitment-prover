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

use crate::errors::CPUTopologyError;
use crate::CTResult;
use crate::LogicalCoreId;
use crate::PhysicalCoreId;

#[cfg_attr(feature = "mockall", mockall::automock)]
pub trait CPUTopology {
    fn physical_cores(&self) -> CTResult<NonEmpty<PhysicalCoreId>>;

    fn logical_cores_for_physical(
        &self,
        core_id: PhysicalCoreId,
    ) -> CTResult<NonEmpty<LogicalCoreId>>;
}

#[derive(Debug)]
pub struct HwlocCPUTopology {
    topology: hwlocality::Topology,
}

impl HwlocCPUTopology {
    pub fn new() -> CTResult<Self> {
        let topology = hwlocality::Topology::new()?;
        Ok(Self { topology })
    }

    pub fn physical_cores_count_len(&self) -> usize {
        self.physical_cores().map(|r| r.len()).unwrap_or(0)
    }
}

impl CPUTopology for HwlocCPUTopology {
    fn physical_cores(&self) -> CTResult<NonEmpty<PhysicalCoreId>> {
        use hwlocality::object::types::ObjectType;

        let physical_core_ids = self
            .topology
            .objects_with_type(ObjectType::Core)
            .map(|value| PhysicalCoreId::from(value.logical_index() as u32))
            .collect::<Vec<_>>();

        NonEmpty::from_vec(physical_core_ids).ok_or_else(|| CPUTopologyError::PhysicalCoresNotFound)
    }

    fn logical_cores_for_physical(
        &self,
        core_id: PhysicalCoreId,
    ) -> CTResult<NonEmpty<LogicalCoreId>> {
        use hwlocality::object::types::ObjectType;

        let core_depth = self.topology.depth_or_below_for_type(ObjectType::Core)?;
        let physical_cores = self
            .topology
            .objects_at_depth(core_depth)
            .collect::<Vec<_>>();

        let physical_core = physical_cores
            .get(<PhysicalCoreId as Into<usize>>::into(core_id))
            .ok_or(CPUTopologyError::physical_core_not_found(core_id))?;

        let physical_core_cpuset = physical_core
            .cpuset()
            .ok_or(CPUTopologyError::cpuset_not_found(core_id))?;

        let logical_core_ids = physical_core_cpuset
            .into_iter()
            .map(usize::from)
            .map(|value| LogicalCoreId::from(value as u32))
            .collect::<Vec<_>>();

        NonEmpty::from_vec(logical_core_ids)
            .ok_or_else(|| CPUTopologyError::logical_cores_not_found(core_id))
    }
}
