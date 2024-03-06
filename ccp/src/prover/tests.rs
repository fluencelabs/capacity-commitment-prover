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

use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use ccp_config::CCPConfig;
use ccp_randomx::ResultHash;
use ccp_shared::proof::{CCProof, CCProofId, ProofIdx};
use ccp_shared::types::LocalNonce;
use ccp_shared::{
    nox_ccp_api::NoxCCPApi,
    types::{CUAllocation, EpochParameters, CUID},
};
use ccp_test_utils::test_values::generate_epoch_params;
use maplit::hashmap;
use test_log::test;

use crate::state_storage::CCPState;
use crate::CCProver;

const GEN_PROOFS_DURATION: Duration = Duration::from_secs(10);

fn get_prover(state_dir: impl Into<PathBuf>) -> CCProver {
    let state_dir = state_dir.into();
    let config = CCPConfig {
        rpc_endpoint: <_>::default(),
        prometheus_endpoint: None,
        optimizations: <_>::default(),
        logs: <_>::default(),
        state_dir,
    };

    CCProver::new(config).unwrap()
}

async fn get_prover_from_saved_state(state_dir: impl Into<PathBuf>) -> CCProver {
    let state_dir = state_dir.into();
    let config = CCPConfig {
        rpc_endpoint: <_>::default(),
        prometheus_endpoint: None,
        optimizations: <_>::default(),
        logs: <_>::default(),
        state_dir,
    };

    CCProver::from_saved_state(config).await.unwrap()
}

fn get_epoch_params() -> EpochParameters {
    generate_epoch_params(1, 50)
}

fn get_cu_allocation() -> CUAllocation {
    hashmap! {
        1.into() => CUID::new([
            33, 247, 206, 99, 242, 79, 217, 190, 58, 45, 87, 221, 151, 162, 217, 11, 43, 151, 160,
            77, 199, 173, 183, 140, 130, 71, 222, 113, 189, 117, 174, 63,
        ]),
        2.into() => CUID::new([
            192, 52, 100, 105, 186, 121, 170, 203, 69, 85, 100, 205, 144, 66, 82, 85, 108, 121,
            68, 68, 227, 24, 101, 29, 154, 84, 84, 26, 234, 134, 65, 54,
        ]),
    }
}

fn load_state(state_dir: &Path) -> Option<CCPState> {
    let state_data = std::fs::read(state_dir.join("state.json")).unwrap();
    serde_json::from_slice(&state_data).unwrap()
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 3))]
async fn prover_on_active_commitment() {
    let state_dir = tempdir::TempDir::new("state").unwrap();

    let mut prover = get_prover(state_dir.path());
    let epoch_params = get_epoch_params();
    let cu_allocation = get_cu_allocation();
    prover
        .on_active_commitment(epoch_params.clone(), cu_allocation.clone())
        .await
        .unwrap();

    let state = load_state(state_dir.path());

    let expected_state: Option<CCPState> = Some(CCPState {
        epoch_params,
        cu_allocation: cu_allocation.clone(),
        msr_state: <_>::default(),
    });

    tokio::time::sleep(GEN_PROOFS_DURATION).await;

    let proofs = prover
        .get_proofs_after("0".parse().unwrap())
        .await
        .expect("reading proofs");

    assert!(
        // it really depends on your hardware; you may need to increase second value
        // in the generate_epoch_params call above.
        proofs.len() > 3,
        "{:?}",
        proofs,
    );

    for proof in proofs {
        assert!(
            cu_allocation
                .values()
                .find(|p| *p == &proof.cu_id)
                .is_some(),
            "{:?}",
            proof
        );
        assert_eq!(
            proof.id.global_nonce, epoch_params.global_nonce,
            "{:?}",
            proof
        );
        assert_eq!(proof.id.difficulty, epoch_params.difficulty, "{:?}", proof);
    }

    prover.stop().await.unwrap();

    assert_eq!(state, expected_state);
    assert!(!state_dir.path().join("state.json.draft").exists());
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 3))]
async fn prover_on_no_active_commitment() {
    let state_dir = tempdir::TempDir::new("state").unwrap();

    let mut prover = get_prover(state_dir.path());
    prover.on_no_active_commitment().await.unwrap();

    let proofs = prover
        .get_proofs_after("0".parse().unwrap())
        .await
        .expect("reading proofs");
    assert!(proofs.is_empty());

    prover.stop().await.unwrap();
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 3))]
#[ignore = "until on_no_active_commitment cleans proofs_dir"]
async fn prover_on_active_no_active_commitment() {
    let state_dir = tempdir::TempDir::new("state").unwrap();
    let mut prover = get_prover(state_dir.path());
    let epoch_params = get_epoch_params();
    let cu_allocation = get_cu_allocation();

    prover
        .on_active_commitment(epoch_params.clone(), cu_allocation.clone())
        .await
        .unwrap();

    tokio::time::sleep(GEN_PROOFS_DURATION).await;

    let proofs_before = prover
        .get_proofs_after("0".parse().unwrap())
        .await
        .expect("reading proofs");
    assert!(!proofs_before.is_empty());

    prover.on_no_active_commitment().await.unwrap();

    // state is cleared on no_active_commitment
    let state = load_state(state_dir.path());
    let expected_state = None;

    let proofs_after = prover
        .get_proofs_after("0".parse().unwrap())
        .await
        .expect("reading proofs");

    prover.stop().await.unwrap();

    assert!(proofs_after.is_empty());
    assert_eq!(state, expected_state);
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 3))]
async fn prover_on_active_reduce_allocation_on_active_commitment() {
    let state_dir = tempdir::TempDir::new("state").unwrap();

    let mut prover = get_prover(state_dir.path());
    let mut cu_allocation = get_cu_allocation();
    let epoch_params = get_epoch_params();
    prover
        .on_active_commitment(epoch_params.clone(), cu_allocation.clone())
        .await
        .unwrap();

    tokio::time::sleep(GEN_PROOFS_DURATION).await;

    cu_allocation.remove(&2.into()).unwrap();
    prover
        .on_active_commitment(epoch_params, cu_allocation)
        .await
        .unwrap();

    prover.stop().await.unwrap();
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 3))]
async fn prover_on_active_reduce_empty_allocation_active_commitment() {
    let state_dir = tempdir::TempDir::new("state").unwrap();

    let mut prover = get_prover(state_dir.path());
    let mut cu_allocation = get_cu_allocation();
    prover
        .on_active_commitment(get_epoch_params(), cu_allocation.clone())
        .await
        .unwrap();

    tokio::time::sleep(GEN_PROOFS_DURATION).await;

    let proofs_before = prover
        .get_proofs_after("0".parse().unwrap())
        .await
        .expect("reading proofs");
    assert!(!proofs_before.is_empty());

    cu_allocation.clear();

    prover
        .on_active_commitment(get_epoch_params(), cu_allocation)
        .await
        .unwrap();

    let proofs_after = prover
        .get_proofs_after("0".parse().unwrap())
        .await
        .expect("reading proofs");
    assert!(proofs_after.is_empty());

    prover.stop().await.unwrap();
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 4))]
async fn prover_on_active_extend_allocation_on_active_commitment() {
    let state_dir = tempdir::TempDir::new("state").unwrap();

    let mut prover = get_prover(state_dir.path());
    let mut cu_allocation = get_cu_allocation();

    prover
        .on_active_commitment(get_epoch_params(), cu_allocation.clone())
        .await
        .unwrap();

    cu_allocation.insert(
        4.into(),
        CUID::new([
            203, 92, 78, 52, 198, 0, 81, 15, 157, 50, 231, 155, 93, 107, 90, 171, 59, 181, 211,
            102, 152, 191, 178, 178, 131, 62, 176, 58, 49, 124, 217, 244,
        ]),
    );

    prover
        .on_active_commitment(get_epoch_params(), cu_allocation)
        .await
        .unwrap();

    prover.stop().await.unwrap();
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 3))]
async fn prover_on_active_reschedule_on_active_commitment() {
    let state_dir = tempdir::TempDir::new("state").unwrap();

    let mut prover = get_prover(state_dir.path());
    let mut cu_allocation = get_cu_allocation();

    prover
        .on_active_commitment(get_epoch_params(), cu_allocation.clone())
        .await
        .unwrap();

    cu_allocation.remove(&2.into()).unwrap();
    cu_allocation.insert(
        4.into(),
        CUID::new([
            203, 92, 78, 52, 198, 0, 81, 15, 157, 50, 231, 155, 93, 107, 90, 171, 59, 181, 211,
            102, 152, 191, 178, 178, 131, 62, 176, 58, 49, 124, 217, 244,
        ]),
    );
    prover
        .on_active_commitment(get_epoch_params(), cu_allocation.clone())
        .await
        .unwrap();

    prover.stop().await.unwrap();
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 4))]
async fn prover_on_active_extend_on_active_commitment_performance() {
    let state_dir = tempdir::TempDir::new("state").unwrap();

    let mut prover = get_prover(state_dir.path());
    let cu_allocation_large = get_cu_allocation();
    let cu_allocation_small = hashmap! {
        1.into() => cu_allocation_large.get(&1.into()).cloned().unwrap(),
    };

    prover
        .on_active_commitment(get_epoch_params(), cu_allocation_small)
        .await
        .unwrap();

    tokio::time::sleep(GEN_PROOFS_DURATION).await;

    let proofs_before = prover
        .get_proofs_after("0".parse().unwrap())
        .await
        .expect("reading proofs");
    assert!(!proofs_before.is_empty());

    prover
        .on_active_commitment(get_epoch_params(), cu_allocation_large)
        .await
        .unwrap();

    tokio::time::sleep(GEN_PROOFS_DURATION).await;

    let proofs_after = prover
        .get_proofs_after("0".parse().unwrap())
        .await
        .expect("reading proofs");
    assert!(
        3 * proofs_before.len() < 2 * proofs_after.len(),
        "should be 1.5*{} < {}",
        proofs_before.len(),
        proofs_after.len()
    );

    prover.stop().await.unwrap();
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 3))]
async fn prover_on_active_change_epoch() {
    let state_dir = tempdir::TempDir::new("state").unwrap();

    let mut prover = get_prover(state_dir.path());
    let cu_allocation = get_cu_allocation();

    let epoch_params_first = get_epoch_params();

    prover
        .on_active_commitment(epoch_params_first, cu_allocation.clone())
        .await
        .unwrap();

    tokio::time::sleep(GEN_PROOFS_DURATION).await;

    let proofs_before = prover
        .get_proofs_after("0".parse().unwrap())
        .await
        .expect("reading proofs");
    assert!(!proofs_before.is_empty());

    let epoch_params_second = generate_epoch_params(2, 50);

    prover
        .on_active_commitment(epoch_params_second, cu_allocation.clone())
        .await
        .unwrap();

    let state = load_state(state_dir.path());
    let expected_state = Some(CCPState {
        epoch_params: epoch_params_second,
        cu_allocation: cu_allocation.clone(),
        msr_state: <_>::default(),
    });

    tokio::time::sleep(GEN_PROOFS_DURATION).await;

    let proofs_after = prover
        .get_proofs_after("0".parse().unwrap())
        .await
        .expect("reading proofs");

    prover.stop().await.unwrap();

    assert_eq!(state, expected_state);

    for proof in proofs_after {
        assert!(
            cu_allocation
                .values()
                .find(|p| *p == &proof.cu_id)
                .is_some(),
            "{:?}",
            proof
        );
        assert_eq!(
            proof.id.global_nonce, epoch_params_second.global_nonce,
            "{:?}",
            proof
        );
        assert_eq!(
            proof.id.difficulty, epoch_params_second.difficulty,
            "{:?}",
            proof
        );
    }
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 3))]
async fn prover_restore_from_state_with_no_proofs() {
    let state_dir = tempdir::TempDir::new("state").unwrap();

    let epoch_params = get_epoch_params();
    let cu_allocation = get_cu_allocation();

    let state_path = state_dir.path().join("state.json");
    let initial_state = Some(CCPState {
        epoch_params,
        cu_allocation: cu_allocation.clone(),
        msr_state: <_>::default(),
    });
    tokio::fs::write(state_path, &serde_json::to_vec(&initial_state).unwrap())
        .await
        .unwrap();

    let prover = get_prover_from_saved_state(state_dir.path()).await;

    tokio::time::sleep(GEN_PROOFS_DURATION).await;

    let state = load_state(state_dir.path());

    let proofs = prover
        .get_proofs_after("0".parse().unwrap())
        .await
        .expect("reading proofs");

    assert!(
        // it really depends on your hardware; you may need to increase second value
        // in the generate_epoch_params call above.
        proofs.len() > 3,
        "{:?}",
        proofs,
    );

    for proof in proofs {
        assert!(
            cu_allocation
                .values()
                .find(|p| *p == &proof.cu_id)
                .is_some(),
            "{:?}",
            proof
        );
        assert_eq!(
            proof.id.global_nonce, epoch_params.global_nonce,
            "{:?}",
            proof
        );
        assert_eq!(proof.id.difficulty, epoch_params.difficulty, "{:?}", proof);
    }

    prover.stop().await.unwrap();

    assert_eq!(state, initial_state);
    assert!(!state_dir.path().join("state.json.draft").exists());
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 3))]
async fn prover_restore_from_state_with_proofs() {
    let state_dir = tempdir::TempDir::new("state").unwrap();
    let proofs_dir = state_dir.path().join("cc_proofs");
    tokio::fs::create_dir(&proofs_dir).await.unwrap();

    let epoch_params = get_epoch_params();
    let cu_allocation = get_cu_allocation();

    let epoch_params_old = generate_epoch_params(2, 50);
    let mut cu_allocation_old = get_cu_allocation();
    cu_allocation_old.remove(&2.into()).expect("Invalid test");

    let state_path = state_dir.path().join("state.json");
    let initial_state = Some(CCPState {
        epoch_params,
        cu_allocation: cu_allocation.clone(),
        msr_state: <_>::default(),
    });
    tokio::fs::write(state_path, &serde_json::to_vec(&initial_state).unwrap())
        .await
        .unwrap();

    // Good proofs
    for good_idx_str in ["100"] {
        let good_idx: ProofIdx = good_idx_str.parse().unwrap();

        let proof_id = CCProofId::new(
            epoch_params.global_nonce.clone(),
            epoch_params.difficulty,
            good_idx,
        );
        let proof = CCProof::new(
            proof_id,
            LocalNonce::random(),
            cu_allocation.values().next().unwrap().clone(),
            ResultHash::from_slice([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ]),
        );
        tokio::fs::write(
            proofs_dir.join(good_idx_str),
            serde_json::to_vec(&proof).unwrap(),
        )
        .await
        .unwrap();
    }

    // Bad proofs
    for bad_idx_str in ["2", "500"] {
        let bad_idx: ProofIdx = bad_idx_str.parse().unwrap();

        let proof_id = CCProofId::new(
            epoch_params_old.global_nonce.clone(),
            epoch_params_old.difficulty,
            bad_idx,
        );
        let proof = CCProof::new(
            proof_id,
            LocalNonce::random(),
            cu_allocation.values().next().unwrap().clone(),
            ResultHash::from_slice([
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ]),
        );
        tokio::fs::write(
            proofs_dir.join(bad_idx_str),
            serde_json::to_vec(&proof).unwrap(),
        )
        .await
        .unwrap();
    }

    let prover = get_prover_from_saved_state(state_dir.path()).await;

    tokio::time::sleep(GEN_PROOFS_DURATION).await;

    let state = load_state(state_dir.path());

    let proofs = prover
        .get_proofs_after("0".parse().unwrap())
        .await
        .expect("reading proofs");

    assert!(
        // it really depends on your hardware; you may need to increase second value
        // in the generate_epoch_params call above.
        proofs.len() > 3,
        "not enough proofs: {:?}",
        proofs,
    );

    for proof in proofs {
        assert!(
            cu_allocation
                .values()
                .find(|p| *p == &proof.cu_id)
                .is_some(),
            "{:?}",
            proof
        );
        assert_eq!(
            proof.id.global_nonce, epoch_params.global_nonce,
            "{:?}",
            proof
        );
        assert_eq!(proof.id.difficulty, epoch_params.difficulty, "{:?}", proof);

        let min_bad_idx: ProofIdx = "100".parse().unwrap();
        assert!(
            min_bad_idx <= proof.id.idx,
            "idx too small: {}",
            proof.id.idx
        );

        let max_bad_idx: ProofIdx = "500".parse().unwrap();
        assert!(proof.id.idx < max_bad_idx, "idx too big: {}", proof.id.idx);
    }

    prover.stop().await.unwrap();

    assert_eq!(state, initial_state);
    assert!(!state_dir.path().join("state.json.draft").exists());
}
