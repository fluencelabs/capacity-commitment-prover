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

use ccp_shared::types::*;

pub fn generate_epoch_params(nonce: u8, difficulty: u8) -> EpochParameters {
    let global_nonce = generate_global_nonce(nonce);
    let difficulty = generate_difficulty(difficulty);

    EpochParameters::new(global_nonce, difficulty)
}

pub fn generate_global_nonce(first_byte: u8) -> GlobalNonce {
    GlobalNonce::new([
        first_byte, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5, 6, 1, 2, 3, 2, 3, 3, 4, 2, 1, 4, 5, 6, 1, 2,
        3, 4, 6, 3, 2,
    ])
}

pub fn generate_local_nonce(first_byte: u8) -> LocalNonce {
    LocalNonce::new([
        first_byte, 2, 3, 4, 3, 4, 3, 1, 2, 4, 4, 5, 6, 1, 2, 3, 2, 3, 3, 4, 2, 1, 4, 5, 6, 1, 2,
        3, 4, 6, 3, 2,
    ])
}

pub fn generate_cu_id(first_byte: u8) -> CUID {
    CUID::new([
        first_byte, 2, 3, 4, 5, 6, 7, 8, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ])
}

pub fn generate_difficulty(difficulty: u8) -> Difficulty {
    Difficulty::new([
        0, difficulty, 3, 4, 3, 4, 3, 1, 2, 4, 4, 5, 6, 1, 2, 3, 2, 3, 3, 4, 2, 1, 4, 5, 6, 1, 2,
        3, 4, 6, 3, 2,
    ])
}

pub fn generate_allocation(cores: &[u8]) -> CUAllocation {
    cores
        .iter()
        .map(|core_id| {
            (
                PhysicalCoreId::from(*core_id as u32),
                generate_cu_id(*core_id),
            )
        })
        .collect()
}

pub fn generate_random_allocation(
    rng: &mut impl rand::Rng,
    size: usize,
    range: std::ops::Range<u8>,
) -> CUAllocation {
    use rand::Rng;
    let distr = rand::distributions::Uniform::from(range);
    rng.sample_iter(distr)
        .take(size as usize)
        .map(|core_id| {
            (
                PhysicalCoreId::from(core_id as u32),
                generate_cu_id(core_id),
            )
        })
        .collect()
}
