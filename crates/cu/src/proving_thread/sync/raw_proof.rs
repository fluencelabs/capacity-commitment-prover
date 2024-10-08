/*
 * Copyright 2024 Fluence DAO
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

use ccp_shared::types::*;
use serde::Deserialize;
use serde::Serialize;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RawProof {
    pub epoch: EpochParameters,
    pub local_nonce: LocalNonce,
    pub cu_id: CUID,
    pub result_hash: ResultHash,
}

impl RawProof {
    pub fn new(
        epoch: EpochParameters,
        local_nonce: LocalNonce,
        cu_id: CUID,
        result_hash: ResultHash,
    ) -> Self {
        Self {
            epoch,
            local_nonce,
            cu_id,
            result_hash,
        }
    }
}

use std::fmt;

impl fmt::Display for RawProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.epoch)?;
        writeln!(f, "local nonce {:?}", self.local_nonce)?;
        writeln!(f, "CU id: {:?}", self.cu_id)?;
        writeln!(f, "result hash: {:?}", self.result_hash)
    }
}
