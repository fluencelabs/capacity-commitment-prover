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

use ccp_shared::types::*;
use randomx::cache::CacheHandle;
use randomx::dataset::DatasetHandle;
use randomx::Cache;
use randomx::Dataset;
use randomx_rust_wrapper as randomx;
use randomx_rust_wrapper::RandomXFlags;

use super::errors::AsyncThreadError;
use crate::cu::proving_thread::facade::ProvingThreadFacade;
use crate::cu::proving_thread::messages::*;
use crate::cu::proving_thread::sync::to_utility_message::ToUtilityInlet;
use crate::cu::proving_thread::sync::ProvingThreadSync;

#[derive(Debug)]
pub(crate) struct ProvingThreadAsync {
    to_sync: AsyncToSyncInlet,
    from_sync: SyncToAsyncOutlet,
    sync_thread: ProvingThreadSync,
}

impl ProvingThreadAsync {
    pub(crate) fn new(core_id: LogicalCoreId, to_utility: ToUtilityInlet) -> Self {
        let (to_sync, from_async) = mpsc::channel::<AsyncToSyncMessage>(1);
        let (to_async, from_sync) = mpsc::channel::<SyncToAsyncMessage>(1);
        let sync_thread = ProvingThreadSync::spawn(core_id, from_async, to_async, to_utility);

        Self {
            to_sync,
            from_sync,
            sync_thread,
        }
    }
}

impl ProvingThreadFacade for ProvingThreadAsync {
    type Error = AsyncThreadError;

    async fn create_cache(
        &mut self,
        global_nonce: GlobalNonce,
        cu_id: CUID,
        flags: RandomXFlags,
    ) -> Result<Cache, Self::Error> {
        let message = CreateCache::new(global_nonce, cu_id, flags);
        let message = AsyncToSyncMessage::CreateCache(message);
        self.to_sync.send(message).await?;

        match self.from_sync.recv().await {
            Some(SyncToAsyncMessage::CacheCreated(params)) => Ok(params.cache),
            Some(message) => Err(AsyncThreadError::channel_error(format!(
                "expected the CacheCreated event, but {message:?} received"
            ))),
            None => Err(AsyncThreadError::channel_error(
                "sync to async channel is closed unexpectedly".to_string(),
            )),
        }
    }

    async fn allocate_dataset(&mut self, flags: RandomXFlags) -> Result<Dataset, Self::Error> {
        let message = AllocateDataset::new(flags);
        let message = AsyncToSyncMessage::AllocateDataset(message);
        self.to_sync.send(message).await?;

        match self.from_sync.recv().await {
            Some(SyncToAsyncMessage::DatasetAllocated(params)) => Ok(params.dataset),
            Some(message) => Err(AsyncThreadError::channel_error(format!(
                "expected the DatasetAllocated event, but {message:?} received"
            ))),
            None => Err(AsyncThreadError::channel_error(
                "sync to async channel is closed unexpectedly".to_string(),
            )),
        }
    }

    async fn initialize_dataset(
        &mut self,
        cache: CacheHandle,
        dataset: DatasetHandle,
        start_item: u64,
        items_count: u64,
    ) -> Result<(), Self::Error> {
        let message = InitializeDataset::new(cache, dataset, start_item, items_count);
        let message = AsyncToSyncMessage::InitializeDataset(message);
        self.to_sync.send(message).await?;

        match self.from_sync.recv().await {
            Some(SyncToAsyncMessage::DatasetInitialized) => Ok(()),
            Some(message) => Err(AsyncThreadError::channel_error(format!(
                "expected the DatasetInitialized event, but {message:?} received"
            ))),
            None => Err(AsyncThreadError::channel_error(
                "sync to async channel is closed unexpectedly".to_string(),
            )),
        }
    }

    async fn run_cc_job(
        &self,
        dataset: DatasetHandle,
        flags: RandomXFlags,
        epoch: EpochParameters,
        cu_id: CUID,
    ) -> Result<(), Self::Error> {
        let message = NewCCJob::new(dataset, flags, epoch, cu_id);
        let message = AsyncToSyncMessage::NewCCJob(message);
        self.to_sync.send(message).await.map_err(Into::into)
    }

    async fn pin(&self, core_id: LogicalCoreId) -> Result<(), Self::Error> {
        let message = AsyncToSyncMessage::PinThread(PinThread { core_id });
        self.to_sync.send(message).await.map_err(Into::into)
    }

    async fn pause(&mut self) -> Result<(), Self::Error> {
        let message = AsyncToSyncMessage::Pause;
        self.to_sync.send(message).await?;

        match self.from_sync.recv().await {
            Some(SyncToAsyncMessage::Paused) => Ok(()),
            Some(message) => Err(AsyncThreadError::channel_error(format!(
                "expected the Paused event, but {message:?} received"
            ))),
            None => Err(AsyncThreadError::channel_error(
                "sync to async channel is closed unexpectedly".to_string(),
            )),
        }
    }

    async fn stop(self) -> Result<(), Self::Error> {
        let message = AsyncToSyncMessage::Stop;
        self.to_sync.send(message).await?;
        self.sync_thread.join().map_err(Into::into)
    }
}
