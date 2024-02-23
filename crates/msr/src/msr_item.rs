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

#[derive(Clone, Copy, Debug)]
pub struct MSRItem {
    register_id: u32,
    value: u64,
    mask: u64,
}

impl MSRItem {
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

    pub fn is_valid(&self) -> bool {
        self.register_id > 0
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

impl Default for MSRItem {
    fn default() -> Self {
        Self::new(0, 0)
    }
}
