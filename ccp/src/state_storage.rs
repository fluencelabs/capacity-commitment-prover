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

use ccp_shared::types::CUAllocation;
use ccp_shared::types::EpochParameters;
use ccp_shared::types::LogicalCoreId;
use serde::Deserialize;
use serde::Serialize;

use crate::utility_thread::save_reliably;

const EXPECT_DEFAULT_DESERIALIZER: &str = "the default serde (de)serializer shouldn't fail";
const STATE_FILE: &str = "state.json";

pub(crate) struct StateStorage {
    state_dir: PathBuf,
    // TODO file lock
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub(crate) struct CCPState {
    pub(crate) epoch_params: EpochParameters,
    pub(crate) cu_allocation: CUAllocation,
    pub(crate) utility_cores: Vec<LogicalCoreId>,
}

impl StateStorage {
    pub(crate) fn new(state_dir: PathBuf) -> Self {
        Self { state_dir }
    }

    pub(crate) async fn save_state(&self, state: Option<&CCPState>) -> tokio::io::Result<()> {
        let data = serde_json::to_vec(&state).expect(EXPECT_DEFAULT_DESERIALIZER);
        let path = self.state_dir.join(STATE_FILE);

        log::info!("Saving state to {:?}", path);

        tokio::task::spawn_blocking(move || save_reliably(&path, &data)).await??;
        Ok(())
    }

    // TODO should it return error on IO problems?
    pub(crate) async fn try_to_load_data(&self) -> tokio::io::Result<Option<CCPState>> {
        log::info!("Try to restore previous state from {:?}", self.state_dir);
        let path = self.state_dir.join(STATE_FILE);

        if !path.exists() {
            return Ok(None);
        }

        let state_data = tokio::fs::read(&path).await?;

        match serde_json::from_slice(&state_data) {
            Ok(data) => Ok(data),
            Err(e) => {
                log::warn!("failed to parse state data from {path:?}, ignoring: {e}");
                Ok(None)
            }
        }
    }
}
