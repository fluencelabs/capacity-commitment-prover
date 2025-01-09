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

use ccp_cu::proving_thread::sync::to_stats_collecting_message::ToStatsCollectingMessage;
pub use ccp_shared::proof::raw::RawProof;
pub use ccp_shared::types::EpochParameters;

pub use crate::ccp_collector_rpc::collector_client::CollectorClient;
pub use crate::ccp_collector_rpc::MalformedRpcProof;
pub use crate::ccp_collector_rpc::ReportRequest;
pub use crate::ccp_collector_rpc::RpcProof;

pub mod ccp_collector_rpc {
    use crate::RawProof;
    use crate::ToStatsCollectingMessage;

    pub const COLLECTOR_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("collector_descriptor");

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

    impl HashRate {
        fn cache_creation(duration_secs: f32) -> Self {
            Self {
                checked_hashes_count: 0,
                cache_creation: duration_secs,
                dataset_initialization: 0.0,
                cc_job_duration_secs: 0.0,
                found_proofs_count: 0,
            }
        }

        fn dataset_initialization(duration_secs: f32) -> Self {
            Self {
                checked_hashes_count: 0,
                cache_creation: 0.0,
                dataset_initialization: duration_secs,
                cc_job_duration_secs: 0.0,
                found_proofs_count: 0,
            }
        }

        fn checked_hashes(duration_secs: f32, count: usize) -> Self {
            Self {
                checked_hashes_count: count as u64,
                cache_creation: 0.0,
                dataset_initialization: 0.0,
                cc_job_duration_secs: duration_secs,
                found_proofs_count: 0,
            }
        }

        fn proofs_found(count: u64) -> Self {
            Self {
                checked_hashes_count: 0,
                cache_creation: 0.0,
                dataset_initialization: 0.0,
                cc_job_duration_secs: 0.0,
                found_proofs_count: count,
            }
        }
    }

    impl TryFrom<&ToStatsCollectingMessage> for HashRate {
        type Error = ();

        fn try_from(value: &ToStatsCollectingMessage) -> Result<Self, Self::Error> {
            use ccp_cu::HashrateRecordType;
            match value {
                ToStatsCollectingMessage::ProofFound { .. } => Ok(HashRate::proofs_found(1)),
                ToStatsCollectingMessage::Hashrate(thread_hashrate_record) => {
                    let duration_secs = thread_hashrate_record.duration.as_secs_f32();

                    match thread_hashrate_record.variant {
                        HashrateRecordType::CacheCreation => {
                            Ok(HashRate::cache_creation(duration_secs))
                        }
                        HashrateRecordType::DatasetInitialization { .. } => {
                            Ok(HashRate::dataset_initialization(duration_secs))
                        }
                        HashrateRecordType::CheckedHashes { count } => {
                            Ok(HashRate::checked_hashes(duration_secs, count))
                        }
                    }
                }
                ToStatsCollectingMessage::Epoch(_epoch_parameters) => Err(()),
            }
        }
    }
}

pub type ClientError = tonic::Status;

// n.b.: the rpc macro also defines CcpCuRpcClient type which is a working async JSON RPC client.
pub struct CcpCuRpcHttpClient {
    proofs_inner: CollectorClient<Channel>,
}

impl CcpCuRpcHttpClient {
    pub async fn new_http(url: String) -> Result<Self, tonic::transport::Error> {
        let proofs_inner = CollectorClient::connect(url.clone()).await?;
        Ok(Self { proofs_inner })
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
        let proofs_inner = CollectorClient::new(channel.clone());

        Ok(Self { proofs_inner })
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

    pub async fn report(
        &mut self,
        proof: &[RawProof],
        metrics: &[ToStatsCollectingMessage],
    ) -> Result<(), ClientError> {
        let proofs: Vec<_> = proof.iter().map(Into::into).collect();
        let metrics: Vec<_> = metrics
            .iter()
            .filter_map(|msg| TryInto::try_into(msg).ok())
            .collect();

        // it can be empty because some metrics are skipped
        if !proofs.is_empty() || !metrics.is_empty() {
            let report_request = ReportRequest { proofs, metrics };
            self.proofs_inner.report(report_request).await?;
        }
        Ok(())
    }
}
