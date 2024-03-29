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

use cpu_utils::LogicalCoreId;

pub(crate) trait ThreadDistributionPolicy {
    /// Returns a particular logical core id where the supplied proving thread id
    /// will be allocated.
    fn distribute(
        &self,
        thread_id: usize,
        logical_cores: &nonempty::NonEmpty<LogicalCoreId>,
    ) -> LogicalCoreId;
}
