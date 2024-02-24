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

use crate::bindings::vm::*;
use crate::cache::Cache;
use crate::cache::CacheRawAPI;
use crate::dataset::Dataset;
use crate::dataset::DatasetRawAPI;
use crate::errors::VmCreationError;
use crate::flags::RandomXFlags;
use crate::result_hash::ResultHash;
use crate::result_hash::ToRawMut;
use crate::try_alloc;
use crate::RResult;

#[derive(Debug)]
pub struct RandomXVM<T> {
    vm: *mut randomx_vm,
    // too ensure that state outlives VM
    _state: T,
}

impl<T> RandomXVM<T>
where
    T: CacheRawAPI,
{
    pub fn light(cache: T, flags: RandomXFlags) -> RResult<Self> {
        if !flags.is_light_mode() {
            return Err(VmCreationError::IncorrectLightModeFlag { flags })?;
        }

        let vm = try_alloc! { randomx_create_vm(flags.bits(), cache.raw(), std::ptr::null_mut()), VmCreationError::AllocationFailed {flags} };

        let vm = RandomXVM { vm, _state: cache };
        Ok(vm)
    }

    /// (from RandomX doc) Reinitializes a virtual machine with a new Cache.
    /// This function should be called anytime the Cache is reinitialized with a new key.
    /// Does nothing if called with a Cache containing the same key value as already set.
    pub fn set_new_cache(&mut self, cache: &'_ Cache) {
        unsafe { randomx_vm_set_cache(self.vm, cache.raw()) }
    }
}

impl<T> RandomXVM<T>
where
    T: DatasetRawAPI,
{
    pub fn fast(dataset: T, flags: RandomXFlags) -> RResult<Self> {
        if !flags.is_fast_mode() {
            return Err(VmCreationError::IncorrectFastModeFlag { flags })?;
        }

        let vm = try_alloc! { randomx_create_vm(flags.bits(), std::ptr::null_mut(), dataset.raw()), VmCreationError::AllocationFailed {flags} };

        let vm = RandomXVM {
            vm,
            _state: dataset,
        };

        Ok(vm)
    }

    /// Reinitializes a virtual machine with a new Dataset.
    pub fn set_new_dataset(&mut self, dataset: &'_ Dataset) {
        unsafe { randomx_vm_set_dataset(self.vm, dataset.raw()) }
    }
}

impl<T> Drop for RandomXVM<T> {
    fn drop(&mut self) {
        unsafe { randomx_destroy_vm(self.vm) }
    }
}

impl<T> RandomXVM<T> {
    /// Calculates a RandomX hash value.
    pub fn hash(&self, local_nonce: &[u8]) -> ResultHash {
        let mut hash = ResultHash::empty();

        unsafe {
            randomx_calculate_hash(
                self.vm,
                local_nonce.as_ptr() as *const std::ffi::c_void,
                local_nonce.len(),
                hash.as_raw_mut(),
            )
        };

        hash
    }

    /// Begins a RandomX hash calculation.
    pub fn hash_first(&self, local_nonce: &[u8]) {
        unsafe {
            randomx_calculate_hash_first(
                self.vm,
                local_nonce.as_ptr() as *const std::ffi::c_void,
                local_nonce.len(),
            )
        };
    }

    /// Output the hash value of the previous input
    /// and begin the calculation of the next hash.
    pub fn hash_next(&self, local_nonce: &[u8]) -> ResultHash {
        let mut hash = ResultHash::empty();

        unsafe {
            randomx_calculate_hash_next(
                self.vm,
                local_nonce.as_ptr() as *const std::ffi::c_void,
                local_nonce.len(),
                hash.as_raw_mut(),
            )
        };

        hash
    }

    /// Output the hash value of the previous input.
    pub fn hash_last(&self) -> ResultHash {
        let mut hash = ResultHash::empty();

        unsafe { randomx_calculate_hash_last(self.vm, hash.as_raw_mut()) };

        hash
    }
}

#[cfg(test)]
mod tests {
    use crate::{Cache, Dataset, RandomXFlags, RandomXVM};

    #[test]
    fn light_no_creates_with_full_mem() {
        let flags = RandomXFlags::recommended_full_mem();
        let cache = Cache::new(&[0, 1], flags).unwrap();
        let vm = RandomXVM::light(cache.handle(), flags);

        assert!(vm.is_err());
    }

    #[test]
    fn fast_no_creates_without_full_mem() {
        let flags = RandomXFlags::recommended();
        let dataset = Dataset::new(&[0, 1], flags).unwrap();
        let vm = RandomXVM::fast(dataset.handle(), flags);

        assert!(vm.is_err());
    }
}
