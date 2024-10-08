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

mod channels_facade;
mod errors;
mod local_nonce;
mod state;
mod thread;
pub mod to_state_operating_message;
pub mod to_stats_collecting_message;

pub(crate) use self::errors::ProvingThreadSyncFacadeError;
pub(crate) use self::thread::ProvingThreadSync;
pub use crate::RawProof;

type STResult<T> = Result<T, errors::ProvingThreadSyncError>;
pub(crate) type STFResult<T> = Result<T, errors::ProvingThreadSyncFacadeError>;
