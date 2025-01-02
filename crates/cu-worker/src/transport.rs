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

use std::future::Future;

use ccp_cu::proving_thread::sync::to_stats_collecting_message::ToStatsCollectingMessage;
use ccp_cu::RawProof;
use ccp_cu_rpc_client::CcpCuRpcHttpClient;

pub(crate) trait ProofUpstreamTransport: Send {
    type Error;

    fn report(
        &mut self,
        proofs: &[RawProof],
        stats: &[ToStatsCollectingMessage],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

impl ProofUpstreamTransport for CcpCuRpcHttpClient {
    type Error = ccp_cu_rpc_client::ClientError;

    async fn report(
        &mut self,
        proof: &[RawProof],
        stats: &[ToStatsCollectingMessage],
    ) -> Result<(), Self::Error> {
        Self::report(self, proof, stats).await
    }
}
