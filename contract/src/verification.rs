use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum VerificationStatus {
    PENDING,
    SUCCESS,
    FAILURE,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct VerificationResult {
    pub code_hash: CodeHash,
    pub code_url: String,
    pub repository: String,
    pub remote: String,
    pub branch: String,
    pub commit: String,
    pub verification_request_id: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct VerificationRequest {
    pub id: u64,
    pub repository: String,
    pub fee: Balance,
    pub status: VerificationStatus,
    pub code_hash: Option<CodeHash>,
    pub created_at: u64,
    pub updated_at: u64,
}
