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

use crate::cu::RawProof;
use ccp_config::ThreadsPerCoreAllocationPolicy;
use ccp_shared::types::{Difficulty, GlobalNonce, CUID};
use ccp_test_utils::randomx::run_light_randomx;
use ccp_test_utils::test_values as test;
use randomx_rust_wrapper::RandomXFlags;

use super::CUProver;
use super::CUProverConfig;
use crate::cu::status::CUStatus;
use crate::cu::status::ToCUStatus;

#[tokio::test]
async fn idle_cu_prover_can_be_stopped() {
    let config = CUProverConfig {
        randomx_flags: RandomXFlags::recommended_full_mem(),
        thread_allocation_policy: ThreadsPerCoreAllocationPolicy::Exact {
            threads_per_physical_core: std::num::NonZeroUsize::new(1).unwrap(),
        },
    };

    let (inlet, _) = mpsc::channel(1);
    let prover = CUProver::create(config, inlet, 3.into()).await.unwrap();

    let actual_status = prover.status();
    assert_eq!(actual_status, CUStatus::Idle);

    let result = prover.stop().await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cu_prover_can_be_stopped() {
    let _ = env_logger::builder().is_test(true).try_init();

    let config = CUProverConfig {
        randomx_flags: RandomXFlags::recommended_full_mem(),
        thread_allocation_policy: ThreadsPerCoreAllocationPolicy::Exact {
            threads_per_physical_core: std::num::NonZeroUsize::new(1).unwrap(),
        },
    };

    let (inlet, mut outlet) = mpsc::channel(1);
    let mut prover = CUProver::create(config, inlet, 3.into()).await.unwrap();
    let cu_id = test::generate_cu_id(1);

    let handle = tokio::spawn(async move { while let Some(_) = outlet.recv().await {} });

    prover
        .new_epoch(
            test::generate_global_nonce(1),
            cu_id,
            test::generate_difficulty(0xFF),
        )
        .await
        .unwrap();

    let actual_status = prover.status();
    assert_eq!(actual_status, CUStatus::Running { cu_id });

    println!("stop");
    let result = prover.stop().await;
    let _ = handle.await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn cu_prover_produces_correct_proof() {
    let _ = env_logger::builder().is_test(true).try_init();

    let config = CUProverConfig {
        randomx_flags: RandomXFlags::recommended_full_mem(),
        thread_allocation_policy: ThreadsPerCoreAllocationPolicy::Exact {
            threads_per_physical_core: std::num::NonZeroUsize::new(4).unwrap(),
        },
    };

    let (inlet, mut outlet) = mpsc::channel(1);
    let mut prover = CUProver::create(config, inlet, 3.into()).await.unwrap();
    let global_nonce = test::generate_global_nonce(1);
    let cu_id = test::generate_cu_id(1);
    let difficulty = test::generate_difficulty(0xFF);

    let handle = tokio::spawn(async move {
        let flags = RandomXFlags::recommended();
        let global_nonce_cu = ccp_utils::compute_global_nonce_cu(&global_nonce, &cu_id);
        let mut found_proof_count = 0;

        while let Some(proof) = outlet.recv().await {
            found_proof_count += 1;
            let expected_result_hash =
                run_light_randomx(global_nonce_cu.as_slice(), &proof.local_nonce, flags);
            assert!(expected_result_hash.into_slice() < difficulty);
        }

        found_proof_count
    });

    prover
        .new_epoch(global_nonce, cu_id, difficulty)
        .await
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(10));
    let result = prover.stop().await;
    let found_proof_count = handle.await.unwrap();
    println!("proofs count is {found_proof_count}");

    assert!(result.is_ok());
    assert!(found_proof_count > 0);
}

fn batch_proof_verification(
    global_nonce: GlobalNonce,
    cu_id: CUID,
    proofs: impl Iterator<Item = RawProof>,
    difficulty: Difficulty,
) -> bool {
    use randomx_rust_wrapper::Cache;
    use randomx_rust_wrapper::RandomXVM;

    let flags = RandomXFlags::recommended();
    let global_nonce_cu = ccp_utils::compute_global_nonce_cu(&global_nonce, &cu_id);
    let cache = Cache::new(&global_nonce_cu, flags).unwrap();
    let vm = RandomXVM::light(cache.handle(), flags).unwrap();

    for proof in proofs {
        let result = vm.hash(&proof.local_nonce);
        if result.into_slice() > difficulty {
            return false;
        }
    }

    true
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn cu_prover_works_with_odd_threads_number() {
    let _ = env_logger::builder().is_test(true).try_init();

    let config = CUProverConfig {
        randomx_flags: RandomXFlags::recommended_full_mem(),
        thread_allocation_policy: ThreadsPerCoreAllocationPolicy::Exact {
            threads_per_physical_core: std::num::NonZeroUsize::new(5).unwrap(),
        },
    };

    let (inlet, mut outlet) = mpsc::channel(1);
    let mut prover = CUProver::create(config, inlet, 3.into()).await.unwrap();
    let global_nonce = test::generate_global_nonce(1);
    let cu_id = test::generate_cu_id(1);
    let difficulty = test::generate_difficulty(0xFF);

    let handle = tokio::spawn(async move {
        let mut proofs = Vec::new();

        while let Some(proof) = outlet.recv().await {
            proofs.push(proof);
        }

        proofs
    });

    prover
        .new_epoch(global_nonce, cu_id, difficulty)
        .await
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(10));
    let result = prover.stop().await;
    let proofs = handle.await.unwrap();

    assert!(result.is_ok());

    assert!(batch_proof_verification(
        global_nonce,
        cu_id,
        proofs.into_iter(),
        difficulty
    ));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cu_prover_changes_epoch_correctly() {
    let _ = env_logger::builder().is_test(true).try_init();

    let config = CUProverConfig {
        randomx_flags: RandomXFlags::recommended_full_mem(),
        thread_allocation_policy: ThreadsPerCoreAllocationPolicy::Exact {
            threads_per_physical_core: std::num::NonZeroUsize::new(2).unwrap(),
        },
    };

    let (inlet, mut outlet) = mpsc::channel(1);
    let mut prover = CUProver::create(config, inlet, 3.into()).await.unwrap();

    let global_nonce = test::generate_global_nonce(1);
    let cu_id = test::generate_cu_id(1);
    let difficulty = test::generate_difficulty(0xFF);

    let handle = tokio::spawn(async move {
        let mut first_epoch_proofs = Vec::new();
        let mut second_epoch_proofs = Vec::new();

        while let Some(proof) = outlet.recv().await {
            if proof.global_nonce == global_nonce {
                first_epoch_proofs.push(proof)
            } else {
                second_epoch_proofs.push(proof)
            }
        }

        (first_epoch_proofs, second_epoch_proofs)
    });

    prover
        .new_epoch(global_nonce, cu_id, difficulty)
        .await
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(10));

    let global_nonce_2 = test::generate_global_nonce(2);
    let cu_id_2 = test::generate_cu_id(2);
    let difficulty_2 = test::generate_difficulty(0x80);

    prover
        .new_epoch(global_nonce_2, cu_id_2, difficulty_2)
        .await
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(10));

    let result = prover.stop().await;
    let (first_epoch_proofs, second_epoch_proofs) = handle.await.unwrap();
    assert!(result.is_ok());
    assert!(!first_epoch_proofs.is_empty());
    assert!(!second_epoch_proofs.is_empty());

    println!("first epoch proofs {}", first_epoch_proofs.len());
    println!("second epoch proofs {}", second_epoch_proofs.len());

    assert!(batch_proof_verification(
        global_nonce,
        cu_id,
        first_epoch_proofs.into_iter(),
        difficulty
    ));
    assert!(batch_proof_verification(
        global_nonce_2,
        cu_id_2,
        second_epoch_proofs.into_iter(),
        difficulty_2
    ));
}
