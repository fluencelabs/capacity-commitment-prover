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

pub mod cu_prover;
mod errors;
mod proving_thread;
mod proving_thread_utils;
pub(crate) mod status;

pub(crate) use cu_prover::CUProver;
pub(crate) use cu_prover::CUProverConfig;
pub(crate) use errors::CUProverError;
pub(crate) use errors::ThreadAllocationError;
pub(crate) use proving_thread::RawProof;

pub(crate) type CUResult<T> = Result<T, errors::CUProverError>;
