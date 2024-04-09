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

use std::collections::HashMap;

use ccp_msr::MSRModeEnforcer;
use ccp_shared::types::*;
use ccp_utils::run_utils::run_unordered;
use futures::future::BoxFuture;
use futures::FutureExt;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::cu::CUProver;
use crate::cu::CUProverConfig;
use crate::cu::CUResult;
use crate::prover::AlignmentPostAction;
use crate::prover::CUProverAction;
use crate::prover::CUProverPreAction;
use crate::prover::RoadmapAlignable;
use crate::status::CCStatus;
use crate::utility_thread::UtilityThread;
use crate::CCProverError;
use crate::CCResult;

use super::actions_state;
use super::CCProverAlignmentRoadmap;

#[derive(Debug)]
pub(crate) struct CuGovernor {
    inlet: mpsc::Receiver<CuGovernorRequest>,

    cu_provers: HashMap<PhysicalCoreId, CUProver>,
    cu_prover_config: CUProverConfig,
    status: CCStatus,

    utility_thread: UtilityThread,
    msr_enforcer: MSRModeEnforcer,
}

impl CuGovernor {
    pub(crate) async fn run(mut self) {
        use CuGovernorRequest::*;
        // TODO consider first stop message or last on_commitment/on_no_commitment
        while let Some(msg) = self.inlet.recv().await {
            match msg {
                OnActiveCommitment {
                    new_epoch,
                    new_allocation,
                    ready,
                } => todo!(),
                OnNoActiveCommitment { ready } => todo!(),
                Stop { ready } => todo!(),
            }
        }
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

    async fn apply_cc_parameters(
        &mut self,
        new_epoch: EpochParameters,
        new_allocation: &HashMap<PhysicalCoreId, CUID>,
    ) -> Result<(), <Self as RoadmapAlignable>::Error> {
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

    pub(self) fn cu_creation(
        &mut self,
        state: actions_state::CreateCUProverState,
        epoch: EpochParameters,
    ) -> BoxFuture<'static, CUResult<AlignmentPostAction>> {
        let prover_config = self.cu_prover_config.clone();
        let to_utility = self.utility_thread.get_to_utility_channel();
        let msr_enforcer = self.msr_enforcer.clone();

        async move {
            let mut prover =
                CUProver::create(prover_config, to_utility, msr_enforcer, state.new_core_id)
                    .await?;
            prover.new_epoch(epoch, state.new_cu_id).await?;

            Ok(AlignmentPostAction::KeepProver(prover))
        }
        .boxed()
    }

    pub(self) fn cu_removal(
        &mut self,
        state: actions_state::RemoveCUProverState,
    ) -> BoxFuture<'static, CUResult<AlignmentPostAction>> {
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
    ) -> BoxFuture<'static, CUResult<AlignmentPostAction>> {
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
    ) -> BoxFuture<'static, CUResult<AlignmentPostAction>> {
        let mut prover = self.cu_provers.remove(&state.current_core_id).unwrap();
        async move {
            prover.pin(state.new_core_id).await?;
            prover.new_epoch(epoch, state.new_cu_id).await?;
            Ok(AlignmentPostAction::KeepProver(prover))
        }
        .boxed()
    }
}

impl RoadmapAlignable for CuGovernor {
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
                // TODO
                // self.proof_drainer.remove_proofs().await?;
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

#[derive(Debug)]
pub(crate) enum CuGovernorRequest {
    OnActiveCommitment {
        new_epoch: EpochParameters,
        new_allocation: CUAllocation,
        ready: oneshot::Sender<CUResult<()>>,
    },
    OnNoActiveCommitment {
        ready: oneshot::Sender<CUResult<()>>,
    },
    Stop {
        ready: oneshot::Sender<CUResult<()>>,
    },
}

#[derive(Debug)]
pub(crate) struct CuGovernorHandle {
    pub(crate) governor_task: JoinHandle<()>, // TODO Result?
    outlet: mpsc::Sender<CuGovernorRequest>,
}

impl CuGovernorHandle {
    pub(crate) fn on_active_commitment(
        &self,
        new_epoch: EpochParameters,
        new_allocation: CUAllocation,
    ) -> BoxFuture<'static, Result<CUResult<()>, oneshot::error::RecvError>> {
        let (sender, receiver) = oneshot::channel();
        self.outlet
            .try_send(CuGovernorRequest::OnActiveCommitment {
                new_epoch,
                new_allocation,
                ready: sender,
            })
            .expect("TODO");
        async { receiver.await }.boxed()
    }

    pub(crate) fn on_no_active_commitment(
        &self,
    ) -> BoxFuture<'static, Result<CUResult<()>, oneshot::error::RecvError>> {
        let (sender, receiver) = oneshot::channel();
        self.outlet
            .try_send(CuGovernorRequest::OnNoActiveCommitment { ready: sender })
            .expect("TODO the other end is dead");
        async { receiver.await }.boxed()
    }

    pub(crate) fn stop(
        &self,
    ) -> BoxFuture<'static, Result<CUResult<()>, oneshot::error::RecvError>> {
        let (sender, receiver) = oneshot::channel();
        self.outlet
            .try_send(CuGovernorRequest::Stop { ready: sender })
            .expect("TODO the other end is dead");
        async { receiver.await }.boxed()
    }
}
