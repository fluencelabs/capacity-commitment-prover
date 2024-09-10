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

use hyper_util::rt::TokioIo;
use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint, Uri};

use ccp_cu::RawProof;
use tower::service_fn;

use crate::ccp_collector_rpc::collector_client::CollectorClient;
use crate::ccp_collector_rpc::ReportRequest;

pub mod ccp_collector_rpc {
    tonic::include_proto!("cu_rpc_client");
}

pub type ClientError = tonic::Status;

// n.b.: the rpc macro also defines CcpCuRpcClient type which is a working async JSON RPC client.
pub struct CcpCuRpcHttpClient {
    inner: CollectorClient<Channel>,
}

impl CcpCuRpcHttpClient {
    pub async fn new_http(url: String) -> Result<Self, tonic::transport::Error> {
        let inner = CollectorClient::connect(url).await?;
        Ok(Self { inner })
    }

    pub async fn new_unix_socket(path: String) -> Result<Self, tonic::transport::Error> {
        // The URL is dummy and is ignored by the connector function
        let channel = Endpoint::try_from("http://[::]:9999")?
            .connect_with_connector(service_fn(move |_: Uri| {
                let path = path.clone();
                async {
                    // Connect to a Uds socket
                    Ok::<_, std::io::Error>(TokioIo::new(UnixStream::connect(path).await?))
                }
            }))
            .await?;
        let inner = CollectorClient::new(channel);
        Ok(Self { inner })
    }

    pub async fn auto(endpoint: String) -> Result<Self, tonic::transport::Error> {
        if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            Self::new_http(endpoint).await
        } else {
            Self::new_unix_socket(endpoint).await
        }
    }

    pub async fn report_proof(&mut self, proof: &RawProof) -> Result<(), ClientError> {
        let epoch_message = self::ccp_collector_rpc::EpochParameters {
            difficulty: proof.epoch.difficulty.to_string(),
            global_nonce: proof.epoch.global_nonce.to_string(),
        };
        let proof_message = ReportRequest {
            epoch: Some(epoch_message),
            cu_id: proof.cu_id.to_string(),
            local_nonce: proof.local_nonce.to_string(),
            result_hash: proof.result_hash.to_string(),
        };

        self.inner.report_proof(proof_message).await?;
        Ok(())
    }
}
