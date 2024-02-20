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
use std::collections::HashSet;

use ccp_shared::types::*;

use super::roadmap::CCProverAlignmentRoadmap;
use super::roadmap::CUProverPreAction;
use super::roadmap::CUProverAction;
use crate::cu::status::ToCUStatus;
use crate::status::CCStatus;

#[derive(Debug)]
pub(super) struct RoadmapBuilderState {
    is_new_epoch: bool,
    epoch: EpochParameters,
    unprepared_allocation_actions: Vec<(PhysicalCoreId, CUID)>,
    unprepared_removal_actions: Vec<PhysicalCoreId>,
    pre_actions: Vec<CUProverPreAction>,
    actions: Vec<CUProverAction>,
}

pub(super) struct RoadmapBuilder {}

impl RoadmapBuilder {
    pub(super) fn from(new_epoch: EpochParameters, current_status: CCStatus) -> BuilderFirstStage {
        let is_new_epoch = match current_status {
            CCStatus::Running { epoch } => new_epoch != epoch,
            CCStatus::Idle => true,
        };

        let state = RoadmapBuilderState {
            is_new_epoch,
            epoch: new_epoch,
            unprepared_allocation_actions: Vec::new(),
            unprepared_removal_actions: Vec::new(),
            pre_actions: Vec::new(),
            actions: Vec::new(),
        };

        let state = Self::clean_proofs_if_new_epoch(state);
        BuilderFirstStage::new(state)
    }

    fn clean_proofs_if_new_epoch(mut state: RoadmapBuilderState) -> RoadmapBuilderState {
        if state.is_new_epoch {
            state.pre_actions.push(CUProverPreAction::cleanup_proof_cache())
        }

        state
    }
}

pub(super) struct BuilderFirstStage {
    state: RoadmapBuilderState,
}

impl BuilderFirstStage {
    pub(super) fn new(state: RoadmapBuilderState) -> Self {
        BuilderFirstStage { state }
    }

    pub(super) fn collect_allocation_and_new_job_actions<Status: ToCUStatus>(
        mut self,
        new_allocation: CUAllocation,
        current_allocation: &HashMap<PhysicalCoreId, Status>,
    ) -> BuilderSecondStage {
        let mut remaining_cu_provides = HashSet::new();
        for (new_core_id, new_cu_id) in new_allocation.into_iter() {
            let status = match current_allocation.get(&new_core_id) {
                Some(status) => {
                    remaining_cu_provides.insert(new_core_id);
                    status
                }
                None => {
                    self.state
                        .unprepared_allocation_actions
                        .push((new_core_id, new_cu_id));
                    continue;
                }
            };

            self.maybe_update_cc_job(new_core_id, new_cu_id, status)
        }

        BuilderSecondStage::new(self.state, remaining_cu_provides)
    }

    fn maybe_update_cc_job<Status: ToCUStatus>(
        &mut self,
        core_id: PhysicalCoreId,
        new_cu_id: CUID,
        status: &Status,
    ) {
        if self.should_update_job(&new_cu_id, status) {
            self.state
                .actions
                .push(CUProverAction::new_cc_job(core_id, new_cu_id));
        }
    }

    fn should_update_job<Status: ToCUStatus>(&self, new_cu_id: &CUID, status: &Status) -> bool {
        use crate::cu::status::CUStatus;

        let current_cu_id = match status.status() {
            CUStatus::Running { cu_id } => cu_id,
            CUStatus::Idle => return true,
        };
        new_cu_id != &current_cu_id || self.state.is_new_epoch
    }
}

pub(super) struct BuilderSecondStage {
    pub(self) state: RoadmapBuilderState,
    // Provides that shouldn't be deleted
    pub(self) remaining_cu_providers: HashSet<PhysicalCoreId>,
}

impl BuilderSecondStage {
    pub(super) fn new(
        state: RoadmapBuilderState,
        remaining_cu_providers: HashSet<PhysicalCoreId>,
    ) -> Self {
        Self {
            state,
            remaining_cu_providers,
        }
    }

    pub(super) fn collect_removal_actions<Status>(
        self,
        current_allocation: &HashMap<PhysicalCoreId, Status>,
    ) -> BuilderThirdStage {
        let BuilderSecondStage {
            mut state,
            remaining_cu_providers,
        } = self;

        state.unprepared_removal_actions = current_allocation
            .keys()
            .filter(|current_core_id| remaining_cu_providers.get(current_core_id).is_none())
            .cloned()
            .collect::<Vec<_>>();

        BuilderThirdStage::new(state)
    }
}

pub(super) struct BuilderThirdStage {
    pub(self) state: RoadmapBuilderState,
}

impl BuilderThirdStage {
    pub(super) fn new(state: RoadmapBuilderState) -> Self {
        Self { state }
    }

    pub(super) fn substitute_removal_and_allocation_actions(self) -> BuilderFourthStage {
        let mut state = self.state;
        let allocation_actions = &mut state.unprepared_allocation_actions;
        let removal_actions = &mut state.unprepared_removal_actions;

        let substitute_actions_count =
            std::cmp::min(allocation_actions.len(), removal_actions.len());

        for _ in 0..substitute_actions_count {
            // it's save because length has been checked before
            let (new_core_id, new_cu_id) = allocation_actions.remove(allocation_actions.len() - 1);
            let current_core_id = removal_actions.remove(removal_actions.len() - 1);

            let action = CUProverAction::new_cc_job_repin(current_core_id, new_core_id, new_cu_id);
            state.actions.push(action);
        }

        BuilderFourthStage::new(state)
    }
}

pub(super) struct BuilderFourthStage {
    pub(self) state: RoadmapBuilderState,
}

impl BuilderFourthStage {
    pub(super) fn new(state: RoadmapBuilderState) -> Self {
        Self { state }
    }

    pub(super) fn prepare_allocation_actions(self) -> BuilderFifthStage {
        let mut state = self.state;
        for (core_id, cu_id) in state.unprepared_allocation_actions.iter() {
            let action = CUProverAction::create_cu_prover(*core_id, *cu_id);
            state.actions.push(action);
        }

        BuilderFifthStage::new(state)
    }
}

pub(super) struct BuilderFifthStage {
    pub(self) state: RoadmapBuilderState,
}

impl BuilderFifthStage {
    pub(super) fn new(state: RoadmapBuilderState) -> Self {
        Self { state }
    }

    pub(super) fn prepare_removal_actions(self) -> BuilderSixStage {
        let mut state = self.state;
        for &current_core_id in state.unprepared_removal_actions.iter() {
            let action = CUProverAction::remove_cu_prover(current_core_id);
            state.actions.push(action)
        }

        BuilderSixStage::new(state)
    }
}

pub(super) struct BuilderSixStage {
    pub(self) state: RoadmapBuilderState,
}

impl BuilderSixStage {
    pub(super) fn new(state: RoadmapBuilderState) -> Self {
        Self { state }
    }

    pub(super) fn build(self) -> CCProverAlignmentRoadmap {
        CCProverAlignmentRoadmap {
            actions: self.state.actions,
            epoch: self.state.epoch,
        }
    }
}
