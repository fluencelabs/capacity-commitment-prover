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
pub(crate) struct Commitment {
    pub(crate) epoch_params: EpochParameters,
    pub(crate) cu_allocation: CUAllocation,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub(crate) struct CCPState {
    pub(crate) commitment: Option<Commitment>,
    pub(crate) msr_state: ccp_msr::state::MSRState,
    pub(crate) utility_cores: Vec<LogicalCoreId>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub(crate) struct CCPStateLegacy {
    // n.b.: `serde(flatten)` of `Commitment` doesn't work
    pub(crate) epoch_params: EpochParameters,
    pub(crate) cu_allocation: CUAllocation,
    pub(crate) msr_state: ccp_msr::state::MSRState,
    pub(crate) utility_cores: Vec<LogicalCoreId>,
}

impl From<Option<CCPStateLegacy>> for CCPState {
    fn from(value: Option<CCPStateLegacy>) -> Self {
        match value {
            Some(state) => Self {
                commitment: Some(Commitment {
                    epoch_params: state.epoch_params,
                    cu_allocation: state.cu_allocation,
                }),
                msr_state: state.msr_state,
                utility_cores: state.utility_cores,
            },
            None => <_>::default(),
        }
    }
}

impl StateStorage {
    pub(crate) fn new(state_dir: PathBuf) -> Self {
        Self { state_dir }
    }

    pub(crate) async fn save_state(&self, state: &CCPState) -> tokio::io::Result<()> {
        let data = serde_json::to_vec(&state).expect(EXPECT_DEFAULT_DESERIALIZER);
        let path = self.state_dir.join(STATE_FILE);

        log::info!("Saving state to {:?}", path);

        tokio::task::spawn_blocking(move || save_reliably(&path, &data)).await??;
        Ok(())
    }

    pub(crate) async fn try_to_load_data(&self) -> tokio::io::Result<CCPState> {
        use std::io::ErrorKind;

        log::info!("Try to restore previous state from {:?}", self.state_dir);
        let path = self.state_dir.join(STATE_FILE);

        let state_data = match tokio::fs::read(&path).await {
            Ok(it) => it,
            Err(err) => match err.kind() {
                ErrorKind::NotFound => <_>::default(),
                _ => return Err(err),
            },
        };

        if let Ok(orig_default) = serde_json::from_slice::<Option<CCPStateLegacy>>(&state_data) {
            return Ok(orig_default.into());
        }

        match serde_json::from_slice(&state_data) {
            Ok(data) => Ok(data),
            Err(e) => {
                log::warn!("failed to parse state data from {path:?}, ignoring: {e}");
                Ok(<_>::default())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ccp_msr::state::MSRState;
    use ccp_shared::types::{Difficulty, GlobalNonce, CUID};

    #[tokio::test]
    async fn test_load_legacy_state_no_commitment() {
        let legacy_data = "null";
        let state_dir = tempdir::TempDir::new("state").unwrap();
        let state_path = state_dir.path().join("state.json");
        tokio::fs::write(&state_path, &legacy_data).await.unwrap();

        let state_storage = StateStorage::new(state_dir.path().to_owned());
        let state = state_storage.try_to_load_data().await.unwrap();

        assert_eq!(
            state,
            CCPState {
                commitment: None,
                msr_state: MSRState::default(),
                utility_cores: vec![],
            }
        );
    }

    #[tokio::test]
    async fn test_load_legacy_state_commitment() {
        let legacy_data = r#"{
            "epoch_params":{
              "global_nonce":[19,220,253,189,81,248,156,137,58,114,97,73,198,62,162,50,7,65,195,219,146,4,65,13,158,165,104,3,61,64,235,230],
              "difficulty":[0,1,20,46,211,35,45,244,66,39,119,16,52,187,177,135,126,192,43,227,206,249,254,64,220,76,59,103,226,253,139,187]
             },
             "cu_allocation":{"1":[192,52,100,105,186,121,170,203,69,85,100,205,144,66,82,85,108,121,68,68,227,24,101,29,154,84,84,26,234,134,65,54]},
             "msr_state":{"msr_preset":{"items":[]}},
             "utility_cores":[3, 8]
        }"#;
        let state_dir = tempdir::TempDir::new("state").unwrap();
        let state_path = state_dir.path().join("state.json");
        tokio::fs::write(&state_path, &legacy_data).await.unwrap();

        let state_storage = StateStorage::new(state_dir.path().to_owned());
        let state = state_storage.try_to_load_data().await.unwrap();

        assert_eq!(
            state,
            CCPState {
                commitment: Some(Commitment {
                    epoch_params: EpochParameters {
                        global_nonce: GlobalNonce::new([
                            19, 220, 253, 189, 81, 248, 156, 137, 58, 114, 97, 73, 198, 62, 162,
                            50, 7, 65, 195, 219, 146, 4, 65, 13, 158, 165, 104, 3, 61, 64, 235,
                            230
                        ]),
                        difficulty: Difficulty::new([
                            0, 1, 20, 46, 211, 35, 45, 244, 66, 39, 119, 16, 52, 187, 177, 135,
                            126, 192, 43, 227, 206, 249, 254, 64, 220, 76, 59, 103, 226, 253, 139,
                            187
                        ])
                    },
                    cu_allocation: maplit::hashmap! {
                        1.into() => CUID::new([192,52,100,105,186,121,170,203,69,85,100,205,144,66,82,85,108,121,68,68,227,24,101,29,154,84,84,26,234,134,65,54])
                    }
                }),
                msr_state: MSRState::default(),
                utility_cores: vec![3.into(), 8.into()],
            }
        );
    }

    #[tokio::test]
    async fn test_load_new_config_no_commitment() {
        let legacy_data = r#"{"commitment":null,"msr_state":{"msr_preset":{"items":[]}}, "utility_cores":[3, 8]}"#;
        let state_dir = tempdir::TempDir::new("state").unwrap();
        let state_path = state_dir.path().join("state.json");
        tokio::fs::write(&state_path, &legacy_data).await.unwrap();

        let state_storage = StateStorage::new(state_dir.path().to_owned());
        let state = state_storage.try_to_load_data().await.unwrap();

        assert_eq!(
            state,
            CCPState {
                commitment: None,
                msr_state: MSRState::default(),
                utility_cores: vec![3.into(), 8.into()],
            }
        );
    }

    #[tokio::test]
    async fn test_load_new_config_commitment() {
        let legacy_data = r#"{
            "commitment":{
                 "epoch_params":{
                     "global_nonce":[19,220,253,189,81,248,156,137,58,114,97,73,198,62,162,50,7,65,195,219,146,4,65,13,158,165,104,3,61,64,235,230],
                     "difficulty":[0,1,20,46,211,35,45,244,66,39,119,16,52,187,177,135,126,192,43,227,206,249,254,64,220,76,59,103,226,253,139,187]
                 },
                 "cu_allocation":{
                     "1":[192,52,100,105,186,121,170,203,69,85,100,205,144,66,82,85,108,121,68,68,227,24,101,29,154,84,84,26,234,134,65,54]
                 }
            },
            "msr_state":{"msr_preset":{"items":[]}},
            "utility_cores":[3, 8]
        }"#;
        let state_dir = tempdir::TempDir::new("state").unwrap();
        let state_path = state_dir.path().join("state.json");
        tokio::fs::write(&state_path, &legacy_data).await.unwrap();

        let state_storage = StateStorage::new(state_dir.path().to_owned());
        let state = state_storage.try_to_load_data().await.unwrap();

        assert_eq!(
            state,
            CCPState {
                commitment: Some(Commitment {
                    epoch_params: EpochParameters {
                        global_nonce: GlobalNonce::new([
                            19, 220, 253, 189, 81, 248, 156, 137, 58, 114, 97, 73, 198, 62, 162,
                            50, 7, 65, 195, 219, 146, 4, 65, 13, 158, 165, 104, 3, 61, 64, 235,
                            230
                        ]),
                        difficulty: Difficulty::new([
                            0, 1, 20, 46, 211, 35, 45, 244, 66, 39, 119, 16, 52, 187, 177, 135,
                            126, 192, 43, 227, 206, 249, 254, 64, 220, 76, 59, 103, 226, 253, 139,
                            187
                        ])
                    },
                    cu_allocation: maplit::hashmap! {
                        1.into() => CUID::new([192,52,100,105,186,121,170,203,69,85,100,205,144,66,82,85,108,121,68,68,227,24,101,29,154,84,84,26,234,134,65,54])
                    }
                }),
                msr_state: MSRState::default(),
                utility_cores: vec![3.into(), 8.into()],
            }
        );
    }
}
