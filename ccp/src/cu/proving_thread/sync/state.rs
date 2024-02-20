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

use ccp_shared::types::EpochParameters;
use ccp_shared::types::CUID;

use ccp_shared::types::LocalNonce;
use randomx::dataset::DatasetHandle;
use randomx::ResultHash;
use randomx_rust_wrapper as randomx;

use super::local_nonce::NonceIterable;
use super::raw_proof::RawProof;
use super::STResult;
use crate::cu::proving_thread::messages::AsyncToSyncMessage;
use crate::cu::proving_thread::messages::NewCCJob;
use crate::utility_thread::message::ToUtilityInlet;
use crate::utility_thread::message::ToUtilityMessage;

/// The state machine of the sync part of proving thread, it
#[derive(Debug)]
pub(crate) enum ThreadState {
    CCJob { job: RandomXJob },
    NewMessage { message: AsyncToSyncMessage },
    WaitForMessage,
    Stop,
}

#[derive(Debug)]
pub(crate) struct RandomXJob {
    vm: randomx::RandomXVM<DatasetHandle>,
    local_nonce: LocalNonce,
    epoch: EpochParameters,
    cu_id: CUID,
    hashes_per_round: usize,
}

impl RandomXJob {
    pub(crate) fn from_cc_job(cc_job: NewCCJob, hashes_per_round: usize) -> STResult<Self> {
        let NewCCJob {
            dataset,
            flags,
            epoch,
            cu_id,
        } = cc_job;

        let vm = randomx::RandomXVM::fast(dataset, flags)?;
        let local_nonce = LocalNonce::random();

        let params = Self {
            vm,
            local_nonce,
            epoch,
            cu_id,
            hashes_per_round,
        };
        Ok(params)
    }

    pub(crate) fn cc_prove(&mut self, to_utility: &ToUtilityInlet) -> STResult<()> {
        use ccp_shared::meet_difficulty::MeetDifficulty;

        self.hash_first();

        for hash_id in 0..self.hashes_per_round {
            let result_hash = if self.is_last_iteration(hash_id) {
                self.hash_last()
            } else {
                self.hash_next()
            };

            if result_hash.meet_difficulty(&self.epoch.difficulty) {
                log::info!("proving_thread_sync: found new golden result hash {result_hash:?}\nfor local_nonce {:?}", self.local_nonce);

                let proof = self.create_golden_proof(result_hash);
                let message = ToUtilityMessage::proof_found(proof);
                to_utility.blocking_send(message)?;
            }
        }

        Ok(())
    }

    fn is_last_iteration(&self, hash_id: usize) -> bool {
        hash_id == self.hashes_per_round - 1
    }

    fn hash_first(&mut self) {
        self.vm.hash_first(self.local_nonce.as_ref());
    }

    fn hash_last(&mut self) -> ResultHash {
        self.local_nonce.next();
        self.vm.hash_last()
    }

    fn hash_next(&mut self) -> ResultHash {
        self.local_nonce.next();
        self.vm.hash_next(self.local_nonce.as_ref())
    }

    fn create_golden_proof(&mut self, result_hash: ResultHash) -> RawProof {
        self.local_nonce.prev();

        let proof = RawProof::new(self.epoch, self.local_nonce, self.cu_id, result_hash);
        self.local_nonce.next();

        proof
    }
}
