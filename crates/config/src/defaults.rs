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

use crate::unresolved_config::LogLevel;

const DEFAULT_HASHES_PER_ROUND: usize = 1024;
const DEFAULT_ASYNC_TO_SYNC_QUEUE_SIZE: usize = 1;
const DEFAULT_SYNC_TO_ASYNC_QUEUE_SIZE: usize = 1;

const DEFAULT_UTILITY_QUEUE_SIZE: usize = 100;
const DEFAULT_FACADE_QUEUE_SIZE: usize = 100;

pub(crate) fn default_log_level() -> LogLevel {
    LogLevel::Error
}

pub(crate) fn default_report_hashrate() -> bool {
    false
}

pub(crate) fn default_state_path() -> PathBuf {
    PathBuf::from("./state")
}

pub(crate) fn default_msr_enabled() -> bool {
    false
}

pub(crate) fn default_hashes_per_round() -> usize {
    DEFAULT_HASHES_PER_ROUND
}

pub(crate) fn default_async_to_sync_queue_size() -> usize {
    DEFAULT_ASYNC_TO_SYNC_QUEUE_SIZE
}

pub(crate) fn default_sync_to_async_queue_size() -> usize {
    DEFAULT_SYNC_TO_ASYNC_QUEUE_SIZE
}

pub(crate) fn default_utility_queue_size() -> usize {
    DEFAULT_UTILITY_QUEUE_SIZE
}

pub(crate) fn default_facade_queue_size() -> usize {
    DEFAULT_FACADE_QUEUE_SIZE
}
