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

use thiserror::Error as ThisError;

use crate::PhysicalCoreId;

#[derive(Debug, ThisError)]
pub enum CPUTopologyError {
    #[error("topology allocation failed, probably not enough free memory")]
    TopologyAllocationFailed,

    #[error("{0:?}")]
    TypeDepthError(hwloc2::TypeDepthError),

    #[error("physical core with {physical_core_id} not found")]
    PhysicalCoreNotFound { physical_core_id: PhysicalCoreId },

    #[error("cpuset for a physical core with {physical_core_id} not found")]
    CPUSetNotFound { physical_core_id: PhysicalCoreId },

    #[error("{0:?}")]
    CPUBindError(hwloc2::CpuBindError),
}

impl CPUTopologyError {
    pub fn physical_core_not_found(physical_core_id: PhysicalCoreId) -> Self {
        Self::PhysicalCoreNotFound { physical_core_id }
    }

    pub fn cpuset_not_found(physical_core_id: PhysicalCoreId) -> Self {
        Self::CPUSetNotFound { physical_core_id }
    }
}

impl From<hwloc2::TypeDepthError> for CPUTopologyError {
    fn from(value: hwloc2::TypeDepthError) -> Self {
        Self::TypeDepthError(value)
    }
}

impl From<hwloc2::CpuBindError> for CPUTopologyError {
    fn from(value: hwloc2::CpuBindError) -> Self {
        Self::CPUBindError(value)
    }
}
