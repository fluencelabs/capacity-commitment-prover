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

use ccp_shared::types::EpochParameters;
use cpu_utils::LogicalCoreId;
use tokio::sync::mpsc;

pub(crate) use ccp::hashrate::ThreadHashrateRecord;

pub(crate) type ToStatsCollectingInlet = mpsc::Sender<ToStatsCollectingMessage>;
pub(crate) type ToStatsCollectingOutlet = mpsc::Receiver<ToStatsCollectingMessage>;

#[derive(Debug)]
pub(crate) enum ToStatsCollectingMessage {
    ProofFound { core_id: LogicalCoreId },
    Hashrate(ThreadHashrateRecord),
    Epoch(EpochParameters),
}

impl ToStatsCollectingMessage {
    pub(crate) fn proof_found(core_id: LogicalCoreId) -> Self {
        Self::ProofFound { core_id }
    }

    pub(crate) fn hashrate(entry: ThreadHashrateRecord) -> Self {
        Self::Hashrate(entry)
    }

    pub(crate) fn epoch(epoch: EpochParameters) -> Self {
        Self::Epoch(epoch)
    }
}
