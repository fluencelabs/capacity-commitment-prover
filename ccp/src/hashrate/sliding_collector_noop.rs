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

use super::ThreadHashrateRecord;

pub(crate) type SlidingHashrate = ();

/// Replacement of SlidingHashrateCollector which does nothing to make the crossterm dep optional.
#[derive(Clone, Debug, Default)]
pub(crate) struct SlidingHashrateCollector {}

impl SlidingHashrateCollector {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn account_record(&mut self, _record: ThreadHashrateRecord) {}

    pub(crate) fn hashrate(&self) -> &SlidingHashrate {
        &()
    }
}
