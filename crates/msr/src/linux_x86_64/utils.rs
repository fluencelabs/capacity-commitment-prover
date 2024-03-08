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

use std::fs::File;
use std::io;

use ccp_shared::types::LogicalCoreId;

use super::errors::MSRError;
use crate::MSRResult;

enum MSRFileOpMode {
    MSRRead,
    MSRWrite,
}

pub(crate) fn read_msr(register_id: u32, core_id: LogicalCoreId) -> MSRResult<u64> {
    use nix::sys::uio::pread;

    let file = open_msr(core_id, MSRFileOpMode::MSRRead)
        .map_err(|error| MSRError::open_for_read(core_id, error))?;

    let mut value = [0u8; 8];
    pread(file, &mut value, register_id as i64)
        .map_err(|errno| MSRError::read_w_no_err(register_id, core_id, errno))?;
    let result = u64::from_le_bytes(value);

    // TODO: print value as hex
    tracing::debug!("Read MSR register_id {register_id} value {value:?} at core id {core_id}");

    Ok(result)
}

pub(crate) fn write_msr(register_id: u32, value: u64, core_id: LogicalCoreId) -> MSRResult<()> {
    use nix::sys::uio::pwrite;

    let file = open_msr(core_id, MSRFileOpMode::MSRWrite)
        .map_err(|error| MSRError::open_for_write(core_id, error))?;

    let value_as_bytes = value.to_le_bytes();
    pwrite(file, &value_as_bytes, register_id as i64)
        .map_err(|errno| MSRError::write_w_no_err(value, register_id, core_id, errno))?;

    Ok(())
}

fn open_msr(core_id: LogicalCoreId, mode: MSRFileOpMode) -> io::Result<File> {
    use std::fs::OpenOptions;

    let path = format!("/dev/cpu/{}/msr", core_id);
    match mode {
        MSRFileOpMode::MSRRead => OpenOptions::new().read(true).open(path),
        MSRFileOpMode::MSRWrite => OpenOptions::new().write(true).open(path),
    }
}
