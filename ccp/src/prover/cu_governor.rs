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

use std::collections::HashMap;

use futures::future;
use futures::FutureExt;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use ccp_config::Optimizations;
use ccp_config::Workers;
use ccp_cu::proving_thread::sync::to_state_operating_message::ToStateOperatingInlet;
use ccp_cu::proving_thread::sync::to_stats_collecting_message::ToStatsCollectingInlet;
use ccp_cu::CUProver;
use ccp_cu::CUProverConfig;
use ccp_cu::CUResult;
use ccp_msr::MSRModeEnforcer;
use ccp_shared::types::*;
use ccp_utils::run_utils::run_unordered;

use crate::alignment_roadmap::*;
use crate::errors::CCProverError;
use crate::prover::BatchRequest;
use crate::prover::BatchResponse;
use crate::prover::ProofIdx;
use crate::state_storage::CCPState;
use crate::status::CCStatus;
use crate::CCResult;

/// An part of CCProver that controls CU units, keeping all the changing state.
pub(crate) struct CUGovernor {
    cu_provers: HashMap<PhysicalCoreId, CUProver>,
    cu_prover_config: CUProverConfig,
    status: CCStatus,
    // in-memory statistics
    stats_collecting_inlet: ToStatsCollectingInlet,
    // collecting proofs
    state_operating_inlnet: ToStateOperatingInlet,
    // collected proofs managment
    state_operating_control_inlet: StateOperatingControlInlet,
    msr_enforcer: MSRModeEnforcer,
    cancellation_token: CancellationToken,
}

impl CUGovernor {
    pub(crate) fn new(
        optimizations_config: Optimizations,
        workers_config: Workers,
        stats_collecting_inlnet: ToStatsCollectingInlet,
        state_operating_inlnet: ToStateOperatingInlet,
        state_operating_control_inlet: StateOperatingControlInlet,
        msr_enforcer: MSRModeEnforcer,
        cancellation_token: CancellationToken,
        target_epoch_params: Option<EpochParameters>,
    ) -> Self {
        let cu_prover_config = CUProverConfig::new(optimizations_config, workers_config);

        let status = match target_epoch_params {
            Some(epoch) => CCStatus::Running { epoch },
            None => CCStatus::Idle,
        };

        Self {
            cu_provers: HashMap::new(),
            cu_prover_config,
            status,
            stats_collecting_inlet: stats_collecting_inlnet,
            state_operating_inlnet,
            state_operating_control_inlet,
            msr_enforcer,
            cancellation_token,
        }
    }

    pub async fn stop(&mut self) -> CCResult<()> {
        // stop all active provers
        self.on_no_active_commitment().await?;
        self.shutdown();
        Ok(())
    }

    pub fn shutdown(&mut self) {
        self.cancellation_token.cancel();
    }

    async fn apply_cc_parameters(
        &mut self,
        new_epoch: EpochParameters,
        new_allocation: &HashMap<PhysicalCoreId, CUID>,
    ) -> Result<(), CCProverError> {
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

    async fn call_utility_cleanup_proof_cache(&self) -> CCResult<()> {
        let (notifier, wait_for) = oneshot::channel();
        let message = StateOperatingControlMessage::CleanProofCache { notifier };
        // if the queue is full, it a sign of serious problem
        self.execute_utility_call(message, wait_for).await
    }

    async fn execute_utility_call<T>(
        &self,
        message: StateOperatingControlMessage,
        wait_for: oneshot::Receiver<Result<T, std::io::Error>>,
    ) -> Result<T, CCProverError> {
        use tokio::sync::mpsc::error::TrySendError;

        self.state_operating_control_inlet
            .try_send(message)
            .map_err(|e| match e {
                // if the queue is full, it a sign of serious problem
                TrySendError::Full(_) => CCProverError::UtilityFullError,
                // if the queue is closed, the problem is even more serios
                TrySendError::Closed(_) => CCProverError::UtilityClosedError,
            })?;
        Ok(wait_for
            .await
            .map_err(|_| CCProverError::UtilityClosedError)??)
    }

    pub(crate) async fn on_active_commitment(
        &mut self,
        new_epoch: EpochParameters,
        new_allocation: CUAllocation,
    ) -> Result<(), CCProverError> {
        self.apply_cc_parameters(new_epoch, &new_allocation)
            .await
            .inspect_err(|e| {
                log::error!("Failed to apply parameters: {e}.  Still trying to save state.");
            })
    }

    pub(crate) async fn on_no_active_commitment(&mut self) -> Result<(), CCProverError> {
        self.stop_provers_nonblocking().await?;
        self.join_provers().await?;

        self.status = CCStatus::Idle;

        Ok(())
    }
}

#[derive(Debug)]
enum AlignmentPostAction {
    KeepProver(CUProver),
    Nothing,
}

impl RoadmapAlignable for CUGovernor {
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
                self.call_utility_cleanup_proof_cache().await?;
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

impl CUGovernor {
    pub(self) fn cu_creation(
        &mut self,
        state: actions_state::CreateCUProverState,
        epoch: EpochParameters,
    ) -> future::BoxFuture<'static, CUResult<AlignmentPostAction>> {
        let prover_config = self.cu_prover_config.clone();
        let to_state_operating = self.state_operating_inlnet.clone();
        let to_stats_collecting = self.stats_collecting_inlet.clone();
        let msr_enforcer = self.msr_enforcer.clone();
        let child_cancel_token = self.cancellation_token.clone();

        async move {
            let mut prover = CUProver::create(
                prover_config,
                to_state_operating,
                to_stats_collecting,
                msr_enforcer,
                state.new_core_id,
            )
            .await?;
            prover
                .new_epoch(epoch, state.new_cu_id, child_cancel_token)
                .await?;

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
            prover.stop_join().await?;
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
        let cancel_token = self.cancellation_token.clone();
        async move {
            prover
                .new_epoch(epoch, state.new_cu_id, cancel_token)
                .await?;
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
        let cancel_token = self.cancellation_token.clone();
        async move {
            prover.pin(state.new_core_id).await?;
            prover
                .new_epoch(epoch, state.new_cu_id, cancel_token)
                .await?;
            Ok(AlignmentPostAction::KeepProver(prover))
        }
        .boxed()
    }
}

impl CUGovernor {
    #[allow(clippy::needless_lifetimes)]
    pub async fn pause<'provers>(&'provers mut self) -> CCResult<()> {
        let closure = move |_: usize, (_, prover): (&PhysicalCoreId, &'provers mut CUProver)| {
            prover.pause().boxed()
        };
        run_unordered(self.cu_provers.iter_mut(), closure).await?;

        self.status = CCStatus::Idle;

        Ok(())
    }

    #[allow(clippy::needless_lifetimes)]
    async fn stop_provers_nonblocking<'provers>(&'provers self) -> CCResult<()> {
        let nonblocking_closure =
            move |_: usize, (_, prover): (&PhysicalCoreId, &'provers CUProver)| {
                prover.stop_nonblocking().boxed()
            };

        run_unordered(self.cu_provers.iter(), nonblocking_closure).await?;
        Ok(())
    }

    async fn join_provers(&mut self) -> CCResult<()> {
        let join_closure =
            move |_: usize, (_, prover): (PhysicalCoreId, CUProver)| prover.join().boxed();

        run_unordered(self.cu_provers.drain(), join_closure).await?;
        Ok(())
    }
}

pub(crate) type StateOperatingControlInlet = mpsc::Sender<StateOperatingControlMessage>;
pub(crate) type StateOperatingControlOutlet = mpsc::Receiver<StateOperatingControlMessage>;

#[derive(Debug)]
pub(crate) enum StateOperatingControlMessage {
    // Why saving config is in the CU???
    SaveConfig {
        notifier: oneshot::Sender<tokio::io::Result<()>>,
        state: Option<CCPState>,
    },
    CleanProofCache {
        notifier: oneshot::Sender<tokio::io::Result<()>>,
    },
    GetAllProofs {
        last_known_proofs: HashMap<CUID, ProofIdx>,
        limit: usize,
        notifier: oneshot::Sender<tokio::io::Result<Vec<BatchResponse>>>,
    },
    GetBatchProofs {
        reqs: HashMap<CUID, BatchRequest>,
        min_batch_count: usize,
        max_batch_count: usize,
        notifier: oneshot::Sender<tokio::io::Result<Vec<BatchResponse>>>,
    },
}
