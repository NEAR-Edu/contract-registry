use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    serde::{Deserialize, Serialize},
};

use crate::{code_hash::CodeHash, sequential_id::SequentialId};

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum VerificationStatus {
    PENDING,
    SUCCESS,
    FAILURE,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Verification {
    pub id: u64,
    pub code_hash: CodeHash,
    pub code_url: String,
    pub repository: String,
    pub remote: String,
    pub branch: String,
    pub commit: String,
    pub request_id: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct VerificationRequest {
    pub id: u64,
    pub repository: String,
    pub fee: U128,
    pub status: VerificationStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

impl SequentialId<u64> for VerificationRequest {
    fn seq_id(&self) -> u64 {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::{serde_json, };

    use super::VerificationRequest;

    #[test]
    fn test() {
        let s = serde_json::to_string(&VerificationRequest {
            id: 0,
            repository: "repository".to_string(),
            fee: (3u128).into(),
            status: super::VerificationStatus::PENDING,
            created_at: 0,
            updated_at: 0,
        });
        println!("{:?}", s);
    }
}
