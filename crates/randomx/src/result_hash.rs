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

pub use ccp_shared::types::ResultHash;
pub use ccp_shared::types::RANDOMX_RESULT_SIZE;

pub(crate) trait ToRawMut {
    fn empty() -> Self;

    fn as_raw_mut(&mut self) -> *mut std::ffi::c_void;
}

impl ToRawMut for ResultHash {
    fn empty() -> Self {
        ResultHash::from_slice([0u8; RANDOMX_RESULT_SIZE])
    }

    fn as_raw_mut(&mut self) -> *mut std::ffi::c_void {
        self.as_mut().as_mut_ptr() as *mut std::ffi::c_void
    }
}
