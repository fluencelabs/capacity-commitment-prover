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

use ccp_shared::proof::BatchRequest;
use ccp_shared::proof::BatchResponse;
use ccp_shared::types::EpochParameters;
use ccp_shared::types::CUID;
use eyre::Context;
use parking_lot::Mutex as PLMutex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::DirEntry;

use ccp_shared::proof::CCProof;
use ccp_shared::proof::ProofIdx;

use crate::proof_queue::ProofMultiQueue;

#[derive(Debug)]
pub(crate) struct ProofStorageDrainer {
    /// Path to a directory containing found proofs.
    proof_directory: PathBuf,
    proof_queue: Arc<PLMutex<ProofMultiQueue>>,
}

impl ProofStorageDrainer {
    pub fn new(proof_directory: PathBuf, proof_queue: Arc<PLMutex<ProofMultiQueue>>) -> Self {
        Self {
            proof_directory,
            proof_queue,
        }
    }

    /// Removes all proofs in the proof directory, it's intended for cleanup storage
    /// when a new epoch happened.
    pub async fn clear_proofs(&mut self) -> eyre::Result<()> {
        self.proof_queue.lock().clear();

        if tokio::fs::try_exists(&self.proof_directory).await? {
            tokio::fs::remove_dir_all(&self.proof_directory).await?;
        }
        Ok(tokio::fs::create_dir(&self.proof_directory).await?)
    }

    /// Get batches proofs from the queue.
    pub async fn get_batch_proofs_after(
        &mut self,
        reqs: &HashMap<CUID, BatchRequest>,
        min_batch_count: usize,
        max_batch_count: usize,
    ) -> eyre::Result<Vec<BatchResponse>> {
        Ok(self
            .proof_queue
            .lock()
            .get_batched_proofs_after(reqs, min_batch_count, max_batch_count))
    }

    /// Gets proofs from the proof directory, which proof_id is strictly bigger than
    /// the provided proof id.
    pub async fn get_all_proofs_after(
        &mut self,
        last_known_proofs: &HashMap<CUID, ProofIdx>,
        limit: usize,
    ) -> eyre::Result<Vec<BatchResponse>> {
        Ok(self
            .proof_queue
            .lock()
            .get_all_proofs_after(last_known_proofs, limit))
    }

    pub async fn validate_proofs(
        &mut self,
        epoch: Option<&EpochParameters>,
    ) -> eyre::Result<ProofIdx> {
        ensure_dir(&self.proof_directory).await?;

        log::info!("validate proofs from disk for epoch {epoch:?}");

        let mut directory = tokio::fs::read_dir(&self.proof_directory).await?;
        let mut proofs = vec![];
        loop {
            match directory.next_entry().await {
                Ok(Some(entry)) => {
                    if let Some(entry_proof_id) = Self::proof_idx_from_filename(&entry).await? {
                        let file_content = tokio::fs::read(entry.path())
                            .await
                            .with_context(|| format!("failed to read {:?}", entry.path()))?;
                        let proof: CCProof =
                            serde_json::from_slice(&file_content).with_context(|| {
                                format!("failed to parse proof at {:?}", entry.path())
                            })?;

                        log::debug!("loaded proof {entry_proof_id}: {proof:?}");

                        let found_epoch: EpochParameters = proof.id.into();
                        if Some(&found_epoch) == epoch {
                            proofs.push(proof);
                        } else {
                            let path = entry.path();
                            log::warn!("removing a proof file with wrong epoch: {path:?}");
                            let _ = tokio::fs::remove_file(path).await;
                        }
                    }
                }
                Ok(None) => {
                    break;
                }
                Err(e) => {
                    return Err(e).with_context(|| {
                        format!("failed to read directory: {:?}", self.proof_directory)
                    })
                }
            }
        }

        proofs.sort_unstable_by_key(|proof| proof.id.idx);
        let mut max_proof_idx = proofs.last().map(|proof| proof.id.idx);
        {
            let mut proof_queue_guard = self.proof_queue.lock();

            for proof in proofs {
                proof_queue_guard.insert(proof);
            }
        }
        // We should return an idx that utility thread can use, i.e. new one.
        if let Some(idx) = max_proof_idx.as_mut() {
            idx.increment()
        }
        Ok(max_proof_idx.unwrap_or_default())
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
