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

use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use ccp_shared::proof::CCProof;

use crate::proof_storage::ensure_dir;

const EXPECT_DEFAULT_SERIALIZER: &str = "the default serde serializer shouldn't fail";

#[derive(Debug)]
pub struct ProofStorage {
    /// Path to a directory containing found proofs.
    proof_directory: PathBuf,
}

/// Intended to store proofs in storage.
impl ProofStorage {
    pub fn new(proof_directory: PathBuf) -> Self {
        Self { proof_directory }
    }

    pub async fn store_new_proof(&self, proof: CCProof) -> tokio::io::Result<()> {
        ensure_dir(&self.proof_directory).await?;
        let proof_as_string = serde_json::to_string(&proof).expect(EXPECT_DEFAULT_SERIALIZER);
        let proof_path = self.proof_directory.join(proof.id.idx.to_string());
        tokio::task::spawn_blocking(move || save_reliably(&proof_path, proof_as_string)).await??;
        Ok(())
    }
}

// this is a sync function to avoid possible tokio peculiarities
pub(crate) fn save_reliably(path: &Path, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    // we might use random name here.  but if the dir will lock'd, it is not necessary.
    let base_dir = path.parent().map(Cow::Borrowed).unwrap_or_default();
    let base_dir_file = File::open(&base_dir)?;

    let (mut draft_file, draft_path) =
        gen_draft_file(&base_dir, path.file_name().unwrap_or_default())?;

    draft_file.write_all(contents.as_ref())?;
    draft_file.flush()?;
    draft_file.sync_all()?;
    std::mem::drop(draft_file);

    // now we have the contents saved reliably on disk as a draft file.
    // if we rename, destination file will be changed atomically
    std::fs::rename(draft_path, path)?;
    // make sure rename is saved
    base_dir_file.sync_all()?;

    Ok(())
}

fn gen_draft_file(base_dir: &Path, prefix: &OsStr) -> std::io::Result<(File, PathBuf)> {
    let named_tmp_file = tempfile::Builder::new()
        .prefix(prefix)
        .tempfile_in(base_dir)?;
    Ok(named_tmp_file.keep()?)
}
