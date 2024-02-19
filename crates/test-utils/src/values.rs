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

pub fn create_global_nonce(first_byte: u8) -> GlobalNonce {
    GlobalNonce::new([
        first_byte, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5, 6, 1, 2, 3, 2, 3, 3, 4, 2, 1, 4, 5, 6, 1, 2, 3, 4, 6,
        3, 2,
    ])
}

pub fn create_local_nonce(first_byte: u8) -> LocalNonce {
    LocalNonce::new([
        first_byte, 2, 3, 4, 3, 4, 3, 1, 2, 4, 4, 5, 6, 1, 2, 3, 2, 3, 3, 4, 2, 1, 4, 5, 6, 1, 2, 3, 4, 6,
        3, 2,
    ])
}

pub fn create_cu_id(first_byte: u8) -> CUID {
    CUID::new([
        first_byte, 2, 4, 4, 1, 6, 0, 2, 2, 3, 4, 5, 6, 1, 2, 3, 2, 3, 3, 4, 2, 1, 4, 5, 6, 1, 2, 3, 4, 6,
        3, 2,
    ])
}

pub fn create_difficulty(first_byte: u8, second_byte: u8) -> Difficulty {
    Difficulty::new([
        first_byte, second_byte, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ])
}
