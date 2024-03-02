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

use ccp_randomx::RandomXFlags;
use ccp_shared::types::LogicalCoreId;

#[derive(Clone, Debug)]
pub struct CCPConfig {
    pub http_server: HTTPServer,
    pub optimizations: Optimizations,
    pub logs: Logs,
    pub state_dir: std::path::PathBuf,
}

#[derive(Clone, Debug)]
pub struct HTTPServer {
    pub host: String,
    pub port: u16,
    pub utility_cores_ids: Vec<LogicalCoreId>,
}

#[derive(Clone, Debug)]
pub struct Optimizations {
    pub randomx_flags: RandomXFlags,
    pub threads_per_core_policy: ThreadsPerCoreAllocationPolicy,
    pub msr_enabled: bool,
}

#[derive(Clone, Debug)]
pub struct Logs {
    pub report_hashrate: bool,
    pub log_level: log::LevelFilter,
}

#[derive(Clone, Debug)]
pub struct State {
    pub state_dir: std::path::PathBuf,
}

#[derive(Clone, Debug, Default)]
pub enum ThreadsPerCoreAllocationPolicy {
    /// CCP will try to run the optimal amount of threads per core,
    /// trying to utilize all benefits of HT and SMT.
    #[default]
    Optimal,
    /// CCP will try run the exact amount
    Exact {
        threads_per_physical_core: std::num::NonZeroUsize,
    },
}

impl Default for HTTPServer {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9383,
            utility_cores_ids: vec![1.into()],
        }
    }
}

impl Default for Optimizations {
    fn default() -> Self {
        Self {
            randomx_flags: RandomXFlags::recommended_full_mem(),
            threads_per_core_policy: <_>::default(),
            msr_enabled: false,
        }
    }
}

impl Default for Logs {
    fn default() -> Self {
        Self {
            report_hashrate: false,
            log_level: log::LevelFilter::Off,
        }
    }
}
