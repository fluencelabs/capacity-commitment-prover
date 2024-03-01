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

use chrono::Timelike;
use std::path::Path;
use std::path::PathBuf;

use super::collector::Hashrate;
use super::HResult;
use super::ThreadHashrateRecord;
use crate::hashrate::record::HashrateRecordType;

const PREV_HASHRATE_FILE_NAME: &str = "prev_epoch_hashrate.json";
const CURRENT_HASHRATE_FILE_NAME: &str = "current_epoch_hashrate.json";
const HASHRATE_DIR: &str = "hashrate";
const INSTANT_HASHRATE_DIR: &str = "sliding_hashrate";

pub(crate) struct HashrateSaver {
    prev_hashrate_path: PathBuf,
    current_hashrate_path: PathBuf,
    instant_hashrate_path: PathBuf,
}

impl HashrateSaver {
    pub(crate) fn from_directory(state_dir_path: PathBuf) -> HResult<Self> {
        let hashrate_dir = state_dir_path.join(HASHRATE_DIR);
        ensure_dir_exists_and_empty(&hashrate_dir)?;

        let prev_hashrate_path = hashrate_dir.join(PREV_HASHRATE_FILE_NAME);
        let current_hashrate_path = hashrate_dir.join(CURRENT_HASHRATE_FILE_NAME);
        let instant_hashrate_path = hashrate_dir.join(INSTANT_HASHRATE_DIR);

        ensure_dir_exists_and_empty(&instant_hashrate_path)?;

        let saver = Self {
            prev_hashrate_path,
            current_hashrate_path,
            instant_hashrate_path,
        };

        Ok(saver)
    }

    pub(crate) fn save_hashrate_previous(&self, hashrate: Hashrate) -> HResult<()> {
        let hashrate = serde_json::to_vec(&hashrate).unwrap();
        std::fs::write(self.prev_hashrate_path.as_path(), hashrate).map_err(Into::into)
    }

    pub(crate) fn save_hashrate_current(&self, hashrate: Hashrate) -> HResult<()> {
        let hashrate = serde_json::to_vec(&hashrate).unwrap();
        std::fs::write(self.current_hashrate_path.as_path(), hashrate).map_err(Into::into)
    }

    pub(crate) fn save_hashrate_entry(&self, record: &ThreadHashrateRecord) -> HResult<()> {
        let core_id: usize = record.core_id.into();
        let path = self.instant_hashrate_path.join(core_id.to_string());
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        let writer = csv::Writer::from_writer(file);
        print_record_as_csv(writer, record)
    }

    pub(crate) fn cleanup_sliding_hashrate(&self) -> HResult<()> {
        ensure_dir_exists_and_empty(&self.instant_hashrate_path).map_err(Into::into)
    }
}

fn ensure_dir_exists_and_empty<P: AsRef<Path>>(dir: P) -> Result<(), std::io::Error> {
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir(dir)
}

fn print_record_as_csv(
    mut writer: csv::Writer<std::fs::File>,
    record: &ThreadHashrateRecord,
) -> HResult<()> {
    use super::hashratable::Hashratable;

    let now = chrono::Utc::now();

    let (record_type, measurement_result) = match record.variant {
        HashrateRecordType::CacheCreation => {
            ("cache creation", record.duration.as_secs_f64().to_string())
        }
        HashrateRecordType::DatasetInitialization { .. } => (
            "dataset initialization",
            record.duration.as_secs_f64().to_string(),
        ),
        HashrateRecordType::CheckedHashes { count } => {
            let hashrate =
                super::hashratable::HashrateCalculator::hashrate(count as u64, record.duration);
            ("cc job", hashrate.to_string())
        }
    };

    writer
        .encode([
            format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second()),
            record_type.to_string(),
            measurement_result,
        ])
        .map_err(Into::into)
}
