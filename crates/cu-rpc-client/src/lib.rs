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

use std::time::Duration;

pub use jsonrpsee::core::ClientError;
use jsonrpsee::proc_macros::rpc;
pub use jsonrpsee::types::ErrorObjectOwned;

use ccp_cu::RawProof;

// n.b.: the rpc macro also defines CcpCuRpcClient type which is a working async JSON RPC client.
#[rpc(server, client, namespace = "ccp_cu")]
pub trait CcpCuRpc {
    #[method(name = "report_proof", param_kind = map)]
    async fn report_proof(&self, proof: RawProof) -> Result<(), ErrorObjectOwned>;
}

pub struct CcpCuRpcHttpClient {
    inner: jsonrpsee::http_client::HttpClient,
}

impl CcpCuRpcHttpClient {
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

    pub async fn report_proof(&self, proof: RawProof) -> Result<(), ClientError> {
        CcpCuRpcClient::report_proof(&self.inner, proof).await
    }
}
