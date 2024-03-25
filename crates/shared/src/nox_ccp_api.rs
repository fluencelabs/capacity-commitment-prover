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

use crate::proof::ProofIdx;

use super::proof::CCProof;
use super::types::*;

pub trait NoxCCPApi: Send {
    type Error: Send;

    /// Aims to apply a (new) allocation of CC compute jobs (which are threads with
    /// particular global_nonce_cu + difficulty) per physical_core_id.
    ///
    /// It checks if a job already exists, in this case it doesn't recreate it,
    /// but probably pin to another physical core according to the supplied allocation map.
    /// If there is no such job, it creates,
    /// if jobs are outdated (global_nonce_cu + difficulty aren't the same as supplied),
    /// then it restart them all with the provided parameters.
    ///
    /// All other cores which aren't defined in the supplied map is supposed to manage
    /// "useful" job in workers and existing jobs for them will be released.
    ///
    /// This function is supposed to be called on any of the following situations:
    ///  - new global nonce or difficulty (== new epoch)
    ///  - CU allocation is changed (e.g. a core is assigned or released to CC)
    ///    after event from the on-chain part
    ///  - Nox (re)started
    fn on_active_commitment(
        &self,
        epoch_parameters: EpochParameters,
        cu_allocation: CUAllocation,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Stops all active jobs.
    fn on_no_active_commitment(
        &self,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;

    /// Returns proofs after the provided proof idx for current epoch.
    fn get_proofs_after(
        &self,
        proof_idx: ProofIdx,
    ) -> impl std::future::Future<Output = Result<Vec<CCProof>, Self::Error>> + Send;

    /// Set utility
    fn realloc_utility_cores(
        &self,
        utility_core_ids: Vec<LogicalCoreId>,
    ) -> impl std::future::Future<Output = ()> + Send;
}
