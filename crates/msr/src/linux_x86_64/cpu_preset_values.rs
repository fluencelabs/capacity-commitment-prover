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

use crate::state::MSRCpuPreset;
use crate::state::MSRPresetItem;

/// This is a set of MSR items that are used to disable CPU cache for
/// a variety of CPU models.
pub(crate) static CPU_MSR_PRESETS: Lazy<Vec<MSRCpuPreset>> = Lazy::new(|| {
    vec![
        // No-op
        MSRCpuPreset::new(vec![]),
        // ModRyzen17h
        MSRCpuPreset::new(vec![
            MSRPresetItem::new(0xc0011020, 0),
            MSRPresetItem::with_mask(0xc0011021, 0x40, !0x20),
            MSRPresetItem::new(0xc0011022, 0x1510000),
            MSRPresetItem::new(0xc001102b, 0x2000cc16),
        ]),
        // ModRyzen19h
        MSRCpuPreset::new(vec![
            MSRPresetItem::new(0xc0011020, 0x0004480000000000),
            MSRPresetItem::with_mask(0xc0011021, 0x001c000200000040, !0x20),
            MSRPresetItem::new(0xc0011022, 0xc000000401570000),
            MSRPresetItem::new(0xc001102b, 0x2000cc10),
        ]),
        // Ryzen19hZen4
        MSRCpuPreset::new(vec![
            MSRPresetItem::new(0xc0011020, 0x0004400000000000),
            MSRPresetItem::with_mask(0xc0011021, 0x0004000000000040, !0x20),
            MSRPresetItem::new(0xc0011022, 0x8680000401570000),
            MSRPresetItem::new(0xc001102b, 0x2040cc10),
        ]),
        // Intel
        MSRCpuPreset::new(vec![MSRPresetItem::new(0x1a4, 0xf)]),
    ]
});
