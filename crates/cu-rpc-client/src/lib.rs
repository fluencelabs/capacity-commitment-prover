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
use tower::service_fn;

pub use ccp_shared::proof::raw::RawProof;
pub use ccp_shared::types::EpochParameters;

pub use crate::ccp_collector_rpc::collector_client::CollectorClient;
pub use crate::ccp_collector_rpc::MalformedRpcProof;
pub use crate::ccp_collector_rpc::ReportRequest;
pub use crate::ccp_collector_rpc::RpcProof;

pub mod ccp_collector_rpc {
    use crate::RawProof;

    tonic::include_proto!("cu_rpc_client");

    impl From<&RawProof> for RpcProof {
        fn from(value: &RawProof) -> Self {
            let epoch = RpcEpochParameters {
                difficulty: value.epoch.difficulty.to_string(),
                global_nonce: value.epoch.global_nonce.to_string(),
            };
            Self {
                epoch: Some(epoch),
                cu_id: value.cu_id.to_string(),
                local_nonce: value.local_nonce.to_string(),
                result_hash: value.result_hash.to_string(),
            }
        }
    }

    #[derive(thiserror::Error, Debug, Clone)]
    pub enum MalformedRpcProof {
        #[error("field {0} is undefined")]
        MissingField(&'static str),
        #[error("field {0} is malformed: {1:?}")]
        MalformedField(&'static str, hex::FromHexError),
    }

    impl TryFrom<&RpcProof> for RawProof {
        type Error = MalformedRpcProof;

        fn try_from(value: &RpcProof) -> Result<Self, Self::Error> {
            use MalformedRpcProof::*;

            let epoch_params = value.epoch.as_ref().ok_or(MissingField("epoch"))?;
            let epoch = ccp_shared::types::EpochParameters {
                global_nonce: epoch_params
                    .global_nonce
                    .parse()
                    .map_err(|e| MalformedField("global_nonce", e))?,
                difficulty: epoch_params
                    .difficulty
                    .parse()
                    .map_err(|e| MalformedField("difficulty", e))?,
            };

            Ok(Self {
                epoch,
                local_nonce: value
                    .local_nonce
                    .parse()
                    .map_err(|e| MalformedField("local_nonce", e))?,
                cu_id: value
                    .cu_id
                    .parse()
                    .map_err(|e| MalformedField("cu_id", e))?,
                result_hash: value
                    .result_hash
                    .parse()
                    .map_err(|e| MalformedField("result_hash", e))?,
            })
        }
    }
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
        if endpoint.starts_with("http://")
            || endpoint.starts_with("https://")
            || endpoint.starts_with("grpc://")
        {
            Self::new_http(endpoint).await
        } else {
            Self::new_unix_socket(endpoint).await
        }
    }

    pub async fn report_proofs(&mut self, proof: &[RawProof]) -> Result<(), ClientError> {
        let proofs = proof.iter().map(Into::into).collect();

        let report_request = ReportRequest { proofs };
        self.inner.report_proof(report_request).await?;
        Ok(())
    }
}
