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

use std::collections::HashMap;

use crate::RANDOMX_RESULT_SIZE;

pub type GlobalNonce = [u8; 32];
pub type LocalNonce = [u8; 32];
pub type Difficulty = [u8; RANDOMX_RESULT_SIZE];
pub type CUID = [u8; 32];
pub use cpu_topology::LogicalCoreId;
pub use cpu_topology::PhysicalCoreId;
pub type CUAllocation = HashMap<PhysicalCoreId, CUID>;
