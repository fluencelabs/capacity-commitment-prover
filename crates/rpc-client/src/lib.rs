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

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![deny(
    dead_code,
    nonstandard_style,
    unused_imports,
    unused_mut,
    unused_variables,
    unused_unsafe,
    unreachable_patterns
)]

use std::collections::HashMap;
use std::time::Duration;

pub use jsonrpsee::core::ClientError;
use jsonrpsee::proc_macros::rpc;
pub use jsonrpsee::types::ErrorObjectOwned;

use ccp_shared::proof::BatchRequest;
use ccp_shared::proof::BatchResponse;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::Difficulty;
use ccp_shared::types::GlobalNonce;
use ccp_shared::types::LogicalCoreId;
use ccp_shared::types::PhysicalCoreId;
use ccp_shared::types::CUID;

// n.b.: the rpc macro also defines CcpRpcClient type which is a working async JSON RPC client.
#[rpc(server, client, namespace = "ccp")]
pub trait CCPRpc {
    #[method(name = "on_active_commitment", param_kind = map)]
    async fn on_active_commitment(
        &self,
        global_nonce: GlobalNonce,
        difficulty: Difficulty,
        cu_allocation: HashMap<PhysicalCoreId, CUID>,
    ) -> Result<(), ErrorObjectOwned>;

    #[method(name = "on_no_active_commitment")]
    async fn on_no_active_commitment(&self) -> Result<(), ErrorObjectOwned>;

    #[method(name = "get_proofs_after")]
    async fn get_proofs_after(
        &self,
        last_known_proofs: HashMap<CUID, ProofIdx>,
        limit: usize,
    ) -> Result<Vec<BatchResponse>, ErrorObjectOwned>;

    #[method(name = "get_batch_proofs_after")]
    async fn get_batch_proofs_after(
        &self,
        reqs: HashMap<CUID, BatchRequest>,
        min_batch_count: usize,
        max_batch_count: usize,
    ) -> Result<Vec<BatchResponse>, ErrorObjectOwned>;

    #[method(name = "realloc_utility_cores", param_kind = map)]
    async fn realloc_utility_cores(&self, utility_core_ids: Vec<LogicalCoreId>);

    #[method(name = "health")]
    async fn health(&self) -> Result<String, ErrorObjectOwned>;
}

pub struct CCPRpcHttpClient {
    inner: jsonrpsee::http_client::HttpClient,
}

impl CCPRpcHttpClient {
    pub async fn new(endpoint_url: String) -> Result<Self, ClientError> {
        let inner = jsonrpsee::http_client::HttpClientBuilder::default().build(endpoint_url)?;

        Ok(Self { inner })
    }

    pub async fn with_timeout(
        endpoint_url: String,
        request_timeout: Duration,
    ) -> Result<Self, ClientError> {
        let builder =
            jsonrpsee::http_client::HttpClientBuilder::default().request_timeout(request_timeout);
        let inner = builder.build(endpoint_url)?;

        Ok(Self { inner })
    }

    #[inline]
    pub fn from_http_client(client: jsonrpsee::http_client::HttpClient) -> Self {
        Self { inner: client }
    }

    pub async fn on_active_commitment(
        &self,
        global_nonce: GlobalNonce,
        difficulty: Difficulty,
        cu_allocation: HashMap<PhysicalCoreId, CUID>,
    ) -> Result<(), ClientError> {
        CCPRpcClient::on_active_commitment(&self.inner, global_nonce, difficulty, cu_allocation)
            .await
    }

    #[inline]
    pub async fn on_no_active_commitment(&self) -> Result<(), ClientError> {
        CCPRpcClient::on_no_active_commitment(&self.inner).await
    }

    #[inline]
    pub async fn get_proofs_after(
        &self,
        last_known_proofs: HashMap<CUID, ProofIdx>,
        limit: usize,
    ) -> Result<Vec<BatchResponse>, ClientError> {
        CCPRpcClient::get_proofs_after(&self.inner, last_known_proofs, limit).await
    }

    #[inline]
    pub async fn get_batch_proofs_after(
        &self,
        reqs: HashMap<CUID, BatchRequest>,
        min_batch_count: usize,
        max_batch_count: usize,
    ) -> Result<Vec<BatchResponse>, ClientError> {
        CCPRpcClient::get_batch_proofs_after(&self.inner, reqs, min_batch_count, max_batch_count)
            .await
    }

    #[inline]
    pub async fn realloc_utility_cores(
        &self,
        utility_core_ids: Vec<LogicalCoreId>,
    ) -> Result<(), ClientError> {
        CCPRpcClient::realloc_utility_cores(&self.inner, utility_core_ids).await
    }

    pub async fn health(&self) -> Result<String, ClientError> {
        CCPRpcClient::health(&self.inner).await
    }
}
