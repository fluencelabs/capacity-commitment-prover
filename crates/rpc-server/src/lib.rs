use std::sync::Arc;

use jsonrpsee::core::async_trait;
use jsonrpsee::server::{Server, ServerHandle};
use tokio::sync::Mutex;

use capacity_commitment_prover::prover::CCProver;
use ccp_rpc_client::CcpRpcServer;
use ccp_shared::types::CUAllocation;
use ccp_shared::types::Difficulty;
use ccp_shared::types::GlobalNonce;

pub struct CcpRcpHttpServer {
    // n.b. if CCProver would have internal mutability, we might get used of the Mutex
    cc_prover: Arc<Mutex<CCProver>>,
    bind_address: String,
}

impl CcpRcpHttpServer {
    pub fn new(cc_prover: Arc<Mutex<CCProver>>, bind_address: String) -> Self {
        Self {
            cc_prover,
            bind_address,
        }
    }

    ///  Run the JSON-RPC HTTP server in the background.
    ///
    ///  The returned handle can be used to maniplate it.
    pub async fn run_server(self) -> Result<ServerHandle, std::io::Error> {
        let server = Server::builder().build(&self.bind_address).await?;

        let handle = server.start(self.into_rpc());

        Ok(handle)
    }
}

#[async_trait]
impl CcpRpcServer for CcpRcpHttpServer {
    async fn on_active_commitment(
        &self,
        global_nonce: GlobalNonce,
        difficulty: Difficulty,
        cu_allocation: CUAllocation,
    ) {
        let mut guard = self.cc_prover.lock().await;
        guard
            .on_active_commitment(global_nonce, difficulty, cu_allocation)
            .await;
    }

    async fn on_no_active_commitment(&self) {
        let mut guard = self.cc_prover.lock().await;
        guard.on_no_active_commitment().await;
    }
}
