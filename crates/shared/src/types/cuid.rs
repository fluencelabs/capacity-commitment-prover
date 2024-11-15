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

use std::str::FromStr;

use hex::FromHex;
use hex::ToHex;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;

pub type CUIDInner = [u8; 32];

#[derive(
    Copy, Clone, Hash, PartialEq, Eq, SerializeDisplay, DeserializeFromStr, Default, PartialOrd, Ord,
)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[repr(transparent)]
pub struct CUID(CUIDInner);

impl CUID {
    pub const fn new(inner: CUIDInner) -> Self {
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

impl ToHex for CUID {
    fn encode_hex<T: std::iter::FromIterator<char>>(&self) -> T {
        ToHex::encode_hex(&self.0)
    }

    fn encode_hex_upper<T: std::iter::FromIterator<char>>(&self) -> T {
        ToHex::encode_hex_upper(&self.0)
    }
}

impl FromStr for CUID {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FromHex::from_hex(s)
    }
}

use std::fmt;

impl fmt::Display for CUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.encode_hex::<String>())
    }
}

impl fmt::Debug for CUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CUID").field(&self.to_string()).finish()
    }
}
