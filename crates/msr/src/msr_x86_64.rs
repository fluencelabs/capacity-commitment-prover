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

use crate::msr_cpu_preset::get_cpu_preset;
use crate::msr_cpu_preset::MSRCpuPreset;
use crate::msr_item::MSRItem;
use crate::msr_mode::MSR_MODE;
use crate::MSRConfig;
use crate::MSRError;
use crate::MSRResult;
use crate::MSR;

use cpu_utils::LogicalCoreId;

enum MSRFileOpMode {
    MSRRead,
    MSRWrite,
}

fn msr_open(core_id: LogicalCoreId, mode: MSRFileOpMode) -> io::Result<File> {
    use std::fs::OpenOptions;
    let path = format!("/dev/cpu/{}/msr", core_id);
    match mode {
        MSRFileOpMode::MSRRead => OpenOptions::new().read(true).open(path),
        MSRFileOpMode::MSRWrite => OpenOptions::new().write(true).open(path),
    }
}

#[derive(Debug)]
pub struct MSRImpl {
    is_enabled: bool,
    original_preset: MSRCpuPreset,
    core_id: LogicalCoreId,
}

impl MSRImpl {
    pub fn new(msr_config: MSRConfig, core_id: LogicalCoreId) -> Self {
        use crate::msr_cpu_preset_x86_64::CPU_MSR_ORIGINAL_PRESET;

        let is_enabled = msr_config.msr_enabled;
        let original_preset = if msr_config.original_msr_preset.is_empty() {
            CPU_MSR_ORIGINAL_PRESET.clone()
        } else {
            msr_config.original_msr_preset
        };
        Self {
            is_enabled,
            original_preset,
            core_id,
        }
    }

    fn rdmsr(&self, register_id: u32, core_id: LogicalCoreId) -> MSRResult<u64> {
        use nix::sys::uio::pread;

        let file = msr_open(core_id, MSRFileOpMode::MSRRead)
            .map_err(|error| MSRError::open_for_read(core_id, error))?;
        let mut value = [0u8; 8];

        pread(file, &mut value, register_id as i64)
            .map_err(|errno| MSRError::read_w_no_err(register_id, core_id, errno))?;

        let result = u64::from_le_bytes(value);

        Ok(result)
    }

    fn wrmsr(&self, register_id: u32, value: u64, core_id: LogicalCoreId) -> MSRResult<()> {
        use nix::sys::uio::pwrite;

        let file = msr_open(core_id, MSRFileOpMode::MSRWrite)
            .map_err(|error| MSRError::open_for_write(core_id, error))?;
        let value_as_bytes = value.to_le_bytes();

        pwrite(file, &value_as_bytes, register_id as i64)
            .map_err(|errno| MSRError::write_w_no_err(value, register_id, core_id, errno))?;

        Ok(())
    }

    pub fn read(&self, register_id: u32, core_id: LogicalCoreId) -> MSRResult<MSRItem> {
        let value = self.rdmsr(register_id, core_id)?;
        Ok(MSRItem::new(register_id, value))
    }

    pub fn write(&self, item: MSRItem, core_id: LogicalCoreId) -> MSRResult<()> {
        let value_to_write = if item.mask() != MSRItem::NO_MASK {
            let old_value = self.rdmsr(item.register_id(), core_id)?;
            MSRItem::masked_value(old_value, item.value(), item.mask())
        } else {
            item.value()
        };
        tracing::debug!(
            "Write MSR register_id {:?} value {:} at logical CPU {}  ",
            item.register_id(),
            value_to_write,
            core_id
        );

        self.wrmsr(item.register_id(), value_to_write, core_id)
    }
}

impl MSR for MSRImpl {
    fn write_preset(&mut self) -> MSRResult<()> {
        if !self.is_enabled {
            tracing::debug!("MSR is disabled.");
            return Ok(());
        }

        let mode = *MSR_MODE;
        let preset = get_cpu_preset(mode);
        tracing::debug!("MSR mode: mode:{:?}.", mode);

        for item in preset.get_valid_items() {
            // TODO Check for errors here and rollback/clean the stored state
            self.write(*item, self.core_id)?;
        }

        Ok(())
    }

    fn repin(&mut self, core_id: LogicalCoreId) -> MSRResult<()> {
        if !self.is_enabled {
            tracing::debug!("MSR is disabled.");
            return Ok(());
        }

        for item in self.original_preset.get_valid_items() {
            self.write(*item, self.core_id)?;
        }

        self.core_id = core_id;
        self.write_preset()
    }

    fn restore(self) -> MSRResult<()> {
        if !self.is_enabled {
            tracing::debug!("MSR is disabled.");
            return Ok(());
        }

        tracing::debug!("Restore MSR state.");

        for item in self.original_preset.get_valid_items() {
            self.write(*item, self.core_id)?;
        }
        Ok(())
    }
}
