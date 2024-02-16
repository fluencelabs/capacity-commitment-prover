use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use jsonrpsee::core::async_trait;
use jsonrpsee::server::Server;
use jsonrpsee::server::ServerHandle;
use jsonrpsee::tracing::instrument;
use jsonrpsee::types::ErrorObjectOwned;
use tokio::net::ToSocketAddrs;
use tokio::sync::Mutex;

use ccp_rpc_client::CCPRpcServer;
use ccp_rpc_client::OrHex;
use ccp_shared::nox_ccp_api::NoxCCPApi;
use ccp_shared::proof::CCProof;
use ccp_shared::types::Difficulty;
use ccp_shared::types::GlobalNonce;
use ccp_shared::types::PhysicalCoreId;
use ccp_shared::types::CUID;

pub struct CCPRcpHttpServer<P> {
    // n.b. if NoxCCPApi would have internal mutability, we might get used of the Mutex
    cc_prover: Arc<Mutex<P>>,
}

impl<P> CCPRcpHttpServer<P> {
    pub fn new(cc_prover: Arc<Mutex<P>>) -> Self {
        Self { cc_prover }
    }
}

impl<P> CCPRcpHttpServer<P>
where
    P: NoxCCPApi + 'static,
    <P as NoxCCPApi>::Error: Error,
{
    ///  Run the JSON-RPC HTTP server in the background.
    ///
    ///  The returned handle can be used to maniplate it.
    pub async fn run_server(
        self,
        bind_address: impl ToSocketAddrs,
    ) -> Result<ServerHandle, std::io::Error> {
        let server = Server::builder().build(bind_address).await?;

        let handle = server.start(self.into_rpc());

        Ok(handle)
    }
}

#[async_trait]
impl<P> CCPRpcServer for CCPRcpHttpServer<P>
where
    P: NoxCCPApi + 'static,
    <P as NoxCCPApi>::Error: Error,
{
    #[instrument(skip(self))]
    async fn on_active_commitment(
        &self,
        global_nonce: OrHex<GlobalNonce>,
        difficulty: OrHex<Difficulty>,
        cu_allocation: HashMap<PhysicalCoreId, OrHex<CUID>>,
    ) -> Result<(), ErrorObjectOwned> {
        let global_nonce: GlobalNonce =
            global_nonce
                .clone()
                .try_into()
                .map_err(|e: Box<dyn Error>| {
                    ErrorObjectOwned::owned(2, e.to_string(), Some(global_nonce))
                })?;
        let difficulty = difficulty.clone().try_into().map_err(|e: Box<dyn Error>| {
            ErrorObjectOwned::owned(2, e.to_string(), Some(difficulty))
        })?;
        let mut cu_allocation_real = HashMap::<_, CUID>::new();
        for (id, cuid) in cu_allocation {
            cu_allocation_real.insert(
                id,
                cuid.clone().try_into().map_err(|e: Box<dyn Error>| {
                    ErrorObjectOwned::owned(2, e.to_string(), Some(cuid))
                })?,
            );
        }

        let mut guard = self.cc_prover.lock().await;
        guard
            .on_active_commitment(global_nonce, difficulty, cu_allocation_real)
            .await
            .map_err(|e| ErrorObjectOwned::owned::<()>(1, e.to_string(), None))?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_no_active_commitment(&self) -> Result<(), ErrorObjectOwned> {
        let mut guard = self.cc_prover.lock().await;
        guard
            .on_no_active_commitment()
            .await
            .map_err(|e| ErrorObjectOwned::owned::<()>(1, e.to_string(), None))?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_proofs_after(&self, proof_idx: u64) -> Result<Vec<CCProof>, ErrorObjectOwned> {
        let guard = self.cc_prover.lock().await;
        let proofs = guard
            .get_proofs_after(proof_idx)
            .await
            .map_err(|e| ErrorObjectOwned::owned::<()>(1, e.to_string(), None))?;
        Ok(proofs)
    }
}
