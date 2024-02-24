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
    #[error("opening MSR file '/dev/cpu/{core_id:}/msr' for read there is an error: {io_error:?}")]
    OpenForRead {
        core_id: LogicalCoreId,
        io_error: std::io::Error,
    },

    #[error(
        "opening MSR file '/dev/cpu/{core_id:}/msr' for write there is an error: {io_error:?}"
    )]
    OpenForWrite {
        core_id: LogicalCoreId,
        io_error: std::io::Error,
    },

    #[error(
        "reading from register_id {register_id:} and MSR file '/dev/cpu/{core_id:}/msr' there is an error: {errno:?}"
    )]
    ReadWNoErr {
        register_id: u32,
        core_id: LogicalCoreId,
        errno: Errno,
    },

    #[error(
            "writing value {value:} for register_id {register_id:} into MSR file '/dev/cpu/{core_id:}/msr' there is an error: {errno:?}"j
        )]
    WriteWNoErr {
        value: u64,
        register_id: u32,
        core_id: LogicalCoreId,
        errno: Errno,
    },
}

#[cfg(target_os = "linux")]
impl MSRError {
    pub(crate) fn open_for_read(core_id: LogicalCoreId, io_error: std::io::Error) -> Self {
        Self::OpenForRead { core_id, io_error }
    }

    pub(crate) fn open_for_write(core_id: LogicalCoreId, io_error: std::io::Error) -> Self {
        Self::OpenForWrite { core_id, io_error }
    }

    pub(crate) fn read_w_no_err(register_id: u32, core_id: LogicalCoreId, errno: Errno) -> Self {
        Self::ReadWNoErr {
            register_id,
            core_id,
            errno,
        }
    }

    pub(crate) fn write_w_no_err(
        value: u64,
        register_id: u32,
        core_id: LogicalCoreId,
        errno: Errno,
    ) -> Self {
        Self::WriteWNoErr {
            value,
            register_id,
            core_id,
            errno,
        }
    }
}
