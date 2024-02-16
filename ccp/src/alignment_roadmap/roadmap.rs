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

use ccp_shared::types::*;

use super::roadmap_builder::RoadmapBuilder;
use crate::cu::running_status::ToRunningStatus;
use crate::epoch::Epoch;

/// Intended to align state between Nox and CCP, contains a set of actions, which
/// being applied to a CCP state, make it aligned with the Nox state.
#[derive(Debug, Eq)]
pub(crate) struct CCProverAlignmentRoadmap {
    pub(crate) actions: Vec<CUProverAction>,
    pub(crate) epoch: Epoch,
}

impl CCProverAlignmentRoadmap {
    pub(crate) fn create_roadmap<T: ToRunningStatus>(
        new_allocation: CUAllocation,
        new_epoch: Epoch,
        current_allocation: &HashMap<PhysicalCoreId, T>,
        current_epoch: Epoch,
    ) -> CCProverAlignmentRoadmap {
        RoadmapBuilder::new(new_epoch, current_epoch)
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

/// A single action intended to align CCP with an incoming CU allocation from Nox.
#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum CUProverAction {
    /// Encourage CCP to creates a new CU prover, intended to prove that the physical core with
    /// new_core_id core id registered with the provided cu_id participates in network during epoch.
    CreateCUProver {
        new_core_id: PhysicalCoreId,
        new_cu_id: CUID,
    },

    /// Encourage CCP to remove CU prover with the given current_core_id.
    RemoveCUProver { current_core_id: PhysicalCoreId },

    /// Encourage CCP to run a new CC job on CU with the same cu_id,
    /// identified by the supplied current_core_id.
    /// Epoch parameters for the CC job will be taken from CCProverAlignmentRoadmap::epoch.
    NewCCJob {
        current_core_id: PhysicalCoreId,
        new_cu_id: CUID,
    },

    /// There was already created CUProver, but allocated on a not appropriate core,
    /// this actions tells CCP to repin the prover and run a new CC job on it.
    /// Epoch parameters for the CC job will be taken from CCProverAlignmentRoadmap::epoch.
    NewCCJobWithRepining {
        current_core_id: PhysicalCoreId,
        new_core_id: PhysicalCoreId,
        new_cu_id: CUID,
    },

    /// Signals CCP to remove all collected proofs, this action is a result of epoch switching
    /// and CCP will clean up old proofs to save space.
    CleanProofCache,
}

impl CUProverAction {
    pub(crate) fn create_cu_prover(new_core_id: PhysicalCoreId, cu_id: CUID) -> Self {
        Self::CreateCUProver {
            new_core_id,
            new_cu_id: cu_id,
        }
    }

    pub(crate) fn remove_cu_prover(current_core_id: PhysicalCoreId) -> Self {
        Self::RemoveCUProver { current_core_id }
    }

    pub(crate) fn new_cc_job(current_core_id: PhysicalCoreId, cu_id: CUID) -> Self {
        Self::NewCCJob {
            current_core_id,
            new_cu_id: cu_id,
        }
    }

    pub(crate) fn new_cc_job_repin(
        current_core_id: PhysicalCoreId,
        new_core_id: PhysicalCoreId,
        cu_id: CUID,
    ) -> Self {
        Self::NewCCJobWithRepining {
            current_core_id,
            new_core_id,
            new_cu_id: cu_id,
        }
    }

    pub(crate) fn clean_proof_cache() -> Self {
        Self::CleanProofCache
    }
}
