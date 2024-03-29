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

use crate::proof::CCProofId;
use crate::types::Difficulty;
use crate::types::GlobalNonce;

/// Describes a single epoch, contains global parameters, which come from the on-chain part.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpochParameters {
    pub global_nonce: GlobalNonce,
    pub difficulty: Difficulty,
}

impl EpochParameters {
    pub fn new(global_nonce: GlobalNonce, difficulty: Difficulty) -> Self {
        Self {
            global_nonce,
            difficulty,
        }
    }
}

impl From<CCProofId> for EpochParameters {
    fn from(id: CCProofId) -> Self {
        Self {
            global_nonce: id.global_nonce,
            difficulty: id.difficulty,
        }
    }
}

use std::fmt;

impl fmt::Display for EpochParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "global nonce: {:?}", self.global_nonce)?;
        writeln!(f, "difficulty: {:?}", self.difficulty)
    }
}
