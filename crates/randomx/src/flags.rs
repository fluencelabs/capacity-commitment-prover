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

use bitflags::bitflags;

use crate::bindings::flags::*;

bitflags! {
    /// Flags to configure RandomX behaviour.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct RandomXFlags: u32 {
        const DEFAULT = randomx_flags_RANDOMX_FLAG_DEFAULT;
        const LARGE_PAGES = randomx_flags_RANDOMX_FLAG_LARGE_PAGES;
        const HARD_AES = randomx_flags_RANDOMX_FLAG_HARD_AES;
        const FULL_MEM = randomx_flags_RANDOMX_FLAG_FULL_MEM;
        const FLAG_JIT = randomx_flags_RANDOMX_FLAG_JIT;
        const FLAG_SECURE = randomx_flags_RANDOMX_FLAG_SECURE;
        const FLAG_ARGON2_SSSE3 = randomx_flags_RANDOMX_FLAG_ARGON2_SSSE3;
        const FLAG_ARGON2_AVX2 = randomx_flags_RANDOMX_FLAG_ARGON2_AVX2;
        const FLAG_ARGON2 = randomx_flags_RANDOMX_FLAG_ARGON2;
    }
}

impl RandomXFlags {
    pub fn is_fast_mode(&self) -> bool {
        self.contains(RandomXFlags::FULL_MEM)
    }

    pub fn is_light_mode(&self) -> bool {
        !self.is_fast_mode()
    }

    pub fn is_large_pages(&self) -> bool {
        self.contains(RandomXFlags::LARGE_PAGES)
    }

    /// (from the RandomX doc) Returns the recommended flags to be used.
    ///
    /// Does not include:
    /// * FLAG_LARGE_PAGES
    /// * FLAG_FULL_MEM
    /// * FLAG_SECURE
    ///
    /// The above flags need to be set manually, if required.
    pub fn recommended() -> Self {
        let recommended = unsafe { randomx_get_flags() };

        // this unwrap is safe b/c the randomx_get_flags function will return only
        // existing flags
        RandomXFlags::from_bits(recommended).unwrap()
    }

    pub fn recommended_full_mem() -> Self {
        let mut recommended = Self::recommended();
        recommended.insert(RandomXFlags::FULL_MEM);

        recommended
    }
}

impl Default for RandomXFlags {
    fn default() -> RandomXFlags {
        RandomXFlags::DEFAULT
    }
}
