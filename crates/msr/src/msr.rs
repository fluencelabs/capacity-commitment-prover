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

use nix::sys::uio::pread;
use nix::sys::uio::pwrite;
use std::fs::{File, OpenOptions};
use std::io;

use crate::cpu_preset::get_cpu_preset;
use crate::msr_item::MsrItem;
use crate::msr_mode::MSR_MODE;
use crate::MSRError;
use cpu_utils::LogicalCoreId;

enum MsrFileOpMode {
    MsrRead,
    MsrWrite,
}

type MSRResult<T> = Result<T, MSRError>;

pub trait Msr {
    fn write_preset(&mut self, store_state: bool) -> MSRResult<()>;
    fn repin(&mut self, core_id: LogicalCoreId) -> MSRResult<()>;
    fn restore(self) -> MSRResult<()>;
}

fn msr_open(core_id: LogicalCoreId, mode: MsrFileOpMode) -> io::Result<File> {
    let path = format!("/dev/cpu/{}/msr", core_id);
    match mode {
        MsrFileOpMode::MsrRead => OpenOptions::new().read(true).open(path),
        MsrFileOpMode::MsrWrite => OpenOptions::new().write(true).open(path),
    }
}

#[derive(Debug)]
pub struct MsrLinux {
    is_enabled: bool,
    stored_state: Vec<MsrItem>,
    core_id: LogicalCoreId,
}

impl MsrLinux {
    pub fn new(is_enabled: bool, core_id: LogicalCoreId) -> Self {
        Self {
            is_enabled,
            stored_state: vec![],
            core_id,
        }
    }

    fn rdmsr(&self, reg: u32, core_id: LogicalCoreId) -> MSRResult<u64> {
        let file = msr_open(core_id, MsrFileOpMode::MsrRead)
            .map_err(|error| MSRError::OpenForRead(core_id, error))?;
        let mut value = [0u8; 8];

        pread(file, &mut value, reg as i64)
            .map_err(|errno| MSRError::ReadWNoErr(reg, core_id, errno))?;

        let result = u64::from_le_bytes(value);

        Ok(result)
    }

    fn wrmsr(&self, reg: u32, value: u64, core_id: LogicalCoreId) -> MSRResult<()> {
        let file = msr_open(core_id, MsrFileOpMode::MsrWrite)
            .map_err(|error| MSRError::OpenForWrite(core_id, error))?;
        let value_as_bytes = value.to_le_bytes();

        pwrite(file, &value_as_bytes, reg as i64)
            .map_err(|errno| MSRError::WriteWNoErr(value, reg, core_id, errno))?;

        Ok(())
    }

    fn read(&self, reg: u32, core_id: LogicalCoreId) -> MSRResult<MsrItem> {
        let value = self.rdmsr(reg, core_id)?;
        Ok(MsrItem::new(reg, value))
    }

    fn write(&self, item: MsrItem, core_id: LogicalCoreId) -> MSRResult<()> {
        let value_to_write = if item.mask() != MsrItem::NO_MASK {
            let old_value = self.rdmsr(item.reg(), core_id)?;
            MsrItem::masked_value(old_value, item.value(), item.mask())
        } else {
            item.value()
        };
        tracing::debug!(
            "Write MSR reg {:?} value {:} at logical CPU {}  ",
            item.reg(),
            value_to_write,
            core_id
        );

        self.wrmsr(item.reg(), value_to_write, core_id)
    }
}

impl Msr for MsrLinux {
    fn write_preset(&mut self, store_state: bool) -> MSRResult<()> {
        if !self.is_enabled {
            tracing::debug!("MSR is disabled.");
            return Ok(());
        }

        let mode = *MSR_MODE;
        let preset = get_cpu_preset(mode);
        tracing::debug!("MSR mode: mode:{:?}.", mode);

        for item in preset.get_items() {
            if store_state && item.is_valid() {
                // TODO Check for errors here and clean the stored state
                let current_item_value = self.read(item.reg(), self.core_id)?;
                self.stored_state.push(current_item_value);
                tracing::debug!("Stored MSR item :{:?}.", current_item_value);
            }
        }

        for item in preset.get_items() {
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

        for item in self.stored_state.iter().filter(|item| item.is_valid()) {
            self.write(*item, self.core_id)?;
        }
        self.stored_state.clear();

        self.core_id = core_id;
        self.write_preset(true)
    }

    fn restore(self) -> MSRResult<()> {
        if !self.is_enabled {
            tracing::debug!("MSR is disabled.");
            return Ok(());
        }

        tracing::debug!("Restore MSR state.");

        for item in self.stored_state.iter().filter(|item| item.is_valid()) {
            self.write(*item, self.core_id)?;
        }
        Ok(())
    }
}
