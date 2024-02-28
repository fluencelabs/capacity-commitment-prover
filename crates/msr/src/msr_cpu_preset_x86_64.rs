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

use crate::msr_cpu_preset::MSRCpuPreset;

use once_cell::sync::Lazy;

/// This global is used with Linux x86_64 only to look for the original
/// MSR state only once.
static CPU_MSR_ORIGINAL_PRESET: Lazy<MSRCpuPreset> = Lazy::new(|| {
    use crate::msr_cpu_preset::get_cpu_preset;
    use crate::msr_item::MSRItem;
    use crate::msr_mode::detect_msr_mode;
    use crate::MSRConfig;
    use crate::MSRImpl;

    let core_id = 0.into();
    let mode = detect_msr_mode();
    let preset = get_cpu_preset(mode);
    let msr_config = MSRConfig::new(true, preset.clone());
    let msr = MSRImpl::new(msr_config, core_id);
    let original_items = preset
        .get_valid_items()
        .map(|preset_item| {
            let item = msr.read(preset_item.register_id(), core_id);
            if let Ok(item) = item {
                item
            } else {
                // Adding an invalid Item to effectively disable the MSR
                MSRItem::new(0, 0)
            }
        })
        .collect();
    MSRCpuPreset::new(original_items)
});

pub fn get_original_cpu_msr_preset() -> MSRCpuPreset {
    CPU_MSR_ORIGINAL_PRESET.clone()
}
