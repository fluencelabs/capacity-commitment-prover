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

use ccp_shared::types::Difficulty;
use tokio::sync::mpsc;

use crate::cu::RawProof;
use ccp_shared::types::*;
use cpu_topology::LogicalCoreId;
use randomx_rust_wrapper::dataset::DatasetHandle;
use randomx_rust_wrapper::Cache;
use randomx_rust_wrapper::RandomXFlags;
use randomx_rust_wrapper::RandomXVM;
use randomx_rust_wrapper::ResultHash;

use super::ProvingThreadAsync;
use super::ProvingThreadFacade;

fn run_light_randomx(global_nonce: &[u8], local_nonce: &[u8], flags: RandomXFlags) -> ResultHash {
    let cache = Cache::new(&global_nonce, flags).unwrap();
    let vm = RandomXVM::light(cache.handle(), flags).unwrap();
    vm.hash(&local_nonce)
}

fn get_test_global_nonce() -> GlobalNonce {
    GlobalNonce::new([
        1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5, 6, 1, 2, 3, 2, 3, 3, 4, 2, 1, 4, 5, 6, 1, 2, 3, 4, 6,
        3, 2,
    ])
}

fn get_test_local_nonce() -> LocalNonce {
    LocalNonce::new([
        1, 2, 3, 4, 3, 4, 3, 1, 2, 4, 4, 5, 6, 1, 2, 3, 2, 3, 3, 4, 2, 1, 4, 5, 6, 1, 2, 3, 4, 6,
        3, 2,
    ])
}

fn get_test_cu_id() -> CUID {
    CUID::new([
        2, 2, 4, 4, 1, 6, 0, 2, 2, 3, 4, 5, 6, 1, 2, 3, 2, 3, 3, 4, 2, 1, 4, 5, 6, 1, 2, 3, 4, 6,
        3, 2,
    ])
}

fn get_test_difficulty(difficulty: u8) -> Difficulty {
    Difficulty::new([
        0, difficulty, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0,
    ])
}

#[allow(dead_code)]
async fn create_thread_init_dataset(
    core_id: LogicalCoreId,
    global_nonce: GlobalNonce,
    cu_id: CUID,
) -> (ProvingThreadAsync, DatasetHandle, mpsc::Receiver<RawProof>) {
    let flags = RandomXFlags::recommended_full_mem();

    let (inlet, outlet) = mpsc::channel(1);

    let mut thread = ProvingThreadAsync::new(core_id, inlet);
    let actual_dataset = thread.allocate_dataset(flags).await.unwrap();
    let actual_cache = thread
        .create_cache(global_nonce, cu_id, flags)
        .await
        .unwrap();
    thread
        .initialize_dataset(
            actual_cache.handle(),
            actual_dataset.handle(),
            0,
            actual_dataset.items_count(),
        )
        .await
        .unwrap();

    (thread, actual_dataset.handle(), outlet)
}

#[tokio::test]
async fn cache_creation_works() {
    let global_nonce = get_test_global_nonce();
    let local_nonce = get_test_local_nonce();
    let cu_id = get_test_cu_id();

    let flags = RandomXFlags::recommended();

    let (inlet, _outlet) = mpsc::channel(1);
    let mut thread = ProvingThreadAsync::new(2.into(), inlet);
    let actual_cache = thread
        .create_cache(global_nonce, cu_id, flags)
        .await
        .unwrap();
    thread.stop().await.unwrap();

    let actual_vm = RandomXVM::light(actual_cache.handle(), flags).unwrap();
    let actual_result_hash = actual_vm.hash(local_nonce.as_ref());

    let global_nonce_cu = ccp_utils::compute_global_nonce_cu(&global_nonce, &cu_id);
    let expected_result_hash =
        run_light_randomx(global_nonce_cu.as_slice(), local_nonce.as_ref(), flags);

    assert_eq!(actual_result_hash, expected_result_hash);
}

#[tokio::test]
async fn dataset_creation_works() {
    let global_nonce = get_test_global_nonce();
    let local_nonce = get_test_local_nonce();
    let cu_id = get_test_cu_id();

    let flags = RandomXFlags::recommended_full_mem();

    let (inlet, _outlet) = mpsc::channel(1);
    let mut thread = ProvingThreadAsync::new(2.into(), inlet);
    let actual_dataset = thread.allocate_dataset(flags).await.unwrap();
    let actual_cache = thread
        .create_cache(global_nonce, cu_id, flags)
        .await
        .unwrap();
    thread
        .initialize_dataset(
            actual_cache.handle(),
            actual_dataset.handle(),
            0,
            actual_dataset.items_count(),
        )
        .await
        .unwrap();
    thread.stop().await.unwrap();

    let flags = RandomXFlags::recommended_full_mem();
    let actual_vm = RandomXVM::fast(actual_dataset.handle(), flags).unwrap();
    let actual_result_hash = actual_vm.hash(local_nonce.as_ref());

    let flags = RandomXFlags::recommended();
    let global_nonce_cu = ccp_utils::compute_global_nonce_cu(&global_nonce, &cu_id);
    let expected_result_hash =
        run_light_randomx(global_nonce_cu.as_slice(), local_nonce.as_ref(), flags);

    assert_eq!(actual_result_hash, expected_result_hash);
}

#[tokio::test]
async fn dataset_creation_works_with_two_threads() {
    let global_nonce = get_test_global_nonce();
    let local_nonce = get_test_local_nonce();
    let cu_id = get_test_cu_id();

    let flags = RandomXFlags::recommended_full_mem();

    let (inlet, _outlet) = mpsc::channel(1);
    let mut thread_1 = ProvingThreadAsync::new(2.into(), inlet.clone());
    let mut thread_2 = ProvingThreadAsync::new(2.into(), inlet);
    let actual_dataset = thread_1.allocate_dataset(flags).await.unwrap();
    let actual_cache = thread_1
        .create_cache(global_nonce, cu_id, flags)
        .await
        .unwrap();

    let items_count = actual_dataset.items_count();
    thread_1
        .initialize_dataset(
            actual_cache.handle(),
            actual_dataset.handle(),
            0,
            items_count / 2,
        )
        .await
        .unwrap();
    thread_2
        .initialize_dataset(
            actual_cache.handle(),
            actual_dataset.handle(),
            items_count / 2,
            items_count / 2,
        )
        .await
        .unwrap();

    thread_1.stop().await.unwrap();
    thread_2.stop().await.unwrap();

    let flags = RandomXFlags::recommended_full_mem();
    let actual_vm = RandomXVM::fast(actual_dataset.handle(), flags).unwrap();
    let actual_result_hash = actual_vm.hash(local_nonce.as_ref());

    let flags = RandomXFlags::recommended();
    let global_nonce_cu = ccp_utils::compute_global_nonce_cu(&global_nonce, &cu_id);
    let expected_result_hash =
        run_light_randomx(global_nonce_cu.as_slice(), local_nonce.as_ref(), flags);

    assert_eq!(actual_result_hash, expected_result_hash);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cc_job_stopable() {
    let global_nonce = get_test_global_nonce();
    let cu_id = get_test_cu_id();
    let (mut thread, actual_dataset, mut outlet) =
        create_thread_init_dataset(2.into(), global_nonce, cu_id).await;

    let test_difficulty = get_test_difficulty(0xFF);
    let flags = RandomXFlags::recommended_full_mem();
    thread
        .run_cc_job(actual_dataset, flags, global_nonce, test_difficulty, cu_id)
        .await
        .unwrap();

    let handle = tokio::spawn(async move {
        let flags = RandomXFlags::recommended();
        let global_nonce_cu = ccp_utils::compute_global_nonce_cu(&global_nonce, &cu_id);

        while let Some(proof) = outlet.recv().await {
            let expected_result_hash = run_light_randomx(
                global_nonce_cu.as_slice(),
                proof.local_nonce.as_ref(),
                flags,
            );
            assert!(expected_result_hash.into_slice() < test_difficulty);
        }
    });

    thread.stop().await.unwrap();
    let _ = handle.await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn prover_works() {
    let global_nonce = get_test_global_nonce();
    let cu_id = get_test_cu_id();
    let (mut thread, actual_dataset, mut outlet) =
        create_thread_init_dataset(2.into(), global_nonce, cu_id).await;

    let test_difficulty = get_test_difficulty(0xFF);
    let flags = RandomXFlags::recommended_full_mem();
    thread
        .run_cc_job(actual_dataset, flags, global_nonce, test_difficulty, cu_id)
        .await
        .unwrap();

    let proof = outlet.recv().await.unwrap();

    let handle = tokio::spawn(async move { while let Some(_) = outlet.recv().await {} });

    thread.stop().await.unwrap();
    let _ = handle.await;

    let flags = RandomXFlags::recommended();
    let global_nonce_cu = ccp_utils::compute_global_nonce_cu(&global_nonce, &cu_id);
    let expected_result_hash = run_light_randomx(
        global_nonce_cu.as_slice(),
        proof.local_nonce.as_ref(),
        flags,
    );

    assert!(expected_result_hash.into_slice() < test_difficulty);
}
