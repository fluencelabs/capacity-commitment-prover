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

pub(crate) mod actions_state;
use std::collections::HashMap;

use ccp_shared::types::*;

use super::roadmap_builder::RoadmapBuilder;
use crate::cu::status::ToCUStatus;
use crate::status::CCStatus;

use actions_state::*;

/// Intended to align state between Nox and CCP, contains a set of actions, which
/// being applied to a CCP state, make it aligned with the Nox state.
#[derive(Debug, Eq)]
pub(crate) struct CCProverAlignmentRoadmap {
    pub(crate) pre_action: CUProverPreAction,
    pub(crate) actions: Vec<CUProverAction>,
    pub(crate) epoch: EpochParameters,
}

impl CCProverAlignmentRoadmap {
    pub(crate) fn create_roadmap<T: ToCUStatus>(
        new_allocation: CUAllocation,
        new_epoch: EpochParameters,
        current_allocation: &HashMap<PhysicalCoreId, T>,
        current_status: CCStatus,
    ) -> CCProverAlignmentRoadmap {
        RoadmapBuilder::from(new_epoch, current_status)
            .collect_allocation_and_new_job_actions(new_allocation, current_allocation)
            .collect_removal_actions(current_allocation)
            .substitute_removal_and_allocation_actions()
            .prepare_allocation_actions()
            .prepare_removal_actions()
            .build()
    }
}

impl PartialEq for CCProverAlignmentRoadmap {
    fn eq(&self, other: &Self) -> bool {
        use std::collections::HashSet;

        if self.epoch != other.epoch {
            return false;
        }

        let self_actions = self.actions.iter().collect::<HashSet<_>>();
        let other_actions = other.actions.iter().collect::<HashSet<_>>();
        self_actions == other_actions
    }
}

/// This action intended to align CCP with an incoming CU allocation from Nox.
/// The action will be made before CUProverAction on stopped provers.
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum CUProverPreAction {
    NoAction,
    /// Signals CCP to remove all collected proofs, this action is a result of epoch switching
    /// and CCP will clean up old proofs to save space.
    CleanupProofCache,
}

/// A single action intended to align CCP with an incoming CU allocation from Nox.
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum CUProverAction {
    /// Encourage CCP to creates a new CU prover, intended to prove that the physical core with
    /// new_core_id core id registered with the provided cu_id participates in network during epoch.
    CreateCUProver(CreateCUProverState),

    /// Encourage CCP to remove CU prover with the given current_core_id.
    RemoveCUProver(RemoveCUProverState),

    /// Encourage CCP to run a new CC job on CU with the same cu_id,
    /// identified by the supplied current_core_id.
    /// Epoch parameters for the CC job will be taken from CCProverAlignmentRoadmap::epoch.
    NewCCJob(NewCCJobState),

    /// There was already created CUProver, but allocated on a not appropriate core,
    /// this actions tells CCP to repin the prover and run a new CC job on it.
    /// Epoch parameters for the CC job will be taken from CCProverAlignmentRoadmap::epoch.
    NewCCJobWithRepining(NewCCJobWithRepiningState),
}

impl CUProverAction {
    pub(crate) fn create_cu_prover(new_core_id: PhysicalCoreId, new_cu_id: CUID) -> Self {
        Self::CreateCUProver(CreateCUProverState::new(new_core_id, new_cu_id))
    }

    pub(crate) fn remove_cu_prover(current_core_id: PhysicalCoreId) -> Self {
        Self::RemoveCUProver(RemoveCUProverState::new(current_core_id))
    }

    pub(crate) fn new_cc_job(current_core_id: PhysicalCoreId, new_cu_id: CUID) -> Self {
        Self::NewCCJob(NewCCJobState::new(current_core_id, new_cu_id))
    }

    pub(crate) fn new_cc_job_repin(
        current_core_id: PhysicalCoreId,
        new_core_id: PhysicalCoreId,
        new_cu_id: CUID,
    ) -> Self {
        Self::NewCCJobWithRepining(NewCCJobWithRepiningState::new(
            current_core_id,
            new_core_id,
            new_cu_id,
        ))
    }
}

impl CUProverPreAction {
    pub(crate) fn cleanup_proof_cache() -> Self {
        Self::CleanupProofCache
    }

    pub(crate) fn no_action() -> Self {
        Self::NoAction
    }
}
