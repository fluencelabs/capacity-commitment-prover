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

/// This crate is MSR control framework for the archs that have ways to control CPU cache
/// via MSR registers manipulation, e.g. Linux on x86_64.
/// For everything else it's a no-op.
/// Please note there are number of globals that are accessed in the main code.
mod errors;
mod msr_cpu_preset;
mod msr_item;
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
mod msr_mode;
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
mod msr_x86_64;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
mod msr_cpu_preset_x86_64;
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
pub use msr_cpu_preset_x86_64::get_original_cpu_msr_preset;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
pub use msr_mode::detect_msr_mode;
#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
pub use msr_x86_64::MSRImpl;

#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
mod msr_other;
#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
pub use msr_non_x86_64::detect_msr_mode;
#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
pub use msr_non_x86_64::get_original_cpu_msr_preset;
#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
pub use msr_non_x86_64::MSRImpl;

use cpu_utils::LogicalCoreId;

pub use errors::MSRError;
pub use msr_cpu_preset::get_cpu_preset;
pub use msr_cpu_preset::MSRCpuPreset;
pub use msr_item::MSRItem;
use serde::{Deserialize, Serialize};

pub type MSRResult<T> = Result<T, MSRError>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MSRConfig {
    pub msr_enabled: bool,
    pub original_msr_preset: MSRCpuPreset,
}

impl MSRConfig {
    pub fn new(msr_enabled: bool, original_msr_preset: MSRCpuPreset) -> MSRConfig {
        Self {
            msr_enabled,
            original_msr_preset,
        }
    }

    pub fn disabled_msr() -> MSRConfig {
        Self {
            msr_enabled: false,
            original_msr_preset: MSRCpuPreset::new(vec![]),
        }
    }
}

pub trait MSR {
    fn write_preset(&mut self) -> MSRResult<()>;
    fn repin(&mut self, core_id: LogicalCoreId) -> MSRResult<()>;
    fn restore(self) -> MSRResult<()>;
}
