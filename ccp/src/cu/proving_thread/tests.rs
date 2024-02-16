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

use tokio::sync::mpsc;

use ccp_shared::types::{GlobalNonce, LocalNonce, CUID};
use randomx_rust_wrapper::Cache;
use randomx_rust_wrapper::Dataset;
use randomx_rust_wrapper::RandomXFlags;
use randomx_rust_wrapper::RandomXVM;
use randomx_rust_wrapper::ResultHash;

use super::ProvingThread;
use super::ProvingThreadAPI;

fn run_light_randomx(global_nonce: &[u8], local_nonce: &[u8], flags: RandomXFlags) -> ResultHash {
    let cache = Cache::new(&global_nonce, flags).unwrap();
    let vm = RandomXVM::light(&cache, flags).unwrap();
    vm.hash(&local_nonce)
}

fn run_fast_randomx(global_nonce: &[u8], local_nonce: &[u8], flags: RandomXFlags) -> ResultHash {
    let dataset = Dataset::new(&global_nonce, flags).unwrap();
    let vm = RandomXVM::fast(&dataset, flags).unwrap();
    vm.hash(&local_nonce)
}

fn get_test_global_nonce() -> GlobalNonce {
    [
        1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5, 6, 1, 2, 3, 2, 3, 3, 4, 2, 1, 4, 5, 6, 1, 2, 3, 4, 6,
        3, 2,
    ]
}

fn get_test_local_nonce() -> LocalNonce {
    [
        1, 2, 3, 4, 3, 4, 3, 1, 2, 4, 4, 5, 6, 1, 2, 3, 2, 3, 3, 4, 2, 1, 4, 5, 6, 1, 2, 3, 4, 6,
        3, 2,
    ]
}

fn get_test_cu_id() -> CUID {
    [
        2, 2, 4, 4, 1, 6, 0, 2, 2, 3, 4, 5, 6, 1, 2, 3, 2, 3, 3, 4, 2, 1, 4, 5, 6, 1, 2, 3, 4, 6,
        3, 2,
    ]
}

#[tokio::test]
async fn cache_creation_works() {
    let global_nonce = get_test_global_nonce();
    let local_nonce = get_test_local_nonce();
    let cu_id = get_test_cu_id();

    let flags = RandomXFlags::recommended();

    let (inlet, outlet) = mpsc::channel(1);
    let mut thread = ProvingThread::new(2, inlet);
    let actual_cache = thread
        .create_cache(global_nonce, cu_id, flags)
        .await
        .unwrap();
    thread.stop().await.unwrap();

    let actual_vm = RandomXVM::light(&actual_cache, flags).unwrap();
    let actual_result_hash = actual_vm.hash(&local_nonce);

    let global_nonce_cu = ccp_utils::compute_global_nonce_cu(&global_nonce, &cu_id);
    let expected_result_hash = run_light_randomx(global_nonce_cu.as_slice(), &local_nonce, flags);

    assert_eq!(actual_result_hash, expected_result_hash);
}

#[tokio::test]
async fn dataset_creation_works() {
    let global_nonce = get_test_global_nonce();
    let local_nonce = get_test_local_nonce();
    let cu_id = get_test_cu_id();

    let flags = RandomXFlags::recommended();

    let (inlet, outlet) = mpsc::channel(1);
    let mut thread = ProvingThread::new(2, inlet);
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
    let actual_vm = RandomXVM::fast(&actual_dataset, flags).unwrap();
    let actual_result_hash = actual_vm.hash(&local_nonce);

    let flags = RandomXFlags::recommended();
    let global_nonce_cu = ccp_utils::compute_global_nonce_cu(&global_nonce, &cu_id);
    let expected_result_hash = run_light_randomx(global_nonce_cu.as_slice(), &local_nonce, flags);

    assert_eq!(actual_result_hash, expected_result_hash);
}

#[tokio::test]
async fn prover_works() {
    let global_nonce = get_test_global_nonce();
    let cu_id = get_test_cu_id();

    let flags = RandomXFlags::recommended_full_mem();

    let (inlet, mut outlet) = mpsc::channel(1);
    let mut thread = ProvingThread::new(2, inlet);
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
    let test_difficulty = [
        0, 0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0,
    ];
    thread
        .run_cc_job(
            actual_dataset.handle(),
            flags,
            global_nonce,
            test_difficulty,
            cu_id,
        )
        .await
        .unwrap();

    let proof = outlet.recv().await.unwrap();
    println!("proof: {proof:?}");

    thread.stop().await.unwrap();
    std::thread::sleep(std::time::Duration::from_secs(2));

    let flags = RandomXFlags::recommended();
    let global_nonce_cu = ccp_utils::compute_global_nonce_cu(&global_nonce, &cu_id);
    let expected_result_hash =
        run_light_randomx(global_nonce_cu.as_slice(), &proof.local_nonce, flags);

    println!("expected_result_hash: {expected_result_hash:?}");
    assert!(expected_result_hash.into_slice() < test_difficulty);
}

#[tokio::test]
async fn cc_job_stopable() {
    let global_nonce = get_test_global_nonce();
    let cu_id = get_test_cu_id();

    let flags = RandomXFlags::recommended();

    let (inlet, mut outlet) = mpsc::channel(1);
    let mut thread = ProvingThread::new(2, inlet);
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
    let test_difficulty = [
        0xFF, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0,
    ];
    thread
        .run_cc_job(
            actual_dataset.handle(),
            flags,
            global_nonce,
            test_difficulty,
            cu_id,
        )
        .await
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(2));
    thread.stop().await.unwrap();
    std::thread::sleep(std::time::Duration::from_secs(2));

    let result = outlet.try_recv();
    println!("result {result:?}");
}
