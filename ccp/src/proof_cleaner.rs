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

#[derive(Debug)]
pub(crate) struct ProofCleaner {
    /// Path to a directory containing found proofs.
    proof_directory: PathBuf,
}

impl ProofCleaner {
    /// Creates a a
    pub fn new(proof_directory: PathBuf) -> Self {
        Self { proof_directory }
    }

    /// Removes all proofs in the proof directory, it's intended for cleanup storage
    /// when a new epoch happened.
    pub async fn remove_proofs(&self) -> tokio::io::Result<()> {
        tokio::fs::remove_dir_all(&self.proof_directory).await?;
        tokio::fs::create_dir(&self.proof_directory).await
    }
}
