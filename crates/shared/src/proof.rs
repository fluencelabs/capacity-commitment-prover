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

use crate::types;

/// Uniquely identifies a proof in one epoch.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CCProofId {
    global_nonce: types::GlobalNonce,
    difficulty: types::Difficulty,
    // Unique in one epoch
    id: u64,
}

/// Contains all necessary information to submit proof to verify it.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CCProof {
    id: CCProofId,
    local_nonce: types::LocalNonce,
    cu_id: types::CUID,
}

impl CCProofId {
    pub fn new(global_nonce: types::GlobalNonce, difficulty: types::Difficulty, id: u64) -> Self {
        Self {
            global_nonce,
            difficulty,
            id,
        }
    }
}

impl CCProof {
    pub fn new(id: CCProofId, local_nonce: types::LocalNonce, cu_id: types::CUID) -> Self {
        Self {
            id,
            local_nonce,
            cu_id,
        }
    }
}
