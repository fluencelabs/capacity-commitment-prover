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

pub(crate) struct ThreadInitIngredients {
    pub(crate) thread: ProvingThreadAsync,
    pub(crate) dataset_handle: DatasetHandle,
    pub(crate) to_utility: ToUtilityOutlet,
}

impl ThreadInitIngredients {
    pub(self) fn new(thread: ProvingThreadAsync,
                     dataset_handle: DatasetHandle,
                     to_utility: ToUtilityOutlet) -> Self {
        Self {
            thread,
            dataset_handle,
            to_utility
        }
    }

    #[allow(dead_code)]
    async fn create_thread_init_dataset(
        core_id: LogicalCoreId,
        epoch: EpochParameters,
        cu_id: CUID,
    ) -> Self {
        let flags = RandomXFlags::recommended_full_mem();

        let (inlet, outlet) = mpsc::channel(1);

        let mut thread = ProvingThreadAsync::new(core_id, inlet, false);
        let dataset = thread.allocate_dataset(flags).await.unwrap();
        let cache = thread
            .create_cache(epoch.global_nonce, cu_id, flags)
            .await
            .unwrap();
        thread
            .initialize_dataset(
                cache.handle(),
                dataset.handle(),
                0,
                dataset.items_count(),
            )
            .await
            .unwrap();

        Self {
            thread,
            dataset_handle: dataset.handle(),
            to_utility: outlet
        }
    }
