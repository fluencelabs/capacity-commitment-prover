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

use ccp_shared::proof::CCProof;

const EXPECT_DEFAULT_SERIALIZER: &str = "the default serde serializer shouldn't fail";

#[derive(Debug)]
pub struct ProofStorage {
    /// Path to a directory containing found proofs.
    proof_directory: PathBuf,
}

impl ProofStorage {
    /// Creates a proof storage worker, it exclusively owns the provided proof directory,
    /// e.g. it can remove it and them creates again to flush its content.
    pub fn new(proof_directory: PathBuf) -> Self {
        Self { proof_directory }
    }

    pub async fn store_new_proof(&self, proof: CCProof) -> tokio::io::Result<()> {
        let proof_as_string = serde_json::to_string(&proof).expect(EXPECT_DEFAULT_SERIALIZER);
        let proof_path = self.proof_directory.join(proof.id.idx.to_string());
        tokio::fs::write(proof_path, proof_as_string).await
    }
}
