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

use crate::bindings::dataset::*;
use crate::cache::{Cache, CacheRawAPI};
use crate::errors::RandomXError::DatasetAllocationError;
use crate::flags::RandomXFlags;
use crate::try_alloc;
use crate::RResult;

#[derive(Debug)]
pub struct Dataset {
    inner: Arc<DatasetInner>,
}

#[derive(Debug)]
struct DatasetInner {
    dataset: *mut randomx_dataset,
}

unsafe impl Send for DatasetInner {}
unsafe impl Sync for DatasetInner {}

/// Contains a handle of Dataset, can't be created from scratch,
/// only obtained from already existing Cache.
#[derive(Clone, Debug)]
pub struct DatasetHandle {
    inner: Arc<DatasetInner>,
}

impl Dataset {
    /// Allocate and initialize a new database with provided global nonce and flags.
    /// Only RANDOMX_FLAG_LARGE_PAGES is supported (can be set or unset),
    /// it forces memory allocation in large pages.
    pub fn new(global_nonce: &[u8], flags: RandomXFlags) -> RResult<Self> {
        let cache = Cache::new(global_nonce, flags)?;
        Self::from_cache(&cache, flags.contains(RandomXFlags::LARGE_PAGES))
    }

    /// Allocate and initialize a new database with the provided cache,
    /// large_pages_enabled forces it to allocate memory in large pages.
    pub fn from_cache(cache: &Cache, large_pages_enabled: bool) -> RResult<Self> {
        let mut dataset = Self::allocate(large_pages_enabled)?;
        let items_count = dataset.items_count();
        dataset.initialize(cache, 0, items_count);

        Ok(dataset)
    }

    /// Allocate a new dataset, but doesn't initialize it.
    pub fn allocate(large_pages_enabled: bool) -> RResult<Self> {
        let flags = if large_pages_enabled {
            RandomXFlags::LARGE_PAGES
        } else {
            RandomXFlags::default()
        };

        let dataset =
            try_alloc! { randomx_alloc_dataset(flags.bits()), DatasetAllocationError { flags } };
        let dataset_inner = DatasetInner { dataset };
        let dataset = Self {
            inner: Arc::new(dataset_inner),
        };
        Ok(dataset)
    }

    /// Return a number of elements that a dataset could contain.
    pub fn items_count(&self) -> u64 {
        unsafe { randomx_dataset_item_count() }
    }

    /// Initialize dataset with the provided cache.
    pub fn initialize(&mut self, cache: &impl CacheRawAPI, start_item: u64, items_count: u64) {
        unsafe { randomx_init_dataset(self.raw(), cache.raw(), start_item, items_count) };
    }

    pub fn handle(&self) -> DatasetHandle {
        DatasetHandle {
            inner: self.inner.clone(),
        }
    }

    pub(crate) fn raw(&self) -> *mut randomx_dataset {
        self.inner.dataset
    }
}

impl DatasetHandle {
    /// Return a number of elements that a dataset could contain.
    pub fn items_count(&self) -> u64 {
        unsafe { randomx_dataset_item_count() }
    }

    /// Initialize dataset with the provided cache.
    pub fn initialize(&mut self, cache: &impl CacheRawAPI, start_item: u64, items_count: u64) {
        unsafe { randomx_init_dataset(self.raw(), cache.raw(), start_item, items_count) };
    }

    pub(crate) fn raw(&self) -> *mut randomx_dataset {
        self.inner.dataset
    }
}

impl Drop for DatasetInner {
    fn drop(&mut self) {
        unsafe { randomx_release_dataset(self.dataset) }
    }
}

pub trait DatasetRawAPI {
    fn raw(&self) -> *mut randomx_dataset;
}

impl DatasetRawAPI for Dataset {
    fn raw(&self) -> *mut randomx_dataset {
        self.raw()
    }
}

impl DatasetRawAPI for DatasetHandle {
    fn raw(&self) -> *mut randomx_dataset {
        self.raw()
    }
}
