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

use std::path::PathBuf;
use std::sync::Arc;

use ccp_shared::proof::CCProof;
use parking_lot::Mutex as PLMutex;

use crate::persistence::save_reliably;
use crate::persistence::SaveReliablyError;
use crate::persistence::StorageMetrics;
use crate::proof_queue::ProofMultiQueue;
use crate::proof_storage::ensure_dir;

const EXPECT_DEFAULT_SERIALIZER: &str = "the default serde serializer shouldn't fail";

#[derive(Debug)]
pub struct ProofStorage {
    /// Path to a directory containing found proofs.
    proof_directory: PathBuf,
    proof_storage_metrics: Arc<StorageMetrics>,
    proof_queue: Arc<PLMutex<ProofMultiQueue>>,
}

/// Intended to store proofs in storage.
impl ProofStorage {
    pub fn new(
        proof_directory: PathBuf,
        proof_storage_metrics: Arc<StorageMetrics>,
        proof_queue: Arc<PLMutex<ProofMultiQueue>>,
    ) -> Self {
        Self {
            proof_directory,
            proof_storage_metrics,
            proof_queue,
        }
    }

    pub async fn store_new_proof(&self, proof: CCProof) -> Result<(), SaveReliablyError> {
        ensure_dir(&self.proof_directory)
            .await
            .map_err(|e| SaveReliablyError::new(&self.proof_directory, e))?;
        let proof_as_string = serde_json::to_string(&proof).expect(EXPECT_DEFAULT_SERIALIZER);
        let proof_path = self.proof_directory.join(proof.id.idx.to_string());
        let proof_path_clone = proof_path.clone();

        let proof_storage_metrics = self.proof_storage_metrics.clone();

        tokio::task::spawn_blocking(move || {
            save_reliably(&proof_path, proof_as_string, &proof_storage_metrics)
        })
        .await
        .map_err(|e| SaveReliablyError::new(proof_path_clone, e.into()))??;

        self.proof_queue.lock().insert(proof);

        Ok(())
    }
}
