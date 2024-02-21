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

use crate::msr_mode::MsrMode;
use crate::MsrItem;

use once_cell::sync::Lazy;

#[allow(dead_code)]
#[derive(Debug)]
pub struct CCPCpuPreset {
    items: Vec<MsrItem>,
}

impl CCPCpuPreset {
    pub fn new(items: Vec<MsrItem>) -> CCPCpuPreset {
        Self { items }
    }

    pub(crate) fn get_items(&self) -> &Vec<MsrItem> {
        &self.items
    }
}

static CCP_CPU_MSR_PRESETS: Lazy<Vec<CCPCpuPreset>> = Lazy::new(|| {
    vec![
        CCPCpuPreset::new(vec![]),
        CCPCpuPreset::new(vec![
            MsrItem::new(0xc0011020, 0),
            MsrItem::with_mask(0xc0011021, 0x40, !0x20),
            MsrItem::new(0xc0011022, 0x1510000),
            MsrItem::new(0xc001102b, 0x2000cc16),
        ]),
        CCPCpuPreset::new(vec![
            MsrItem::new(0xc0011020, 0x0004480000000000),
            MsrItem::with_mask(0xc0011021, 0x001c000200000040, !0x20),
            MsrItem::new(0xc0011022, 0xc000000401570000),
            MsrItem::new(0xc001102b, 0x2000cc10),
        ]),
        CCPCpuPreset::new(vec![
            MsrItem::new(0xc0011020, 0x0004400000000000),
            MsrItem::with_mask(0xc0011021, 0x0004000000000040, !0x20),
            MsrItem::new(0xc0011022, 0x8680000401570000),
            MsrItem::new(0xc001102b, 0x2040cc10),
        ]),
        CCPCpuPreset::new(vec![MsrItem::new(0x1a4, 0xf)]),
        CCPCpuPreset::new(vec![]),
    ]
});

pub fn get_cpu_preset(mode: MsrMode) -> &'static CCPCpuPreset {
    match mode {
        MsrMode::MsrModNone => &CCP_CPU_MSR_PRESETS[0],
        MsrMode::MsrModRyzen17h => &CCP_CPU_MSR_PRESETS[1],
        MsrMode::MsrModRyzen19h => &CCP_CPU_MSR_PRESETS[2],
        MsrMode::MsrModRyzen19hZen4 => &CCP_CPU_MSR_PRESETS[3],
        MsrMode::MsrModIntel => &CCP_CPU_MSR_PRESETS[4],
        MsrMode::MsrModCustom => &CCP_CPU_MSR_PRESETS[5],
        MsrMode::MsrModMax => &CCP_CPU_MSR_PRESETS[6],
    }
}
