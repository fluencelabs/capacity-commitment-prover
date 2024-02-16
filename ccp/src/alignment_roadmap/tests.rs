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

use crate::alignment_roadmap::CUProverAction;
use ccp_shared::types::CUAllocation;
use ccp_shared::types::GlobalNonce;
use ccp_shared::types::LocalNonce;
use ccp_shared::types::CUID;

use super::CCProverAlignmentRoadmap;
use crate::cu::running_status::RunningStatus;
use crate::cu::running_status::ToRunningStatus;
use crate::epoch::Epoch;

fn test_cu_id(id: u8) -> CUID {
    [
        id, 2, 3, 4, 5, 6, 7, 8, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ]
}

fn test_global_nonce(id: u8) -> GlobalNonce {
    [
        id, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5, 6, 1, 2, 3, 2, 3, 3, 4, 2, 1, 4, 5, 6, 1, 2, 3, 4, 6,
        3, 2,
    ]
}

fn test_local_nonce(id: u8) -> LocalNonce {
    [
        id, 2, 3, 4, 3, 4, 3, 1, 2, 4, 4, 5, 6, 1, 2, 3, 2, 3, 3, 4, 2, 1, 4, 5, 6, 1, 2, 3, 4, 6,
        3, 2,
    ]
}

fn test_difficulty(id: u8) -> LocalNonce {
    [
        0, id, 3, 4, 3, 4, 3, 1, 2, 4, 4, 5, 6, 1, 2, 3, 2, 3, 3, 4, 2, 1, 4, 5, 6, 1, 2, 3, 4, 6,
        3, 2,
    ]
}

struct DumpProvider {
    pub(self) status: RunningStatus,
}

impl DumpProvider {
    pub(self) fn idle() -> Self {
        Self {
            status: RunningStatus::Idle,
        }
    }

    pub(self) fn running(cu_id: CUID) -> Self {
        Self {
            status: RunningStatus::Running { cu_id },
        }
    }
}

impl ToRunningStatus for DumpProvider {
    fn to_status(&self) -> RunningStatus {
        self.status
    }
}

#[test]
fn alignment_works_if_current_state_empty() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test_cu_id(1));
    new_allocation.insert(allocation_1.0, allocation_1.1);

    let allocation_2 = (2, test_cu_id(2));
    new_allocation.insert(allocation_2.0, allocation_2.1);

    let allocation_3 = (3, test_cu_id(3));
    new_allocation.insert(allocation_3.0, allocation_3.1);

    let new_epoch = Epoch::new(test_local_nonce(1), test_difficulty(1));

    let current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    let current_epoch = Epoch::new(test_local_nonce(2), test_difficulty(1));

    let actual_roadmap = CCProverAlignmentRoadmap::create_roadmap(
        new_allocation,
        new_epoch,
        &current_allocation,
        current_epoch,
    );
    let expected_actions = vec![
        CUProverAction::clean_proof_cache(),
        CUProverAction::create_cu_prover(allocation_1.0, allocation_1.1),
        CUProverAction::create_cu_prover(allocation_2.0, allocation_2.1),
        CUProverAction::create_cu_prover(allocation_3.0, allocation_3.1),
    ];
    let expected_roadmap = CCProverAlignmentRoadmap {
        actions: expected_actions,
        epoch: new_epoch,
    };

    assert_eq!(actual_roadmap, expected_roadmap);
}

#[test]
fn applying_same_roadmap_idempotent() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test_cu_id(1));
    new_allocation.insert(allocation_1.0, allocation_1.1);

    let allocation_2 = (2, test_cu_id(2));
    new_allocation.insert(allocation_2.0, allocation_2.1);

    let epoch = Epoch::new(test_local_nonce(1), test_difficulty(1));

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0, DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0, DumpProvider::running(allocation_2.1));

    let actual_roadmap = CCProverAlignmentRoadmap::create_roadmap(
        new_allocation.clone(),
        epoch,
        &current_allocation,
        epoch,
    );

    assert!(actual_roadmap.actions.is_empty());
}

#[test]
fn add_new_peer() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test_cu_id(1));
    new_allocation.insert(allocation_1.0, allocation_1.1);

    let allocation_2 = (2, test_cu_id(2));
    new_allocation.insert(allocation_2.0, allocation_2.1);

    let allocation_3 = (3, test_cu_id(3));
    new_allocation.insert(allocation_3.0, allocation_3.1);

    let epoch = Epoch::new(test_local_nonce(1), test_difficulty(1));

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0, DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0, DumpProvider::running(allocation_2.1));

    let actual_roadmap = CCProverAlignmentRoadmap::create_roadmap(
        new_allocation.clone(),
        epoch,
        &current_allocation,
        epoch,
    );

    let expected_actions = vec![CUProverAction::create_cu_prover(
        allocation_3.0,
        allocation_3.1,
    )];
    let expected_roadmap = CCProverAlignmentRoadmap {
        actions: expected_actions,
        epoch,
    };

    assert_eq!(actual_roadmap, expected_roadmap);
}

#[test]
fn remove_peer() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test_cu_id(1));
    new_allocation.insert(allocation_1.0, allocation_1.1);

    let allocation_2 = (2, test_cu_id(2));
    new_allocation.insert(allocation_2.0, allocation_2.1);

    let epoch = Epoch::new(test_local_nonce(1), test_difficulty(1));

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0, DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0, DumpProvider::running(allocation_2.1));
    let allocation_3 = (3, test_cu_id(3));
    current_allocation.insert(allocation_3.0, DumpProvider::running(allocation_3.1));

    let actual_roadmap = CCProverAlignmentRoadmap::create_roadmap(
        new_allocation.clone(),
        epoch,
        &current_allocation,
        epoch,
    );

    let expected_actions = vec![CUProverAction::remove_cu_prover(allocation_3.0)];
    let expected_roadmap = CCProverAlignmentRoadmap {
        actions: expected_actions,
        epoch,
    };

    assert_eq!(actual_roadmap, expected_roadmap);
}

#[test]
fn new_epoch() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test_cu_id(1));
    new_allocation.insert(allocation_1.0, allocation_1.1);

    let allocation_2 = (2, test_cu_id(2));
    new_allocation.insert(allocation_2.0, allocation_2.1);

    let allocation_3 = (3, test_cu_id(3));
    new_allocation.insert(allocation_3.0, allocation_3.1);

    let new_epoch = Epoch::new(test_local_nonce(2), test_difficulty(1));

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0, DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0, DumpProvider::running(allocation_2.1));
    current_allocation.insert(allocation_3.0, DumpProvider::running(allocation_3.1));
    let current_epoch = Epoch::new(test_local_nonce(1), test_difficulty(1));

    let actual_roadmap = CCProverAlignmentRoadmap::create_roadmap(
        new_allocation.clone(),
        new_epoch,
        &current_allocation,
        current_epoch,
    );

    let expected_actions = vec![
        CUProverAction::clean_proof_cache(),
        CUProverAction::new_cc_job(allocation_1.0, allocation_1.1),
        CUProverAction::new_cc_job(allocation_2.0, allocation_2.1),
        CUProverAction::new_cc_job(allocation_3.0, allocation_3.1),
    ];
    let expected_roadmap = CCProverAlignmentRoadmap {
        actions: expected_actions,
        epoch: new_epoch,
    };

    assert_eq!(actual_roadmap, expected_roadmap);
}

#[test]
fn same_epoch_new_jobs() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test_cu_id(1));
    let allocation_2 = (2, test_cu_id(2));
    let allocation_3 = (3, test_cu_id(3));

    new_allocation.insert(allocation_1.0, allocation_2.1);
    new_allocation.insert(allocation_2.0, allocation_3.1);
    new_allocation.insert(allocation_3.0, allocation_1.1);

    let epoch = Epoch::new(test_local_nonce(2), test_difficulty(1));

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0, DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0, DumpProvider::running(allocation_2.1));
    current_allocation.insert(allocation_3.0, DumpProvider::running(allocation_3.1));

    let actual_roadmap = CCProverAlignmentRoadmap::create_roadmap(
        new_allocation.clone(),
        epoch,
        &current_allocation,
        epoch,
    );

    let expected_actions = vec![
        CUProverAction::new_cc_job(allocation_1.0, allocation_2.1),
        CUProverAction::new_cc_job(allocation_2.0, allocation_3.1),
        CUProverAction::new_cc_job(allocation_3.0, allocation_1.1),
    ];
    let expected_roadmap = CCProverAlignmentRoadmap {
        actions: expected_actions,
        epoch,
    };

    assert_eq!(actual_roadmap, expected_roadmap);
}

#[test]
fn repinning_works() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test_cu_id(1));
    let allocation_2 = (2, test_cu_id(2));
    let allocation_3 = (3, test_cu_id(3));
    let allocation_4 = (4, test_cu_id(4));

    new_allocation.insert(allocation_2.0, allocation_2.1);
    new_allocation.insert(allocation_3.0, allocation_3.1);
    new_allocation.insert(allocation_4.0, allocation_4.1);

    let epoch = Epoch::new(test_local_nonce(2), test_difficulty(1));

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0, DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0, DumpProvider::running(allocation_2.1));
    current_allocation.insert(allocation_3.0, DumpProvider::running(allocation_3.1));

    let actual_roadmap = CCProverAlignmentRoadmap::create_roadmap(
        new_allocation.clone(),
        epoch,
        &current_allocation,
        epoch,
    );

    let expected_actions = vec![CUProverAction::new_cc_job_repin(
        allocation_1.0,
        allocation_4.0,
        allocation_4.1,
    )];
    let expected_roadmap = CCProverAlignmentRoadmap {
        actions: expected_actions,
        epoch,
    };

    assert_eq!(actual_roadmap, expected_roadmap);
}

#[test]
fn create_more_then_remove() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test_cu_id(1));
    let allocation_2 = (2, test_cu_id(2));
    let allocation_3 = (3, test_cu_id(3));
    let allocation_4 = (4, test_cu_id(4));
    let allocation_5 = (5, test_cu_id(5));

    new_allocation.insert(allocation_2.0, allocation_2.1);
    new_allocation.insert(allocation_3.0, allocation_3.1);
    new_allocation.insert(allocation_4.0, allocation_4.1);
    new_allocation.insert(allocation_5.0, allocation_5.1);

    let epoch = Epoch::new(test_local_nonce(2), test_difficulty(1));

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0, DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0, DumpProvider::running(allocation_2.1));
    current_allocation.insert(allocation_3.0, DumpProvider::running(allocation_3.1));

    let actual_roadmap = CCProverAlignmentRoadmap::create_roadmap(
        new_allocation.clone(),
        epoch,
        &current_allocation,
        epoch,
    );

    let expected_actions_1 = vec![
        CUProverAction::new_cc_job_repin(allocation_1.0, allocation_4.0, allocation_4.1),
        CUProverAction::create_cu_prover(allocation_5.0, allocation_5.1),
    ];
    let expected_roadmap_1 = CCProverAlignmentRoadmap {
        actions: expected_actions_1,
        epoch,
    };

    let expected_actions_2 = vec![
        CUProverAction::new_cc_job_repin(allocation_1.0, allocation_5.0, allocation_5.1),
        CUProverAction::create_cu_prover(allocation_4.0, allocation_4.1),
    ];
    let expected_roadmap_2 = CCProverAlignmentRoadmap {
        actions: expected_actions_2,
        epoch,
    };

    // we can assign core 1 to the 4th task and remove 5th or
    //        assign core 1 to the 5th task and remove 4th
    assert!((actual_roadmap == expected_roadmap_1) || (actual_roadmap == expected_roadmap_2));
}

#[test]
fn remove_more_then_create() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test_cu_id(1));
    let allocation_2 = (2, test_cu_id(2));
    let allocation_3 = (3, test_cu_id(3));
    let allocation_4 = (4, test_cu_id(4));

    new_allocation.insert(allocation_3.0, allocation_3.1);
    new_allocation.insert(allocation_4.0, allocation_4.1);

    let epoch = Epoch::new(test_local_nonce(2), test_difficulty(1));

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0, DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0, DumpProvider::running(allocation_2.1));
    current_allocation.insert(allocation_3.0, DumpProvider::running(allocation_3.1));

    let actual_roadmap = CCProverAlignmentRoadmap::create_roadmap(
        new_allocation.clone(),
        epoch,
        &current_allocation,
        epoch,
    );

    let expected_actions_1 = vec![
        CUProverAction::new_cc_job_repin(allocation_1.0, allocation_4.0, allocation_4.1),
        CUProverAction::remove_cu_prover(allocation_2.0),
    ];
    let expected_roadmap_1 = CCProverAlignmentRoadmap {
        actions: expected_actions_1,
        epoch,
    };

    let expected_actions_2 = vec![
        CUProverAction::new_cc_job_repin(allocation_2.0, allocation_4.0, allocation_4.1),
        CUProverAction::remove_cu_prover(allocation_1.0),
    ];
    let expected_roadmap_2 = CCProverAlignmentRoadmap {
        actions: expected_actions_2,
        epoch,
    };

    // we can assign prover 1 to the 4th task and remove the 2nd prover or
    //        assign prover 2 to the 4th task and remove the 1st prover
    assert!((actual_roadmap == expected_roadmap_1) || (actual_roadmap == expected_roadmap_2));
}
