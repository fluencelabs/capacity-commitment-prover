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

use hex::FromHex;
use serde::{Deserialize, Serialize};

pub type CUIDInner = [u8; 32];

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
#[repr(transparent)]
pub struct CUID(CUIDInner);

impl CUID {
    pub fn new(inner: CUIDInner) -> Self {
        Self(inner)
    }
}

impl AsRef<CUIDInner> for CUID {
    fn as_ref(&self) -> &CUIDInner {
        &self.0
    }
}

impl FromHex for CUID {
    type Error = <CUIDInner as FromHex>::Error;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        CUIDInner::from_hex(hex).map(Self)
    }
}
