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

use ccp_shared::types::PhysicalCoreId;
use ccp_shared::types::CUID;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct CreateCUProverState {
    pub(crate) new_core_id: PhysicalCoreId,
    pub(crate) new_cu_id: CUID,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct RemoveCUProverState {
    pub(crate) current_core_id: PhysicalCoreId,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct NewCCJobState {
    pub(crate) current_core_id: PhysicalCoreId,
    pub(crate) new_cu_id: CUID,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct NewCCJobWithRepiningState {
    pub(crate) current_core_id: PhysicalCoreId,
    pub(crate) new_core_id: PhysicalCoreId,
    pub(crate) new_cu_id: CUID,
}

impl CreateCUProverState {
    pub(crate) fn new(new_core_id: PhysicalCoreId, new_cu_id: CUID) -> Self {
        Self {
            new_core_id,
            new_cu_id,
        }
    }
}

impl RemoveCUProverState {
    pub(crate) fn new(current_core_id: PhysicalCoreId) -> Self {
        Self { current_core_id }
    }
}

impl NewCCJobState {
    pub(crate) fn new(current_core_id: PhysicalCoreId, new_cu_id: CUID) -> Self {
        Self {
            current_core_id,
            new_cu_id,
        }
    }
}

impl NewCCJobWithRepiningState {
    pub(crate) fn new(
        current_core_id: PhysicalCoreId,
        new_core_id: PhysicalCoreId,
        new_cu_id: CUID,
    ) -> Self {
        Self {
            current_core_id,
            new_core_id,
            new_cu_id,
        }
    }
}
