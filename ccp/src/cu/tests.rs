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
use tokio::sync::mpsc;

use ccp_config::ThreadsPerCoreAllocationPolicy;
use ccp_randomx::RandomXFlags;
use ccp_shared::meet_difficulty::MeetDifficulty;
use ccp_shared::types::EpochParameters;
use ccp_shared::types::CUID;
use ccp_test_utils::test_values as test;

use super::CUProver;
use super::CUProverConfig;
use crate::cu::status::CUStatus;
use crate::cu::status::ToCUStatus;
use crate::utility_thread::message::RawProof;
use crate::utility_thread::message::ToUtilityMessage;

fn batch_proof_verification(
    epoch: EpochParameters,
    cu_id: CUID,
    proofs: impl Iterator<Item = RawProof>,
) -> bool {
    use ccp_randomx::Cache;
    use ccp_randomx::RandomXVM;

    let flags = RandomXFlags::recommended();
    let global_nonce_cu = ccp_utils::hash::compute_global_nonce_cu(&epoch.global_nonce, &cu_id);
    let cache = Cache::new(&global_nonce_cu, flags).unwrap();
    let vm = RandomXVM::light(cache.handle(), flags).unwrap();

    for proof in proofs {
        let result = vm.hash(proof.local_nonce.as_ref());
        if !result.meet_difficulty(&epoch.difficulty) {
            return false;
        }
    }

    true
}

fn batch_proof_verification_local(proofs: impl Iterator<Item = RawProof>) -> bool {
    use ccp_randomx::Cache;
    use ccp_randomx::RandomXVM;

    let flags = RandomXFlags::recommended();

    for proof in proofs {
        let global_nonce_cu =
            ccp_utils::hash::compute_global_nonce_cu(&proof.epoch.global_nonce, &proof.cu_id);
        let cache = Cache::new(&global_nonce_cu, flags).unwrap();
        let vm = RandomXVM::light(cache.handle(), flags).unwrap();

        let result = vm.hash(proof.local_nonce.as_ref());
        if result != proof.result_hash || !result.meet_difficulty(&proof.epoch.difficulty) {
            return false;
        }
    }

    true
}

#[tokio::test]
async fn idle_cu_prover_can_be_stopped() {
    let config = CUProverConfig {
        randomx_flags: RandomXFlags::recommended_full_mem(),
        thread_allocation_policy: ThreadsPerCoreAllocationPolicy::Exact {
            threads_per_physical_core: std::num::NonZeroUsize::new(1).unwrap(),
        },
    };

    let (inlet, mut outlet) = mpsc::channel(1);
    let handle = tokio::spawn(async move { while let Some(_) = outlet.recv().await {} });
    let prover = CUProver::create(config, inlet, 3.into()).await.unwrap();

    let actual_status = prover.status();
    assert_eq!(actual_status, CUStatus::Idle);

    let result = prover.stop().await;
    assert!(result.is_ok());

    let result = handle.await;
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

    let epoch = test::generate_epoch_params(1, 0xFF);
    prover.new_epoch(epoch, cu_id).await.unwrap();

    let actual_status = prover.status();
    assert_eq!(actual_status, CUStatus::Running { cu_id });

    let result = prover.stop().await;
    let _ = handle.await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cu_prover_can_be_paused() {
    use std::sync;

    let config = CUProverConfig {
        randomx_flags: RandomXFlags::recommended_full_mem(),
        thread_allocation_policy: ThreadsPerCoreAllocationPolicy::Exact {
            threads_per_physical_core: std::num::NonZeroUsize::new(1).unwrap(),
        },
    };

    let (inlet, mut outlet) = mpsc::channel(1);
    let mut prover = CUProver::create(config, inlet, 3.into()).await.unwrap();
    let cu_id = test::generate_cu_id(1);

    let is_thread_paused = sync::Arc::new(sync::Mutex::new(std::cell::RefCell::new(false)));

    let is_thread_paused_cloned = is_thread_paused.clone();
    let handle = tokio::spawn(async move {
        let mut proofs_before_pause = Vec::new();
        let mut proofs_after_pause = Vec::new();

        while let Some(ToUtilityMessage::ProofFound(proof)) = outlet.recv().await {
            let is_thread_paused_locked = is_thread_paused_cloned.lock().unwrap();
            if !*is_thread_paused_locked.borrow() {
                proofs_before_pause.push(proof);
            } else {
                proofs_after_pause.push(proof);
            }
        }

        (proofs_before_pause, proofs_after_pause)
    });

    let epoch = test::generate_epoch_params(1, 0xFF);
    prover.new_epoch(epoch, cu_id).await.unwrap();

    let actual_status = prover.status();
    assert_eq!(actual_status, CUStatus::Running { cu_id });

    std::thread::sleep(std::time::Duration::from_secs(10));
    prover.pause().await.unwrap();

    let status = prover.status();
    assert_eq!(status, CUStatus::Idle);

    {
        let is_thread_paused_locked = is_thread_paused.lock().unwrap();
        *is_thread_paused_locked.borrow_mut() = true;
    }

    std::thread::sleep(std::time::Duration::from_secs(10));
    prover.stop().await.unwrap();

    let (proofs_before_pause, proofs_after_pause) = handle.await.unwrap();
    assert!(!proofs_before_pause.is_empty());
    assert!(proofs_after_pause.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn cu_prover_produces_correct_proofs() {
    let _ = env_logger::builder().is_test(true).try_init();

    let config = CUProverConfig {
        randomx_flags: RandomXFlags::recommended_full_mem(),
        thread_allocation_policy: ThreadsPerCoreAllocationPolicy::Exact {
            threads_per_physical_core: std::num::NonZeroUsize::new(2).unwrap(),
        },
    };

    let (inlet, mut outlet) = mpsc::channel(1);
    println!("1");
    let mut prover = CUProver::create(config, inlet, 3.into()).await.unwrap();
    println!("2");

    let epoch_1 = test::generate_epoch_params(1, 0x80);
    let cu_id_1 = test::generate_cu_id(1);
    println!("3");

    let handle = tokio::spawn(async move {
        let mut proofs = Vec::new();
        println!("inside closure");

        while let Some(ToUtilityMessage::ProofFound(proof)) = outlet.recv().await {
            println!("5");
            proofs.push(proof)
        }
        proofs
    });

    println!("6");
    prover.new_epoch(epoch_1, cu_id_1).await.unwrap();
    println!("7");
    std::thread::sleep(std::time::Duration::from_secs(10));

    let epoch_2 = test::generate_epoch_params(2, 0xFF);
    let cu_id_2 = test::generate_cu_id(2);

    prover.new_epoch(epoch_2, cu_id_2).await.unwrap();
    std::thread::sleep(std::time::Duration::from_secs(10));

    let result = prover.stop().await;
    let proofs = handle.await.unwrap();

    assert!(result.is_ok());
    assert!(batch_proof_verification_local(proofs.into_iter()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn cu_prover_works_with_odd_threads_number() {
    let config = CUProverConfig {
        randomx_flags: RandomXFlags::recommended_full_mem(),
        thread_allocation_policy: ThreadsPerCoreAllocationPolicy::Exact {
            threads_per_physical_core: std::num::NonZeroUsize::new(5).unwrap(),
        },
    };

    let (inlet, mut outlet) = mpsc::channel(1);
    let mut prover = CUProver::create(config, inlet, 3.into()).await.unwrap();

    let epoch = test::generate_epoch_params(1, 0xFF);
    let cu_id = test::generate_cu_id(1);

    let handle = tokio::spawn(async move {
        let mut proofs = Vec::new();

        while let Some(ToUtilityMessage::ProofFound(proof)) = outlet.recv().await {
            proofs.push(proof);
        }

        proofs
    });

    prover.new_epoch(epoch, cu_id).await.unwrap();

    std::thread::sleep(std::time::Duration::from_secs(10));
    let result = prover.stop().await;
    let proofs = handle.await.unwrap();

    assert!(result.is_ok());
    assert!(batch_proof_verification(epoch, cu_id, proofs.into_iter(),));
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

    let epochs_count = 3usize;

    let params = (0..epochs_count)
        .map(|id| {
            let epoch = test::generate_epoch_params(id as u8, 0x80 + id as u8);
            let cu_id = test::generate_cu_id(id as u8);
            (epoch, cu_id)
        })
        .collect::<Vec<_>>();

    let handle = tokio::spawn(async move {
        use std::collections::hash_map::Entry;

        let mut proofs: HashMap<EpochParameters, Vec<_>> = HashMap::new();

        while let Some(ToUtilityMessage::ProofFound(proof)) = outlet.recv().await {
            match proofs.entry(proof.epoch) {
                Entry::Vacant(entry) => {
                    entry.insert(vec![proof]);
                }
                Entry::Occupied(mut entry) => entry.get_mut().push(proof),
            }
        }

        proofs
    });

    for param_id in 0..epochs_count {
        prover
            .new_epoch(params[param_id].0, params[param_id].1)
            .await
            .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(10));
    }

    let result = prover.stop().await;
    let mut proofs = handle.await.unwrap();

    assert!(result.is_ok());
    assert!(!proofs.is_empty());

    for (epoch, cu_id) in params {
        assert!(proofs.get(&epoch).is_some());

        let current_proofs = proofs.remove(&epoch).unwrap();
        assert!(!current_proofs.is_empty());
        assert!(batch_proof_verification(
            epoch,
            cu_id,
            current_proofs.into_iter()
        ));
    }
}
