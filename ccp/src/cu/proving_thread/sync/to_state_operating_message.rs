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

use std::collections::HashMap;

use ccp_shared::proof::CCBatchedProofs;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::CUID;
use cpu_utils::LogicalCoreId;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub(crate) use super::errors::ProvingThreadSyncError;
pub(crate) use super::raw_proof::RawProof;
pub(crate) use crate::prover::GovernorMessage;
use crate::state_storage::CCPState;

pub(crate) type ToStateOperatingInlet = mpsc::Sender<ToStateOperatingMessage>;
pub(crate) type ToStateOperatingOutlet = mpsc::Receiver<ToStateOperatingMessage>;
pub(crate) type StateOperatingControlInlet = mpsc::Sender<StateOperatingControlMessage>;
pub(crate) type StateOperatingControlOutlet = mpsc::Receiver<StateOperatingControlMessage>;
pub(crate) type GovernorOperatingInlet = mpsc::Sender<GovernorMessage>;
pub(crate) type GovernorOperatingOutlet = mpsc::Receiver<GovernorMessage>;

#[derive(Debug)]
pub(crate) enum ToStateOperatingMessage {
    ProofFound {
        proof: RawProof,
    },
    ErrorHappened {
        core_id: LogicalCoreId,
        error: ProvingThreadSyncError,
    },
}

impl ToStateOperatingMessage {
    pub(crate) fn proof_found(proof: RawProof) -> Self {
        Self::ProofFound { proof }
    }

    pub(crate) fn error_happened(core_id: LogicalCoreId, error: ProvingThreadSyncError) -> Self {
        Self::ErrorHappened { core_id, error }
    }
}

#[derive(Debug)]
pub(crate) enum StateOperatingControlMessage {
    SaveConfig {
        notifier: oneshot::Sender<tokio::io::Result<()>>,
        state: Option<CCPState>,
    },
    CleanProofCache {
        notifier: oneshot::Sender<tokio::io::Result<()>>,
    },
    GetAllProofs {
        last_known_proofs: HashMap<CUID, ProofIdx>,
        limit: usize,
        notifier: oneshot::Sender<tokio::io::Result<CCBatchedProofs>>,
    },
    GetBatchProofs {
        batch_length: usize,
        batch_count: usize,
        last_known_proofs: HashMap<CUID, ProofIdx>,
        notifier: oneshot::Sender<tokio::io::Result<CCBatchedProofs>>,
    },
}
