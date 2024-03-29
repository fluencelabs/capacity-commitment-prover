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

mod core;
mod cuid;
mod difficulty;
mod epoch_parameters;
mod global_nonce;
mod local_nonce;
mod result_hash;

use std::collections::HashMap;

pub use core::CPUIdType;
pub use core::LogicalCoreId;
pub use core::PhysicalCoreId;
pub use cuid::CUIDInner;
pub use cuid::CUID;
pub use difficulty::Difficulty;
pub use difficulty::DifficultyInner;
pub use epoch_parameters::EpochParameters;
pub use global_nonce::GlobalNonce;
pub use global_nonce::GlobalNonceInner;
pub use local_nonce::LocalNonce;
pub use local_nonce::LocalNonceInner;
pub use result_hash::ResultHash;
pub use result_hash::ResultHashInner;
pub use result_hash::RANDOMX_RESULT_SIZE;

pub type CUAllocation = HashMap<PhysicalCoreId, CUID>;
