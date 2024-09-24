/*
 * Copyright 2024 Fluence DAO
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

mod chunk;
mod config;
pub mod cu_prover;
mod errors;
pub mod hashrate_record;
mod memory_dumper;
pub mod proving_thread;
mod proving_thread_utils;
pub mod status;
#[cfg(test)]
mod tests;

pub use crate::proving_thread::sync::RawProof;
pub use config::CUProverConfig;
pub use cu_prover::CUProver;
pub use errors::CUProverError;
pub use errors::ThreadAllocationError;
pub use hashrate_record::HashrateRecordType;
pub use hashrate_record::ThreadHashrateRecord;

pub type CUResult<T> = Result<T, errors::CUProverError>;

pub(crate) use ccp_shared::types::*;

pub use anyhow::Error as AnyhowError;
