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

mod ptt_structs;
mod ttp_structs;

pub(crate) use ptt_structs::*;
pub(crate) use ttp_structs::*;

use ccp_shared::types::LocalNonce;
use ccp_shared::types::CUID;

#[derive(Debug)]
pub(crate) enum ProverToThreadMessage {
    CreateCache(CreateCache),
    AllocateDataset(AllocateDataset),
    InitializeDataset(InitializeDataset),
    NewCCJob(NewCCJob),
    Stop,
}

#[derive(Debug)]
pub(crate) enum ThreadToProverMessage {
    CacheCreated(CacheCreated),
    DatasetAllocated(DatasetAllocated),
    DatasetInitialized,
}

#[derive(Debug)]
pub(crate) struct RawProof {
    pub(crate) local_nonce: LocalNonce,
    pub(crate) cu_id: CUID,
}

impl RawProof {
    pub(crate) fn new(local_nonce: LocalNonce, cu_id: CUID) -> Self {
        Self { local_nonce, cu_id }
    }
}
