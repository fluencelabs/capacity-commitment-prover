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

#[cfg(target_os = "linux")]
mod cpu_preset;
mod errors;
mod msr_item;
#[cfg(target_os = "linux")]
mod msr_linux;
#[cfg(target_os = "linux")]
mod msr_mode;

#[cfg(target_os = "linux")]
pub use msr_linux::MSRImpl;

#[cfg(not(target_os = "linux"))]
mod msr_other;
#[cfg(not(target_os = "linux"))]
pub use msr_other::MSRImpl;

use cpu_utils::LogicalCoreId;

pub use errors::MSRError;
pub use msr_item::MSRItem;

pub type MSRResult<T> = Result<T, MSRError>;

pub trait MSR {
    fn write_preset(&mut self, store_state: bool) -> MSRResult<()>;
    fn repin(&mut self, core_id: LogicalCoreId) -> MSRResult<()>;
    fn restore(self) -> MSRResult<()>;
}
