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

use crate::LogicalCoreId;
use crate::PhysicalCoreId;

#[derive(Debug, ThisError)]
pub enum CPUTopologyError {
    #[error(transparent)]
    RawHwlocError(#[from] hwlocality::errors::RawHwlocError),

    #[error(transparent)]
    TypeToDepthError(#[from] hwlocality::object::depth::TypeToDepthError),

    #[error(transparent)]
    CPUBindingError(#[from] hwlocality::cpu::binding::CpuBindingError),

    #[error("topology allocation failed, probably not enough free memory")]
    TopologyAllocationFailed,

    #[error("physical core id {core_id} is too big to be represented as a signed int")]
    LogicalCoreIdTooBig { core_id: LogicalCoreId },

    #[error("physical core with {core_id} id not found")]
    PhysicalCoreNotFound { core_id: PhysicalCoreId },

    #[error("logical cores for physical core with {core_id} id not found")]
    LogicalCoresNotFound { core_id: PhysicalCoreId },

    #[error("cpuset for a physical core with {core_id} not found")]
    CPUSetNotFound { core_id: PhysicalCoreId },
}

impl CPUTopologyError {
    pub fn physical_core_not_found(core_id: PhysicalCoreId) -> Self {
        Self::PhysicalCoreNotFound { core_id }
    }

    pub fn logical_cores_not_found(core_id: PhysicalCoreId) -> Self {
        Self::PhysicalCoreNotFound { core_id }
    }

    pub fn cpuset_not_found(core_id: PhysicalCoreId) -> Self {
        Self::CPUSetNotFound { core_id }
    }

    pub fn logical_core_too_big(core_id: LogicalCoreId) -> Self {
        Self::LogicalCoreIdTooBig { core_id }
    }
}
