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

use raw_cpuid::CpuId;
use raw_cpuid::CpuIdReaderNative;

use once_cell::sync::Lazy;

use crate::state::MSRCpuPreset;

pub static MSR_MODE: Lazy<MSRMode> = Lazy::new(MSRMode::detect);

#[derive(Copy, Clone, Debug)]
pub enum MSRMode {
    MSRModNone,
    MSRModRyzen17h,
    MSRModRyzen19h,
    MSRModRyzen19hZen4,
    MSRModIntel,
}

impl MSRMode {
    pub fn detect() -> Self {
        use MSRMode::*;

        let cpuid = CpuId::new();
        match cpuid.get_vendor_info() {
            Some(vendor_info) if vendor_info.as_str() == "AuthenticAMD" => {
                Self::detect_amd_mode(&cpuid)
            }
            Some(vendor_info) if vendor_info.as_str() == "GenuineIntel" => MSRModIntel,
            _ => MSRModNone,
        }
    }

    pub fn get_optimal_cpu_preset(&self) -> &'static MSRCpuPreset {
        use super::cpu_preset_values::CPU_MSR_PRESETS;
        use MSRMode::*;

        match self {
            MSRModNone => &CPU_MSR_PRESETS[0],
            MSRModRyzen17h => &CPU_MSR_PRESETS[1],
            MSRModRyzen19h => &CPU_MSR_PRESETS[2],
            MSRModRyzen19hZen4 => &CPU_MSR_PRESETS[3],
            MSRModIntel => &CPU_MSR_PRESETS[4],
        }
    }

    fn detect_amd_mode(cpuid: &CpuId<CpuIdReaderNative>) -> MSRMode {
        use MSRMode::*;

        let (family_id, model_id) = if let Some(cpu_info) = cpuid.get_feature_info() {
            (cpu_info.family_id(), cpu_info.model_id())
        } else {
            return MSRModNone;
        };

        match family_id {
            0x17 => MSRModRyzen17h,
            0x19 => match model_id {
                0x61 => MSRModRyzen19hZen4,
                _ => MSRModRyzen19h,
            },
            _ => MSRModNone,
        }
    }
}
