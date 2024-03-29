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

mod channels_facade;
mod errors;
mod local_nonce;
mod raw_proof;
mod state;
mod thread;
pub(crate) mod to_utility_message;

pub(crate) use errors::ProvingThreadSyncFacadeError;
pub(crate) use thread::ProvingThreadSync;

type STResult<T> = Result<T, errors::ProvingThreadSyncError>;
pub(crate) type STFResult<T> = Result<T, errors::ProvingThreadSyncFacadeError>;
