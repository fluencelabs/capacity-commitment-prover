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

use crate::flags::RandomXFlags;

#[derive(ThisError, Debug, Clone)]
pub enum RandomXError {
    #[error("cache allocation with flags {flags:?} failed")]
    CacheAllocationFailed { flags: RandomXFlags },

    #[error("dataset allocation with flags {flags:?} failed")]
    DatasetAllocationError { flags: RandomXFlags },

    #[error(transparent)]
    VMCreationFailed(#[from] VmCreationError),
}

#[derive(ThisError, Debug, Clone)]
pub enum VmCreationError {
    #[error("vm allocation with flags {flags:?} failed")]
    AllocationFailed { flags: RandomXFlags },

    #[error("to allocate vm in the fast mode, flags {flags:?} must contain the full mem option")]
    IncorrectFastModeFlag { flags: RandomXFlags },

    #[error(
        "to allocate vm in the light mode, flags {flags:?} must no contain the full mem option"
    )]
    IncorrectLightModeFlag { flags: RandomXFlags },
}
