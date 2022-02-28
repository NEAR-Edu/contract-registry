use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    serde::{Deserialize, Serialize},
};

use crate::code_hash::CodeHash;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq, Debug)]
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

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct VerificationRequest {
    pub id: u64,
    pub repository: String,
    pub fee: U128,
    pub status: VerificationStatus,
    pub created_at: u64,
    pub updated_at: u64,
}
