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

use std::path::PathBuf;

use super::collector::Hashrate;

const PREV_HASHRATE_FILE_NAME: &str = "prev_epoch_hashrate.json";
const CURRENT_HASHRATE_FILE_NAME: &str = "current_epoch_hashrate.json";

pub(crate) struct HashrateSaver {
    prev_file_path: PathBuf,
    current_file_path: PathBuf,
}

impl HashrateSaver {
    pub(crate) fn from_directory(dir: PathBuf) -> Self {
        let prev_file_path = dir.join(PREV_HASHRATE_FILE_NAME);
        let current_file_path = dir.join(CURRENT_HASHRATE_FILE_NAME);

        Self {
            prev_file_path,
            current_file_path,
        }
    }

    pub(crate) fn save_hashrate_previous(&self, hashrate: Hashrate) -> Result<(), std::io::Error> {
        let hashrate = serde_json::to_vec(&hashrate).unwrap();
        std::fs::write(self.prev_file_path.as_path(), hashrate)
    }

    pub(crate) fn save_hashrate_current(&self, hashrate: Hashrate) -> Result<(), std::io::Error> {
        let hashrate = serde_json::to_vec(&hashrate).unwrap();
        std::fs::write(self.current_file_path.as_path(), hashrate)
    }
}
