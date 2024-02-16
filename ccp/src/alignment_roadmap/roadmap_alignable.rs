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

use super::CCProverAlignmentRoadmap;

pub trait RoadmapAlignable: Send {
    type Error: Send;

    /// Apply the given roadmap (a set of action) to align states of Nox and CCP.
    fn align_with<'futures, 'prover: 'futures>(
        &'prover mut self,
        roadmap: CCProverAlignmentRoadmap,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send;
}
