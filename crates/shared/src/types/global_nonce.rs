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

use std::str::FromStr;

use hex::ToHex;
use serde::Deserialize;
use serde::Serialize;

use hex::FromHex;

pub type GlobalNonceInner = [u8; 32];

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct GlobalNonce(GlobalNonceInner);

impl GlobalNonce {
    pub fn new(inner: GlobalNonceInner) -> Self {
        Self(inner)
    }
}

impl AsRef<GlobalNonceInner> for GlobalNonce {
    fn as_ref(&self) -> &GlobalNonceInner {
        &self.0
    }
}

impl FromHex for GlobalNonce {
    type Error = <[u8; 32] as FromHex>::Error;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        GlobalNonceInner::from_hex(hex).map(Self)
    }
}

impl ToHex for GlobalNonce {
    fn encode_hex<T: std::iter::FromIterator<char>>(&self) -> T {
        ToHex::encode_hex(&self.0)
    }

    fn encode_hex_upper<T: std::iter::FromIterator<char>>(&self) -> T {
        ToHex::encode_hex_upper(&self.0)
    }
}

impl std::fmt::Display for GlobalNonce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.encode_hex::<String>())
    }
}

impl FromStr for GlobalNonce {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FromHex::from_hex(s)
    }
}
