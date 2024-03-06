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

use super::msr_mode::MSRMode;
use crate::state::MSRCpuPreset;
use crate::state::MSRPresetItem;
use crate::MSREnforce;
use crate::MSRError;
use crate::MSRResult;

const DEFAULT_CORE_TO_READ_MSR: LogicalCoreId = LogicalCoreId::from(0);

enum MSRFileOpMode {
    MSRRead,
    MSRWrite,
}

#[derive(Clone, Debug)]
pub struct MSRModeEnforcer {
    is_enabled: bool,
    original_preset: MSRCpuPreset,
}

impl MSRModeEnforcer {
    pub fn from_os(is_enabled: bool) -> Self {
        let mode = MSRMode::detect();
        let preset = mode.get_optimal_cpu_preset();

        let original_items = preset
            .items()
            .filter_map(|item| {
                let register_id = item.register_id();
                let value = read_msr(register_id, DEFAULT_CORE_TO_READ_MSR).ok()?;
                Some(MSRPresetItem::new(register_id, value))
            })
            .collect();
        let original_preset = MSRCpuPreset::new(original_items);

        Self {
            is_enabled,
            original_preset,
        }
    }

    pub fn from_preset(is_enabled: bool, original_preset: MSRCpuPreset) -> Self {
        Self {
            is_enabled,
            original_preset,
        }
    }
}

pub fn write(item: MSRPresetItem, core_id: LogicalCoreId) -> MSRResult<()> {
    let value_to_write = if item.mask() != MSRPresetItem::NO_MASK {
        let old_value = read_msr(item.register_id(), core_id)?;
        MSRPresetItem::masked_value(old_value, item.value(), item.mask())
    } else {
        item.value()
    };

    tracing::debug!(
            "Write MSR register_id {:?} value {:} at logical CPU {}  ",
            item.register_id(),
            value_to_write,
            core_id
        );
    write_msr(item.register_id(), value_to_write, core_id)
}

fn read_msr(register_id: u32, core_id: LogicalCoreId) -> MSRResult<u64> {
    use nix::sys::uio::pread;

    let file = open_msr(core_id, MSRFileOpMode::MSRRead)
        .map_err(|error| MSRError::open_for_read(core_id, error))?;

    let mut value = [0u8; 8];
    pread(file, &mut value, register_id as i64)
        .map_err(|errno| MSRError::read_w_no_err(register_id, core_id, errno))?;
    let result = u64::from_le_bytes(value);

    tracing::debug!(
            "Read MSR register_id {register_id:?} value {value:} at core id {core_id}",
        );

    Ok(result)
}

fn write_msr(register_id: u32, value: u64, core_id: LogicalCoreId) -> MSRResult<()> {
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

impl MSREnforce for MSRModeEnforcer {
    fn enforce(&mut self, core_id: LogicalCoreId) -> MSRResult<()> {
        use super::msr_mode::MSR_MODE;

        if !self.is_enabled {
            return Ok(());
        }

        let mode = *MSR_MODE;
        tracing::debug!("Global MSR mode: mode:{:?}.", mode);

        let preset = mode.get_optimal_cpu_preset();
        for item in preset.items() {
            // TODO Check for errors here and rollback/clean the stored state
            write(*item, self.core_id)?;
        }

        Ok(())
    }

    fn cease(&self, core_id: LogicalCoreId) -> MSRResult<()> {
        if !self.is_enabled {
            return Ok(());
        }

        for item in self.original_preset.items() {
            write(*item, self.core_id)?;
        }
        Ok(())
    }
}
