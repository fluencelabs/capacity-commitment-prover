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
mod msr_item;

pub use msr_item::MsrItem;

use nix::sys::uio::pread;
use nix::sys::uio::pwrite;
use once_cell::sync::Lazy;
use std::fs::{File, OpenOptions};
use std::io::{self};

#[allow(dead_code)]
#[derive(Debug)]
pub struct CCPCpuPreset {
    items: Vec<MsrItem>,
}

impl CCPCpuPreset {
    pub fn new(items: Vec<MsrItem>) -> CCPCpuPreset {
        Self { items }
    }
}

static CCP_CPU_MSR_PRESETS: Lazy<Vec<CCPCpuPreset>> = Lazy::new(|| {
    vec![
        CCPCpuPreset::new(vec![]),
        CCPCpuPreset::new(vec![
            MsrItem::new(0xC0011020, 0),
            MsrItem::with_mask(0xC0011021, 0x40, !0x20),
            MsrItem::new(0xC0011022, 0x1510000),
            MsrItem::new(0xC001102b, 0x2000cc16),
        ]),
        CCPCpuPreset::new(vec![
            MsrItem::new(0xC0011020, 0x0004480000000000),
            MsrItem::with_mask(0xC0011021, 0x001c000200000040, !0x20),
            MsrItem::new(0xC0011022, 0xc000000401570000),
            MsrItem::new(0xC001102b, 0x2000cc10),
        ]),
        CCPCpuPreset::new(vec![
            MsrItem::new(0xC0011020, 0x0004400000000000),
            MsrItem::with_mask(0xC0011021, 0x0004000000000040, !0x20),
            MsrItem::new(0xC0011022, 0x8680000401570000),
            MsrItem::new(0xC001102b, 0x2040cc10),
        ]),
        CCPCpuPreset::new(vec![MsrItem::new(0x1a4, 0xf)]),
        CCPCpuPreset::new(vec![]),
    ]
});

pub enum CCPMsrMode {
    MsrModNone,
    MsrModRyzen17h,
    MsrModRyzen19h,
    MsrModRyzen19hZen4,
    MsrModIntel,
    MsrModCustom,
    MsrModMax,
}

pub fn get_cpu_preset(mode: CCPMsrMode) -> &'static CCPCpuPreset {
    match mode {
        CCPMsrMode::MsrModNone => &CCP_CPU_MSR_PRESETS[0],
        CCPMsrMode::MsrModRyzen17h => &CCP_CPU_MSR_PRESETS[1],
        CCPMsrMode::MsrModRyzen19h => &CCP_CPU_MSR_PRESETS[2],
        CCPMsrMode::MsrModRyzen19hZen4 => &CCP_CPU_MSR_PRESETS[3],
        CCPMsrMode::MsrModIntel => &CCP_CPU_MSR_PRESETS[4],
        CCPMsrMode::MsrModCustom => &CCP_CPU_MSR_PRESETS[5],
        CCPMsrMode::MsrModMax => &CCP_CPU_MSR_PRESETS[6],
    }
}

#[derive(Debug)]
pub enum MSRError {
    ReadError,
    WriteError,
}

enum MsrFileOpMode {
    MsrRead,
    MsrWrite,
}

type MSRResult<T> = Result<T, MSRError>;

pub trait CCPMsr {
    // fn read(&self, reg: u32, cpu: i32, verbose: bool) -> MSRResult<MsrItem>;
    // fn write(&self, reg: u32, item: MsrItem, cpu: i32, mask: u64, verbose: bool) -> MSRResult<()>;
    fn write_preset(
        &mut self,
        preset: &CCPCpuPreset,
        save_state: bool,
        verbose: bool,
    ) -> MSRResult<()>;
    fn restore(&mut self, verbose: bool) -> MSRResult<()>;
}

fn msr_open(cpu: i32, mode: MsrFileOpMode) -> io::Result<File> {
    let cpu = if cpu < 0 { 0 } else { cpu };
    let path = format!("/dev/cpu/{}/msr", cpu);
    match mode {
        MsrFileOpMode::MsrRead => OpenOptions::new().read(true).open(path),
        MsrFileOpMode::MsrWrite => OpenOptions::new().write(true).open(path),
    }
}

pub struct CCPMsrLinux {
    saved_state: Vec<MsrItem>,
}

impl CCPMsrLinux {
    pub fn new() -> Self {
        Self {
            saved_state: vec![],
        }
    }

    fn rdmsr(&self, reg: u32, cpu: i32) -> MSRResult<u64> {
        let file = msr_open(cpu, MsrFileOpMode::MsrRead).map_err(|_| MSRError::ReadError)?;
        let mut value = [0u8; 8];
        // WIP check for 0 bytes read
        let bytes_read = pread(file, &mut value, reg as i64).map_err(|_| MSRError::ReadError);
        let result = u64::from_le_bytes(value); // WIP endianess ???
        Ok(result)
    }

    fn wrmsr(&self, reg: u32, value: u64, cpu: i32) -> MSRResult<()> {
        let file = msr_open(cpu, MsrFileOpMode::MsrWrite).map_err(|_| MSRError::WriteError)?;
        let value = value.to_le_bytes(); // WIP endianess ???

        // WIP check for 0 bytes read
        let bytes_writen = pwrite(file, &value, reg as i64).map_err(|_| MSRError::ReadError);
        Ok(())
    }

    // WIP change vis
    pub fn read(&self, reg: u32, cpu: i32, verbose: bool) -> MSRResult<MsrItem> {
        let value = self.rdmsr(reg, cpu)?;
        Ok(MsrItem::new(reg, value))
    }

    fn write(&self, item: MsrItem, cpu: i32, verbose: bool) -> MSRResult<()> {
        let value_to_write = if item.mask() != MsrItem::NO_MASK {
            let old_value = self.rdmsr(item.reg(), cpu)?;
            MsrItem::masked_value(old_value, item.value(), item.mask())
        } else {
            item.value()
        };
        self.wrmsr(item.reg(), value_to_write, cpu)
    }
}

impl CCPMsr for CCPMsrLinux {
    fn write_preset(
        &mut self,
        preset: &CCPCpuPreset,
        save_state: bool,
        verbose: bool,
    ) -> MSRResult<()> {
        for item in &preset.items {
            if save_state && item.is_valid() {
                let current_item_value = self.read(item.reg(), 0, verbose)?;
                self.saved_state.push(current_item_value);
            }
        }

        for item in &preset.items {
            self.write(*item, 0, verbose)?;
        }

        Ok(())
    }

    fn restore(&mut self, verbose: bool) -> MSRResult<()> {
        for item in &self.saved_state {
            self.write(*item, 0, verbose)?;
        }
        self.saved_state.clear();
        Ok(())
    }
}
