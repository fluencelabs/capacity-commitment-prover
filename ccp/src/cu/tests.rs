/*
 * Copyright 2024 Fluence DAO
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
use std::sync;

use ccp_config::ThreadsPerCoreAllocationPolicy;
use ccp_msr::MSRModeEnforcer;
use ccp_randomx::RandomXFlags;
use ccp_shared::meet_difficulty::MeetDifficulty;
use ccp_shared::types::EpochParameters;
use ccp_shared::types::CUID;
use ccp_test_utils::test_values as test;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::proving_thread::sync::to_stats_collecting_message::ToStatsCollectingOutlet;
use super::CUProver;
use super::CUProverConfig;
use crate::cu::proving_thread::sync::to_state_operating_message::ToStateOperatingMessage;
use crate::cu::status::CUStatus;
use crate::cu::status::ToCUStatus;
use crate::state_operating_thread::message::RawProof;
use crate::state_operating_thread::message::ToStateOperatingOutlet;

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

fn create_config(cores_count: usize) -> CUProverConfig {
    CUProverConfig {
        randomx_flags: RandomXFlags::recommended_full_mem(),
        threads_per_core_policy: ThreadsPerCoreAllocationPolicy::Exact {
            threads_per_physical_core: std::num::NonZeroUsize::new(cores_count).unwrap(),
        },
        hashes_per_round: 1024,
        async_to_sync_queue_size: 1,
        sync_to_async_queue_size: 1,
        // ca. 1 second
        dataset_chunk_size: 500_000,
    }
}

pub(crate) async fn proof_sink(
    mut to_state_operating: ToStateOperatingOutlet,
    mut to_stats_collecting: ToStatsCollectingOutlet,
) {
    loop {
        tokio::select! {
            state = to_state_operating.recv() => {
                match state {
                    Some(_) => {}
                    None => break,
                }
            }
            _ = to_stats_collecting.recv() => {}
        }
    }
}

pub(crate) async fn collect_proofs(
    mut to_state_operating: ToStateOperatingOutlet,
    mut to_stats_collecting: ToStatsCollectingOutlet,
) -> Vec<RawProof> {
    let mut proofs = vec![];
    loop {
        tokio::select! {
            state = to_state_operating.recv() => {
                match state {
                    Some(ToStateOperatingMessage::ProofFound { proof, .. }) => proofs.push(proof),
                    Some(_) => {}
                    None => break,
                }
            }
            _ = to_stats_collecting.recv() => {}
        }
    }
    proofs
}

pub(crate) async fn collect_proofs_with_flag(
    mut to_state_operating: ToStateOperatingOutlet,
    mut to_stats_collecting: ToStatsCollectingOutlet,
    flag: sync::Arc<sync::Mutex<bool>>,
) -> (Vec<RawProof>, Vec<RawProof>) {
    let mut proofs_before = vec![];
    let mut proofs_after = vec![];
    loop {
        tokio::select! {
            state = to_state_operating.recv() => {
                match state {
                    Some(ToStateOperatingMessage::ProofFound { proof, .. }) => {
                        if *flag.lock().unwrap() {
                            proofs_after.push(proof)
                        } else {
                            proofs_before.push(proof)
                        }
                    },
                    Some(_) => {}
                    None => break,
                }
            }
            _ = to_stats_collecting.recv() => {}
        }
    }
    (proofs_before, proofs_after)
}

#[tokio::test]
async fn idle_cu_prover_can_be_stopped() {
    let (to_state_operating_inlet, to_state_operating_outlet) = mpsc::channel(1);
    let (to_stats_collecting_inlet, to_stats_collecting_outlet) = mpsc::channel(1);

    let handle = tokio::spawn(proof_sink(
        to_state_operating_outlet,
        to_stats_collecting_outlet,
    ));

    let config = create_config(1);
    let msr_enforcer = MSRModeEnforcer::from_preset(false, <_>::default());
    let prover = CUProver::create(
        config,
        to_state_operating_inlet,
        to_stats_collecting_inlet,
        msr_enforcer,
        3.into(),
    )
    .await
    .unwrap();

    let actual_status = prover.status();
    assert_eq!(actual_status, CUStatus::Idle);

    let result = prover.stop_join().await;
    assert!(result.is_ok());

    let result = handle.await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cu_prover_can_be_stopped() {
    let _ = env_logger::builder().is_test(true).try_init();

    let config = create_config(1);
    let (to_state_operating_inlet, to_state_operating_outlet) = mpsc::channel(1);
    let (to_stats_collecting_inlet, to_stats_collecting_outlet) = mpsc::channel(1);
    let msr_enforcer = MSRModeEnforcer::from_preset(false, <_>::default());
    let mut prover = CUProver::create(
        config,
        to_state_operating_inlet,
        to_stats_collecting_inlet,
        msr_enforcer,
        3.into(),
    )
    .await
    .unwrap();
    let handle = tokio::spawn(proof_sink(
        to_state_operating_outlet,
        to_stats_collecting_outlet,
    ));

    let epoch = test::generate_epoch_params(1, 0xFF);
    let cu_id = test::generate_cu_id(1);
    let cancel_token = CancellationToken::new();
    prover.new_epoch(epoch, cu_id, cancel_token).await.unwrap();

    let actual_status = prover.status();
    assert_eq!(actual_status, CUStatus::Running { cu_id });

    let result = prover.stop_join().await;
    let _ = handle.await;
    assert!(result.is_ok());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cu_prover_can_be_paused() {
    use std::sync;

    let _ = env_logger::builder().is_test(true).try_init();

    let config = create_config(1);
    let (to_state_operating_inlet, to_state_operating_outlet) = mpsc::channel(1);
    let (to_stats_collecting_inlet, to_stats_collecting_outlet) = mpsc::channel(1);
    let msr_enforcer = MSRModeEnforcer::from_preset(false, <_>::default());
    let mut prover = CUProver::create(
        config,
        to_state_operating_inlet,
        to_stats_collecting_inlet,
        msr_enforcer,
        3.into(),
    )
    .await
    .unwrap();

    let is_thread_paused = sync::Arc::new(sync::Mutex::new(false));

    let handle = tokio::spawn(collect_proofs_with_flag(
        to_state_operating_outlet,
        to_stats_collecting_outlet,
        is_thread_paused.clone(),
    ));

    let epoch = test::generate_epoch_params(1, 0xFF);
    let cu_id = test::generate_cu_id(1);
    let cancel_token = CancellationToken::new();
    prover.new_epoch(epoch, cu_id, cancel_token).await.unwrap();

    let actual_status = prover.status();
    assert_eq!(actual_status, CUStatus::Running { cu_id });

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    prover.pause().await.unwrap();

    let status = prover.status();
    assert_eq!(status, CUStatus::Idle);

    {
        let mut is_thread_paused_locked = is_thread_paused.lock().unwrap();
        *is_thread_paused_locked = true;
    }

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    prover.stop_join().await.unwrap();

    let (proofs_before_pause, proofs_after_pause) = handle.await.unwrap();
    assert!(!proofs_before_pause.is_empty());
    assert!(proofs_after_pause.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn cu_prover_produces_correct_proofs() {
    let _ = env_logger::builder().is_test(true).try_init();

    let config = create_config(2);
    let (to_state_operating_inlet, to_state_operating_outlet) = mpsc::channel(1);
    let (to_stats_collecting_inlet, to_stats_collecting_outlet) = mpsc::channel(1);
    let msr_enforcer = MSRModeEnforcer::from_preset(false, <_>::default());
    let mut prover = CUProver::create(
        config,
        to_state_operating_inlet,
        to_stats_collecting_inlet,
        msr_enforcer,
        3.into(),
    )
    .await
    .unwrap();

    let epoch_1 = test::generate_epoch_params(1, 0x80);
    let cu_id_1 = test::generate_cu_id(1);

    let handle = tokio::spawn(collect_proofs(
        to_state_operating_outlet,
        to_stats_collecting_outlet,
    ));

    let cancel_token = CancellationToken::new();
    prover
        .new_epoch(epoch_1, cu_id_1, cancel_token.clone())
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let epoch_2 = test::generate_epoch_params(2, 0xFF);
    let cu_id_2 = test::generate_cu_id(2);

    prover
        .new_epoch(epoch_2, cu_id_2, cancel_token)
        .await
        .unwrap();
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let result = prover.stop_join().await;
    let proofs = handle.await.unwrap();

    assert!(result.is_ok());
    assert!(batch_proof_verification_local(proofs.into_iter()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
async fn cu_prover_works_with_odd_threads_number() {
    let config = create_config(5);
    let (to_state_operating_inlet, to_state_operating_outlet) = mpsc::channel(1);
    let (to_stats_collecting_inlet, to_stats_collecting_outlet) = mpsc::channel(1);

    let handle = tokio::spawn(collect_proofs(
        to_state_operating_outlet,
        to_stats_collecting_outlet,
    ));

    let msr_enforcer = MSRModeEnforcer::from_preset(false, <_>::default());

    let mut prover = CUProver::create(
        config,
        to_state_operating_inlet,
        to_stats_collecting_inlet,
        msr_enforcer,
        3.into(),
    )
    .await
    .unwrap();

    let epoch = test::generate_epoch_params(1, 0xFF);
    let cu_id = test::generate_cu_id(1);

    let cancel_token = CancellationToken::new();
    prover.new_epoch(epoch, cu_id, cancel_token).await.unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    let result = prover.stop_join().await;
    let proofs = handle.await.unwrap();

    assert!(result.is_ok());
    assert!(batch_proof_verification(epoch, cu_id, proofs.into_iter(),));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn cu_prover_changes_epoch_correctly() {
    let _ = env_logger::builder().is_test(true).try_init();

    let config = create_config(2);
    let (to_state_operating_inlet, mut to_state_operating_outlet) = mpsc::channel(1);
    let (to_stats_collecting_inlet, mut to_stats_collecting_outlet) = mpsc::channel(1);

    let handle = tokio::spawn(async move {
        let mut proofs: HashMap<EpochParameters, Vec<_>> = HashMap::new();
        loop {
            tokio::select! {
                message = to_state_operating_outlet.recv() => {
                    match message {
                        Some(ToStateOperatingMessage::ProofFound { proof, .. }) => {
                            proofs.entry(proof.epoch).or_default().push(proof);
                        }
                        Some(_) => {}
                        None => break,
                    }
                }
                _ = to_stats_collecting_outlet.recv() => {}
            }
        }

        proofs
    });

    let msr_enforcer = MSRModeEnforcer::from_preset(false, <_>::default());
    let mut prover = CUProver::create(
        config,
        to_state_operating_inlet,
        to_stats_collecting_inlet,
        msr_enforcer,
        3.into(),
    )
    .await
    .unwrap();

    let epochs_count = 3usize;

    let params = (0..epochs_count)
        .map(|id| {
            let epoch = test::generate_epoch_params(id as u8, 0x80 + id as u8);
            let cu_id = test::generate_cu_id(id as u8);
            (epoch, cu_id)
        })
        .collect::<Vec<_>>();

    let cancel_token = CancellationToken::new();
    for param_id in 0..epochs_count {
        prover
            .new_epoch(params[param_id].0, params[param_id].1, cancel_token.clone())
            .await
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    }

    let result = prover.stop_join().await;
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
