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

/// This type allows to use both hex strings (e.g. from JS) and byte arrays in RPC arguments.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum OrHex<T> {
    String(String),
    Data(T),
}

impl<T> OrHex<T> {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: impl Into<String>) -> OrHex<T> {
        Self::String(s.into())
    }
}

use std::fmt;
impl<T: fmt::Display> fmt::Display for OrHex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrHex::String(s) => write!(f, "{s}"),
            OrHex::Data(d) => write!(f, "{d}"),
        }
    }
}

impl<T: FromHex> From<T> for OrHex<T> {
    fn from(value: T) -> Self {
        OrHex::Data(value)
    }
}

impl<T: FromHex> OrHex<T> {
    pub fn unhex(self) -> Result<T, <T as FromHex>::Error> {
        match self {
            OrHex::String(s) => from_hex_with_prefix(&s),
            OrHex::Data(data) => Ok(data),
        }
    }
}

fn from_hex_with_prefix<T: FromHex>(s: &str) -> Result<T, <T as FromHex>::Error> {
    <_>::from_hex(s.trim_start_matches("0x"))
}

#[cfg(test)]
mod tests {
    use ccp_shared::types::Difficulty;
    use serde_json::json;

    use super::*;

    fn a<T>(_: impl Into<OrHex<T>>) {}

    #[test]
    fn test_from_str() {
        a::<Difficulty>(OrHex::from_str(
            "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        ));
    }

    #[test]
    fn test_from_string() {
        a::<Difficulty>(OrHex::from_str(
            "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
        ));
    }

    #[test]
    fn test_from_data() {
        let d = Difficulty::default();
        a::<Difficulty>(d);
    }

    #[test]
    fn test_str_into_data_prefix() {
        let dx: OrHex<Difficulty> =
            OrHex::from_str("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
        let _d: Difficulty = dx.unhex().unwrap();
    }

    #[test]
    fn test_str_into_data_bare() {
        let dx: OrHex<Difficulty> =
            OrHex::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
        let _d: Difficulty = dx.unhex().unwrap();
    }

    #[test]
    fn test_data_into_data() {
        let dx: OrHex<Difficulty> = Difficulty::default().into();
        let _d: Difficulty = dx.unhex().unwrap();
    }

    #[test]
    fn test_invalid_hex() {
        let dx: OrHex<Difficulty> =
            OrHex::from_str("FGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGFFFFFFFFFF");
        assert!(dx.unhex().is_err());
    }

    #[test]
    fn test_invalid_len() {
        let dx: OrHex<Difficulty> =
            OrHex::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
        assert!(dx.unhex().is_err());
    }

    #[test]
    fn serialize_str() {
        let a = OrHex::<Difficulty>::String(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".to_owned(),
        );
        let j = serde_json::to_value(&a).unwrap();
        assert_eq!(
            j,
            json!("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF")
        );
    }

    #[test]
    fn serialize_vec() {
        let a = OrHex::<Difficulty>::Data(Difficulty::new([0xFF; 32]));
        let j = serde_json::to_value(&a).unwrap();
        assert_eq!(j, json!(vec![0xFF; 32]));
    }

    #[test]
    fn deserialize_from_array() {
        let a: Difficulty = Difficulty::new([0xFF; 32]);
        let j = serde_json::to_string(&a).unwrap();
        let o: OrHex<Difficulty> = serde_json::from_str(&j).unwrap();
        let a2: Difficulty = o.unhex().unwrap();
        assert_eq!(a, a2);
    }

    #[test]
    fn deserialize_from_string() {
        let a: Difficulty = Difficulty::new([0xFF; 32]);
        let astr = "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";
        let j = serde_json::to_string(&astr).unwrap();
        let o: OrHex<Difficulty> = serde_json::from_str(&j).unwrap();
        let a2: Difficulty = o.unhex().unwrap();
        assert_eq!(a, a2);
    }
}
