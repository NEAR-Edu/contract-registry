use model::{
    code_hash::CodeHash,
    verification::{Verification, VerificationRequest, VerificationStatus},
};
use near_sdk::{
    assert_one_yocto,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{UnorderedMap, Vector},
    env,
    json_types::{U128, U64},
    near_bindgen, require, AccountId, BorshStorageKey, PanicOnDefault,
};

use crate::{
    impl_ownership,
    ownership::{Ownable, Ownership},
    utils::storage_refund,
};

#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKey {
    OWNERSHIP,
    REQUESTS,
    VERIFICATIONS,
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    pub ownership: Ownership,
    pub requests: Vector<VerificationRequest>,
    pub verifications: UnorderedMap<CodeHash, Verification>,
    pub verification_fee: u128,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, verification_fee: U128) -> Self {
        Self {
            ownership: Ownership::new(StorageKey::OWNERSHIP, owner_id),
            requests: Vector::new(StorageKey::REQUESTS),
            verifications: UnorderedMap::new(StorageKey::VERIFICATIONS),
            verification_fee: verification_fee.into(),
        }
    }

    pub fn get_verification_fee(&self) -> U128 {
        self.verification_fee.into()
    }

    #[payable]
    pub fn set_verification_fee(&mut self, verification_fee: U128) {
        assert_one_yocto();
        self.ownership.assert_owner();
        self.verification_fee = verification_fee.into();
    }

    pub fn get_verification_request(&self, id: U64) -> Option<VerificationRequest> {
        self.requests.get(id.into())
    }

    pub fn get_verification_result(&self, request_id: U64) -> Option<Verification> {
        let request_id = u64::from(request_id);
        self.verifications
            .values()
            .find(|v| v.request_id == request_id)
    }

    pub fn verify_code_hash(&self, code_hash: CodeHash) -> Option<Verification> {
        self.verifications.get(&code_hash)
    }

    pub fn get_pending_requests(&self) -> Vec<VerificationRequest> {
        self.requests
            .iter()
            .filter(|r| r.status == VerificationStatus::PENDING)
            .collect()
    }

    #[payable]
    pub fn request_verification(&mut self, repository: String, checkout: String, path: String, fee: U128) -> VerificationRequest {
        let attached_deposit = env::attached_deposit();
        require!(attached_deposit > 0, "Deposit required");

        let verification_fee: u128 = fee.into();
        require!(
            attached_deposit >= verification_fee,
            "Deposit less than indicated fee"
        );
        require!(
            verification_fee >= self.verification_fee,
            "Fee less than minimum requirement"
        );

        let storage_usage_start = env::storage_usage();

        let id = self.requests.len();
        let now = env::block_timestamp();

        let request = VerificationRequest {
            id,
            repository,
            checkout,
            path,
            fee: verification_fee.into(),
            status: VerificationStatus::PENDING,
            created_at: now,
            updated_at: now,
        };

        self.requests.push(&request);

        storage_refund(storage_usage_start, verification_fee);

        request
    }

    fn resolve(&mut self, id: u64, result: Option<Verification>) {
        let attached_deposit = env::attached_deposit();
        require!(attached_deposit > 0, "Deposit required");

        let storage_usage_start = env::storage_usage();

        self.ownership.assert_owner();

        let request = self
            .requests
            .get(id)
            .unwrap_or_else(|| env::panic_str("Request ID does not exist"));

        require!(
            request.status == VerificationStatus::PENDING,
            "Request already resolved"
        );

        let now = env::block_timestamp();
        if let Some(result) = &result {
            self.verifications.insert(&result.code_hash, &result);
            self.requests.replace(
                id,
                &VerificationRequest {
                    status: VerificationStatus::SUCCESS,
                    updated_at: now,
                    ..request
                },
            );
        } else {
            self.requests.replace(
                id,
                &VerificationRequest {
                    status: VerificationStatus::FAILURE,
                    updated_at: now,
                    ..request
                },
            );
        }

        storage_refund(storage_usage_start, 0);
    }

    #[payable]
    pub fn verification_success(&mut self, result: Verification) {
        self.resolve(result.request_id, Some(result));
    }

    #[payable]
    pub fn verification_failure(&mut self, id: U64) {
        self.resolve(id.into(), None);
    }
}

impl_ownership!(Contract, ownership);
