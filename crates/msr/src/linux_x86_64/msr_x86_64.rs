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

use super::msr_mode::MSRMode;
use super::utils;
use crate::state::MSRCpuPreset;
use crate::state::MSRPresetItem;
use crate::MSREnforce;
use crate::MSRResult;

const DEFAULT_CORE_TO_READ_MSR: LogicalCoreId = LogicalCoreId::new(0);

#[derive(Clone, Debug)]
pub struct MSRModeEnforcer {
    is_enabled: bool,
    original_preset: MSRCpuPreset,
}

impl MSRModeEnforcer {
    /// Initialize enforces by reading the original values from OS,
    /// this method should typically be called only once in CCP.
    pub fn from_os(is_enabled: bool) -> Self {
        if !is_enabled {
            return Self {
                is_enabled,
                original_preset: <_>::default(),
            };
        }

        let mode = MSRMode::detect();
        let preset = mode.get_optimal_cpu_preset();

        let original_items = preset
            .items()
            .filter_map(|item| {
                let register_id = item.register_id();
                let value = utils::read_msr(register_id, DEFAULT_CORE_TO_READ_MSR).ok()?;
                Some(MSRPresetItem::new(register_id, value))
            })
            .collect();
        let original_preset = MSRCpuPreset::new(original_items);

        Self {
            is_enabled,
            original_preset,
        }
    }

    pub fn from_preset(is_enabled: bool, original_preset: MSRCpuPreset) -> Self {
        Self {
            is_enabled,
            original_preset,
        }
    }

    pub fn original_preset(&self) -> &MSRCpuPreset {
        &self.original_preset
    }
}

impl MSREnforce for MSRModeEnforcer {
    fn enforce(&mut self, core_id: LogicalCoreId) -> MSRResult<()> {
        use super::msr_mode::MSR_MODE;

        if !self.is_enabled {
            return Ok(());
        }

        let mode = *MSR_MODE;
        tracing::debug!("Global MSR mode: mode:{:?}.", mode);

        let preset = mode.get_optimal_cpu_preset();
        for item in preset.items() {
            // TODO Check for errors here and rollback/clean the stored state
            write(*item, core_id)?;
        }

        Ok(())
    }

    fn cease(&self, core_id: LogicalCoreId) -> MSRResult<()> {
        if !self.is_enabled {
            return Ok(());
        }

        for item in self.original_preset.items() {
            write(*item, core_id)?;
        }
        Ok(())
    }
}

pub fn write(item: MSRPresetItem, core_id: LogicalCoreId) -> MSRResult<()> {
    let value_to_write = if item.mask() != MSRPresetItem::NO_MASK {
        let old_value = utils::read_msr(item.register_id(), core_id)?;
        MSRPresetItem::masked_value(old_value, item.value(), item.mask())
    } else {
        item.value()
    };

    tracing::debug!(
        "Write MSR register_id {} value {:#X} at logical CPU {}  ",
        item.register_id(),
        value_to_write,
        core_id
    );
    utils::write_msr(item.register_id(), value_to_write, core_id)
}
