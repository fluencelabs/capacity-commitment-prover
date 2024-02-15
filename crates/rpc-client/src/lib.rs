use ccp_shared::types::CUAllocation;
use ccp_shared::types::Difficulty;
use ccp_shared::types::GlobalNonce;
use ccp_shared::types::PhysicalCoreId;

use jsonrpsee::core::ClientError;
use jsonrpsee::proc_macros::rpc;

// n.b.: the rpc macro also defines CcpRpcClient type which is a working async JSON RPC client.
#[rpc(server, client, namespace = "ccp")]
pub trait CCPRpc {
    #[method(name = "on_active_commitment", param_kind = map)]
    async fn on_active_commitment(
        &self,
        global_nonce: GlobalNonce,
        difficulty: Difficulty,
        cu_allocation: CUAllocation,
    );

    #[method(name = "on_no_active_commitment")]
    async fn on_no_active_commitment(&self);
}

pub struct CCPRpcHttpClient {
    inner: jsonrpsee::http_client::HttpClient,
}

impl CCPRpcHttpClient {
    pub async fn new(
        &self,
        endpoint_url: String,
        _client_cpu_id: PhysicalCoreId,
    ) -> Result<Self, ClientError> {
        let inner = jsonrpsee::http_client::HttpClientBuilder::default().build(endpoint_url)?;

        Ok(Self { inner })
    }

    pub async fn on_active_commitment(
        &self,
        global_nonce: GlobalNonce,
        difficulty: Difficulty,
        cu_allocation: CUAllocation,
    ) -> Result<(), ClientError> {
        CCPRpcClient::on_active_commitment(&self.inner, global_nonce, difficulty, cu_allocation)
            .await
    }

    pub async fn on_no_active_commitment(&self) -> Result<(), ClientError> {
        CCPRpcClient::on_no_active_commitment(&self.inner).await
    }
}
