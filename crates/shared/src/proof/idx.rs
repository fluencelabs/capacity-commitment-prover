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

use std::fmt::Display;
use std::str::FromStr;

use serde::Deserialize;
use serde::Serialize;

#[derive(
    Debug, Copy, Clone, Hash, Default, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize,
)]
#[repr(transparent)]
#[serde(transparent)]
pub struct ProofIdx(u64);

impl ProofIdx {
    pub fn zero() -> Self {
        Self(0)
    }

    pub fn increment(&mut self) {
        self.0 += 1;
    }
}

impl Display for ProofIdx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for ProofIdx {
    type Err = <u64 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        u64::from_str(s).map(ProofIdx)
    }
}
