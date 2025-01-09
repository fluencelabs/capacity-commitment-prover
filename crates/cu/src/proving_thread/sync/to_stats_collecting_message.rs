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

use ccp_cu_rpc_client::HashRate;
use ccp_shared::types::EpochParameters;
use cpu_utils::LogicalCoreId;
use tokio::sync::mpsc;

use crate::HashrateRecordType;
pub(crate) use crate::ThreadHashrateRecord;

pub type ToStatsCollectingInlet = mpsc::Sender<ToStatsCollectingMessage>;
pub type ToStatsCollectingOutlet = mpsc::Receiver<ToStatsCollectingMessage>;

#[derive(Debug, Clone)]
pub enum ToStatsCollectingMessage {
    ProofFound {
        core_id: Option<LogicalCoreId>,
        epoch: EpochParameters,
    },
    Hashrate(ThreadHashrateRecord),
    Epoch(EpochParameters),
}

impl ToStatsCollectingMessage {
    pub(crate) fn proof_found(core_id: Option<LogicalCoreId>, epoch: EpochParameters) -> Self {
        Self::ProofFound { core_id, epoch }
    }

    pub(crate) fn hashrate(entry: ThreadHashrateRecord) -> Self {
        Self::Hashrate(entry)
    }

    pub(crate) fn epoch(epoch: EpochParameters) -> Self {
        Self::Epoch(epoch)
    }
}

impl TryFrom<&ToStatsCollectingMessage> for HashRate {
    type Error = ();

    fn try_from(value: &ToStatsCollectingMessage) -> Result<Self, Self::Error> {
        match value {
            &ToStatsCollectingMessage::ProofFound { epoch, .. } => {
                Ok(HashRate::proofs_found(1, epoch))
            }
            ToStatsCollectingMessage::Hashrate(thread_hashrate_record) => {
                let duration_secs = thread_hashrate_record.duration.as_secs_f32();

                match thread_hashrate_record.variant {
                    HashrateRecordType::CacheCreation => Ok(HashRate::cache_creation(
                        duration_secs,
                        thread_hashrate_record.epoch,
                    )),
                    HashrateRecordType::DatasetInitialization { .. } => {
                        Ok(HashRate::dataset_initialization(
                            duration_secs,
                            thread_hashrate_record.epoch,
                        ))
                    }
                    HashrateRecordType::CheckedHashes { count } => Ok(HashRate::checked_hashes(
                        duration_secs,
                        count,
                        thread_hashrate_record.epoch,
                    )),
                }
            }
            ToStatsCollectingMessage::Epoch(_epoch_parameters) => Err(()),
        }
    }
}
