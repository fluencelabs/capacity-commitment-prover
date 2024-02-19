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

use sha3::digest::core_api::CoreWrapper;
use sha3::digest::Output;
use sha3::Keccak256Core;

use ccp_shared::types::GlobalNonce;
use ccp_shared::types::CUID;

/// Computes global nonce specific for a particular CU by
/// keccak(global_nonce + cu_id)
pub fn compute_global_nonce_cu(
    global_nonce: &GlobalNonce,
    cu_id: &CUID,
) -> Output<CoreWrapper<Keccak256Core>> {
    use sha3::Digest;

    let mut hasher = sha3::Keccak256::new();
    hasher.update(global_nonce.as_ref());
    hasher.update(cu_id.as_ref());

    hasher.finalize()
}
