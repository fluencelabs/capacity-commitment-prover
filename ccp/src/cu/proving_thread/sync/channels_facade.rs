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
use ccp_shared::types::EpochParameters;
use ccp_shared::types::LogicalCoreId;

use super::to_state_operating_message::ProvingThreadSyncError;
use super::to_state_operating_message::RawProof;
use super::to_state_operating_message::ToStateOperatingInlet;
use super::to_state_operating_message::ToStateOperatingMessage;
use super::to_stats_collecting_message::ToStatsCollectingInlet;
use super::to_stats_collecting_message::ToStatsCollectingMessage;
use super::STResult;
use crate::cu::proving_thread::messages::*;
use crate::hashrate::ThreadHashrateRecord;

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
pub(crate) struct ToStateOperating {
    to_state_operating: ToStateOperatingInlet,
}

impl ToStateOperating {
    pub(crate) fn new(to_state_operating: ToStateOperatingInlet) -> Self {
        Self { to_state_operating }
    }

    pub(crate) fn send_proof(&self, proof: RawProof) -> STResult<()> {
        let message = ToStateOperatingMessage::proof_found(proof);
        self.to_state_operating
            .blocking_send(message)
            .map_err(Into::into)
    }

    pub(crate) fn send_error(
        &self,
        core_id: LogicalCoreId,
        error: ProvingThreadSyncError,
    ) -> STResult<()> {
        let message = ToStateOperatingMessage::error_happened(core_id, error);
        self.to_state_operating
            .blocking_send(message)
            .map_err(Into::into)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ToStatsCollecting {
    to_stats_collecting: ToStatsCollectingInlet,
}

impl ToStatsCollecting {
    pub(crate) fn new(to_stats_collecting: ToStatsCollectingInlet) -> Self {
        Self {
            to_stats_collecting,
        }
    }

    pub(crate) fn send_proof(&self, core_id: LogicalCoreId) -> STResult<()> {
        let message = ToStatsCollectingMessage::proof_found(core_id);
        self.to_stats_collecting
            .blocking_send(message)
            .map_err(Into::into)
    }

    pub(crate) fn send_hashrate(&self, hashrate_message: ThreadHashrateRecord) -> STResult<()> {
        let message = ToStatsCollectingMessage::hashrate(hashrate_message);
        self.to_stats_collecting
            .blocking_send(message)
            .map_err(Into::into)
    }

    pub(crate) fn send_epoch(&self, epoch: EpochParameters) -> STResult<()> {
        let message = ToStatsCollectingMessage::epoch(epoch);
        self.to_stats_collecting
            .blocking_send(message)
            .map_err(Into::into)
    }
}
