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

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
#[path = "linux_x86_64/mod.rs"]
mod msr_impl;
#[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
#[path = "other/mod.rs"]
mod msr_impl;

pub mod state;

use ccp_shared::types::LogicalCoreId;

pub use msr_impl::*;

pub type MSRResult<T> = Result<T, MSRError>;

pub trait MSREnforce {
    /// Applies chosen MSR policy to current core.
    fn enforce(&mut self, _core_id: LogicalCoreId) -> MSRResult<()>;

    /// Cease applied policy to original presets.
    fn cease(self, _core_id: LogicalCoreId) -> MSRResult<()>;
}
