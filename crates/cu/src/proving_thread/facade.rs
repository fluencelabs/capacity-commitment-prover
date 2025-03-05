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

use ccp_randomx::RandomXFlags;
use ccp_shared::types::*;

pub trait ProvingThreadFacade {
    type Error;

    async fn run_cc_job(
        &self,
        epoch: EpochParameters,
        flags: RandomXFlags,
        cu_id: CUID,
    ) -> Result<(), Self::Error>;

    async fn pin(&mut self, logical_core_id: LogicalCoreId) -> Result<(), Self::Error>;

    /// Pauses proving thread till the next message.
    async fn pause(&mut self) -> Result<(), Self::Error>;

    /// Send a stop message to thread and do not join on it.
    async fn stop_nonblocking(&self) -> Result<(), Self::Error>;

    /// Send a stop message to thread and join on it till it ends.
    async fn stop_join(self) -> Result<(), Self::Error>;

    /// Join on thread till it ends.
    async fn join(self) -> Result<(), Self::Error>;
}
