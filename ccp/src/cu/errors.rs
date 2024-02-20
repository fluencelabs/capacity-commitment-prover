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

use thiserror::Error as ThisError;
use tokio::sync::mpsc;

use cpu_utils::CPUTopologyError;
use cpu_utils::PhysicalCoreId;
use randomx_rust_wrapper::errors::RandomXError;

use super::proving_thread::ProvingThreadError;

#[derive(ThisError, Debug)]
pub enum CUProverError {
    #[error(transparent)]
    ThreadError(#[from] ProvingThreadError),

    #[error("multiple threads failed: {0:?}")]
    ThreadErrors(Vec<ProvingThreadError>),

    #[error(transparent)]
    RandomXError(#[from] RandomXError),

    #[error("")]
    ChannelError(#[source] anyhow::Error),

    #[error(transparent)]
    ThreadAllocation(#[from] ThreadAllocationError),

    #[error(transparent)]
    TopologyError(#[from] CPUTopologyError),
}

#[derive(ThisError, Debug)]
pub enum ThreadAllocationError {
    #[error(transparent)]
    TopologyError(#[from] CPUTopologyError),

    #[error("no logical CPUs found for physical core with id {core_id}")]
    LogicalCPUNotFound { core_id: PhysicalCoreId },
}

impl CUProverError {
    pub fn logical_cpus_not_found(core_id: PhysicalCoreId) -> Self {
        let thread_allocation_error = ThreadAllocationError::LogicalCPUNotFound { core_id };
        Self::ThreadAllocation(thread_allocation_error)
    }
}

impl From<Vec<ProvingThreadError>> for CUProverError {
    fn from(errors: Vec<ProvingThreadError>) -> Self {
        Self::ThreadErrors(errors)
    }
}

impl<T> From<mpsc::error::SendError<T>> for CUProverError {
    fn from(value: mpsc::error::SendError<T>) -> Self {
        CUProverError::ChannelError(anyhow::anyhow!("prover channel error: {value}"))
    }
}

impl<T> From<mpsc::error::TrySendError<T>> for CUProverError {
    fn from(value: mpsc::error::TrySendError<T>) -> Self {
        CUProverError::ChannelError(anyhow::anyhow!("prover channel error: {value}"))
    }
}

impl From<mpsc::error::TryRecvError> for CUProverError {
    fn from(value: mpsc::error::TryRecvError) -> Self {
        CUProverError::ChannelError(anyhow::anyhow!("prover channel error: {value}"))
    }
}
