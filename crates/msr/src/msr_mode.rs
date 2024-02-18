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

#[derive(Debug)]
pub enum MsrMode {
    MsrModNone,
    MsrModRyzen17h,
    MsrModRyzen19h,
    MsrModRyzen19hZen4,
    MsrModIntel,
    MsrModCustom,
    MsrModMax,
}

pub fn detect_msr_mode() -> MsrMode {
    use MsrMode::*;

    let cpuid = CpuId::new();
    match cpuid.get_vendor_info() {
        Some(vendor_info) if vendor_info.as_str() == "AuthenticAMD" => detect_amd_msr_mode(&cpuid),
        Some(vendor_info) if vendor_info.as_str() == "GenuineIntel" => MsrModIntel,
        _ => MsrModNone,
    }
}

fn detect_amd_msr_mode(cpuid: &CpuId<CpuIdReaderNative>) -> MsrMode {
    use MsrMode::*;

    let (family_id, model_id) = if let Some(cpu_info) = cpuid.get_feature_info() {
        (cpu_info.family_id(), cpu_info.model_id())
    } else {
        return MsrModNone;
    };

    match family_id {
        0x17 => MsrMode::MsrModRyzen17h,
        0x19 => match model_id {
            0x61 => MsrMode::MsrModRyzen19hZen4,
            _ => MsrMode::MsrModRyzen19h,
        },
        _ => MsrMode::MsrModNone,
    }
}
