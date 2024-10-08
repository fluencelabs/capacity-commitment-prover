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

use cpu_utils::LogicalCoreId;
use tokio::sync::mpsc;

pub(crate) use super::errors::ProvingThreadSyncError;
pub(crate) use super::RawProof;

pub type ToStateOperatingInlet = mpsc::Sender<ToStateOperatingMessage>;
pub type ToStateOperatingOutlet = mpsc::Receiver<ToStateOperatingMessage>;

#[derive(Debug)]
pub enum ToStateOperatingMessage {
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
