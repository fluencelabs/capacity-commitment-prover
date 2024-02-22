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
use serde::Deserialize;
use serde::Serialize;

use crate::utility_thread::save_reliably;

const STATE_FILE: &str = "state.json";

pub(crate) struct StateStorage {
    state_dir: PathBuf,
    // TODO file lock
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CCPState {
    pub(crate) epoch_params: EpochParameters,
    pub(crate) cu_allocation: CUAllocation,
}

impl StateStorage {
    pub(crate) fn new(state_dir: PathBuf) -> Self {
        Self { state_dir }
    }

    pub(crate) async fn save_state(&self, state: Option<&CCPState>) -> tokio::io::Result<()> {
        let data = serde_json::to_vec(&state).expect("TODO");
        let path = self.state_dir.join(&STATE_FILE);

        log::info!("Saving state to {:?}", path);

        tokio::task::spawn_blocking(move || save_reliably(&path, &data))
            .await
            .unwrap()
    }

    #[allow(dead_code)]
    pub(crate) async fn try_to_load_data(&self) -> Option<CCPState> {
        log::info!("Try to restore previous state from {:?}", self.state_dir);
        todo!()
    }
}
