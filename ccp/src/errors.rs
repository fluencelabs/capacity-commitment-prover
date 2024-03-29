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
use tokio::task::JoinError;

use crate::cu::CUProverError;
use crate::hashrate::HashrateError;
use crate::utility_thread::UtilityThreadError;

#[derive(ThisError, Debug)]
pub enum CCProverError {
    #[error(transparent)]
    CUProverError(#[from] CUProverError),

    #[error("CU prover errors are happened: {0:?}")]
    CUProverErrors(Vec<CUProverError>),

    #[error(transparent)]
    HashrateError(#[from] HashrateError),

    #[error(transparent)]
    JoinError(#[from] JoinError),

    #[error(transparent)]
    UtilityThreadError(#[from] UtilityThreadError),

    #[error(transparent)]
    IOError(#[from] tokio::io::Error),
}

impl From<Vec<CUProverError>> for CCProverError {
    fn from(errors: Vec<CUProverError>) -> Self {
        Self::CUProverErrors(errors)
    }
}
