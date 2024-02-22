use std::path::{Path, PathBuf};

use crate::{state_storage::CCPState, CCProver};
use ccp_config::CCPConfig;
use ccp_shared::{
    nox_ccp_api::NoxCCPApi,
    types::{CUAllocation, EpochParameters, CUID},
};
use ccp_test_utils::test_values::generate_epoch_params;
use randomx_rust_wrapper::RandomXFlags;
use test_log::test;

fn get_prover(
    dir_to_store_proofs: impl Into<PathBuf>,
    dir_to_store_persistent_state: impl Into<PathBuf>,
) -> CCProver {
    let dir_to_store_proofs = dir_to_store_proofs.into();
    let dir_to_store_persistent_state = dir_to_store_persistent_state.into();
    let config = CCPConfig {
        thread_allocation_policy: ccp_config::ThreadsPerCoreAllocationPolicy::Exact {
            threads_per_physical_core: 1.try_into().unwrap(),
        },
        randomx_flags: RandomXFlags::recommended_full_mem(),
        dir_to_store_proofs,
        dir_to_store_persistent_state,
    };

    CCProver::new(0.into(), config)
}

fn get_epoch_params() -> EpochParameters {
    generate_epoch_params(1, 25)
}

fn get_cu_allocation() -> CUAllocation {
    maplit::hashmap! {
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

fn load_state(state_dir: &Path) -> CCPState {
    let state_data = std::fs::read(state_dir.join("state.json")).unwrap();
    serde_json::from_slice(&state_data).unwrap()
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 3))]
async fn prover_on_active_commitment() {
    let proofs_dir = tempdir::TempDir::new("proofs").unwrap();
    let state_dir = tempdir::TempDir::new("state").unwrap();

    let mut prover = get_prover(proofs_dir.path(), state_dir.path());
    let epoch_params = get_epoch_params();
    let cu_allocation = get_cu_allocation();
    prover
        .on_active_commitment(epoch_params.clone(), cu_allocation.clone())
        .await
        .unwrap();

    let state = load_state(state_dir.path());
    let expected_state = CCPState { epoch_params, cu_allocation };

    prover.stop().await.unwrap();

    assert_eq!(state, expected_state);
    assert!(!state_dir.path().join("state.json.draft").exists());

}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 3))]
async fn prover_on_no_active_commitment() {
    let proofs_dir = tempdir::TempDir::new("proofs").unwrap();
    let state_dir = tempdir::TempDir::new("state").unwrap();

    let mut prover = get_prover(proofs_dir.path(), state_dir.path());
    prover.on_no_active_commitment().await.unwrap();
    prover.stop().await.unwrap();
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 3))]
async fn prover_on_active_no_active_commitment() {
    let proofs_dir = tempdir::TempDir::new("proofs").unwrap();
    let state_dir = tempdir::TempDir::new("state").unwrap();
    let mut prover = get_prover(proofs_dir.path(), state_dir.path());
    let epoch_params = get_epoch_params();
    let cu_allocation = get_cu_allocation();

    prover
        .on_active_commitment(epoch_params.clone(), cu_allocation.clone())
        .await
        .unwrap();
    prover.on_no_active_commitment().await.unwrap();

    // state is cleared on no_active_commitment
    let state = load_state(state_dir.path());
    let expected_state = CCPState { epoch_params, cu_allocation: <_>::default() };

    prover.stop().await.unwrap();

    assert_eq!(state, expected_state);
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 3))]
async fn prover_on_active_reduce_on_active_commitment() {
    let proofs_dir = tempdir::TempDir::new("proofs").unwrap();
    let state_dir = tempdir::TempDir::new("state").unwrap();

    let mut prover = get_prover(proofs_dir.path(), state_dir.path());
    let mut cu_allocation = get_cu_allocation();
    let epoch_params = get_epoch_params();
    prover
        .on_active_commitment(epoch_params.clone(), cu_allocation.clone())
        .await
        .unwrap();

    cu_allocation.remove(&2.into()).unwrap();
    prover
        .on_active_commitment(epoch_params, cu_allocation)
        .await
        .unwrap();

    prover.stop().await.unwrap();
}

#[test(tokio::test(flavor = "multi_thread", worker_threads = 4))]
async fn prover_on_active_extend_on_active_commitment() {
    let proofs_dir = tempdir::TempDir::new("proofs").unwrap();
    let state_dir = tempdir::TempDir::new("state").unwrap();

    let mut prover = get_prover(proofs_dir.path(), state_dir.path());
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
    let proofs_dir = tempdir::TempDir::new("proofs").unwrap();
    let state_dir = tempdir::TempDir::new("state").unwrap();

    let mut prover = get_prover(proofs_dir.path(), state_dir.path());
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
