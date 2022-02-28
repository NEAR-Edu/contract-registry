use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    env, require, AccountId, IntoStorageKey,
};

use crate::utils::prefix_key;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Ownership {
    pub owner: Option<AccountId>,
    pub proposed_owner: LazyOption<AccountId>,
}

impl Ownership {
    pub fn new<S>(storage_key_prefix: S, owner_id: AccountId) -> Self
    where
        S: IntoStorageKey,
    {
        let k = storage_key_prefix.into_storage_key();

        Self {
            owner: Some(owner_id),
            proposed_owner: LazyOption::new(prefix_key(&k, b"p"), None),
        }
    }

    pub fn assert_owner(&self) {
        require!(
            &env::predecessor_account_id()
                == self
                    .owner
                    .as_ref()
                    .unwrap_or_else(|| env::panic_str("No owner")),
            "Owner only"
        );
    }

    pub fn renounce_owner(&mut self) {
        self.assert_owner();
        self.owner = None;
        self.proposed_owner.remove();
    }

    pub fn propose_owner(&mut self, account_id: Option<AccountId>) {
        self.assert_owner();
        if let Some(a) = account_id {
            self.proposed_owner.set(&a);
        } else {
            self.proposed_owner.remove();
        }
    }

    pub fn accept_owner(&mut self) {
        let proposed_owner = self
            .proposed_owner
            .take()
            .unwrap_or_else(|| env::panic_str("No proposed owner"));
        require!(
            &env::predecessor_account_id() == &proposed_owner,
            "Proposed owner only"
        );
        self.owner = Some(proposed_owner);
    }
}

pub trait Ownable {
    fn own_get_owner(&self) -> Option<AccountId>;
    fn own_get_proposed_owner(&self) -> Option<AccountId>;
    fn own_renounce_owner(&mut self);
    fn own_propose_owner(&mut self, account_id: Option<AccountId>);
    fn own_accept_owner(&mut self);
}

#[macro_export]
macro_rules! impl_ownership {
    ($contract: ident, $ownership: ident) => {
        #[near_bindgen]
        impl Ownable for $contract {
            fn own_get_owner(&self) -> Option<AccountId> {
                self.$ownership.owner.clone()
            }

            fn own_get_proposed_owner(&self) -> Option<AccountId> {
                self.$ownership.proposed_owner.get()
            }

            #[payable]
            fn own_renounce_owner(&mut self) {
                assert_one_yocto();
                self.$ownership.renounce_owner()
            }

            #[payable]
            fn own_propose_owner(&mut self, account_id: Option<AccountId>) {
                assert_one_yocto();
                self.$ownership.propose_owner(account_id);
            }

            #[payable]
            fn own_accept_owner(&mut self) {
                assert_one_yocto();
                self.$ownership.accept_owner();
            }
        }
    };
}
