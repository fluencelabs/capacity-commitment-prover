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

use ccp_shared::proof::CCProof;
use ccp_shared::proof::ProofIdx;
use cpu_utils::LogicalCoreId;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub(crate) use super::errors::ProvingThreadSyncError;
pub(crate) use super::raw_proof::RawProof;
pub(crate) use crate::hashrate::ThreadHashrateRecord;
use crate::state_storage::CCPState;

pub(crate) type ToUtilityInlet = mpsc::Sender<ToUtilityMessage>;
pub(crate) type ToUtilityOutlet = mpsc::Receiver<ToUtilityMessage>;
pub(crate) type UtilityControlInlet = mpsc::Sender<UtilityControlMessage>;
pub(crate) type UtilityControlOutlet = mpsc::Receiver<UtilityControlMessage>;

#[derive(Debug)]
pub(crate) enum ToUtilityMessage {
    ProofFound {
        core_id: LogicalCoreId,
        proof: RawProof,
    },
    ErrorHappened {
        core_id: LogicalCoreId,
        error: ProvingThreadSyncError,
    },
    Hashrate(ThreadHashrateRecord),
}

impl ToUtilityMessage {
    pub(crate) fn proof_found(core_id: LogicalCoreId, proof: RawProof) -> Self {
        Self::ProofFound { core_id, proof }
    }

    pub(crate) fn error_happened(core_id: LogicalCoreId, error: ProvingThreadSyncError) -> Self {
        Self::ErrorHappened { core_id, error }
    }

    pub(crate) fn hashrate(entry: ThreadHashrateRecord) -> Self {
        Self::Hashrate(entry)
    }
}

#[derive(Debug)]
pub(crate) enum UtilityControlMessage {
    SaveConfig {
        notifier: oneshot::Sender<tokio::io::Result<()>>,
        state: Option<CCPState>,
    },
    CleanProofCache {
        notifier: oneshot::Sender<tokio::io::Result<()>>,
    },
    GetProofs {
        proof_idx: ProofIdx,
        notifier: oneshot::Sender<tokio::io::Result<Vec<CCProof>>>,
    },
}
