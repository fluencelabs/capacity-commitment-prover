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

#[derive(Debug)]
pub(crate) enum AsyncToSyncMessage {
    CreateCache(CreateCache),
    AllocateDataset(AllocateDataset),
    InitializeDataset(InitializeDataset),
    NewCCJob(NewCCJob, usize),
    PinThread(PinThread),
    Pause,
    Stop,
}

#[derive(Debug)]
pub(crate) enum SyncToAsyncMessage {
    CacheCreated(CacheCreated),
    DatasetAllocated(DatasetAllocated),
    DatasetInitialized,
    Paused,
}
