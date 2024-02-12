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

use randomx::dataset::DatasetHandle;
use randomx::RandomXFlags;
use randomx_rust_wrapper as randomx;

use super::LocalNonce;
use super::PTResult;
use crate::cu::proving_thread::messages::RawProof;
use crate::Difficulty;

#[derive(Debug)]
pub(crate) struct RandomXJobParams<'vm> {
    pub(crate) vm: randomx::RandomXVM<'vm, DatasetHandle>,
    pub(crate) local_nonce: LocalNonce,
    pub(crate) difficulty: Difficulty,
    pub(crate) proof_receiver_inlet: mpsc::Sender<RawProof>,
}

impl<'params> RandomXJobParams<'params> {
    pub(crate) fn new(
        dataset: DatasetHandle,
        flags: RandomXFlags,
        difficulty: Difficulty,
        proof_receiver_inlet: mpsc::Sender<RawProof>,
    ) -> PTResult<Self> {
        let vm = randomx::RandomXVM::fast(&dataset, flags)?;
        let local_nonce = LocalNonce::random();

        let params = Self {
            vm,
            local_nonce,
            difficulty,
            proof_receiver_inlet,
        };
        Ok(params)
    }

    pub(crate) fn from_vm<'vm: 'params>(
        vm: randomx::RandomXVM<'vm, DatasetHandle>,
        local_nonce: LocalNonce,
        difficulty: Difficulty,
        proof_receiver_inlet: mpsc::Sender<RawProof>,
    ) -> Self {
        Self {
            vm,
            local_nonce,
            difficulty,
            proof_receiver_inlet,
        }
    }
}

#[derive(Debug)]
pub(crate) enum ThreadState<'vm> {
    CCJob { parameters: RandomXJobParams<'vm> },
    Stop,
    WaitForMessage,
}
