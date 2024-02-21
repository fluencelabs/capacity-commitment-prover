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

use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

use cpu_utils::LogicalCoreId;

use super::hashrate_entry::HashrateCUEntry;

pub(crate) struct HashrateCollector {
    epoch_started: Option<Instant>,
    entries: HashMap<LogicalCoreId, HashrateCUInfo>,
}

pub(crate) struct HashrateCUInfo {
    pub(crate) entries: Vec<HashrateCUEntry>,
}

impl HashrateCollector {
    pub(crate) fn new() -> Self {
        Self {
            epoch_started: None,
            entries: HashMap::new(),
        }
    }

    pub(crate) fn add_entry(&mut self, new_entry: HashrateCUEntry, core_id: LogicalCoreId) {
        use std::collections::hash_map::Entry;

        match self.entries.entry(core_id) {
            Entry::Vacant(entry) => {
                let cu_info = HashrateCUInfo {
                    entries: vec![new_entry]
                };

                entry.insert(cu_info);
            }
            Entry::Occupied(entry) => {
                entry.into_mut().push(new_entry);
            }
        }
    }

    pub(crate) fn collect(&mut self) {

    }
}
