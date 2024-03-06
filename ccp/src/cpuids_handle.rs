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

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use cpu_utils::LogicalCoreId;
use parking_lot::Mutex;

#[derive(Clone)]
pub struct CpuIdsHandle(Arc<CpuIdsHandleInner>);

impl CpuIdsHandle {
    pub fn new(initial_state: Vec<LogicalCoreId>) -> Self {
        Self(Arc::new(CpuIdsHandleInner {
            cpu_ids: Mutex::new(initial_state),
            epoch: AtomicU32::new(0),
        }))
    }

    pub fn get(&self) -> Vec<LogicalCoreId> {
        let guard = self.0.cpu_ids.lock();
        (*guard).clone()
    }

    pub fn set(&self, new_state: Vec<LogicalCoreId>) {
        let mut guard = self.0.cpu_ids.lock();
        self.0.epoch.fetch_add(1, Ordering::Relaxed);
        *guard = new_state;
    }

    pub fn get_epoch_relaxed(&self) -> u32 {
        self.0.epoch.load(Ordering::Relaxed)
    }
}

struct CpuIdsHandleInner {
    cpu_ids: Mutex<Vec<LogicalCoreId>>,
    epoch: AtomicU32,
}
