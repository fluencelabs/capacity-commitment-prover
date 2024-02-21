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

use std::path::PathBuf;
use tokio::fs::DirEntry;

use ccp_shared::proof::CCProof;
use ccp_shared::proof::ProofIdx;

const EXPECT_DEFAULT_DESERIALIZER: &str = "the default serde deserializer shouldn't fail";

#[derive(Debug)]
pub(crate) struct ProofStorageDrainer {
    /// Path to a directory containing found proofs.
    proof_directory: PathBuf,
}

impl ProofStorageDrainer {
    pub fn new(proof_directory: PathBuf) -> Self {
        Self { proof_directory }
    }

    /// Removes all proofs in the proof directory, it's intended for cleanup storage
    /// when a new epoch happened.
    pub async fn remove_proofs(&self) -> tokio::io::Result<()> {
        tokio::fs::remove_dir_all(&self.proof_directory).await?;
        tokio::fs::create_dir(&self.proof_directory).await
    }

    /// Gets proofs from the proof directory, which proof_id is strictly bigger than
    /// the provided proof id.
    pub async fn get_proofs_after(&self, proof_idx: ProofIdx) -> tokio::io::Result<Vec<CCProof>> {
        let mut proofs = Vec::new();

        let mut directory = tokio::fs::read_dir(&self.proof_directory).await?;
        loop {
            match directory.next_entry().await {
                Ok(Some(entry)) => {
                    if !Self::is_file_suitable(&entry, proof_idx).await? {
                        continue;
                    }

                    let file_content = tokio::fs::read(entry.path()).await?;
                    let proof: CCProof =
                        serde_json::from_slice(&file_content).expect(EXPECT_DEFAULT_DESERIALIZER);

                    proofs.push(proof);
                }
                Ok(None) => {
                    return Ok(proofs);
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn is_file_suitable(entry: &DirEntry, proof_idx: ProofIdx) -> tokio::io::Result<bool> {
        use std::str::FromStr;

        if !entry.file_type().await?.is_file() {
            return Ok(false);
        }

        let file_name = entry.file_name();
        let file_name_str = match file_name.to_str() {
            Some(name) => name,
            // file is not utf-8, someone else put a file into the proof directory, ignore it
            None => return Ok(false),
        };

        match ProofIdx::from_str(file_name_str) {
            Ok(current_proof_idx) => Ok(proof_idx < current_proof_idx),
            // if the file name isn't u64, then again someone else put a file into
            // the proof directory, ignore it
            Err(_) => Ok(false),
        }
    }
}
