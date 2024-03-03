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

use std::path::PathBuf;

use ccp_randomx::RandomXFlags;

use crate::config_loader::load_config;
use crate::CCPConfig;
use crate::HTTPServer;
use crate::Logs;
use crate::Optimizations;
use crate::ThreadsPerCoreAllocationPolicy;

#[test]
fn parse_basic_config() {
    let mut manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_path.push("src/tests/default.toml");

    let actual_config = load_config(manifest_path.as_os_str().to_str().unwrap()).unwrap();

    let http_server = HTTPServer {
        host: "127.0.0.1".to_string(),
        port: 9383,
        utility_cores_ids: vec![1.into(), 2.into()],
    };

    let mut randomx_flags = RandomXFlags::default();
    randomx_flags.set(RandomXFlags::HARD_AES, true);
    randomx_flags.set(RandomXFlags::FULL_MEM, true);
    randomx_flags.set(RandomXFlags::FLAG_JIT, true);
    randomx_flags.set(RandomXFlags::FLAG_SECURE, true);
    randomx_flags.set(RandomXFlags::FLAG_ARGON2, true);

    let optimizations = Optimizations {
        randomx_flags,
        threads_per_core_policy: ThreadsPerCoreAllocationPolicy::Exact {
            threads_per_physical_core: 2.try_into().unwrap(),
        },
        msr_enabled: true,
    };
    let logs = Logs {
        report_hashrate: true,
        log_level: log::LevelFilter::Warn,
    };
    let expected_config = CCPConfig {
        http_server,
        optimizations,
        logs,
        state_dir: "../test".into(),
    };

    assert_eq!(actual_config, expected_config);
}
