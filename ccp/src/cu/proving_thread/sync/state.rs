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
use crate::cu::proving_thread::messages::NewCCJob;

#[derive(Debug)]
pub(crate) enum ThreadState {
    CCJob { parameters: RandomXJob },
    Stop,
    WaitForMessage,
}

#[derive(Debug)]
pub(crate) struct RandomXJob {
    pub(crate) vm: randomx::RandomXVM<DatasetHandle>,
    pub(crate) local_nonce: LocalNonce,
    pub(crate) epoch: EpochParameters,
    pub(crate) cu_id: CUID,
}

impl RandomXJob {
    pub(crate) fn from_cc_job(cc_job: NewCCJob) -> STResult<Self> {
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
        };
        Ok(params)
    }

    pub(crate) fn hash_first(&mut self) {
        self.vm.hash_first(self.local_nonce.as_ref());
    }

    pub(crate) fn hash_last(&mut self) -> ResultHash {
        self.local_nonce.next();
        self.vm.hash_last()
    }

    pub(crate) fn hash_next(&mut self) -> ResultHash {
        self.local_nonce.next();
        self.vm.hash_next(self.local_nonce.as_ref())
    }

    pub(crate) fn create_golden_proof(&mut self, result_hash: ResultHash) -> RawProof {
        self.local_nonce.prev();

        let proof = RawProof::new(self.epoch, self.local_nonce, self.cu_id, result_hash);
        self.local_nonce.next();

        proof
    }
}