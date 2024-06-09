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

use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::ops::RangeBounds;

use ccp_shared::proof::BatchRequest;
use ccp_shared::proof::BatchResponse;
use ccp_shared::proof::CCBatchedProofs;
use ccp_shared::proof::CCProof;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::CUID;
use itertools::Itertools;

// TODO it is not quite a queue; cache?
#[derive(Debug, Default)]
pub(crate) struct ProofQueue {
    cu_proofs: HashMap<CUID, CuState>,
}

#[derive(Debug, Default)]
struct CuState {
    proofs: VecDeque<CCProof>,
}

impl CuState {
    fn push_back(&mut self, proof: CCProof) {
        self.proofs.push_back(proof)
    }

    fn len(&self) -> usize {
        self.proofs.len()
    }

    fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }

    fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> impl Iterator<Item = CCProof> + '_ {
        self.proofs.drain(range)
    }
}

impl<'state> IntoIterator for &'state CuState {
    type Item = &'state CCProof;

    type IntoIter = <&'state VecDeque<CCProof> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.proofs.iter()
    }
}

impl ProofQueue {
    pub(crate) fn insert(&mut self, proof: CCProof) {
        self.cu_proofs
            .entry(proof.cu_id)
            .or_default()
            .push_back(proof);
    }

    pub(crate) fn clear(&mut self) {
        self.cu_proofs.clear();
    }

    pub(crate) fn get_batched_proofs_after(
        &mut self,
        reqs: &HashMap<CUID, BatchRequest>,
        min_batch_count: usize,
        max_batch_count: usize,
    ) -> CCBatchedProofs {
        if max_batch_count == 0 {
            return <_>::default();
        }

        // this is the only destructive operation in this function
        // TODO update from disk if lags? it needs a data source for it
        self.rewind_forward(
            reqs.iter()
                .map(|(cuid, batch_req)| (cuid, &batch_req.last_seen_proof_idx)),
        );

        self.collect_batches(reqs, min_batch_count, max_batch_count)
    }

    fn collect_batches(
        &self,
        reqs: &HashMap<CUID, BatchRequest>,
        min_batch_count: usize,
        max_batch_count: usize,
    ) -> Vec<BatchResponse> {
        // first, check that the required number of batches is available
        let available_batches_count: usize = self
            .cu_proofs
            .iter()
            .filter_map(|(cid, state)| {
                reqs.get(cid)
                    .and_then(|batch_req| state.len().checked_div(batch_req.proof_batch_size))
            })
            .sum();
        if available_batches_count < min_batch_count {
            return <_>::default();
        }

        let batches_to_collect = std::cmp::min(available_batches_count, max_batch_count);

        // we have to wrap the modifiable value into a RefCell to make the iter clonable and usable in the Cycle
        let cu_iters = self
            .cu_proofs
            .iter()
            .map(|(cuid, state)| (cuid, RefCell::new(state.iter())))
            .collect_vec();
        // we try to get proofs in round-robin fashion.  Alternative algorithms are possible too, for example,
        // preferring longest queues (with a binary heap)
        let mut cu_round_robin_iters = cu_iters.iter().cycle();

        let mut collected_batches = Vec::<BatchResponse>::with_capacity(min_batch_count);

        while collected_batches.len() < batches_to_collect {
            // it doesn't loop infinitely because we have verified to have enough batches (`available_batches_count`)
            for (&cuid, cu_proofs_iter) in &mut cu_round_robin_iters {
                let mut cu_proofs_iter = cu_proofs_iter.borrow_mut();

                if let Some(batch_req) = reqs.get(&cuid) {
                    if cu_proofs_iter.len() >= batch_req.proof_batch_size {
                        let batch = (&mut *cu_proofs_iter)
                            .take(batch_req.proof_batch_size)
                            .cloned()
                            .collect_vec();
                        collected_batches.push(BatchResponse::new(cuid, batch));

                        // we have found a batch; now retry the search from the next CU if needed, until
                        // exact number of batches is collected
                        break;
                    }
                }
            }
        }

        collected_batches
    }

    pub(crate) fn get_all_proofs_after(
        &mut self,
        last_known_proofs: &HashMap<CUID, ProofIdx>,
        mut limit: usize,
    ) -> CCBatchedProofs {
        self.rewind_forward(last_known_proofs.iter());

        let mut output = CCBatchedProofs::default();

        for (cuid, state) in self.cu_proofs.iter_mut() {
            if state.len() > 0 {
                let remnant = std::cmp::min(limit, state.len());
                output.push(BatchResponse::new(*cuid, state.drain(..remnant).collect()));
                limit -= remnant;
            }
            if limit == 0 {
                break;
            }
        }
        output
    }

    fn rewind_forward<'a>(
        &mut self,
        last_known_proofs: impl Iterator<Item = (&'a CUID, &'a ProofIdx)>,
    ) {
        for (cid, last_known_idx) in last_known_proofs {
            if let Some(n) = self.cu_proofs.get_mut(cid) {
                while let Some(head) = n.proofs.front() {
                    if head.id.idx <= *last_known_idx {
                        n.proofs.pop_front();
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::{
        collections::{HashMap, HashSet},
        str::FromStr,
    };

    use ccp_randomx::ResultHash;
    use ccp_shared::{
        proof::{BatchRequest, BatchResponse, CCProof, CCProofId, ProofIdx},
        types::{Difficulty, GlobalNonce, LocalNonce, CUID},
    };

    use crate::proof_queue::ProofQueue;

    const CUID1: CUID = CUID::new([
        33, 247, 206, 99, 242, 79, 217, 190, 58, 45, 87, 221, 151, 162, 217, 11, 43, 151, 160, 77,
        199, 173, 183, 140, 130, 71, 222, 113, 189, 117, 174, 63,
    ]);
    const CUID2: CUID = CUID::new([
        192, 52, 100, 105, 186, 121, 170, 203, 69, 85, 100, 205, 144, 66, 82, 85, 108, 121, 68, 68,
        227, 24, 101, 29, 154, 84, 84, 26, 234, 134, 65, 54,
    ]);
    const CUID3: CUID = CUID::new([
        12, 241, 204, 143, 159, 82, 32, 206, 5, 86, 97, 254, 160, 206, 69, 152, 73, 240, 81, 169,
        178, 86, 202, 101, 130, 208, 210, 4, 54, 135, 86, 112,
    ]);
    const CUID4: CUID = CUID::new([
        155, 155, 197, 6, 137, 232, 33, 106, 32, 129, 163, 210, 155, 80, 108, 136, 2, 86, 136, 118,
        70, 62, 148, 186, 108, 247, 130, 220, 239, 249, 205, 102,
    ]);
    const CUID5: CUID = CUID::new([
        14, 11, 219, 226, 211, 166, 132, 247, 214, 15, 70, 220, 220, 22, 217, 252, 49, 246, 168,
        205, 187, 145, 129, 243, 53, 14, 38, 4, 248, 246, 36, 233,
    ]);

    fn gen_queue(count: u8) -> ProofQueue {
        let mut queue = ProofQueue::default();

        let global_nonce = GlobalNonce::from_str(
            "851bdb35e2e7cd1645beed2893e27bb247e80b22a4d5fa11696352d74a4b8fe0",
        )
        .unwrap();
        let local_nonce: LocalNonce = LocalNonce::from_str(
            "50c830c9fa5bf8bd62e69f48db3e5c7bb13eb3ce393b08cbe62f193e43cdab04",
        )
        .unwrap();
        let difficulty = Difficulty::from_str(
            "622ae4af56f0aa2ea407e5e1dddf29a8a5a3b276248b16e3193135cf68dde846",
        )
        .unwrap();

        let var_name = [CUID1, CUID2, CUID3, CUID4, CUID5];
        let var_name = var_name;
        let cu_s = var_name;

        for (id, cu_id) in cu_s.iter().cycle().enumerate().take(count as usize) {
            let proof_idx = ProofIdx::from_str(&(id + 1).to_string()).unwrap();
            let proof_id = CCProofId::new(global_nonce, difficulty, proof_idx);
            let result_hash = ResultHash::from_slice([
                id as u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0,
            ]);
            let proof = CCProof::new(proof_id, local_nonce, *cu_id, result_hash);
            queue.insert(proof);
        }

        queue
    }

    fn get_simple_requests(proof_batch_size: usize) -> HashMap<CUID, BatchRequest> {
        let last_seen_proof_idx = <_>::default();
        let request = BatchRequest {
            last_seen_proof_idx,
            proof_batch_size,
        };
        maplit::hashmap! {
            CUID1 => request.clone(),
            CUID2 => request.clone(),
            CUID3 => request.clone(),
            CUID4 => request.clone(),
            CUID5 => request,
        }
    }

    fn update_request(req: &mut HashMap<CUID, BatchRequest>, proofs: &Vec<BatchResponse>) {
        for resp in proofs {
            let proof_idx = resp
                .proof_batches
                .iter()
                .map(|p| p.id.idx)
                .max()
                .unwrap_or_default();
            req.get_mut(&resp.cu_id).unwrap().last_seen_proof_idx = proof_idx;
        }
    }

    #[test]
    fn test_empty() {
        let mut queue = ProofQueue::default();
        let req = get_simple_requests(6);
        assert!(queue.get_batched_proofs_after(&req, 5, 10).is_empty());
    }

    #[test]
    fn test_min_exactly() {
        let mut queue = gen_queue(30);
        let req = get_simple_requests(6);
        assert!(!queue.get_batched_proofs_after(&req, 5, 6).is_empty());
    }

    #[test]
    fn test_min_one_short() {
        let mut queue = gen_queue(29);
        let req = get_simple_requests(6);
        assert!(queue.get_batched_proofs_after(&req, 5, 5).is_empty());
    }

    #[test]
    fn test_min_one_more() {
        let mut queue = gen_queue(31);
        let req = get_simple_requests(6);
        assert!(!queue.get_batched_proofs_after(&req, 5, 5).is_empty());
    }

    #[test]
    fn test_exactly_uneven() {
        let mut queue = gen_queue(7);
        let req = get_simple_requests(1);
        let proofs = queue.get_batched_proofs_after(&req, 5, 7);

        let mut counts = HashMap::<&CUID, usize>::new();
        for res in &proofs {
            *counts.entry(&res.cu_id).or_default() += res.proof_batches.len();
        }
        let long_count = counts.values().filter(|&&count| count > 1).count();
        assert_eq!(long_count, 2);
    }

    #[test]
    fn test_request_idempotent() {
        let mut queue = gen_queue(14);
        let req = get_simple_requests(1);
        let proofs1 = queue.get_batched_proofs_after(&req, 7, 7);

        let proofs2 = queue.get_batched_proofs_after(&req, 7, 7);
        let all_proofs1 = proofs1
            .iter()
            .map(|res| &res.proof_batches)
            .flatten()
            .collect::<HashSet<_>>();
        let all_proofs2 = proofs2
            .iter()
            .map(|res| &res.proof_batches)
            .flatten()
            .collect::<HashSet<_>>();

        assert_eq!(all_proofs1, all_proofs2);
    }

    #[test]
    fn test_rewind_on_success1() {
        let mut queue = gen_queue(14);
        let req = get_simple_requests(1);
        let proofs1 = queue.get_batched_proofs_after(&req, 7, 7);

        let mut rewind_request = req.clone();
        update_request(&mut rewind_request, &proofs1);
        let _ = queue.get_batched_proofs_after(&rewind_request, 7, 7);

        // rerun original request
        let proofs2 = queue.get_batched_proofs_after(&req, 7, 7);

        let all_proofs1 = proofs1
            .iter()
            .flat_map(|res| &res.proof_batches)
            .collect::<HashSet<_>>();
        let all_proofs2 = proofs2
            .iter()
            .flat_map(|res| &res.proof_batches)
            .collect::<HashSet<_>>();

        assert_eq!(
            all_proofs1.intersection(&all_proofs2).count(),
            0,
            "{:?}",
            all_proofs1.intersection(&all_proofs2).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_rewind_on_success2() {
        let mut queue = gen_queue(13);
        let mut req = get_simple_requests(1);

        let proofs1 = queue.get_batched_proofs_after(&req, 7, 7);
        assert!(!proofs1.is_empty());

        update_request(&mut req, &proofs1);

        let res2 = queue.get_batched_proofs_after(&req, 7, 7);
        assert!(res2.is_empty());
    }

    #[test]
    fn test_no_rewind_on_failure() {
        let mut queue = gen_queue(6);
        let req = get_simple_requests(1);
        let res1 = queue.get_batched_proofs_after(&req, 7, 7);
        let res2 = queue.get_batched_proofs_after(&req, 6, 6);

        assert!(res1.is_empty());
        assert!(!res2.is_empty());
    }

    #[test]
    fn test_reset() {
        let mut queue = gen_queue(14);
        let req = get_simple_requests(1);

        let _ = queue.get_batched_proofs_after(&req, 7, 7);
        queue.clear();
        let proofs2 = queue.get_batched_proofs_after(&req, 7, 7);
        assert!(proofs2.is_empty());
    }

    #[test]
    fn test_max_queue_1() {
        let mut queue = gen_queue(17); // 3cuids * 3proofs + 2 * 4
        let req = get_simple_requests(2);

        let proofs = queue.get_batched_proofs_after(&req, 3, 6);
        assert_eq!(proofs.len(), 6);
    }

    #[test]
    fn test_max_queue_2() {
        let mut queue = gen_queue(17); // 3cuids * 3proofs + 2 * 4
        let req = get_simple_requests(2);

        let proofs = queue.get_batched_proofs_after(&req, 3, 8);
        assert_eq!(proofs.len(), 7);
    }

    #[test]
    fn test_max_queue_exact() {
        let mut queue = gen_queue(17);
        let req = get_simple_requests(2);

        let proofs = queue.get_batched_proofs_after(&req, 3, 7);
        assert_eq!(proofs.len(), 7);
    }

    #[test]
    fn test_uneven_requests() {
        let mut queue = gen_queue(5 * 4);

        let mut req = get_simple_requests(4);
        req.insert(
            CUID1,
            BatchRequest {
                last_seen_proof_idx: <_>::default(),
                proof_batch_size: 2,
            },
        );

        let proofs = queue.get_batched_proofs_after(&req, 5, 6);
        assert_eq!(proofs.len(), 6);
    }
}
