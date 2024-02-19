mod or_hex;

use std::collections::HashMap;

use ccp_shared::proof::ProofIdx;
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
    async fn get_proofs_after(&self, proof_idx: ProofIdx)
        -> Result<Vec<CCProof>, ErrorObjectOwned>;
}

pub struct CCPRpcHttpClient {
    inner: jsonrpsee::http_client::HttpClient,
}

impl CCPRpcHttpClient {
    pub async fn new(
        endpoint_url: String,
        _client_cpu_id: PhysicalCoreId,
    ) -> Result<Self, ClientError> {
        let inner = jsonrpsee::http_client::HttpClientBuilder::default().build(endpoint_url)?;

        Ok(Self { inner })
    }

    pub async fn on_active_commitment(
        &self,
        global_nonce: impl Into<OrHex<GlobalNonce>>,
        difficulty: impl Into<OrHex<Difficulty>>,
        cu_allocation: impl Into<HashMap<PhysicalCoreId, OrHex<CUID>>>,
    ) -> Result<(), ClientError> {
        CCPRpcClient::on_active_commitment(
            &self.inner,
            global_nonce.into(),
            difficulty.into(),
            cu_allocation.into(),
        )
        .await
    }

    pub async fn on_no_active_commitment(&self) -> Result<(), ClientError> {
        CCPRpcClient::on_no_active_commitment(&self.inner).await
    }

    pub async fn get_proofs_after(&self, proof_idx: ProofIdx) -> Result<Vec<CCProof>, ClientError> {
        CCPRpcClient::get_proofs_after(&self.inner, proof_idx).await
    }
}
