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
#[cfg(target_os = "linux")]
use raw_cpuid::CpuIdReaderNative;

use once_cell::sync::Lazy;

pub static MSR_MODE: Lazy<MSRMode> = Lazy::new(detect_msr_mode);

#[derive(Copy, Clone, Debug)]
pub enum MSRMode {
    MSRModNone,
    MSRModRyzen17h,
    MSRModRyzen19h,
    MSRModRyzen19hZen4,
    MSRModIntel,
}

pub fn detect_msr_mode() -> MSRMode {
    use MSRMode::*;

    let cpuid = CpuId::new();
    match cpuid.get_vendor_info() {
        #[cfg(target_os = "linux")]
        Some(vendor_info) if vendor_info.as_str() == "AuthenticAMD" => detect_amd_msr_mode(&cpuid),
        #[cfg(target_os = "linux")]
        Some(vendor_info) if vendor_info.as_str() == "GenuineIntel" => MSRModIntel,
        _ => MSRModNone,
    }
}
#[cfg(target_os = "linux")]
fn detect_amd_msr_mode(cpuid: &CpuId<CpuIdReaderNative>) -> MSRMode {
    use MSRMode::*;

    let (family_id, model_id) = if let Some(cpu_info) = cpuid.get_feature_info() {
        (cpu_info.family_id(), cpu_info.model_id())
    } else {
        return MSRModNone;
    };

    match family_id {
        0x17 => MSRMode::MSRModRyzen17h,
        0x19 => match model_id {
            0x61 => MSRMode::MSRModRyzen19hZen4,
            _ => MSRMode::MSRModRyzen19h,
        },
        _ => MSRMode::MSRModNone,
    }
}
