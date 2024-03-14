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

use crate::defaults::default_facade_queue_size;
use crate::defaults::default_log_level;
use crate::defaults::default_msr_enabled;
use crate::defaults::default_report_hashrate;
use crate::defaults::default_utility_queue_size;
use crate::unresolved_config::UnresolvedWorkers;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CCPConfig {
    pub rpc_endpoint: RpcEndpoint,
    pub prometheus_endpoint: Option<PrometheusEndpoint>,
    pub optimizations: Optimizations,
    pub logs: Logs,
    pub state_dir: std::path::PathBuf,
    pub workers: Workers,
    pub tokio: Tokio,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RpcEndpoint {
    pub host: String,
    pub port: u16,
    pub utility_queue_size: usize,
    pub facade_queue_size: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrometheusEndpoint {
    pub host: String,
    pub port: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Optimizations {
    pub randomx_flags: RandomXFlags,
    pub threads_per_core_policy: ThreadsPerCoreAllocationPolicy,
    pub msr_enabled: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Logs {
    pub report_hashrate: bool,
    pub log_level: tracing_subscriber::filter::LevelFilter,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    pub state_dir: std::path::PathBuf,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Workers {
    pub hashes_per_round: usize,
    pub async_to_sync_queue_size: usize,
    pub sync_to_async_queue_size: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Tokio {
    pub worker_threads: Option<usize>,
    pub max_blocking_threads: Option<usize>,
    pub utility_cores_ids: Vec<LogicalCoreId>,
}

impl Default for RpcEndpoint {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9383,
            utility_queue_size: default_utility_queue_size(),
            facade_queue_size: default_facade_queue_size(),
        }
    }
}

impl Default for Optimizations {
    fn default() -> Self {
        Self {
            randomx_flags: RandomXFlags::recommended_full_mem(),
            threads_per_core_policy: <_>::default(),
            msr_enabled: default_msr_enabled(),
        }
    }
}

impl Default for Logs {
    fn default() -> Self {
        Self {
            report_hashrate: default_report_hashrate(),
            log_level: default_log_level().to_tracing_filter(),
        }
    }
}

impl Default for Workers {
    fn default() -> Self {
        UnresolvedWorkers::default().resolve()
    }
}
