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

use ccp_shared::types::*;

pub(crate) struct RawProof {
    pub(crate) epoch: EpochParameters,
    pub(crate) local_nonce: LocalNonce,
    pub(crate) cu_id: CUID,
    pub(crate) result_hash: ResultHash,
}

impl RawProof {
    pub(crate) fn new(
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
