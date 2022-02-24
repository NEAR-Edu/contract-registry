use sha2::{Digest, Sha256};

pub fn hash_bytes(b: impl AsRef<[u8]>) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(b);
    hasher.finalize().to_vec()
}

pub fn encode_bs58(b: impl AsRef<[u8]>) -> String {
    bs58::encode(b).into_string()
}

pub fn hash_code(b: impl AsRef<[u8]>) -> String {
    encode_bs58(hash_bytes(b))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::hash_code::{encode_bs58, hash_bytes};

    #[test]
    fn test_encode() {
        assert_eq!("2yGEbwRGRKr9Udf39", encode_bs58("hello, world"));
    }

    #[test]
    fn test_hash_file() {
        let file_bytes = fs::read("./out.wasm").unwrap();
        let hash = hash_bytes(file_bytes);
        println!("{:?}", hash);
        println!("{:?}", encode_bs58(hash));
    }
}
