use std::error::Error;

use serde::{Deserialize, Serialize};

/// This type allows to use both hex strings (e.g. from JS) and byte arrays in RPC arguments.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum OrHex<T> {
    String(String),
    Data(T),
}

impl<T> From<String> for OrHex<T> {
    fn from(value: String) -> Self {
        OrHex::String(value)
    }
}

impl<T> From<&str> for OrHex<T> {
    fn from(value: &str) -> Self {
        OrHex::String(value.to_owned())
    }
}

impl<const N: usize> From<[u8; N]> for OrHex<[u8; N]> {
    fn from(value: [u8; N]) -> Self {
        OrHex::Data(value)
    }
}

impl<const N: usize> TryFrom<OrHex<[u8; N]>> for [u8; N] {
    type Error = Box<dyn Error>;

    fn try_from(value: OrHex<[u8; N]>) -> Result<Self, Self::Error> {
        match value {
            OrHex::String(s) => parse_hex_string(&s),
            OrHex::Data(data) => Ok(data),
        }
    }
}

fn parse_hex_string<const N: usize>(s: &str) -> Result<[u8; N], Box<dyn Error>> {
    let bytes = hex::decode(s.trim_start_matches("0x"))?;
    let arr: [u8; N] = bytes.try_into().map_err(|_| "invalid hex string length")?;
    Ok(arr)
}

#[cfg(test)]
mod tests {
    use ccp_shared::types::Difficulty;
    use serde_json::json;

    use super::*;

    fn a<T>(_: impl Into<OrHex<T>>) {}

    #[test]
    fn test_from_str() {
        a::<Difficulty>("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    }

    #[test]
    fn test_from_string() {
        a::<Difficulty>("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF");
    }

    #[test]
    fn test_from_data() {
        let d = Difficulty::default();
        a::<Difficulty>(d);
    }

    #[test]
    fn test_str_into_data_prefix() {
        let dx: OrHex<Difficulty> =
            "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".into();
        let _d: Difficulty = dx.try_into().unwrap();
    }

    #[test]
    fn test_str_into_data_bare() {
        let dx: OrHex<Difficulty> =
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".into();
        let _d: Difficulty = dx.try_into().unwrap();
    }

    #[test]
    fn test_data_into_data() {
        let dx: OrHex<Difficulty> = Difficulty::default().into();
        let _d: Difficulty = dx.try_into().unwrap();
    }

    #[test]
    fn test_invalid_hex() {
        let dx: OrHex<Difficulty> =
            "FGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGFFFFFFFFFF".into();
        assert!(TryInto::<Difficulty>::try_into(dx).is_err());
    }

    #[test]
    fn test_invalid_len() {
        let dx: OrHex<Difficulty> =
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF".into();
        assert!(TryInto::<Difficulty>::try_into(dx).is_err());
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
        let a = OrHex::<Difficulty>::Data([0xFF; 32]);
        let j = serde_json::to_value(&a).unwrap();
        assert_eq!(j, json!(vec![0xFF; 32]));
    }

    #[test]
    fn deserialize_from_array() {
        let a: Difficulty = [0xFF; 32];
        let j = serde_json::to_string(&a).unwrap();
        let o: OrHex<Difficulty> = serde_json::from_str(&j).unwrap();
        let a2: Difficulty = o.try_into().unwrap();
        assert_eq!(a, a2);
    }

    #[test]
    fn deserialize_from_string() {
        let a: Difficulty = [0xFF; 32];
        let astr = "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF";
        let j = serde_json::to_string(&astr).unwrap();
        let o: OrHex<Difficulty> = serde_json::from_str(&j).unwrap();
        let a2: Difficulty = o.try_into().unwrap();
        assert_eq!(a, a2);
    }
}
