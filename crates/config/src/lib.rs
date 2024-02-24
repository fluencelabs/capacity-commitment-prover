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

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![deny(
    dead_code,
    nonstandard_style,
    unused_imports,
    unused_mut,
    unused_variables,
    unused_unsafe,
    unreachable_patterns
)]

use std::path::PathBuf;

use ccp_randomx::RandomXFlags;

#[derive(Clone, Debug)]
pub struct CCPConfig {
    pub thread_allocation_policy: ThreadsPerCoreAllocationPolicy,
    pub randomx_flags: RandomXFlags,
    pub dir_to_store_proofs: PathBuf,
    pub dir_to_store_persistent_state: PathBuf,
    pub enable_msr: bool,
}

#[derive(Clone, Debug)]
pub enum ThreadsPerCoreAllocationPolicy {
    /// CCP will try to run the optimal amount of threads per core,
    /// trying to utilize all benefits of HT and SMT.
    Optimal,
    /// CCP will try run the exact amount
    Exact {
        threads_per_physical_core: std::num::NonZeroUsize,
    },
}
