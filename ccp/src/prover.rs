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

#[cfg(test)]
mod tests;

use ccp_msr::get_original_cpu_msr_preset;
use ccp_msr::MSRConfig;
use ccp_msr::MSRCpuPreset;
use futures::future;
use futures::FutureExt;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use ccp_config::CCPConfig;
use ccp_shared::nox_ccp_api::NoxCCPApi;
use ccp_shared::proof::CCProof;
use ccp_shared::proof::ProofIdx;
use ccp_shared::types::*;
use ccp_utils::run_utils::run_unordered;

use crate::alignment_roadmap::*;
use crate::cu::CUProver;
use crate::cu::CUProverConfig;
use crate::cu::CUResult;
use crate::errors::CCProverError;
use crate::hashrate::prometheus::PrometheusEndpoint;
use crate::hashrate::HashrateCollector;
use crate::hashrate::HashrateHandler;
use crate::proof_storage::ProofStorageDrainer;
use crate::state_storage::CCPState;
use crate::state_storage::StateStorage;
use crate::status::CCStatus;
use crate::status::ToCCStatus;
use crate::utility_thread::UtilityThread;

const PROOF_DIR: &str = "cc_proofs";

pub type CCResult<T> = Result<T, CCProverError>;

pub struct CCProver {
    cu_provers: HashMap<PhysicalCoreId, CUProver>,
    cu_prover_config: CUProverConfig,
    status: CCStatus,
    utility_thread: UtilityThread,
    prometheus_endpoint: Option<PrometheusEndpoint>,
    proof_drainer: ProofStorageDrainer,
    state_storage: StateStorage,
}

impl NoxCCPApi for CCProver {
    type Error = CCProverError;

    async fn on_active_commitment(
        &mut self,
        new_epoch: EpochParameters,
        new_allocation: CUAllocation,
    ) -> Result<(), Self::Error> {
        let apply_resut = self
            .apply_cc_parameters(new_epoch, &new_allocation)
            .await
            .inspect_err(|e| {
                log::error!("Failed to apply parameters: {e}.  Still trying to save state.");
            });
        self.save_state(new_epoch, new_allocation.clone())
            .await
            .inspect_err(|e| {
                log::error!("Failed to save state: {e}");
            })?;

        apply_resut
    }

    async fn on_no_active_commitment(&mut self) -> Result<(), Self::Error> {
        let closure =
            move |_: usize, (_, prover): (PhysicalCoreId, CUProver)| prover.stop().boxed();

        run_unordered(self.cu_provers.drain(), closure).await?;
        self.status = CCStatus::Idle;

        self.save_no_state().await?;

        Ok(())
    }

    async fn get_proofs_after(&self, proof_idx: ProofIdx) -> Result<Vec<CCProof>, Self::Error> {
        self.proof_drainer
            .get_proofs_after(proof_idx)
            .await
            .map_err(Into::into)
    }
}

impl ToCCStatus for CCProver {
    fn status(&self) -> CCStatus {
        self.status
    }
}

impl CCProver {
    pub fn new(config: CCPConfig) -> CCResult<Self> {
        let proof_dir = config.state_dir.join(PROOF_DIR);
        let proof_drainer = ProofStorageDrainer::new(proof_dir.clone());
        let hashrate_collector = Arc::new(Mutex::new(HashrateCollector::new()));
        let hashrate_handler = HashrateHandler::new(
            hashrate_collector.clone(),
            config.state_dir.clone(),
            config.logs.report_hashrate,
        )?;
        let utility_thread = UtilityThread::spawn(
            config.rpc_endpoint.utility_cores_ids,
            ProofIdx::zero(),
            proof_dir,
            None,
            hashrate_handler,
        );
        let prometheus_endpoint = config.prometheus_endpoint.as_ref().map(|endpoint_cfg| {
            PrometheusEndpoint::new(
                (endpoint_cfg.host.clone(), endpoint_cfg.port),
                hashrate_collector,
            )
        });
        let cu_prover_config = config.optimizations.into();
        let state_storage = StateStorage::new(config.state_dir);

        let prover = Self {
            cu_provers: HashMap::new(),
            cu_prover_config,
            status: CCStatus::Idle,
            utility_thread,
            prometheus_endpoint,
            proof_drainer,
            state_storage,
        };
        Ok(prover)
    }

    #[allow(clippy::needless_lifetimes)]
    pub async fn pause<'provers>(&'provers mut self) -> CCResult<()> {
        let closure = move |_: usize, (_, prover): (&PhysicalCoreId, &'provers mut CUProver)| {
            prover.pause().boxed()
        };
        run_unordered(self.cu_provers.iter_mut(), closure).await?;

        self.status = CCStatus::Idle;

        Ok(())
    }

    pub async fn stop(mut self) -> CCResult<()> {
        // stop all active provers
        self.on_no_active_commitment().await?;
        // stop background thread
        self.utility_thread.stop().await?;

        if let Some(prometheus_endpoint) = self.prometheus_endpoint {
            prometheus_endpoint.stop().await?;
        }

        Ok(())
    }

    pub async fn from_saved_state(config: CCPConfig) -> CCResult<Self> {
        let proof_dir = config.state_dir.join(PROOF_DIR);
        let mut proof_cleaner = ProofStorageDrainer::new(proof_dir.clone());
        let state_storage = StateStorage::new(config.state_dir.clone());

        let prev_state = state_storage.try_to_load_data().await?;
        let start_proof_idx = proof_cleaner
            .validate_proofs(prev_state.as_ref().map(|state| &state.epoch_params))
            .await?;

        log::info!("continuing from proof index {start_proof_idx}");

        let original_msr_preset = if config.enable_msr {
            prev_state
                .as_ref()
                .map_or_else(get_original_cpu_msr_preset, |state| {
                    state.msr_config.original_msr_preset.clone()
                })
        } else {
            MSRCpuPreset::empty()
        };

        let msr_config = MSRConfig {
            enable_msr: config.enable_msr,
            original_msr_preset,
        };

        let original_msr_preset = if config.enable_msr {
            prev_state
                .as_ref()
                .map_or_else(get_original_cpu_msr_preset, |state| {
                    state.msr_config.original_msr_preset.clone()
                })
        } else {
            MSRCpuPreset::empty()
        };

        let msr_config = MSRConfig {
            enable_msr: config.enable_msr,
            original_msr_preset,
        };

        let hashrate_collector = Arc::new(Mutex::new(HashrateCollector::new()));
        let hashrate_handler = HashrateHandler::new(
            hashrate_collector.clone(),
            config.state_dir.clone(),
            config.logs.report_hashrate,
        )?;
        let utility_thread = UtilityThread::spawn(
            config.rpc_endpoint.utility_cores_ids,
            start_proof_idx,
            proof_dir,
            prev_state
                .as_ref()
                .map(|state| state.epoch_params.global_nonce),
            hashrate_handler,
        );
        let prometheus_endpoint = config.prometheus_endpoint.as_ref().map(|endpoint_cfg| {
            PrometheusEndpoint::new(
                (endpoint_cfg.host.clone(), endpoint_cfg.port),
                hashrate_collector,
            )
        });

        let cu_prover_config = config.optimizations.into();
        let mut self_ = Self {
            cu_provers: HashMap::new(),
            cu_prover_config,
            status: CCStatus::Idle,
            utility_thread,
            prometheus_endpoint,
            proof_drainer: proof_cleaner,
            state_storage,
        };

        if let Some(prev_state) = prev_state {
            self_
                .apply_cc_parameters(prev_state.epoch_params, &prev_state.cu_allocation)
                .await?;
        }

        Ok(self_)
    }

    pub async fn save_state(
        &self,
        epoch_state: EpochParameters,
        cu_allocation: CUAllocation,
    ) -> tokio::io::Result<()> {
        let msr_config = self.cu_prover_config.msr_config.clone();

        let state = CCPState {
            epoch_params: epoch_state,
            cu_allocation,
            msr_config,
        };
        self.state_storage.save_state(Some(&state)).await
    }

    pub async fn save_no_state(&self) -> tokio::io::Result<()> {
        self.state_storage.save_state(None).await
    }

    async fn apply_cc_parameters(
        &mut self,
        new_epoch: EpochParameters,
        new_allocation: &HashMap<PhysicalCoreId, CUID>,
    ) -> Result<(), <CCProver as NoxCCPApi>::Error> {
        let roadmap = CCProverAlignmentRoadmap::make(
            new_allocation.clone(),
            new_epoch,
            &self.cu_provers,
            self.status,
        );
        self.align_with(roadmap).await?;
        self.status = CCStatus::Running { epoch: new_epoch };

        Ok(())
    }
}

#[derive(Debug)]
enum AlignmentPostAction {
    KeepProver(CUProver),
    Nothing,
}

impl RoadmapAlignable for CCProver {
    type Error = CCProverError;

    async fn align_with(&mut self, roadmap: CCProverAlignmentRoadmap) -> Result<(), Self::Error> {
        use futures::stream::FuturesUnordered;
        use futures::StreamExt;

        let CCProverAlignmentRoadmap {
            pre_action,
            actions,
            epoch,
        } = roadmap;

        match pre_action {
            CUProverPreAction::NoAction => {}
            CUProverPreAction::CleanupProofCache => {
                self.pause().await?;
                self.proof_drainer.remove_proofs().await?;
            }
        }

        let actions_as_futures = actions
            .into_iter()
            .map(|action| match action {
                CUProverAction::CreateCUProver(state) => self.cu_creation(state, epoch),
                CUProverAction::RemoveCUProver(state) => self.cu_removal(state),
                CUProverAction::NewCCJob(state) => self.new_cc_job(state, epoch),
                CUProverAction::NewCCJobWithRepining(state) => self.new_cc_job_repin(state, epoch),
            })
            .collect::<FuturesUnordered<_>>();

        let (provers, errors) = actions_as_futures
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .partition::<Vec<_>, _>(Result::is_ok);

        let provers_to_keep = provers
            .into_iter()
            .map(Result::unwrap)
            .flat_map(|prover_post_action| match prover_post_action {
                AlignmentPostAction::KeepProver(prover) => Some((prover.pinned_core_id(), prover)),
                AlignmentPostAction::Nothing => None,
            })
            .collect::<Vec<_>>();
        self.cu_provers.extend(provers_to_keep);

        if errors.is_empty() {
            return Ok(());
        }

        let errors = errors
            .into_iter()
            .map(Result::unwrap_err)
            .collect::<Vec<_>>();

        Err(CCProverError::CUProverErrors(errors))
    }
}

impl CCProver {
    pub(self) fn cu_creation(
        &mut self,
        state: actions_state::CreateCUProverState,
        epoch: EpochParameters,
    ) -> future::BoxFuture<'static, CUResult<AlignmentPostAction>> {
        let prover_config = self.cu_prover_config.clone();
        let to_utility = self.utility_thread.get_to_utility_channel();

        async move {
            let mut prover = CUProver::create(prover_config, to_utility, state.new_core_id).await?;
            prover.new_epoch(epoch, state.new_cu_id).await?;

            Ok(AlignmentPostAction::KeepProver(prover))
        }
        .boxed()
    }

    pub(self) fn cu_removal(
        &mut self,
        state: actions_state::RemoveCUProverState,
    ) -> future::BoxFuture<'static, CUResult<AlignmentPostAction>> {
        let prover = self.cu_provers.remove(&state.current_core_id).unwrap();
        async move {
            prover.stop().await?;
            Ok(AlignmentPostAction::Nothing)
        }
        .boxed()
    }

    pub(self) fn new_cc_job(
        &mut self,
        state: actions_state::NewCCJobState,
        epoch: EpochParameters,
    ) -> future::BoxFuture<'static, CUResult<AlignmentPostAction>> {
        let mut prover = self.cu_provers.remove(&state.current_core_id).unwrap();
        async move {
            prover.new_epoch(epoch, state.new_cu_id).await?;
            Ok(AlignmentPostAction::KeepProver(prover))
        }
        .boxed()
    }

    pub(self) fn new_cc_job_repin(
        &mut self,
        state: actions_state::NewCCJobWithRepiningState,
        epoch: EpochParameters,
    ) -> future::BoxFuture<'static, CUResult<AlignmentPostAction>> {
        let mut prover = self.cu_provers.remove(&state.current_core_id).unwrap();
        async move {
            prover.pin(state.new_core_id).await?;
            prover.new_epoch(epoch, state.new_cu_id).await?;
            Ok(AlignmentPostAction::KeepProver(prover))
        }
        .boxed()
    }
}
