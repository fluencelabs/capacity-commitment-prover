/*
 * Copyright 2024 Fluence DAO
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
use serde::Serialize;
use serde_with::serde_as;
use serde_with::skip_serializing_none;

use crate::defaults::default_control_queue_size;
use crate::defaults::default_cumulative_hashrate_update_interval_secs;
use crate::defaults::default_facade_queue_size;
use crate::defaults::default_log_level;
use crate::defaults::default_msr_enabled;
use crate::defaults::default_report_hashrate;
use crate::defaults::default_utility_queue_size;
use crate::unresolved_config::UnresolvedWorkers;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CCPConfig {
    pub rpc_endpoint: RpcEndpoint,
    pub prometheus_endpoint: Option<PrometheusEndpoint>,
    pub optimizations: Optimizations,
    pub logs: Logs,
    pub workers: Workers,
    pub tokio: Tokio,
    pub state_dir: std::path::PathBuf,
    pub print_config: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RpcEndpoint {
    pub host: String,
    pub port: u16,
    pub utility_queue_size: usize,
    pub facade_queue_size: usize,
    pub control_queue_size: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PrometheusEndpoint {
    pub host: String,
    pub port: u16,
}

#[serde_as]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Optimizations {
    #[serde(serialize_with = "serialize_debug")]
    pub randomx_flags: RandomXFlags,
    pub threads_per_core_policy: ThreadsPerCoreAllocationPolicy,
    pub msr_enabled: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize)]
pub struct Logs {
    pub report_hashrate: bool,
    pub cumulative_hashrate_update_interval_secs: f64,
    #[serde(serialize_with = "serialize_debug")]
    pub log_level: tracing_subscriber::filter::LevelFilter,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct State {
    pub state_dir: std::path::PathBuf,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Workers {
    pub hashes_per_round: usize,
    pub async_to_sync_queue_size: usize,
    pub sync_to_async_queue_size: usize,
    pub dataset_chunk_size: u64,
}

#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize)]
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
            control_queue_size: default_control_queue_size(),
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
            cumulative_hashrate_update_interval_secs:
                default_cumulative_hashrate_update_interval_secs(),
            log_level: default_log_level().to_tracing_filter(),
        }
    }
}

impl Default for Workers {
    fn default() -> Self {
        UnresolvedWorkers::default().resolve()
    }
}

fn serialize_debug<S, T>(value: &T, _s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
    T: std::fmt::Debug,
{
    _s.serialize_str(&format!("{value:?}"))
}
