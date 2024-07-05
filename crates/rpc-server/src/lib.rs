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
use std::sync::Arc;

use ccp_shared::proof::BatchRequest;
use ccp_shared::proof::BatchResponse;
use ccp_shared::types::LogicalCoreId;
use jsonrpsee::core::async_trait;
use jsonrpsee::server::Server;
use jsonrpsee::server::ServerHandle;
use jsonrpsee::tracing::instrument;
use jsonrpsee::types::ErrorObjectOwned;
use tokio::net::ToSocketAddrs;

use ccp_rpc_client::CCPRpcServer;
use ccp_shared::nox_ccp_api::NoxCCPApi;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::Difficulty;
use ccp_shared::types::EpochParameters;
use ccp_shared::types::GlobalNonce;
use ccp_shared::types::PhysicalCoreId;
use ccp_shared::types::CUID;
use cpu_utils::pinning::ThreadPinner;

use tokio::sync::RwLock;

pub struct CCPRcpHttpServer<P> {
    // n.b. if NoxCCPApi would have internal mutability, we might get used of the Mutex
    cc_prover: Arc<RwLock<P>>,
}

impl<P> CCPRcpHttpServer<P> {
    pub fn new(cc_prover: Arc<RwLock<P>>) -> Self {
        Self { cc_prover }
    }
}

impl<P> CCPRcpHttpServer<P>
where
    P: NoxCCPApi + Send + Sync + 'static,
    <P as NoxCCPApi>::Error: ToString,
{
    ///  Run the JSON-RPC HTTP server in the background.
    ///
    ///  The returned handle can be used to maniplate it.
    pub async fn run_server(
        self,
        bind_address: impl ToSocketAddrs + std::fmt::Debug,
    ) -> Result<ServerHandle, std::io::Error> {
        let server = Server::builder().build(bind_address).await?;

        let handle = server.start(self.into_rpc());

        Ok(handle)
    }
}

#[async_trait]
impl<P> CCPRpcServer for CCPRcpHttpServer<P>
where
    P: NoxCCPApi + Send + Sync + 'static,
    <P as NoxCCPApi>::Error: ToString,
{
    #[instrument(skip(self))]
    async fn on_active_commitment(
        &self,
        global_nonce: GlobalNonce,
        difficulty: Difficulty,
        cu_allocation: HashMap<PhysicalCoreId, CUID>,
    ) -> Result<(), ErrorObjectOwned> {
        let mut guard = self.cc_prover.write().await;
        let epoch = EpochParameters::new(global_nonce, difficulty);
        let _init_recv = guard
            .on_active_commitment(epoch, cu_allocation)
            .await
            .map_err(|e| ErrorObjectOwned::owned::<()>(1, e.to_string(), None))?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn on_no_active_commitment(&self) -> Result<(), ErrorObjectOwned> {
        let mut guard = self.cc_prover.write().await;
        guard
            .on_no_active_commitment()
            .await
            .map_err(|e| ErrorObjectOwned::owned::<()>(1, e.to_string(), None))?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn get_proofs_after(
        &self,
        last_known_proofs: HashMap<CUID, ProofIdx>,
        limit: usize,
    ) -> Result<Vec<BatchResponse>, ErrorObjectOwned> {
        let guard = self.cc_prover.read().await;
        let proofs = guard
            .get_proofs_after(last_known_proofs, limit)
            .await
            .map_err(|e| ErrorObjectOwned::owned::<()>(1, e.to_string(), None))?;
        Ok(proofs)
    }

    #[instrument(skip(self))]
    async fn get_batch_proofs_after(
        &self,
        reqs: HashMap<CUID, BatchRequest>,
        min_batch_count: usize,
        max_batch_count: usize,
    ) -> Result<Vec<BatchResponse>, ErrorObjectOwned> {
        let guard = self.cc_prover.read().await;
        let proofs = guard
            .get_batch_proofs_after(reqs, min_batch_count, max_batch_count)
            .await
            .map_err(|e| ErrorObjectOwned::owned::<()>(1, e.to_string(), None))?;
        Ok(proofs)
    }

    #[instrument(skip(self))]
    async fn realloc_utility_cores(&self, utility_core_ids: Vec<LogicalCoreId>) {
        // optimization: schedule current Tokio thread immediately, not waiting
        // for it to park-unpark
        let pid = std::thread::current().id();
        tracing::info!("Instantly Repinning tokio thread {pid:?} to cores {utility_core_ids:?}");
        if !cpu_utils::pinning::DEFAULT.pin_current_thread_to_cpuset(&utility_core_ids) {
            tracing::error!("Tokio thread repinning failed");
        }

        let guard = self.cc_prover.read().await;
        guard.realloc_utility_cores(utility_core_ids).await;
    }
}
