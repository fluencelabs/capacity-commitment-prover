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

use once_cell::sync::Lazy;

use crate::msr_mode::MSRMode;
use crate::MSRItem;

pub struct CCPCpuPreset {
    items: Vec<MSRItem>,
}

impl CCPCpuPreset {
    pub fn new(items: Vec<MSRItem>) -> CCPCpuPreset {
        Self { items }
    }

    pub(crate) fn get_items(&self) -> &Vec<MSRItem> {
        &self.items
    }
}

static CCP_CPU_MSR_PRESETS: Lazy<Vec<CCPCpuPreset>> = Lazy::new(|| {
    vec![
        CCPCpuPreset::new(vec![]),
        CCPCpuPreset::new(vec![
            MSRItem::new(0xc0011020, 0),
            MSRItem::with_mask(0xc0011021, 0x40, !0x20),
            MSRItem::new(0xc0011022, 0x1510000),
            MSRItem::new(0xc001102b, 0x2000cc16),
        ]),
        CCPCpuPreset::new(vec![
            MSRItem::new(0xc0011020, 0x0004480000000000),
            MSRItem::with_mask(0xc0011021, 0x001c000200000040, !0x20),
            MSRItem::new(0xc0011022, 0xc000000401570000),
            MSRItem::new(0xc001102b, 0x2000cc10),
        ]),
        CCPCpuPreset::new(vec![
            MSRItem::new(0xc0011020, 0x0004400000000000),
            MSRItem::with_mask(0xc0011021, 0x0004000000000040, !0x20),
            MSRItem::new(0xc0011022, 0x8680000401570000),
            MSRItem::new(0xc001102b, 0x2040cc10),
        ]),
        CCPCpuPreset::new(vec![MSRItem::new(0x1a4, 0xf)]),
        CCPCpuPreset::new(vec![]),
    ]
});

pub fn get_cpu_preset(mode: MSRMode) -> &'static CCPCpuPreset {
    match mode {
        MSRMode::MSRModNone => &CCP_CPU_MSR_PRESETS[0],
        MSRMode::MSRModRyzen17h => &CCP_CPU_MSR_PRESETS[1],
        MSRMode::MSRModRyzen19h => &CCP_CPU_MSR_PRESETS[2],
        MSRMode::MSRModRyzen19hZen4 => &CCP_CPU_MSR_PRESETS[3],
        MSRMode::MSRModIntel => &CCP_CPU_MSR_PRESETS[4],
    }
}
