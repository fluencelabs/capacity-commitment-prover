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

use crate::cpu_preset::get_cpu_preset;
use crate::msr_item::MsrItem;
use crate::msr_mode::MSR_MODE;
use cpu_utils::LogicalCoreId;

use nix::sys::uio::pread;
use nix::sys::uio::pwrite;
use std::fs::{File, OpenOptions};
use std::io::{self};

#[derive(Debug)]
pub enum MSRError {
    ReadError,
    WriteError,
}

pub(crate) enum MsrFileOpMode {
    MsrRead,
    MsrWrite,
}

type MSRResult<T> = Result<T, MSRError>;

pub trait Msr {
    fn write_preset(&mut self, store_state: bool) -> MSRResult<()>;
    fn restore(self) -> MSRResult<()>;
}

fn msr_open(core_id: LogicalCoreId, mode: MsrFileOpMode) -> io::Result<File> {
    let path = format!("/dev/cpu/{}/msr", core_id);
    match mode {
        MsrFileOpMode::MsrRead => OpenOptions::new().read(true).open(path),
        MsrFileOpMode::MsrWrite => OpenOptions::new().write(true).open(path),
    }
}

#[derive(Debug, Clone)]
pub struct MsrLinux {
    stored_state: Vec<MsrItem>,
    core_id: LogicalCoreId,
}

impl MsrLinux {
    pub fn new(core_id: LogicalCoreId) -> Self {
        Self {
            stored_state: vec![],
            core_id,
        }
    }

    fn rdmsr(&self, reg: u32, core_id: LogicalCoreId) -> MSRResult<u64> {
        let file = msr_open(core_id, MsrFileOpMode::MsrRead).map_err(|_| MSRError::ReadError)?;
        let mut value = [0u8; 8];
        // WIP check for 0 bytes read
        let _ = pread(file, &mut value, reg as i64).map_err(|_| MSRError::ReadError)?;
        let result = u64::from_le_bytes(value); // WIP endianess ???
        Ok(result)
    }

    fn wrmsr(&self, reg: u32, value: u64, core_id: LogicalCoreId) -> MSRResult<()> {
        let file = msr_open(core_id, MsrFileOpMode::MsrWrite).map_err(|_| MSRError::WriteError)?;
        let value = value.to_le_bytes(); // WIP endianess ???

        // WIP check for 0 bytes read
        let _ = pwrite(file, &value, reg as i64).map_err(|_| MSRError::ReadError)?;
        Ok(())
    }

    // WIP change vis
    pub fn read(&self, reg: u32, core_id: LogicalCoreId) -> MSRResult<MsrItem> {
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
        let mode = *MSR_MODE;
        let preset = get_cpu_preset(mode);
        tracing::debug!("MSR mode: mode:{:?}.", mode);

        for item in preset.get_items() {
            if store_state && item.is_valid() {
                // Check for errors here and clean the stored state
                let current_item_value = self.read(item.reg(), self.core_id)?;
                self.stored_state.push(current_item_value);
                tracing::debug!("Stored MSR item :{:?}.", current_item_value);
            }
        }

        for item in preset.get_items() {
            // Check for errors here and rollback/clean the stored state
            self.write(*item, self.core_id)?;
        }

        Ok(())
    }

    fn restore(self) -> MSRResult<()> {
        tracing::debug!("Restore MSR state.");

        for item in &self.stored_state {
            self.write(*item, self.core_id)?;
        }
        Ok(())
    }
}
