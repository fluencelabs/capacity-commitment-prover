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

use rand::rngs::SmallRng;
use rand::Rng;

use ccp_shared::types::CUAllocation;
use ccp_shared::types::PhysicalCoreId;
use ccp_shared::types::CUID;
use ccp_test_utils::test_values as test;

use super::CCProverAlignmentRoadmap;
use crate::alignment_roadmap::CUProverAction;
use crate::alignment_roadmap::CUProverPreAction;
use crate::cu::status::CUStatus;
use crate::cu::status::ToCUStatus;
use crate::status::CCStatus;

struct DumpProvider {
    pub(self) status: CUStatus,
}

impl DumpProvider {
    pub(self) fn running(cu_id: CUID) -> Self {
        Self {
            status: CUStatus::Running { cu_id },
        }
    }
}

impl ToCUStatus for DumpProvider {
    fn status(&self) -> CUStatus {
        self.status
    }
}

#[test]
fn alignment_works_if_prover_idle() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test::generate_cu_id(1));
    new_allocation.insert(allocation_1.0.into(), allocation_1.1);

    let allocation_2 = (2, test::generate_cu_id(2));
    new_allocation.insert(allocation_2.0.into(), allocation_2.1);

    let allocation_3 = (3, test::generate_cu_id(3));
    new_allocation.insert(allocation_3.0.into(), allocation_3.1);

    let new_epoch = test::generate_epoch_params(1, 1);
    let current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    let current_status = CCStatus::Idle;

    let actual_roadmap = CCProverAlignmentRoadmap::make(
        new_allocation,
        new_epoch,
        &current_allocation,
        current_status,
    );
    let pre_action = CUProverPreAction::cleanup_proof_cache();
    let expected_actions = vec![
        CUProverAction::create_cu_prover(allocation_1.0.into(), allocation_1.1),
        CUProverAction::create_cu_prover(allocation_2.0.into(), allocation_2.1),
        CUProverAction::create_cu_prover(allocation_3.0.into(), allocation_3.1),
    ];
    let expected_roadmap = CCProverAlignmentRoadmap {
        pre_action,
        actions: expected_actions,
        epoch: new_epoch,
    };

    assert_eq!(actual_roadmap, expected_roadmap);
}

#[test]
fn applying_same_roadmap_idempotent() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test::generate_cu_id(1));
    new_allocation.insert(allocation_1.0.into(), allocation_1.1);

    let allocation_2 = (2, test::generate_cu_id(2));
    new_allocation.insert(allocation_2.0.into(), allocation_2.1);

    let epoch = test::generate_epoch_params(1, 1);
    let current_status = CCStatus::Running { epoch };

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0.into(), DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0.into(), DumpProvider::running(allocation_2.1));

    let actual_roadmap = CCProverAlignmentRoadmap::make(
        new_allocation.clone(),
        epoch,
        &current_allocation,
        current_status,
    );

    assert!(actual_roadmap.actions.is_empty());
}

#[test]
fn add_new_peer() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test::generate_cu_id(1));
    new_allocation.insert(allocation_1.0.into(), allocation_1.1);

    let allocation_2 = (2, test::generate_cu_id(2));
    new_allocation.insert(allocation_2.0.into(), allocation_2.1);

    let allocation_3 = (3, test::generate_cu_id(3));
    new_allocation.insert(allocation_3.0.into(), allocation_3.1);

    let epoch = test::generate_epoch_params(1, 1);
    let current_status = CCStatus::Running { epoch };

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0.into(), DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0.into(), DumpProvider::running(allocation_2.1));

    let actual_roadmap = CCProverAlignmentRoadmap::make(
        new_allocation.clone(),
        epoch,
        &current_allocation,
        current_status,
    );

    let pre_action = CUProverPreAction::cleanup_proof_cache();
    let expected_actions = vec![CUProverAction::create_cu_prover(
        allocation_3.0.into(),
        allocation_3.1,
    )];
    let expected_roadmap = CCProverAlignmentRoadmap {
        pre_action,
        actions: expected_actions,
        epoch,
    };

    assert_eq!(actual_roadmap, expected_roadmap);
}

#[test]
fn remove_peer() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test::generate_cu_id(1));
    new_allocation.insert(allocation_1.0.into(), allocation_1.1);

    let allocation_2 = (2, test::generate_cu_id(2));
    new_allocation.insert(allocation_2.0.into(), allocation_2.1);

    let epoch = test::generate_epoch_params(1, 1);
    let current_status = CCStatus::Running { epoch };

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0.into(), DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0.into(), DumpProvider::running(allocation_2.1));
    let allocation_3 = (3, test::generate_cu_id(3));
    current_allocation.insert(allocation_3.0.into(), DumpProvider::running(allocation_3.1));

    let actual_roadmap = CCProverAlignmentRoadmap::make(
        new_allocation.clone(),
        epoch,
        &current_allocation,
        current_status,
    );

    let pre_action = CUProverPreAction::cleanup_proof_cache();
    let expected_actions = vec![CUProverAction::remove_cu_prover(allocation_3.0.into())];
    let expected_roadmap = CCProverAlignmentRoadmap {
        pre_action,
        actions: expected_actions,
        epoch,
    };

    assert_eq!(actual_roadmap, expected_roadmap);
}

#[test]
fn new_epoch() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test::generate_cu_id(1));
    new_allocation.insert(allocation_1.0.into(), allocation_1.1);

    let allocation_2 = (2, test::generate_cu_id(2));
    new_allocation.insert(allocation_2.0.into(), allocation_2.1);

    let allocation_3 = (3, test::generate_cu_id(3));
    new_allocation.insert(allocation_3.0.into(), allocation_3.1);

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0.into(), DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0.into(), DumpProvider::running(allocation_2.1));
    current_allocation.insert(allocation_3.0.into(), DumpProvider::running(allocation_3.1));
    let current_epoch = test::generate_epoch_params(1, 1);
    let current_status = CCStatus::Running {
        epoch: current_epoch,
    };

    let new_epoch = test::generate_epoch_params(2, 1);
    let actual_roadmap = CCProverAlignmentRoadmap::make(
        new_allocation.clone(),
        new_epoch,
        &current_allocation,
        current_status,
    );

    let pre_action = CUProverPreAction::cleanup_proof_cache();
    let expected_actions = vec![
        CUProverAction::new_cc_job(allocation_1.0.into(), allocation_1.1),
        CUProverAction::new_cc_job(allocation_2.0.into(), allocation_2.1),
        CUProverAction::new_cc_job(allocation_3.0.into(), allocation_3.1),
    ];
    let expected_roadmap = CCProverAlignmentRoadmap {
        pre_action,
        actions: expected_actions,
        epoch: new_epoch,
    };

    assert_eq!(actual_roadmap, expected_roadmap);
}

#[test]
fn same_epoch_new_jobs() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test::generate_cu_id(1));
    let allocation_2 = (2, test::generate_cu_id(2));
    let allocation_3 = (3, test::generate_cu_id(3));

    new_allocation.insert(allocation_1.0.into(), allocation_2.1);
    new_allocation.insert(allocation_2.0.into(), allocation_3.1);
    new_allocation.insert(allocation_3.0.into(), allocation_1.1);

    let epoch = test::generate_epoch_params(2, 1);
    let current_status = CCStatus::Running { epoch };

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0.into(), DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0.into(), DumpProvider::running(allocation_2.1));
    current_allocation.insert(allocation_3.0.into(), DumpProvider::running(allocation_3.1));

    let actual_roadmap = CCProverAlignmentRoadmap::make(
        new_allocation.clone(),
        epoch,
        &current_allocation,
        current_status,
    );

    let pre_action = CUProverPreAction::cleanup_proof_cache();
    let expected_actions = vec![
        CUProverAction::new_cc_job(allocation_1.0.into(), allocation_2.1),
        CUProverAction::new_cc_job(allocation_2.0.into(), allocation_3.1),
        CUProverAction::new_cc_job(allocation_3.0.into(), allocation_1.1),
    ];
    let expected_roadmap = CCProverAlignmentRoadmap {
        pre_action,
        actions: expected_actions,
        epoch,
    };

    assert_eq!(actual_roadmap, expected_roadmap);
}

#[test]
fn repinning_works() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test::generate_cu_id(1));
    let allocation_2 = (2, test::generate_cu_id(2));
    let allocation_3 = (3, test::generate_cu_id(3));
    let allocation_4 = (4, test::generate_cu_id(4));

    new_allocation.insert(allocation_2.0.into(), allocation_2.1);
    new_allocation.insert(allocation_3.0.into(), allocation_3.1);
    new_allocation.insert(allocation_4.0.into(), allocation_4.1);

    let epoch = test::generate_epoch_params(2, 1);
    let current_status = CCStatus::Running { epoch };

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0.into(), DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0.into(), DumpProvider::running(allocation_2.1));
    current_allocation.insert(allocation_3.0.into(), DumpProvider::running(allocation_3.1));

    let actual_roadmap = CCProverAlignmentRoadmap::make(
        new_allocation.clone(),
        epoch,
        &current_allocation,
        current_status,
    );

    let pre_action = CUProverPreAction::cleanup_proof_cache();
    let expected_actions = vec![CUProverAction::new_cc_job_repin(
        allocation_1.0.into(),
        allocation_4.0.into(),
        allocation_4.1,
    )];
    let expected_roadmap = CCProverAlignmentRoadmap {
        pre_action,
        actions: expected_actions,
        epoch,
    };

    assert_eq!(actual_roadmap, expected_roadmap);
}

#[test]
fn create_more_then_remove() {
    let mut new_allocation = CUAllocation::new();
    let allocation_1 = (1, test::generate_cu_id(1));
    let allocation_2 = (2, test::generate_cu_id(2));
    let allocation_3 = (3, test::generate_cu_id(3));
    let allocation_4 = (4, test::generate_cu_id(4));
    let allocation_5 = (5, test::generate_cu_id(5));

    new_allocation.insert(allocation_2.0.into(), allocation_2.1);
    new_allocation.insert(allocation_3.0.into(), allocation_3.1);
    new_allocation.insert(allocation_4.0.into(), allocation_4.1);
    new_allocation.insert(allocation_5.0.into(), allocation_5.1);

    let epoch = test::generate_epoch_params(2, 1);
    let current_status = CCStatus::Running { epoch };

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0.into(), DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0.into(), DumpProvider::running(allocation_2.1));
    current_allocation.insert(allocation_3.0.into(), DumpProvider::running(allocation_3.1));

    let actual_roadmap = CCProverAlignmentRoadmap::make(
        new_allocation.clone(),
        epoch,
        &current_allocation,
        current_status,
    );

    let pre_action_1 = CUProverPreAction::cleanup_proof_cache();
    let expected_actions_1 = vec![
        CUProverAction::new_cc_job_repin(
            allocation_1.0.into(),
            allocation_4.0.into(),
            allocation_4.1,
        ),
        CUProverAction::create_cu_prover(allocation_5.0.into(), allocation_5.1),
    ];
    let expected_roadmap_1 = CCProverAlignmentRoadmap {
        pre_action: pre_action_1,
        actions: expected_actions_1,
        epoch,
    };

    let pre_action_2 = CUProverPreAction::cleanup_proof_cache();
    let expected_actions_2 = vec![
        CUProverAction::new_cc_job_repin(
            allocation_1.0.into(),
            allocation_5.0.into(),
            allocation_5.1,
        ),
        CUProverAction::create_cu_prover(allocation_4.0.into(), allocation_4.1),
    ];
    let expected_roadmap_2 = CCProverAlignmentRoadmap {
        pre_action: pre_action_2,
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
    let allocation_1 = (1, test::generate_cu_id(1));
    let allocation_2 = (2, test::generate_cu_id(2));
    let allocation_3 = (3, test::generate_cu_id(3));
    let allocation_4 = (4, test::generate_cu_id(4));

    new_allocation.insert(allocation_3.0.into(), allocation_3.1);
    new_allocation.insert(allocation_4.0.into(), allocation_4.1);

    let epoch = test::generate_epoch_params(2, 1);
    let current_status = CCStatus::Running { epoch };

    let mut current_allocation: HashMap<_, DumpProvider> = HashMap::new();
    current_allocation.insert(allocation_1.0.into(), DumpProvider::running(allocation_1.1));
    current_allocation.insert(allocation_2.0.into(), DumpProvider::running(allocation_2.1));
    current_allocation.insert(allocation_3.0.into(), DumpProvider::running(allocation_3.1));

    let actual_roadmap = CCProverAlignmentRoadmap::make(
        new_allocation.clone(),
        epoch,
        &current_allocation,
        current_status,
    );

    let pre_action_1 = CUProverPreAction::cleanup_proof_cache();
    let expected_actions_1 = vec![
        CUProverAction::new_cc_job_repin(
            allocation_1.0.into(),
            allocation_4.0.into(),
            allocation_4.1,
        ),
        CUProverAction::remove_cu_prover(allocation_2.0.into()),
    ];
    let expected_roadmap_1 = CCProverAlignmentRoadmap {
        pre_action: pre_action_1,
        actions: expected_actions_1,
        epoch,
    };

    let pre_action_2 = CUProverPreAction::cleanup_proof_cache();
    let expected_actions_2 = vec![
        CUProverAction::new_cc_job_repin(
            allocation_2.0.into(),
            allocation_4.0.into(),
            allocation_4.1,
        ),
        CUProverAction::remove_cu_prover(allocation_1.0.into()),
    ];
    let expected_roadmap_2 = CCProverAlignmentRoadmap {
        pre_action: pre_action_2,
        actions: expected_actions_2,
        epoch,
    };

    // we can assign prover 1 to the 4th task and remove the 2nd prover or
    //        assign prover 2 to the 4th task and remove the 1st prover
    assert!((actual_roadmap == expected_roadmap_1) || (actual_roadmap == expected_roadmap_2));
}

#[test]
fn consecutive_repinnings() {
    use rand::SeedableRng;
    const ITERATIONS: usize = 1000;
    const CORES: u8 = 128;
    const SEED: u64 = 0x0123456789ABCDEF;

    let mut rng = SmallRng::seed_from_u64(SEED);
    let cores_distribution = rand::distributions::Uniform::from(0..CORES + 1);

    let mut current_allocation: Option<CUAllocation> = None;
    let mut prover_state = HashMap::<PhysicalCoreId, MockProver>::new();
    let mut current_epoch = test::generate_epoch_params(0xFF, 1);

    let apply_roadmap = |prover_state: &mut HashMap<PhysicalCoreId, MockProver>,
                         roadmap: CCProverAlignmentRoadmap| {
        for action in roadmap.actions {
            match action {
                CUProverAction::CreateCUProver(state) => {
                    let result = prover_state.insert(state.new_core_id, state.new_cu_id.into());
                    assert!(result.is_none())
                }
                CUProverAction::RemoveCUProver(state) => {
                    let result = prover_state.remove(&state.current_core_id);
                    assert!(result.is_some())
                }
                CUProverAction::NewCCJob(state) => {
                    *prover_state.get_mut(&state.current_core_id).unwrap() = state.new_cu_id.into();
                }
                CUProverAction::NewCCJobWithRepining(state) => {
                    let result = prover_state.remove(&state.current_core_id);
                    assert!(result.is_some());
                    let result = prover_state.insert(state.new_core_id, state.new_cu_id.into());
                    assert!(result.is_none())
                }
            }
        }
    };

    let check_prover_state = |prover_state: &HashMap<PhysicalCoreId, MockProver>,
                              current_allocation: &CUAllocation| {
        for (core_id, prover) in prover_state.iter() {
            let allocated_cu_id = current_allocation.get(core_id).unwrap();
            match prover.status {
                CUStatus::Idle => panic!("prover running but was not allocated"),
                CUStatus::Running { cu_id } if cu_id != *allocated_cu_id => {
                    panic!("expected matching cu_id")
                }
                _ => continue,
            }
        }

        for (allocated_core_id, allocated_cu_id) in current_allocation.iter() {
            let prover = prover_state.get(allocated_core_id).unwrap();
            match prover.status {
                CUStatus::Idle => panic!("expected running prover"),
                CUStatus::Running { cu_id } if cu_id != *allocated_cu_id => {
                    panic!("expected matching cu_id")
                }
                _ => continue,
            }
        }
    };

    for i in 0..ITERATIONS {
        let allocation_size = rng.sample(&cores_distribution);
        let new_allocation =
            test::generate_random_allocation(&mut rng, allocation_size as usize, 0..CORES);
        let new_epoch = test::generate_epoch_params((i % 256) as u8, 1);
        let status = if current_allocation.is_some() {
            CCStatus::Running {
                epoch: current_epoch,
            }
        } else {
            CCStatus::Idle
        };

        let roadmap = CCProverAlignmentRoadmap::make(
            new_allocation.clone(),
            new_epoch,
            &prover_state,
            status,
        );

        apply_roadmap(&mut prover_state, roadmap);
        current_epoch = new_epoch;
        current_allocation = Some(new_allocation);
        check_prover_state(&prover_state, &current_allocation.as_ref().unwrap())
    }
}

#[derive(Debug)]
struct MockProver {
    #[allow(unused)]
    pub status: CUStatus,
}

impl MockProver {
    pub fn new(cu_id: CUID) -> Self {
        Self {
            status: CUStatus::Running { cu_id },
        }
    }
}

impl From<&CUID> for MockProver {
    fn from(value: &CUID) -> Self {
        Self::new(value.clone())
    }
}

impl From<CUID> for MockProver {
    fn from(value: CUID) -> Self {
        Self::new(value)
    }
}

impl ToCUStatus for MockProver {
    fn status(&self) -> CUStatus {
        self.status
    }
}
