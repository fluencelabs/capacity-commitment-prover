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

use nix::errno::Errno;
use thiserror::Error as ThisError;

use cpu_utils::LogicalCoreId;

#[derive(ThisError, Debug)]
pub enum MSRError {
    #[error("opening MSR file '/dev/cpu/{0:}/msr' for read there is an error: {1:?}")]
    OpenForRead(LogicalCoreId, std::io::Error),

    #[error("opening MSR file '/dev/cpu/{0:}/msr' for write there is an error: {1:?}")]
    OpenForWrite(LogicalCoreId, std::io::Error),

    #[error("reading from reg {0:} and MSR file '/dev/cpu/{1:}/msr' there is an error: {2:?}")]
    ReadWNoErr(u32, LogicalCoreId, Errno),

    #[error(
            "writing value {0:} for reg {1:} into MSR file '/dev/cpu/{3:}/msr' there is an error: {3:?}"
        )]
    WriteWNoErr(u64, u32, LogicalCoreId, Errno),
}
