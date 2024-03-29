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

use crate::cu::proving_thread::sync::ProvingThreadSyncFacadeError;
use ccp_msr::MSRError;

#[derive(ThisError, Debug)]
pub enum ProvingThreadAsyncError {
    #[error(transparent)]
    ChannelError(#[from] anyhow::Error),

    #[error(transparent)]
    SyncThreadError(#[from] ProvingThreadSyncFacadeError),

    #[error("error happened while waiting the sync part to complete {0:?}")]
    JoinThreadFailed(Box<dyn Any + Send>),

    #[error(transparent)]
    MsrError(#[from] MSRError),
}

impl ProvingThreadAsyncError {
    pub fn channel_error(error_message: impl ToString) -> Self {
        Self::ChannelError(anyhow::anyhow!(error_message.to_string()))
    }

    pub fn join_error(error: Box<dyn Any + Send>) -> Self {
        Self::JoinThreadFailed(error)
    }
}

impl<T> From<mpsc::error::SendError<T>> for ProvingThreadAsyncError {
    fn from(value: mpsc::error::SendError<T>) -> Self {
        ProvingThreadAsyncError::channel_error(anyhow::anyhow!("prover channel error: {value}"))
    }
}
