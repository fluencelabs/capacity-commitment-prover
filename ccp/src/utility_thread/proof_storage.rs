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
use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use ccp_shared::proof::CCProof;

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
        let proof_as_string = serde_json::to_string(&proof).expect(EXPECT_DEFAULT_SERIALIZER);
        let proof_path = self.proof_directory.join(proof.id.idx.to_string());
        tokio::task::spawn_blocking(move || save_reliably(&proof_path, proof_as_string))
            .await
            // unwrap spawn_blocking outer result: if something is wrong here,
            // the system is screwed.
            .unwrap()
    }
}

// This is a non-sync function to avoid possible tokio peculiarities.
fn save_reliably(path: &Path, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
    // We might use random name here.  But if the dir is lock'd, it is not necessary.
    let extension = draft_extension(path);
    let mut draft_path = path.to_owned();
    draft_path.set_extension(extension);

    let base_dir = path.parent().map(Cow::Borrowed).unwrap_or_default();
    // Yep; we open a directory as a file.  It's ok.
    let base_dir_file = File::open(base_dir)?;

    // We do not bother removing draft file if everything goes wrong.
    let mut draft_file = File::create(&draft_path)?;
    draft_file.write_all(contents.as_ref())?;
    draft_file.flush()?; // Yep, flush is noop here, but lets do it anyway.
    draft_file.sync_all()?;
    std::mem::drop(draft_file);

    // Now we have the contents saved reliably on disk as a draft file.
    // If we rename, destination file will be changed atomically.
    std::fs::rename(draft_path, path)?;
    // Make sure rename is saved.
    base_dir_file.sync_all()?;

    Ok(())
}

fn draft_extension(path: &Path) -> OsString {
    let mut extension = path.extension().unwrap_or_default().to_owned();
    let os_string = OsString::try_from(".draft").unwrap();
    extension.extend([os_string].into_iter());
    extension
}
