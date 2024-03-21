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

use hex::{FromHex, ToHex};
use serde::{Deserialize, Serialize};

pub type LocalNonceInner = [u8; 32];

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct LocalNonce(LocalNonceInner);

impl LocalNonce {
    pub fn new(inner: LocalNonceInner) -> Self {
        Self(inner)
    }

    /// Creates a new random nonce.
    /// It uses random generator to be sure that in the next start with the same parameters,
    /// CCP won't do the same job twice.
    pub fn random() -> Self {
        use rand::RngCore;

        let mut rng = rand::thread_rng();
        let mut nonce_inner = LocalNonceInner::default();

        rng.fill_bytes(&mut nonce_inner);

        LocalNonce::new(nonce_inner)
    }
}

impl AsRef<LocalNonceInner> for LocalNonce {
    fn as_ref(&self) -> &LocalNonceInner {
        &self.0
    }
}

impl AsMut<LocalNonceInner> for LocalNonce {
    fn as_mut(&mut self) -> &mut LocalNonceInner {
        &mut self.0
    }
}

impl FromHex for LocalNonce {
    type Error = <LocalNonceInner as FromHex>::Error;

    fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
        LocalNonceInner::from_hex(hex).map(Self)
    }
}

impl ToHex for LocalNonce {
    fn encode_hex<T: std::iter::FromIterator<char>>(&self) -> T {
        ToHex::encode_hex(&self.0)
    }

    fn encode_hex_upper<T: std::iter::FromIterator<char>>(&self) -> T {
        ToHex::encode_hex_upper(&self.0)
    }
}

impl std::fmt::Display for LocalNonce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.encode_hex::<String>())
    }
}

impl FromStr for LocalNonce {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FromHex::from_hex(s)
    }
}
