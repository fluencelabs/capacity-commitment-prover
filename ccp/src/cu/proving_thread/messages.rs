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

use tokio::sync::mpsc;

mod async_to_sync;
mod sync_to_async;

pub(crate) use async_to_sync::*;
pub(crate) use sync_to_async::*;

pub(crate) type AsyncToSyncInlet = mpsc::Sender<AsyncToSyncMessage>;
pub(crate) type AsyncToSyncOutlet = mpsc::Receiver<AsyncToSyncMessage>;

pub(crate) type SyncToAsyncInlet = mpsc::Sender<SyncToAsyncMessage>;
pub(crate) type SyncToAsyncOutlet = mpsc::Receiver<SyncToAsyncMessage>;

use ccp_shared::types::*;

#[derive(Debug)]
pub(crate) enum AsyncToSyncMessage {
    CreateCache(CreateCache),
    AllocateDataset(AllocateDataset),
    InitializeDataset(InitializeDataset),
    NewCCJob(NewCCJob),
    PinThread(PinThread),
    Stop,
}

#[derive(Debug)]
pub(crate) enum SyncToAsyncMessage {
    CacheCreated(CacheCreated),
    DatasetAllocated(DatasetAllocated),
    DatasetInitialized,
}

#[derive(Debug)]
pub(crate) struct RawProof {
    pub(crate) global_nonce: GlobalNonce,
    pub(crate) difficulty: Difficulty,
    pub(crate) local_nonce: LocalNonce,
    pub(crate) cu_id: CUID,
    pub(crate) result_hash: ResultHash,
}

impl RawProof {
    pub(crate) fn new(
        global_nonce: GlobalNonce,
        difficulty: Difficulty,
        local_nonce: impl Into<LocalNonceInner>,
        cu_id: CUID,
        result_hash: ResultHash,
    ) -> Self {
        let local_nonce = LocalNonce::new(local_nonce.into());
        Self {
            global_nonce,
            difficulty,
            local_nonce,
            cu_id,
            result_hash,
        }
    }
}
