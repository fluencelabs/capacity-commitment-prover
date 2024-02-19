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

use ccp_shared::types::EpochParameters;
use randomx::cache::CacheHandle;
use randomx::dataset::DatasetHandle;
use randomx::Cache;
use randomx::Dataset;
use randomx_rust_wrapper as randomx;
use randomx_rust_wrapper::RandomXFlags;
use tokio::sync::mpsc;

use super::errors::ProvingThreadError;
use super::facade::ProvingThreadFacade;
use super::messages::*;
use super::sync::ProvingThreadSync;
use crate::GlobalNonce;
use crate::LogicalCoreId;
use crate::CUID;

#[derive(Debug)]
pub(crate) struct ProvingThreadAsync {
    inlet: AsyncToSyncInlet,
    outlet: SyncToAsyncOutlet,
    sync_thread: ProvingThreadSync,
}

impl ProvingThreadAsync {
    pub(crate) fn new(
        core_id: LogicalCoreId,
        proof_receiver_inlet: mpsc::Sender<RawProof>,
    ) -> Self {
        let (ats_inlet, ats_outlet) = mpsc::channel::<AsyncToSyncMessage>(1);
        let (sta_inlet, sta_outlet) = mpsc::channel::<SyncToAsyncMessage>(1);
        let sync_thread =
            ProvingThreadSync::spawn(core_id, ats_outlet, sta_inlet, proof_receiver_inlet);

        Self {
            inlet: ats_inlet,
            outlet: sta_outlet,
            sync_thread,
        }
    }
}

impl ProvingThreadFacade for ProvingThreadAsync {
    type Error = ProvingThreadError;

    async fn create_cache(
        &mut self,
        global_nonce: GlobalNonce,
        cu_id: CUID,
        flags: RandomXFlags,
    ) -> Result<Cache, Self::Error> {
        let message = CreateCache::new(global_nonce, cu_id, flags);
        let message = AsyncToSyncMessage::CreateCache(message);
        self.inlet.send(message).await?;

        match self.outlet.recv().await {
            Some(SyncToAsyncMessage::CacheCreated(params)) => Ok(params.cache),
            Some(message) => Err(ProvingThreadError::channel_error(format!(
                "expected the CacheCreated event, but {message:?} received"
            ))),
            None => Err(ProvingThreadError::channel_error(
                "sync to async channel is closed unexpectedly".to_string(),
            )),
        }
    }

    async fn allocate_dataset(&mut self, flags: RandomXFlags) -> Result<Dataset, Self::Error> {
        let message = AllocateDataset::new(flags);
        let message = AsyncToSyncMessage::AllocateDataset(message);
        self.inlet.send(message).await?;

        match self.outlet.recv().await {
            Some(SyncToAsyncMessage::DatasetAllocated(params)) => Ok(params.dataset),
            Some(message) => Err(ProvingThreadError::channel_error(format!(
                "expected the DatasetAllocated event, but {message:?} received"
            ))),
            None => Err(ProvingThreadError::channel_error(
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
        self.inlet.send(message).await?;

        match self.outlet.recv().await {
            Some(SyncToAsyncMessage::DatasetInitialized) => Ok(()),
            Some(message) => Err(ProvingThreadError::channel_error(format!(
                "expected the DatasetInitialized event, but {message:?} received"
            ))),
            None => Err(ProvingThreadError::channel_error(
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
        self.inlet.send(message).await.map_err(Into::into)
    }

    async fn pin(&self, core_id: LogicalCoreId) -> Result<(), Self::Error> {
        let message = AsyncToSyncMessage::PinThread(PinThread { core_id });
        self.inlet.send(message).await.map_err(Into::into)
    }

    async fn pause(&mut self) -> Result<(), Self::Error> {
        let message = AsyncToSyncMessage::Pause;
        self.inlet.send(message).await?;

        match self.outlet.recv().await {
            Some(SyncToAsyncMessage::Paused) => Ok(()),
            Some(message) => Err(ProvingThreadError::channel_error(format!(
                "expected the Paused event, but {message:?} received"
            ))),
            None => Err(ProvingThreadError::channel_error(
                "sync to async channel is closed unexpectedly".to_string(),
            )),
        }
    }

    async fn stop(self) -> Result<(), Self::Error> {
        let message = AsyncToSyncMessage::Stop;
        self.inlet.send(message).await?;
        self.sync_thread.join()?;

        Ok(())
    }
}
