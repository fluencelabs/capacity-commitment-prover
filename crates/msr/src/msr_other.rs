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

use cpu_utils::LogicalCoreId;

use crate::MSRResult;
use crate::MSR;

#[derive(Debug)]
pub struct MSRImpl {}

impl MSRImpl {
    pub fn new(_is_enabled: bool, _core_id: LogicalCoreId) -> Self {
        Self {}
    }
}

impl MSR for MSRImpl {
    fn write_preset(&mut self, _store_state: bool) -> MSRResult<()> {
        Ok(())
    }

    fn repin(&mut self, _core_id: LogicalCoreId) -> MSRResult<()> {
        Ok(())
    }

    fn restore(self) -> MSRResult<()> {
        Ok(())
    }
}
