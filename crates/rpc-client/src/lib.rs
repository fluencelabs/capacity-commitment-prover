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

mod or_hex;

use std::collections::HashMap;
use std::time::Duration;

use ccp_shared::proof::ProofIdx;
use ccp_shared::types::LogicalCoreId;
use jsonrpsee::core::ClientError;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::ErrorObjectOwned;

use ccp_shared::proof::CCProof;
use ccp_shared::types::Difficulty;
use ccp_shared::types::GlobalNonce;
use ccp_shared::types::PhysicalCoreId;
use ccp_shared::types::CUID;

pub use crate::or_hex::OrHex;

// n.b.: the rpc macro also defines CcpRpcClient type which is a working async JSON RPC client.
#[rpc(server, client, namespace = "ccp")]
pub trait CCPRpc {
    #[method(name = "on_active_commitment", param_kind = map)]
    async fn on_active_commitment(
        &self,
        global_nonce: OrHex<GlobalNonce>,
        difficulty: OrHex<Difficulty>,
        cu_allocation: HashMap<PhysicalCoreId, OrHex<CUID>>,
    ) -> Result<(), ErrorObjectOwned>;

    #[method(name = "on_no_active_commitment")]
    async fn on_no_active_commitment(&self) -> Result<(), ErrorObjectOwned>;

    #[method(name = "get_proofs_after")]
    async fn get_proofs_after(
        &self,
        proof_idx: ProofIdx,
        limit: usize,
    ) -> Result<Vec<CCProof>, ErrorObjectOwned>;

    #[method(name = "realloc_utility_cores", param_kind = map)]
    async fn realloc_utility_cores(&self, utility_core_ids: Vec<LogicalCoreId>);
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
        let cu_allocation = cu_allocation
            .into_iter()
            .map(|(k, v)| (k, v.into()))
            .collect();
        CCPRpcClient::on_active_commitment(
            &self.inner,
            global_nonce.into(),
            difficulty.into(),
            cu_allocation,
        )
        .await
    }

    #[inline]
    pub async fn on_no_active_commitment(&self) -> Result<(), ClientError> {
        CCPRpcClient::on_no_active_commitment(&self.inner).await
    }

    #[inline]
    pub async fn get_proofs_after(
        &self,
        proof_idx: ProofIdx,
        limit: usize,
    ) -> Result<Vec<CCProof>, ClientError> {
        CCPRpcClient::get_proofs_after(&self.inner, proof_idx, limit).await
    }

    #[inline]
    pub async fn realloc_utility_cores(
        &self,
        utility_core_ids: Vec<LogicalCoreId>,
    ) -> Result<(), ClientError> {
        CCPRpcClient::realloc_utility_cores(&self.inner, utility_core_ids).await
    }
}
