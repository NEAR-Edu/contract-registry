mod ownership;
mod utils;

mod contract;
pub use contract::*;

#[cfg(test)]
mod tests {
    use model::verification::VerificationStatus;
    use near_sdk::{test_utils::*, testing_env, AccountId};

    use crate::{ownership::Ownable, Contract};

    const ONE_NEAR: u128 = u128::pow(10, 24);

    fn account_contract() -> AccountId {
        "contract".parse::<AccountId>().unwrap()
    }

    fn account_owner() -> AccountId {
        "owner".parse::<AccountId>().unwrap()
    }

    fn account_user1() -> AccountId {
        "alice".parse::<AccountId>().unwrap()
    }

    fn account_user2() -> AccountId {
        "bob".parse::<AccountId>().unwrap()
    }

    const VERIFICATION_FEE: u128 = ONE_NEAR * 1;
    const REPOSITORY_URL: &'static str = "https://github.com/NEAR-Edu/stats.gallery-dapp.git";

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(account_contract())
            .account_balance(15 * ONE_NEAR)
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn initialization() {
        let context = get_context(account_owner());
        testing_env!(context.build());

        let contract = Contract::new(account_owner(), VERIFICATION_FEE.into());

        assert_eq!(
            contract.own_get_owner(),
            Some(account_owner()),
            "Account owner set correctly"
        );

        assert_eq!(
            u128::from(contract.get_verification_fee()),
            VERIFICATION_FEE,
            "Verification fee set correctly"
        );

        assert_eq!(
            contract.get_pending_requests().len(),
            0,
            "Empty pending requests on init"
        );
    }

    #[test]
    fn create_request() {
        let context = get_context(account_owner());
        testing_env!(context.build());

        let mut contract = Contract::new(account_owner(), VERIFICATION_FEE.into());

        let mut context = get_context(account_user1());
        context.attached_deposit(VERIFICATION_FEE + u128::pow(10, 22) /* storage fee */);
        testing_env!(context.build());

        let request =
            contract.request_verification(REPOSITORY_URL.to_string(), VERIFICATION_FEE.into());

        assert_eq!(
            request.repository, REPOSITORY_URL,
            "Repository URL set correctly"
        );

        assert_eq!(
            u128::from(request.fee),
            VERIFICATION_FEE,
            "Fee set correctly"
        );

        assert_eq!(
            request.status,
            VerificationStatus::PENDING,
            "Status is pending"
        );

        let by_id = contract
            .get_verification_request(request.id.into())
            .expect("Can get request by ID");

        assert_eq!(by_id, request, "Request by ID matches");
    }

    #[test]
    #[should_panic(expected = "Insufficient deposit")]
    fn create_request_insufficient_deposit() {
        let context = get_context(account_owner());
        testing_env!(context.build());

        let mut contract = Contract::new(account_owner(), VERIFICATION_FEE.into());

        let mut context = get_context(account_user1());
        context.attached_deposit(VERIFICATION_FEE); // no storage fee
        testing_env!(context.build());

        contract.request_verification(REPOSITORY_URL.to_string(), VERIFICATION_FEE.into());
    }

    #[test]
    #[should_panic(expected = "Deposit required")]
    fn create_request_no_deposit() {
        let context = get_context(account_owner());
        testing_env!(context.build());

        let mut contract = Contract::new(account_owner(), VERIFICATION_FEE.into());

        let context = get_context(account_user1());
        // context.attached_deposit(...); // no deposit
        testing_env!(context.build());

        contract.request_verification(REPOSITORY_URL.to_string(), VERIFICATION_FEE.into());
    }
}
