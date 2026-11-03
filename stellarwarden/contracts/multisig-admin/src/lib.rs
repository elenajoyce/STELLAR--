//! # Multisig Admin
//!
//! M-of-N multisig authorization. Deploy this contract and use its address as
//! the `admin` of any other stellarwarden primitive. Admin operations on those
//! contracts then require M-of-N signatures from the registered signers.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Env, Vec,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Signers,
    Threshold,
    Nonce,
    Approval(u64, Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum MultisigError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotASigner = 3,
    AlreadyApproved = 4,
    ThresholdNotMet = 5,
    InvalidThreshold = 6,
    WrongNonce = 7,
}

#[contract]
pub struct MultisigAdmin;

#[contractimpl]
impl MultisigAdmin {
    /// Initialise with signers and approval threshold M.
    pub fn initialize(env: Env, signers: Vec<Address>, threshold: u32) {
        if env.storage().instance().has(&DataKey::Signers) {
            panic_with_error!(&env, MultisigError::AlreadyInitialized);
        }
        if threshold == 0 || threshold as usize > signers.len() as usize {
            panic_with_error!(&env, MultisigError::InvalidThreshold);
        }
        env.storage().instance().set(&DataKey::Signers, &signers);
        env.storage()
            .instance()
            .set(&DataKey::Threshold, &threshold);
        env.storage().instance().set(&DataKey::Nonce, &0u64);
    }

    fn check_is_signer(env: &Env, address: &Address) -> bool {
        let signers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .unwrap_or_else(|| panic_with_error!(env, MultisigError::NotInitialized));
        for s in signers.iter() {
            if s == *address {
                return true;
            }
        }
        false
    }

    fn get_nonce(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::Nonce)
            .unwrap_or(0u64)
    }

    fn count_approvals(env: &Env, nonce: u64) -> u32 {
        let signers: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::Signers)
            .unwrap_or_else(|| panic_with_error!(env, MultisigError::NotInitialized));
        let mut count = 0u32;
        for s in signers.iter() {
            if env
                .storage()
                .temporary()
                .get::<DataKey, bool>(&DataKey::Approval(nonce, s))
                .unwrap_or(false)
            {
                count += 1;
            }
        }
        count
    }

    /// A registered signer submits their approval for the current nonce.
    pub fn approve(env: Env, signer: Address, nonce: u64) {
        signer.require_auth();
        let current = Self::get_nonce(&env);
        if nonce != current {
            panic_with_error!(&env, MultisigError::WrongNonce);
        }
        if !Self::check_is_signer(&env, &signer) {
            panic_with_error!(&env, MultisigError::NotASigner);
        }
        if env
            .storage()
            .temporary()
            .get::<DataKey, bool>(&DataKey::Approval(nonce, signer.clone()))
            .unwrap_or(false)
        {
            panic_with_error!(&env, MultisigError::AlreadyApproved);
        }
        env.storage()
            .temporary()
            .set(&DataKey::Approval(nonce, signer), &true);
    }

    /// Execute if threshold approvals collected. Bumps the nonce.
    pub fn execute(env: Env) -> u64 {
        let nonce = Self::get_nonce(&env);
        let threshold: u32 = env
            .storage()
            .instance()
            .get(&DataKey::Threshold)
            .unwrap_or_else(|| panic_with_error!(&env, MultisigError::NotInitialized));
        if Self::count_approvals(&env, nonce) < threshold {
            panic_with_error!(&env, MultisigError::ThresholdNotMet);
        }
        let new_nonce = nonce + 1;
        env.storage().instance().set(&DataKey::Nonce, &new_nonce);
        new_nonce
    }

    /// Returns the current nonce.
    pub fn nonce(env: Env) -> u64 {
        Self::get_nonce(&env)
    }

    /// Returns the approval threshold.
    pub fn threshold(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Threshold)
            .unwrap_or_else(|| panic_with_error!(&env, MultisigError::NotInitialized))
    }

    /// Returns the registered signers.
    pub fn signers(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::Signers)
            .unwrap_or_else(|| panic_with_error!(&env, MultisigError::NotInitialized))
    }

    /// Returns how many approvals collected for the current nonce.
    pub fn approval_count(env: Env) -> u32 {
        let nonce = Self::get_nonce(&env);
        Self::count_approvals(&env, nonce)
    }

    /// Returns `true` if the address is a registered signer.
    pub fn is_signer(env: Env, address: Address) -> bool {
        Self::check_is_signer(&env, &address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, vec, Address, Env};

    fn setup_2of3() -> (Env, Address, Address, Address, MultisigAdminClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, MultisigAdmin);
        let client = MultisigAdminClient::new(&env, &contract_id);
        let s1 = Address::generate(&env);
        let s2 = Address::generate(&env);
        let s3 = Address::generate(&env);
        let signers = vec![&env, s1.clone(), s2.clone(), s3.clone()];
        client.initialize(&signers, &2);
        (env, s1, s2, s3, client)
    }

    #[test]
    fn test_initialize() {
        let (_env, _s1, _s2, _s3, client) = setup_2of3();
        assert_eq!(client.threshold(), 2);
        assert_eq!(client.nonce(), 0);
    }

    #[test]
    fn test_approve_and_execute() {
        let (_env, s1, s2, _s3, client) = setup_2of3();
        client.approve(&s1, &0);
        client.approve(&s2, &0);
        let new_nonce = client.execute();
        assert_eq!(new_nonce, 1);
        assert_eq!(client.nonce(), 1);
    }

    #[test]
    #[should_panic]
    fn test_execute_fails_without_threshold() {
        let (_env, s1, _s2, _s3, client) = setup_2of3();
        client.approve(&s1, &0);
        client.execute();
    }

    #[test]
    #[should_panic]
    fn test_non_signer_cannot_approve() {
        let (env, _s1, _s2, _s3, client) = setup_2of3();
        let outsider = Address::generate(&env);
        client.approve(&outsider, &0);
    }

    #[test]
    #[should_panic]
    fn test_double_approve_panics() {
        let (_env, s1, _s2, _s3, client) = setup_2of3();
        client.approve(&s1, &0);
        client.approve(&s1, &0);
    }

    #[test]
    fn test_is_signer() {
        let (env, s1, _s2, _s3, client) = setup_2of3();
        let outsider = Address::generate(&env);
        assert!(client.is_signer(&s1));
        assert!(!client.is_signer(&outsider));
    }
}
