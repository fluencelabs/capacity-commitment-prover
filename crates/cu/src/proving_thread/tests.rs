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

use ccp_msr::MSRModeEnforcer;
use tokio::sync::mpsc;

use ccp_randomx::dataset::DatasetHandle;
use ccp_randomx::RandomXFlags;
use ccp_randomx::RandomXVM;
use ccp_shared::meet_difficulty::MeetDifficulty;
use ccp_shared::types::*;
use ccp_test_utils::randomx::run_light_randomx;
use ccp_test_utils::test_values as test;
use ccp_utils::run_utils::run_unordered;

use super::sync::to_stats_collecting_message::ToStatsCollectingOutlet;
use super::ProvingThreadAsync;
use super::ProvingThreadConfig;
use super::ProvingThreadFacade;
use crate::proving_thread::sync::to_state_operating_message::ToStateOperatingMessage;
use crate::proving_thread::sync::to_state_operating_message::ToStateOperatingOutlet;
use crate::proving_thread::sync::RawProof;
use crate::tests::collect_proofs;
use crate::tests::collect_proofs_with_flag;
use crate::tests::proof_sink;

pub(crate) struct ThreadInitIngredients {
    pub(crate) thread: ProvingThreadAsync,
    pub(crate) dataset: DatasetHandle,
    pub(crate) to_state_operating: ToStateOperatingOutlet,
    pub(crate) to_stats_collecting: ToStatsCollectingOutlet,
}

impl ThreadInitIngredients {
    #[allow(dead_code)]
    pub(self) async fn create(core_id: LogicalCoreId, epoch: EpochParameters, cu_id: CUID) -> Self {
        let flags = RandomXFlags::recommended_full_mem();

        let (state_operating_inlet, to_state_operating_outlet) = mpsc::channel(3);
        let (stats_collecting_inlet, to_stats_collecting_outlet) = mpsc::channel(3);

        let thread_cfg = ProvingThreadConfig {
            hashes_per_round: 1024,
            async_to_sync_queue_size: 3,
            sync_to_async_queue_size: 3,
        };
        let msr_enforcer = MSRModeEnforcer::from_preset(false, <_>::default());
        let mut thread = ProvingThreadAsync::new(
            Some(core_id),
            msr_enforcer,
            state_operating_inlet,
            stats_collecting_inlet,
            thread_cfg,
        );
        let dataset = thread.allocate_dataset(flags).await.unwrap();
        let cache = thread.create_cache(epoch, cu_id, flags).await.unwrap();
        thread
            .initialize_dataset(cache.handle(), dataset.handle(), 0, dataset.items_count())
            .await
            .unwrap();

        Self {
            thread,
            dataset: dataset.handle(),
            to_state_operating: to_state_operating_outlet,
            to_stats_collecting: to_stats_collecting_outlet,
        }
    }
}

#[tokio::test]
async fn cache_creation_works() {
    let epoch = test::generate_epoch_params(1, 0xFF);
    let local_nonce = test::generate_local_nonce(1);
    let cu_id = test::generate_cu_id(1);

    let flags = RandomXFlags::recommended();

    let (state_operating_inlet, _state_operating_outlet) = mpsc::channel(3);
    let (stats_collecting_inlet, _stats_collecting_outlet) = mpsc::channel(3);

    let msr_enforcer = MSRModeEnforcer::from_preset(false, <_>::default());
    let thread_cfg = ProvingThreadConfig {
        hashes_per_round: 1024,
        async_to_sync_queue_size: 1,
        sync_to_async_queue_size: 1,
    };
    let mut thread = ProvingThreadAsync::new(
        Some(2.into()),
        msr_enforcer,
        state_operating_inlet,
        stats_collecting_inlet,
        thread_cfg,
    );
    let actual_cache = thread.create_cache(epoch, cu_id, flags).await.unwrap();
    thread.stop_join().await.unwrap();

    let actual_vm = RandomXVM::light(actual_cache.handle(), flags).unwrap();
    let actual_result_hash = actual_vm.hash(local_nonce.as_ref());

    let global_nonce_cu = ccp_utils::hash::compute_global_nonce_cu(&epoch.global_nonce, &cu_id);
    let expected_result_hash =
        run_light_randomx(global_nonce_cu.as_slice(), local_nonce.as_ref(), flags);

    assert_eq!(actual_result_hash, expected_result_hash);
}

#[tokio::test]
async fn dataset_creation_works() {
    let epoch = test::generate_epoch_params(2, 0xFF);
    let local_nonce = test::generate_local_nonce(2);
    let cu_id = test::generate_cu_id(2);

    let flags = RandomXFlags::recommended_full_mem();

    let (state_operating_inlet, to_state_operating_outlet) = mpsc::channel(3);
    let (stats_collecting_inlet, stats_collecting_outlet) = mpsc::channel(3);

    let msr_enforcer = MSRModeEnforcer::from_preset(false, <_>::default());
    let thread_cfg = ProvingThreadConfig {
        hashes_per_round: 1024,
        async_to_sync_queue_size: 1,
        sync_to_async_queue_size: 1,
    };
    let mut thread = ProvingThreadAsync::new(
        Some(2.into()),
        msr_enforcer,
        state_operating_inlet,
        stats_collecting_inlet,
        thread_cfg,
    );
    let actual_dataset = thread.allocate_dataset(flags).await.unwrap();
    let actual_cache = thread.create_cache(epoch, cu_id, flags).await.unwrap();
    let handle = tokio::spawn(proof_sink(
        to_state_operating_outlet,
        stats_collecting_outlet,
    ));

    thread
        .initialize_dataset(
            actual_cache.handle(),
            actual_dataset.handle(),
            0,
            actual_dataset.items_count(),
        )
        .await
        .unwrap();

    thread.stop_join().await.unwrap();
    handle.await.unwrap();

    let flags = RandomXFlags::recommended_full_mem();
    let actual_vm = RandomXVM::fast(actual_dataset.handle(), flags).unwrap();
    let actual_result_hash = actual_vm.hash(local_nonce.as_ref());

    let flags = RandomXFlags::recommended();
    let global_nonce_cu = ccp_utils::hash::compute_global_nonce_cu(&epoch.global_nonce, &cu_id);
    let expected_result_hash =
        run_light_randomx(global_nonce_cu.as_slice(), local_nonce.as_ref(), flags);

    assert_eq!(actual_result_hash, expected_result_hash);
}

#[tokio::test]
async fn dataset_creation_works_with_three_threads() {
    use futures::FutureExt;

    let _ = env_logger::builder().is_test(true).try_init();

    let epoch = test::generate_epoch_params(3, 0xFF);
    let local_nonce = test::generate_local_nonce(3);
    let cu_id = test::generate_cu_id(3);

    let flags = RandomXFlags::recommended_full_mem();

    let (state_operating_inlet, _state_operating_outlet) = mpsc::channel(100);
    let (stats_collecting_inlet, _stats_collecting_outlet) = mpsc::channel(100);
    let threads_count = 3u32;

    let msr_enforcer = MSRModeEnforcer::from_preset(false, <_>::default());
    let mut threads = (0..threads_count)
        .map(|thread_id| {
            let thread_cfg = ProvingThreadConfig {
                hashes_per_round: 1024,
                async_to_sync_queue_size: 1,
                sync_to_async_queue_size: 1,
            };
            ProvingThreadAsync::new(
                Some((2 + thread_id).into()),
                msr_enforcer.clone(),
                state_operating_inlet.clone(),
                stats_collecting_inlet.clone(),
                thread_cfg,
            )
        })
        .collect::<Vec<_>>();

    let thread_1 = &mut threads[0];
    let actual_dataset = thread_1.allocate_dataset(flags).await.unwrap();
    let actual_cache = thread_1.create_cache(epoch, cu_id, flags).await.unwrap();

    let dataset_size = actual_dataset.items_count();

    let closure = |thread_id: usize, mut thread: ProvingThreadAsync| {
        let thread_id = thread_id as u64;
        let threads_count = threads_count as u64;

        let start_item = (dataset_size * thread_id) / threads_count;
        let next_start_item = (dataset_size * (thread_id + 1)) / threads_count;
        let items_count = next_start_item - start_item;

        let cache = actual_cache.handle();
        let dataset = actual_dataset.handle();

        async move {
            thread
                .initialize_dataset(cache, dataset, start_item, items_count)
                .await
                .unwrap();

            Ok::<_, ()>(thread)
        }
        .boxed()
    };
    let threads = run_unordered(threads.into_iter(), closure).await.unwrap();

    let closure = |_: usize, thread: ProvingThreadAsync| thread.stop_join().boxed();
    run_unordered(threads.into_iter(), closure).await.unwrap();

    let flags = RandomXFlags::recommended_full_mem();
    let actual_vm = RandomXVM::fast(actual_dataset.handle(), flags).unwrap();
    let actual_result_hash = actual_vm.hash(local_nonce.as_ref());

    let flags = RandomXFlags::recommended();
    let global_nonce_cu = ccp_utils::hash::compute_global_nonce_cu(&epoch.global_nonce, &cu_id);
    let expected_result_hash =
        run_light_randomx(global_nonce_cu.as_slice(), local_nonce.as_ref(), flags);

    assert_eq!(actual_result_hash, expected_result_hash);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cc_job_stopable() {
    let epoch = test::generate_epoch_params(4, 0xFF);
    let cu_id = test::generate_cu_id(4);
    let mut ingredients = ThreadInitIngredients::create(2.into(), epoch, cu_id).await;

    let flags = RandomXFlags::recommended_full_mem();
    ingredients
        .thread
        .run_cc_job(epoch, ingredients.dataset, flags, cu_id)
        .await
        .unwrap();

    let handle = tokio::spawn(async move {
        let flags = RandomXFlags::recommended();
        let global_nonce_cu = ccp_utils::hash::compute_global_nonce_cu(&epoch.global_nonce, &cu_id);

        loop {
            tokio::select! {
                message = ingredients.to_state_operating.recv() => {
                    match message {
                        Some(ToStateOperatingMessage::ProofFound { proof, .. }) => {
                            let expected_result_hash = run_light_randomx(
                                global_nonce_cu.as_slice(),
                                proof.local_nonce.as_ref(),
                                flags,
                            );
                            assert!(expected_result_hash.meet_difficulty(&epoch.difficulty));
                        }
                        Some(_) => {}
                        None => break,
                    }
                }
                _ = ingredients.to_stats_collecting.recv() => {}
            }
        }
    });

    ingredients.thread.stop_join().await.unwrap();
    let _ = handle.await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cc_job_pausable() {
    use std::sync;

    let epoch = test::generate_epoch_params(5, 0xFF);
    let cu_id = test::generate_cu_id(5);
    let mut ingredients = ThreadInitIngredients::create(3.into(), epoch, cu_id).await;

    let flags = RandomXFlags::recommended_full_mem();
    ingredients
        .thread
        .run_cc_job(epoch, ingredients.dataset, flags, cu_id)
        .await
        .unwrap();

    let is_thread_paused = sync::Arc::new(sync::Mutex::new(false));

    let handle = tokio::spawn(collect_proofs_with_flag(
        ingredients.to_state_operating,
        ingredients.to_stats_collecting,
        is_thread_paused.clone(),
    ));

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    ingredients.thread.pause().await.unwrap();

    {
        let mut is_thread_paused_locked = is_thread_paused.lock().unwrap();
        *is_thread_paused_locked = true;
    }

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    ingredients.thread.stop_join().await.unwrap();

    let (proofs_before_pause, proofs_after_pause) = handle.await.unwrap();
    assert!(!proofs_before_pause.is_empty());
    assert!(proofs_after_pause.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn proving_thread_works() {
    let _ = env_logger::builder().is_test(true).try_init();

    let epoch = test::generate_epoch_params(6, 0xFF);
    let cu_id = test::generate_cu_id(6);
    let ingredients = ThreadInitIngredients::create(4.into(), epoch, cu_id).await;

    let flags = RandomXFlags::recommended_full_mem();
    ingredients
        .thread
        .run_cc_job(epoch, ingredients.dataset, flags, cu_id)
        .await
        .unwrap();

    let handle = tokio::spawn(collect_proofs(
        ingredients.to_state_operating,
        ingredients.to_stats_collecting,
    ));

    ingredients.thread.stop_join().await.unwrap();
    let proofs = handle.await.unwrap();
    let proof = proofs[0];

    let flags = RandomXFlags::recommended();
    let global_nonce_cu = ccp_utils::hash::compute_global_nonce_cu(&epoch.global_nonce, &cu_id);
    let expected_result_hash = run_light_randomx(
        global_nonce_cu.as_slice(),
        proof.local_nonce.as_ref(),
        flags,
    );

    assert!(expected_result_hash.meet_difficulty(&epoch.difficulty));
}

fn batch_proof_verification(proofs: impl Iterator<Item = RawProof>, difficulty: Difficulty) {
    use ccp_randomx::cache::Cache;

    let flags = RandomXFlags::recommended();

    for proof in proofs {
        let global_nonce_cu =
            ccp_utils::hash::compute_global_nonce_cu(&proof.epoch.global_nonce, &proof.cu_id);
        let cache = Cache::new(&global_nonce_cu, flags).unwrap();
        let vm = RandomXVM::light(cache.handle(), flags).unwrap();

        let result = vm.hash(proof.local_nonce.as_ref());
        assert_eq!(result, proof.result_hash);
        assert!(result.meet_difficulty(&difficulty));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn proving_therad_produces_repeatable_hashes() {
    let epoch = test::generate_epoch_params(7, 0xFF);
    let cu_id = test::generate_cu_id(7);
    let ingredients = ThreadInitIngredients::create(5.into(), epoch, cu_id).await;

    let handle = tokio::spawn(collect_proofs(
        ingredients.to_state_operating,
        ingredients.to_stats_collecting,
    ));

    let flags = RandomXFlags::recommended_full_mem();
    ingredients
        .thread
        .run_cc_job(epoch, ingredients.dataset, flags, cu_id)
        .await
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    ingredients.thread.stop_join().await.unwrap();
    let proofs = handle.await.unwrap();
    batch_proof_verification(proofs.into_iter(), epoch.difficulty);
}
