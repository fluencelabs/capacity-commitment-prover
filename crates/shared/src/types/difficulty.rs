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

use hex::{FromHex, ToHex};
use serde::{Deserialize, Serialize};

use crate::RANDOMX_RESULT_SIZE;

pub type DifficultyInner = [u8; RANDOMX_RESULT_SIZE];

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Difficulty(DifficultyInner);

impl Difficulty {
    pub fn new(inner: DifficultyInner) -> Self {
        Self(inner)
    }
}

impl AsRef<DifficultyInner> for Difficulty {
    fn as_ref(&self) -> &DifficultyInner {
        &self.0
    }
}

impl FromHex for Difficulty {
    type Error = <DifficultyInner as FromHex>::Error;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        DifficultyInner::from_hex(hex).map(Self)
    }
}

impl PartialEq<Difficulty> for DifficultyInner {
    fn eq(&self, other: &Difficulty) -> bool {
        self.eq(&other.0)
    }
}

impl PartialOrd<Difficulty> for DifficultyInner {
    fn partial_cmp(&self, other: &Difficulty) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl ToHex for Difficulty {
    fn encode_hex<T: std::iter::FromIterator<char>>(&self) -> T {
        ToHex::encode_hex(&self.0)
    }

    fn encode_hex_upper<T: std::iter::FromIterator<char>>(&self) -> T {
        ToHex::encode_hex_upper(&self.0)
    }
}

impl std::fmt::Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.encode_hex::<String>())
    }
}
