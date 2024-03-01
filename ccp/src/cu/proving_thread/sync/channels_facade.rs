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

use ccp_randomx::cache::Cache;
use ccp_randomx::Dataset;
use ccp_shared::types::LogicalCoreId;

use super::to_utility_message::ToUtilityInlet;
use super::to_utility_message::ToUtilityMessage;
use super::STResult;
use crate::cu::proving_thread::messages::*;
use crate::hashrate::ThreadHashrateRecord;
use crate::utility_thread::message::ProvingThreadSyncError;
use crate::utility_thread::message::RawProof;

#[derive(Clone, Debug)]
pub(crate) struct ToAsync {
    to_async: SyncToAsyncInlet,
}

impl ToAsync {
    pub(crate) fn new(to_async: SyncToAsyncInlet) -> Self {
        Self { to_async }
    }

    pub(crate) fn send_cache(&self, cache: Cache) -> STResult<()> {
        let to_async_message = CacheCreated::new(cache);
        let to_async_message = SyncToAsyncMessage::CacheCreated(to_async_message);
        self.to_async
            .blocking_send(to_async_message)
            .map_err(Into::into)
    }

    pub(crate) fn send_dataset(&self, dataset: Dataset) -> STResult<()> {
        let to_async_message = DatasetAllocated::new(dataset);
        let to_async_message = SyncToAsyncMessage::DatasetAllocated(to_async_message);
        self.to_async
            .blocking_send(to_async_message)
            .map_err(Into::into)
    }

    pub(crate) fn notify_dataset_initialized(&self) -> STResult<()> {
        let to_async_message = SyncToAsyncMessage::DatasetInitialized;
        self.to_async
            .blocking_send(to_async_message)
            .map_err(Into::into)
    }

    pub(crate) fn notify_paused(&self) -> STResult<()> {
        let to_async_message = SyncToAsyncMessage::Paused;
        self.to_async
            .blocking_send(to_async_message)
            .map_err(Into::into)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ToUtility {
    to_utility: ToUtilityInlet,
}

impl ToUtility {
    pub(crate) fn new(to_utility: ToUtilityInlet) -> Self {
        Self { to_utility }
    }

    pub(crate) fn send_proof(&self, core_id: LogicalCoreId, proof: RawProof) -> STResult<()> {
        let message = ToUtilityMessage::proof_found(core_id, proof);
        self.to_utility.blocking_send(message).map_err(Into::into)
    }

    pub(crate) fn send_hashrate(&self, hashrate_message: ThreadHashrateRecord) -> STResult<()> {
        let to_utility_message = ToUtilityMessage::hashrate(hashrate_message);
        self.to_utility
            .blocking_send(to_utility_message)
            .map_err(Into::into)
    }

    pub(crate) fn send_error(
        &self,
        core_id: LogicalCoreId,
        error: ProvingThreadSyncError,
    ) -> STResult<()> {
        let to_utility_message = ToUtilityMessage::error_happened(core_id, error);
        self.to_utility
            .blocking_send(to_utility_message)
            .map_err(Into::into)
    }
}
