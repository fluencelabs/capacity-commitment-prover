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

use std::path::Path;

use eyre::eyre;
use serde::Deserialize;
use serde::Serialize;

use super::defaults::default_async_to_sync_queue_size;
use super::defaults::default_facade_queue_size;
use super::defaults::default_hashes_per_round;
use super::defaults::default_log_level;
use super::defaults::default_msr_enabled;
use super::defaults::default_report_hashrate;
use super::defaults::default_state_path;
use super::defaults::default_sync_to_async_queue_size;
use super::defaults::default_utility_queue_size;

use crate::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UnresolvedCCPConfig {
    pub rpc_endpoint: UnresolvedRpcEndpoint,
    pub prometheus_endpoint: Option<UnresolvedPrometheusEndpoint>,
    #[serde(default)]
    pub optimizations: UnresolvedOptimizations,
    #[serde(default)]
    pub logs: UnresolvedLogs,
    #[serde(default)]
    pub state: State,
    #[serde(default)]
    pub workers: UnresolvedWorkers,
    #[serde(default)]
    pub tokio: UnresolvedTokio,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UnresolvedRpcEndpoint {
    pub host: String,
    pub port: u16,
    #[serde(default = "default_utility_queue_size")]
    pub utility_queue_size: usize,
    #[serde(default = "default_facade_queue_size")]
    pub facade_queue_size: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UnresolvedPrometheusEndpoint {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UnresolvedOptimizations {
    #[serde(flatten)]
    pub randomx: UnresolvedRandomX,

    #[serde(default = "default_msr_enabled")]
    pub msr_enabled: bool,

    pub threads_per_core: Option<usize>,
}

impl Default for UnresolvedOptimizations {
    fn default() -> Self {
        Self {
            randomx: Default::default(),
            msr_enabled: default_msr_enabled(),
            threads_per_core: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct UnresolvedRandomX {
    pub large_pages: Option<bool>,
    pub hard_aes: Option<bool>,
    pub jit: Option<bool>,
    pub secure: Option<bool>,
    pub argon2: Option<Argon2Impl>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UnresolvedLogs {
    #[serde(default = "default_report_hashrate")]
    pub report_hashrate: bool,

    #[serde(default = "default_log_level")]
    pub log_level: LogLevel,
}

impl Default for UnresolvedLogs {
    fn default() -> Self {
        UnresolvedLogs {
            report_hashrate: default_report_hashrate(),
            log_level: default_log_level(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    #[serde(default = "default_state_path")]
    pub path: std::path::PathBuf,
}

impl Default for State {
    fn default() -> Self {
        State {
            path: default_state_path(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct UnresolvedWorkers {
    #[serde(default = "default_hashes_per_round")]
    pub hashes_per_round: usize,
    #[serde(default = "default_async_to_sync_queue_size")]
    pub async_to_sync_queue_size: usize,
    #[serde(default = "default_sync_to_async_queue_size")]
    pub sync_to_async_queue_size: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UnresolvedTokio {
    pub worker_threads: Option<usize>,
    pub max_blocking_threads: Option<usize>,
    #[serde(default)]
    pub utility_thread_ids: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Argon2Impl {
    AVX2,
    SSSE3,
    Default,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl UnresolvedCCPConfig {
    pub fn resolve(self, config_path: impl AsRef<Path>) -> eyre::Result<CCPConfig> {
        let config_dir = config_path.as_ref().parent().ok_or_else(|| {
            eyre!(
                "config resolver was provided with invalid config path: {}",
                config_path.as_ref().display()
            )
        })?;

        let rpc_endpoint = self.rpc_endpoint.resolve();
        let prometheus_endpoint = self.prometheus_endpoint.map(|cfg| cfg.resolve());
        let optimization = self.optimizations.resolve()?;
        let logs = self.logs.resolve();
        let workers = self.workers.resolve();
        let tokio = self.tokio.resolve();

        let config = CCPConfig {
            rpc_endpoint,
            prometheus_endpoint,
            optimizations: optimization,
            logs,
            state_dir: config_dir.join(self.state.path),
            workers,
            tokio,
        };
        Ok(config)
    }
}

impl UnresolvedRpcEndpoint {
    pub fn resolve(self) -> RpcEndpoint {
        RpcEndpoint {
            host: self.host,
            port: self.port,
            utility_queue_size: self.utility_queue_size,
            facade_queue_size: self.facade_queue_size,
        }
    }
}

impl UnresolvedPrometheusEndpoint {
    pub fn resolve(self) -> PrometheusEndpoint {
        PrometheusEndpoint {
            host: self.host,
            port: self.port,
        }
    }
}

impl UnresolvedOptimizations {
    pub fn resolve(self) -> eyre::Result<Optimizations> {
        let randomx_flags = self.randomx.resolve();
        let msr_config = self.msr_enabled;
        let threads_per_core_policy = match self.threads_per_core {
            Some(threads_count) => ThreadsPerCoreAllocationPolicy::Exact {
                threads_per_physical_core: threads_count.try_into()?,
            },
            None => ThreadsPerCoreAllocationPolicy::Optimal,
        };

        let opt = Optimizations {
            randomx_flags,
            msr_enabled: msr_config,
            threads_per_core_policy,
        };
        Ok(opt)
    }
}

impl UnresolvedRandomX {
    pub fn resolve(self) -> RandomXFlags {
        let mut randomx_flags = RandomXFlags::recommended_full_mem();

        if let Some(value) = self.large_pages {
            randomx_flags.set(RandomXFlags::LARGE_PAGES, value);
        }

        if let Some(value) = self.hard_aes {
            randomx_flags.set(RandomXFlags::HARD_AES, value);
        }

        if let Some(value) = self.jit {
            randomx_flags.set(RandomXFlags::FLAG_JIT, value);
        }

        if let Some(value) = self.secure {
            randomx_flags.set(RandomXFlags::FLAG_SECURE, value);
        }

        match self.argon2 {
            Some(Argon2Impl::AVX2) => randomx_flags.set(RandomXFlags::FLAG_ARGON2_AVX2, true),
            Some(Argon2Impl::SSSE3) => randomx_flags.set(RandomXFlags::FLAG_ARGON2_SSSE3, true),
            Some(Argon2Impl::Default) => randomx_flags.set(RandomXFlags::FLAG_ARGON2, true),
            None => {}
        }

        randomx_flags
    }
}

impl LogLevel {
    pub fn to_tracing_filter(&self) -> tracing_subscriber::filter::LevelFilter {
        use tracing_subscriber::filter::LevelFilter;

        match self {
            LogLevel::Off => LevelFilter::OFF,
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Trace => LevelFilter::TRACE,
        }
    }
}

impl UnresolvedLogs {
    pub fn resolve(self) -> Logs {
        Logs {
            report_hashrate: self.report_hashrate,
            log_level: self.log_level.to_tracing_filter(),
        }
    }
}

impl UnresolvedWorkers {
    pub fn resolve(self) -> Workers {
        Workers {
            hashes_per_round: self.hashes_per_round,
            async_to_sync_queue_size: self.async_to_sync_queue_size,
            sync_to_async_queue_size: self.sync_to_async_queue_size,
        }
    }
}

impl Default for UnresolvedWorkers {
    fn default() -> Self {
        Self {
            hashes_per_round: default_hashes_per_round(),
            async_to_sync_queue_size: default_async_to_sync_queue_size(),
            sync_to_async_queue_size: default_sync_to_async_queue_size(),
        }
    }
}

impl UnresolvedTokio {
    pub fn resolve(self) -> Tokio {
        let utility_thread_ids = self
            .utility_thread_ids
            .into_iter()
            .map(Into::into)
            .collect();

        Tokio {
            utility_cores_ids: utility_thread_ids,
            worker_threads: self.worker_threads,
            max_blocking_threads: self.max_blocking_threads,
        }
    }
}
