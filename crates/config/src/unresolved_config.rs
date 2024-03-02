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

use serde::Deserialize;
use serde::Serialize;

use ccp_randomx::RandomXFlags;

use super::config::Optimizations;
//use super::defaults::default_log_level;
use super::defaults::default_msr_enabled;
use super::defaults::report_hashrate;
use crate::*;

const DEFAULT_UTILITY_THREAD_ID: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnresolvedCCPConfig {
    #[serde(rename = "http-server")]
    pub http_server: UnresolvedHTTPServer,
    pub optimizations: UnresolvedOptimizations,
    pub logs: UnresolvedLogs,
    pub state: State,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnresolvedHTTPServer {
    pub host: String,
    pub port: u16,
    #[serde(rename = "utility-thread-ids")]
    pub utility_thread_ids: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnresolvedOptimizations {
    #[serde(flatten)]
    pub randomx: UnresolvedRandomX,

    #[serde(default = "default_msr_enabled")]
    #[serde(rename = "msr-enabled")]
    pub msr_enabled: bool,

    pub threads_per_core_policy: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnresolvedRandomX {
    #[serde(rename = "large-pages")]
    pub large_pages: Option<bool>,
    pub hard_aes: Option<bool>,
    pub jit: Option<bool>,
    pub secure: Option<bool>,
    pub argon2: Option<Argon2Impl>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnresolvedLogs {
    #[serde(default = "report_hashrate")]
    #[serde(rename = "report-hashrate")]
    pub report_hashrate: bool,

    //#[serde(default = "default_log_level")]
    #[serde(rename = "log-level")]
    #[serde(flatten)]
    pub log_level: LogLevel,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct State {
    pub path: std::path::PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Argon2Impl {
    #[serde(rename = "avx2")]
    AVX2,
    #[serde(rename = "ssse3")]
    SSSE3,
    #[serde(rename = "default")]
    Default,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LogLevel {
    #[serde(rename = "off")]
    Off,

    #[serde(rename = "error")]
    Error,

    #[serde(rename = "warn")]
    Warn,

    #[serde(rename = "info")]
    Info,

    #[serde(rename = "debug")]
    Debug,

    #[serde(rename = "trace")]
    Trace,
}

impl UnresolvedCCPConfig {
    pub fn resolve(self) -> eyre::Result<CCPConfig> {
        let http_server = self.http_server.resolve();
        let optimization = self.optimizations.resolve()?;
        let logs = self.logs.resolve();

        let config = CCPConfig {
            http_server,
            optimizations: optimization,
            logs,
            state_dir: self.state.path,
        };
        Ok(config)
    }
}

impl UnresolvedHTTPServer {
    pub fn resolve(self) -> HTTPServer {
        let mut utility_thread_ids = self.utility_thread_ids;
        if utility_thread_ids.is_empty() {
            utility_thread_ids.push(DEFAULT_UTILITY_THREAD_ID);
        }

        let utility_thread_ids = utility_thread_ids
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();

        HTTPServer {
            host: self.host,
            port: self.port,
            utility_cores_ids: utility_thread_ids,
        }
    }
}

impl UnresolvedOptimizations {
    pub fn resolve(self) -> eyre::Result<Optimizations> {
        let randomx_flags = self.randomx.resolve();
        let msr_enabled = self.msr_enabled;
        let threads_per_core_policy = match self.threads_per_core_policy {
            Some(threads_count) => ThreadsPerCoreAllocationPolicy::Exact {
                threads_per_physical_core: threads_count.try_into()?,
            },
            None => ThreadsPerCoreAllocationPolicy::Optimal,
        };

        let opt = Optimizations {
            randomx_flags,
            msr_enabled,
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

impl UnresolvedLogs {
    pub fn resolve(self) -> Logs {
        let log_level = match self.log_level {
            LogLevel::Off => log::LevelFilter::Off,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        };

        Logs {
            report_hashrate: self.report_hashrate,
            log_level,
        }
    }
}
