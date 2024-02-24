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

use std::sync::Arc;

use crate::bindings::cache::*;
use crate::flags::RandomXFlags;
use crate::try_alloc;
use crate::RResult;

#[derive(Debug)]
pub struct Cache {
    inner: Arc<CacheInner>,
}

#[derive(Debug)]
struct CacheInner {
    cache: *mut randomx_cache,
}

unsafe impl Send for CacheInner {}
unsafe impl Sync for CacheInner {}

/// Contains a Cache handle, can't be created from scratch,
/// only obtained from already existing Cache.
#[derive(Clone, Debug)]
pub struct CacheHandle {
    inner: Arc<CacheInner>,
}

impl Cache {
    /// Creates RandomX cache with the provided global_nonce.
    /// Flags is any combination of these 2 flags (each flag can be set or not set):
    ///  - RANDOMX_FLAG_LARGE_PAGES - allocate memory in large pages
    ///  - RANDOMX_FLAG_JIT - create cache structure with JIT compilation support; this makes
    ///                                     subsequent Dataset initialization faster
    /// Optionally, one of these two flags may be selected:
    ///  - RANDOMX_FLAG_ARGON2_SSSE3 - optimized Argon2 for CPUs with the SSSE3 instruction set
    ///                                makes subsequent cache initialization faster
    ///   - RANDOMX_FLAG_ARGON2_AVX2 - optimized Argon2 for CPUs with the AVX2 instruction set
    ///                                makes subsequent cache initialization faster
    pub fn new(global_nonce: &[u8], flags: RandomXFlags) -> RResult<Self> {
        let cache = try_alloc!(
            randomx_alloc_cache(flags.bits()),
            crate::RandomXError::CacheAllocationFailed { flags }
        );

        let cache_inner = CacheInner { cache };
        let mut cache = Self {
            inner: Arc::new(cache_inner),
        };
        cache.initialize(global_nonce);
        Ok(cache)
    }

    /// Initializes the cache memory using the provided global nonce value.
    /// Does nothing if called with the same value again.
    pub fn initialize(&mut self, global_nonce: &[u8]) {
        unsafe {
            randomx_init_cache(
                self.raw(),
                global_nonce.as_ptr() as *const std::ffi::c_void,
                global_nonce.len(),
            )
        };
    }

    pub fn handle(&self) -> CacheHandle {
        CacheHandle {
            inner: self.inner.clone(),
        }
    }

    pub(crate) fn raw(&self) -> *mut randomx_cache {
        self.inner.cache
    }
}

impl CacheHandle {
    pub fn raw(&self) -> *mut randomx_cache {
        self.inner.cache
    }
}

impl Drop for CacheInner {
    fn drop(&mut self) {
        unsafe { randomx_release_cache(self.cache) }
    }
}

pub trait CacheRawAPI {
    fn raw(&self) -> *mut randomx_cache;
}

impl CacheRawAPI for Cache {
    fn raw(&self) -> *mut randomx_cache {
        self.raw()
    }
}

impl CacheRawAPI for CacheHandle {
    fn raw(&self) -> *mut randomx_cache {
        self.raw()
    }
}
