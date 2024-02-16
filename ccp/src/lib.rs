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

mod alignment_roadmap;
mod cu;
mod epoch;
mod errors;
mod proof_storage_worker;
pub mod prover;
pub mod status;

pub use errors::CCProverError;
pub use prover::CCProver;
pub use prover::CCResult;

pub(crate) use ccp_shared::types::*;
pub(crate) type LogicalCoreId = usize;
