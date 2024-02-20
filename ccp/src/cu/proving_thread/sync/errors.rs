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

use std::any::Any;

use thiserror::Error as ThisError;
use tokio::sync::mpsc;

use cpu_utils::LogicalCoreId;
use randomx_rust_wrapper::errors::RandomXError;

/// Errors arisen inside sync thread,
/// they are intended to be passed into the utility thread.
#[derive(ThisError, Debug)]
pub enum SyncThreadError {
    #[error(transparent)]
    RandomXError(#[from] RandomXError),

    #[error(transparent)]
    ChannelError(#[from] anyhow::Error),

    #[error("thread pinning to logical core {core_id} failed")]
    ThreadPinFailed { core_id: LogicalCoreId },
}

/// Errors arisen in the sync thread facade.
#[derive(ThisError, Debug)]
pub enum SyncThreadFacadeError {
    #[error("error happened while waiting the sync part to complete {0:?}")]
    JoinError(Box<dyn Any + Send>),

    #[error(transparent)]
    ChannelError(#[from] anyhow::Error),
}

impl SyncThreadError {
    pub fn channel_error(error_message: impl ToString) -> Self {
        Self::ChannelError(anyhow::anyhow!(error_message.to_string()))
    }
}

impl SyncThreadFacadeError {
    pub(crate) fn join_error(error: Box<dyn Any + Send>) -> Self {
        Self::JoinError(error)
    }
}

impl<T> From<mpsc::error::SendError<T>> for SyncThreadError {
    fn from(value: mpsc::error::SendError<T>) -> Self {
        SyncThreadError::ChannelError(anyhow::anyhow!("prover channel error: {value}"))
    }
}

impl<T> From<mpsc::error::TrySendError<T>> for SyncThreadError {
    fn from(value: mpsc::error::TrySendError<T>) -> Self {
        SyncThreadError::ChannelError(anyhow::anyhow!("prover channel error: {value}"))
    }
}

impl From<mpsc::error::TryRecvError> for SyncThreadError {
    fn from(value: mpsc::error::TryRecvError) -> Self {
        SyncThreadError::ChannelError(anyhow::anyhow!("prover channel error: {value}"))
    }
}

impl<T> From<mpsc::error::SendError<T>> for SyncThreadFacadeError {
    fn from(value: mpsc::error::SendError<T>) -> Self {
        SyncThreadFacadeError::ChannelError(anyhow::anyhow!("prover channel error: {value}"))
    }
}

impl<T> From<mpsc::error::TrySendError<T>> for SyncThreadFacadeError {
    fn from(value: mpsc::error::TrySendError<T>) -> Self {
        SyncThreadFacadeError::ChannelError(anyhow::anyhow!("prover channel error: {value}"))
    }
}

impl From<mpsc::error::TryRecvError> for SyncThreadFacadeError {
    fn from(value: mpsc::error::TryRecvError) -> Self {
        SyncThreadFacadeError::ChannelError(anyhow::anyhow!("prover channel error: {value}"))
    }
}
