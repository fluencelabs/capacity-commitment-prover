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

use tempfile::tempfile_in;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

#[derive(Debug)]
pub(crate) struct MemoryDumper {
    dump_file: File,
    dump_start: *mut u8,
    dump_len: usize,
}

unsafe impl Send for MemoryDumper {}
unsafe impl Sync for MemoryDumper {}

impl MemoryDumper {
    pub(crate) async unsafe fn new(
        tmp_dir: &Path,
        // n.b.: this is actually a pointer, but async doesn't work well with pointer (they are not Send)
        dump_start: usize,
        dump_len: usize,
    ) -> std::io::Result<Self> {
        // aligning page size
        // madvise needs to aligned memory; but RandomX doesn't give an aligned base address (it stores header,
        // probably).  Round to upper size, as it is better to dump and remove only subset of data than remove
        // data needed for generation of other parts.
        let page_alignment = page_size::get();
        let aligned_dump_start = dump_start.next_multiple_of(page_alignment);
        let aligned_dump_len = if aligned_dump_start == dump_start {
            dump_len
        } else {
            dump_len - page_alignment
        };

        // N.B. tempfile_in creates and removes a file, keeping an open handle to it.
        // So, the file is removed automagically when the handle is closed, e.g. when
        // the file object is dropped.
        let dump_file: File = tempfile_in(tmp_dir)?.into();

        // Hide the pointer into a Send object, otherwise async code fails to compile
        let mut self_ = Self {
            dump_file,
            dump_start: aligned_dump_start as _,
            dump_len: aligned_dump_len,
        };

        let dump_buffer = std::slice::from_raw_parts(self_.dump_start, self_.dump_len);
        self_.dump_file.write_all(dump_buffer).await?;

        dont_need_helper(self_.dump_start as _, self_.dump_len)?;

        Ok(self_)
    }

    pub(crate) async unsafe fn restore(mut self) -> tokio::io::Result<()> {
        self.dump_file.rewind().await?;
        let dump_buffer = std::slice::from_raw_parts_mut(self.dump_start, self.dump_len);
        self.dump_file.read_exact(dump_buffer).await?;
        Ok(())
    }
}

unsafe fn dont_need_helper(dump_start: *mut u8, dump_len: usize) -> std::io::Result<()> {
    let ret = libc::madvise(dump_start as _, dump_len, libc::MADV_DONTNEED);
    if ret != 0 {
        log::error!("dontneed madvice ret: {}, errno {}", ret, errno::errno());
    }
    Ok(())
}
