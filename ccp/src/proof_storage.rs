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
use ccp_shared::types::CUID;
use parking_lot::Mutex as PLMutex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::DirEntry;

use ccp_shared::proof::CCProof;
use ccp_shared::proof::ProofIdx;

use crate::proof_queue::ProofQueue;

#[derive(Debug)]
pub(crate) struct ProofStorageDrainer {
    /// Path to a directory containing found proofs.
    proof_directory: PathBuf,
    pub(crate) proof_queue: Arc<PLMutex<ProofQueue>>,
}

impl ProofStorageDrainer {
    pub fn new(proof_directory: PathBuf) -> Self {
        Self {
            proof_directory,
            proof_queue: <_>::default(),
        }
    }

    /// Removes all proofs in the proof directory, it's intended for cleanup storage
    /// when a new epoch happened.
    pub async fn clear_proofs(&mut self) -> tokio::io::Result<()> {
        self.proof_queue.lock().clear();

        if tokio::fs::try_exists(&self.proof_directory).await? {
            tokio::fs::remove_dir_all(&self.proof_directory).await?;
        }
        tokio::fs::create_dir(&self.proof_directory).await
    }

    /// Get batches proofs from the queue.
    pub async fn get_batch_proofs_after(
        &mut self,
        batch_length: usize,
        batch_count: usize,
        last_known_proofs: &HashMap<CUID, ProofIdx>,
    ) -> tokio::io::Result<Vec<(CUID, Vec<CCProof>)>> {
        Ok(self.proof_queue.lock().get_batched_proofs_after(
            batch_length,
            batch_count,
            last_known_proofs,
        ))
    }

    /// Gets proofs from the proof directory, which proof_id is strictly bigger than
    /// the provided proof id.
    pub async fn get_all_proofs_after(
        &mut self,
        last_known_proofs: &HashMap<CUID, ProofIdx>,
    ) -> tokio::io::Result<HashMap<CUID, Vec<CCProof>>> {
        Ok(self
            .proof_queue
            .lock()
            .get_all_proofs_after(last_known_proofs))
    }

    pub async fn validate_proofs(
        &mut self,
        epoch: Option<&EpochParameters>,
    ) -> tokio::io::Result<ProofIdx> {
        let mut max_proof_idx = None;
        ensure_dir(&self.proof_directory).await?;

        let mut directory = tokio::fs::read_dir(&self.proof_directory).await?;
        loop {
            match directory.next_entry().await {
                Ok(Some(entry)) => {
                    if let Some(entry_proof_id) = Self::proof_idx_from_filename(&entry).await? {
                        let file_content = tokio::fs::read(entry.path()).await?;
                        let proof: CCProof = serde_json::from_slice(&file_content)?;

                        log::debug!("loaded proof {entry_proof_id}: {proof:?}");

                        let found_epoch: EpochParameters = proof.id.into();
                        if Some(&found_epoch) == epoch {
                            max_proof_idx = Some(std::cmp::max(
                                max_proof_idx.unwrap_or_default(),
                                entry_proof_id,
                            ));
                        } else {
                            let path = entry.path();
                            log::warn!("removing a proof file with wrong epoch: {path:?}");
                            let _ = tokio::fs::remove_file(path).await;
                        }
                    }
                }
                Ok(None) => {
                    // We should return an idx that utility thread can use, i.e. new one.
                    if let Some(idx) = max_proof_idx.as_mut() {
                        idx.increment()
                    }
                    return Ok(max_proof_idx.unwrap_or_default());
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn proof_idx_from_filename(entry: &DirEntry) -> tokio::io::Result<Option<ProofIdx>> {
        use std::str::FromStr;

        if !entry.file_type().await?.is_file() {
            return Ok(None);
        }

        let file_name = entry.file_name();
        let file_name_str = match file_name.to_str() {
            Some(name) => name,
            // file is not utf-8, someone else put a file into the proof directory, ignore it
            None => {
                log::warn!("non-utf-8 file name: {file_name:?}, ignoring");
                return Ok(None);
            }
        };

        match ProofIdx::from_str(file_name_str) {
            Ok(current_proof_idx) => Ok(Some(current_proof_idx)),
            // if the file name isn't u64, then again someone else put a file into
            // the proof directory, ignore it
            Err(_) => {
                log::warn!("non-numeric file name: {file_name:?}, ignoring");
                Ok(None)
            }
        }
    }
}

pub(crate) async fn ensure_dir(path: &PathBuf) -> tokio::io::Result<()> {
    match tokio::fs::create_dir(path).await {
        Ok(()) => Ok(()),
        Err(e) => match e.kind() {
            std::io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(e),
        },
    }
}
