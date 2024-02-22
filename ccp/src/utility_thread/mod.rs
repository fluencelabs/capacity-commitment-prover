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

mod errors;
mod proof_storage;
mod thread;

pub use errors::UtilityThreadError;
pub(crate) use thread::*;
pub(crate) use proof_storage::save_reliably;

pub(crate) mod message {
    pub(crate) use crate::cu::proving_thread::sync::to_utility_message::*;
}

pub(crate) type UTResult<T> = Result<T, errors::UtilityThreadError>;
