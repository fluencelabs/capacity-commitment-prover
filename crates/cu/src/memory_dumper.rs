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

use std::path::Path;

use anyhow::Context;
use tempfile::tempfile_in;
use thiserror::Error;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

const DUMPER_CHUNK_SIZE: usize = 1024 * 1024;

#[derive(Debug)]
pub(crate) struct MemoryDumper {
    dump_file: File,
    dump_start: *mut u8,
    dump_len: usize,
}

unsafe impl Send for MemoryDumper {}
unsafe impl Sync for MemoryDumper {}

impl MemoryDumper {
    /// Safety: dump_start is a pointer to initialized mmap-allocated memory range of length dump_len.
    ///
    /// It's OK if memory range is not page-aligned.
    pub(crate) async unsafe fn new(
        temp_dir: &Path,
        // n.b.: this is actually a pointer, but async doesn't work well with pointer (they are not Send)
        dump_start: usize,
        dump_len: usize,
    ) -> anyhow::Result<Self> {
        // aligning page size
        // madvise needs to aligned memory; but RandomX doesn't give an aligned base address (it stores header,
        // probably).  Round to upper size, as it is better to dump and remove only subset of data than remove
        // data needed for generation of other parts.
        let page_alignment = page_size::get();
        if dump_len % page_alignment != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "MemoryDumper dump_len has to be multiple of page_size".to_owned(),
            )
            .into());
        }
        let aligned_dump_start = dump_start.next_multiple_of(page_alignment);
        let aligned_dump_len = if aligned_dump_start == dump_start {
            dump_len
        } else {
            dump_len - page_alignment
        };

        // N.B. tempfile_in creates and removes a file, keeping an open handle to it.
        // So, the file is removed automagically when the handle is closed, e.g. when
        // the file object is dropped.
        let mut dump_file: File = tempfile_in(temp_dir)?.into();

        dump(&mut dump_file, aligned_dump_start, aligned_dump_len).await?;

        // Hide the pointer into a Send object, otherwise async code fails to compile
        let self_ = Self {
            dump_file,
            dump_start: aligned_dump_start as _,
            dump_len: aligned_dump_len,
        };

        Ok(self_)
    }

    /// Safety: memory range passed to `new` is still valid and can be written to.
    pub(crate) async unsafe fn restore(mut self) -> anyhow::Result<()> {
        self.dump_file
            .rewind()
            .await
            .context("Failed to rewind a dump file")?;
        let dump_buffer = std::slice::from_raw_parts_mut(self.dump_start, self.dump_len);
        self.dump_file
            .read_exact(dump_buffer)
            .await
            .context("Failed to restore dump file")?;
        Ok(())
    }
}

/// This function marks mmap-allocated memory as unneeded, but still allocated.  A error is logged, but ignored.
///
/// Safety: dump_start is a pointer to a mmap-allocated memory range of length dump_len.
unsafe fn release_memory(dump_start: *mut u8, dump_len: usize) {
    log::debug!("Releasing memory: {dump_start:p} of {dump_len} bytes");
    #[cfg(target_os = "linux")]
    let ret = libc::madvise(dump_start as _, dump_len, libc::MADV_PAGEOUT);
    #[cfg(not(target_os = "linux"))]
    let ret = libc::madvise(dump_start as _, dump_len, libc::MADV_DONTNEED);
    if ret != 0 {
        log::error!(
            "freeing madvice ret: {}, errno {}. Ignoring.",
            ret,
            errno::errno().0
        );
    }
}

#[derive(Error, Debug)]
pub enum DumpError {
    #[error(transparent)]
    Write(std::io::Error),
    #[error(transparent)]
    Flush(std::io::Error),
    #[error(transparent)]
    Sync(std::io::Error),
}

pub async unsafe fn dump(
    dump_file: &mut File,
    // n.b.: this is actually a pointer, but async doesn't work well with pointer (they are not Send)
    dump_start: usize,
    dump_len: usize,
) -> Result<(), DumpError> {
    let dump_buffer = std::slice::from_raw_parts(dump_start as _, dump_len);
    for chunk in dump_buffer.chunks(DUMPER_CHUNK_SIZE) {
        dump_file
            .write_all(chunk)
            .await
            .map_err(|err| DumpError::Write(err))?;

        dump_file
            .flush()
            .await
            .map_err(|err| DumpError::Flush(err))?;

        dump_file
            .sync_all()
            .await
            .map_err(|err| DumpError::Sync(err))?;
    }

    release_memory(dump_start as _, dump_len);

    Ok(())
}
