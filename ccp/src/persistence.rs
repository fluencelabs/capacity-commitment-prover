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

use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::Instant;

use prometheus_client::metrics;

// changing buckets boundaries is possible, but discuraged;
// for that reason, making it a config option is a bad idea
const HISTOGRAM_BUCKETS: [f64; 9] = [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 4.0];

#[derive(Debug)]
pub struct StorageMetrics {
    pub(crate) io_error: metrics::counter::Counter,
    pub(crate) writing_time: metrics::histogram::Histogram,
}

impl StorageMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for StorageMetrics {
    fn default() -> Self {
        Self {
            io_error: metrics::counter::Counter::default(),
            writing_time: metrics::histogram::Histogram::new(HISTOGRAM_BUCKETS.into_iter()),
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("failed to save {path}: {nested}")]
pub struct SaveReliablyError {
    pub path: PathBuf,
    #[source]
    pub nested: std::io::Error,
}

impl SaveReliablyError {
    pub(crate) fn new(path: impl Into<PathBuf>, nested: std::io::Error) -> Self {
        Self {
            path: path.into(),
            nested,
        }
    }
}

// this is a sync function to avoid possible tokio peculiarities
pub(crate) fn save_reliably(
    path: &Path,
    contents: impl AsRef<[u8]>,
    metrics: &StorageMetrics,
) -> Result<(), SaveReliablyError> {
    let started = Instant::now();
    let res = save_reliably_inner(path, contents.as_ref());
    let duration = started.elapsed();

    if res.is_err() {
        metrics.io_error.inc();
    }

    metrics.writing_time.observe(duration.as_secs_f64());

    res
}

pub(crate) fn save_reliably_inner(path: &Path, contents: &[u8]) -> Result<(), SaveReliablyError> {
    // we might use random name here.  but if the dir will lock'd, it is not necessary.
    let base_dir = path.parent().map(Cow::Borrowed).unwrap_or_default();
    let base_dir_file =
        File::open(&base_dir).map_err(|e| SaveReliablyError::new(base_dir.as_ref(), e))?;

    let (mut draft_file, draft_path) =
        gen_draft_file(&base_dir, path.file_name().unwrap_or_default())?;

    draft_file
        .write_all(contents.as_ref())
        .map_err(|e| SaveReliablyError::new(&draft_path, e))?;
    draft_file
        .flush()
        .map_err(|e| SaveReliablyError::new(&draft_path, e))?;
    draft_file
        .sync_all()
        .map_err(|e| SaveReliablyError::new(&draft_path, e))?;
    std::mem::drop(draft_file);

    // now we have the contents saved reliably on disk as a draft file.
    // if we rename, destination file will be changed atomically
    std::fs::rename(draft_path, path).map_err(|e| SaveReliablyError::new(path, e))?;
    // make sure rename is saved
    base_dir_file
        .sync_all()
        .map_err(|e| SaveReliablyError::new(base_dir.as_ref(), e))?;

    Ok(())
}

fn gen_draft_file(base_dir: &Path, prefix: &OsStr) -> Result<(File, PathBuf), SaveReliablyError> {
    let named_tmp_file = tempfile::Builder::new()
        .prefix(prefix)
        .tempfile_in(base_dir)
        .map_err(|e| SaveReliablyError::new(base_dir, e))?;
    let named_tmp_file_path = named_tmp_file.path().to_owned();
    named_tmp_file
        .keep()
        .map_err(|e| SaveReliablyError::new(named_tmp_file_path, e.into()))
}
