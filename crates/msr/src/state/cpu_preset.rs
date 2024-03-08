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

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MSRCpuPreset {
    items: Vec<MSRPresetItem>,
}

impl MSRCpuPreset {
    pub fn new(items: Vec<MSRPresetItem>) -> MSRCpuPreset {
        Self { items }
    }

    pub fn items(&self) -> impl Iterator<Item = &MSRPresetItem> {
        self.items.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MSRPresetItem {
    register_id: u32,
    value: u64,
    mask: u64,
}

impl MSRPresetItem {
    pub const NO_MASK: u64 = u64::MAX;

    pub fn new(register_id: u32, value: u64) -> Self {
        Self {
            register_id,
            value,
            mask: Self::NO_MASK,
        }
    }

    pub fn with_mask(register_id: u32, value: u64, mask: u64) -> Self {
        Self {
            register_id,
            value,
            mask,
        }
    }

    pub fn register_id(&self) -> u32 {
        self.register_id
    }

    pub fn value(&self) -> u64 {
        self.value
    }

    pub fn mask(&self) -> u64 {
        self.mask
    }

    pub fn masked_value(old_value: u64, new_value: u64, mask: u64) -> u64 {
        (new_value & mask) | (old_value & !mask)
    }
}
