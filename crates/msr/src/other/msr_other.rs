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

/// This module is no-op implementation to allow the code to compile on non-x86_64 archs.
use ccp_shared::types::LogicalCoreId;

use crate::state::MSRCpuPreset;
use crate::MSREnforce;
use crate::MSRResult;

#[derive(Clone, Debug)]
pub struct MSRModeEnforcer {
    preset: MSRCpuPreset,
}

impl MSRModeEnforcer {
    pub fn from_os(_is_enabled: bool) -> Self {
        Self {
            preset: MSRCpuPreset::default(),
        }
    }

    pub fn from_preset(_is_enabled: bool, preset: MSRCpuPreset) -> Self {
        Self { preset }
    }

    pub fn preset(&self) -> &MSRCpuPreset {
        &self.preset
    }
}

impl MSREnforce for MSRModeEnforcer {
    fn enforce(&mut self, _core_id: LogicalCoreId) -> MSRResult<()> {
        Ok(())
    }

    fn cease(self, _core_id: LogicalCoreId) -> MSRResult<()> {
        Ok(())
    }
}
