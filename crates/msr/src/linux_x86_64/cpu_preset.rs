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

use ccp_shared::types::LogicalCoreId;
use once_cell::sync::Lazy;

use super::msr_x86_64::MSRModeEnforcer;
use crate::state::MSRCpuPreset;

const DEFAULT_CORE_TO_READ_MSR: LogicalCoreId = LogicalCoreId(0);

/// This global is used with Linux x86_64 only to look for the original
/// MSR state only once.
static CPU_MSR_ORIGINAL_PRESET: Lazy<MSRCpuPreset> = Lazy::new(get_cpu_preset_);

fn get_cpu_preset_() -> MSRCpuPreset {
    use crate::config::MSRConfig;
    use crate::linux_x86_64::msr_mode::MSRMode;
    use crate::MSRImpl;

    let mode = MSRMode::detect();
    let preset = mode.get_cpu_preset();
    let msr_config = MSRConfig::new(true, preset.clone());
    let msr = MSRModeEnforcer::new(msr_config, core_id);

    let original_items = preset
        .items()
        .filter_map(|item| msr.read(item.register_id(), core_id).ok())
        .collect();

    MSRCpuPreset::new(original_items)
}

pub fn get_original_cpu_msr_preset() -> MSRCpuPreset {
    CPU_MSR_ORIGINAL_PRESET.clone()
}
