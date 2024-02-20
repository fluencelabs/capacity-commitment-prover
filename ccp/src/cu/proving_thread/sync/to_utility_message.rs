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

use cpu_utils::LogicalCoreId;
use tokio::sync::mpsc;

pub(crate) use super::errors::SyncThreadError;
pub(crate) use super::raw_proof::RawProof;

pub(crate) type ToUtilityInlet = mpsc::Sender<ToUtilityMessage>;
pub(crate) type ToUtilityOutlet = mpsc::Receiver<ToUtilityMessage>;

pub(crate) enum ToUtilityMessage {
    ProofFound(RawProof),
    ErrorHappened {
        thread_location: LogicalCoreId,
        error: SyncThreadError,
    },
}

impl ToUtilityMessage {
    pub(crate) fn proof_found(proof: RawProof) -> Self {
        Self::ProofFound(proof)
    }

    pub(crate) fn error_happened(thread_location: LogicalCoreId, error: SyncThreadError) -> Self {
        Self::ErrorHappened {
            thread_location,
            error,
        }
    }
}