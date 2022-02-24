use std::fmt::Display;

use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    serde::{de::*, Serialize, Serializer},
};
use sha2::{Digest, Sha256};

#[derive(BorshDeserialize, BorshSerialize, PartialEq, Clone, Debug)]
pub struct CodeHash(pub Vec<u8>); // vs [u8; 32] ?

impl CodeHash {
    pub fn hash_bytes(bytes: impl AsRef<[u8]>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Self(hasher.finalize().to_vec())
    }
}

impl From<&[u8]> for CodeHash {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

impl From<String> for CodeHash {
    fn from(s: String) -> Self {
        todo!()
    }
}

impl Display for CodeHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", bs58::encode(&self.0).into_string())
    }
}

impl Serialize for CodeHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let CodeHash(v) = self;
        let encoded = bs58::encode(v).into_string();
        serializer.serialize_str(&encoded)
    }
}

struct CodeHashVisitor;

impl<'de> Visitor<'de> for CodeHashVisitor {
    type Value = CodeHash;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("base58-encoded hash")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: near_sdk::serde::de::Error,
    {
        bs58::decode(v)
            .into_vec()
            .map(|v| CodeHash(v))
            .map_err(|e| E::custom(format!("base58 decode error: {}", e)))
    }
}

impl<'de> Deserialize<'de> for CodeHash {
    fn deserialize<D>(deserializer: D) -> Result<CodeHash, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(CodeHashVisitor)
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::serde_json;

    use crate::CodeHash;

    #[test]
    fn code_hash_serialize() {
        let expected = "\"2yGEbwRGRKr9Udf39\"";
        let actual = serde_json::to_string(&CodeHash("hello, world".as_bytes().to_vec()))
            .expect("Cannot serialize");
        assert_eq!(expected, actual);
    }

    #[test]
    fn code_hash_deserialize() {
        let expected = CodeHash("hello, world".as_bytes().to_vec());
        let actual: CodeHash =
            serde_json::from_str("\"2yGEbwRGRKr9Udf39\"").expect("Cannot deserialize");
        assert_eq!(expected, actual);
    }
}
