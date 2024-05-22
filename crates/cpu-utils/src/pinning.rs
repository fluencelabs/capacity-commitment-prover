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

use ccp_core_affinity::CoreId;

use crate::LogicalCoreId;

pub const DUMMY: DummyThreadPinner = DummyThreadPinner {};
pub const DEFAULT: DefaultThreadPinner = DefaultThreadPinner {};

#[cfg_attr(feature = "mockall", mockall::automock)]
pub trait ThreadPinner: Send + Sync {
    fn pin_current_thread_to(&self, core_id: LogicalCoreId) -> bool;

    fn pin_current_thread_to_cpuset(&self, core_ids: &[LogicalCoreId]) -> bool;
}

pub struct DefaultThreadPinner;

impl ThreadPinner for DefaultThreadPinner {
    #[inline]
    fn pin_current_thread_to(&self, core_id: LogicalCoreId) -> bool {
        let core_id = CoreId { id: core_id.into() };
        ccp_core_affinity::set_for_current(core_id)
    }

    #[inline]
    fn pin_current_thread_to_cpuset(&self, core_ids: &[LogicalCoreId]) -> bool {
        let core_ids = core_ids
            .iter()
            .map(|core_id| {
                let id = (*core_id).into();
                CoreId { id }
            })
            .collect::<Vec<_>>();

        ccp_core_affinity::set_mask_for_current(&core_ids)
    }
}

pub struct DummyThreadPinner;

impl ThreadPinner for DummyThreadPinner {
    #[inline]
    fn pin_current_thread_to(&self, _core_id: LogicalCoreId) -> bool {
        true
    }

    #[inline]
    fn pin_current_thread_to_cpuset(&self, _core_ids: &[LogicalCoreId]) -> bool {
        true
    }
}
