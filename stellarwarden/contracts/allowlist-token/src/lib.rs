//! # Allowlist Token
//!
//! A token wrapper that only permits transfers between two addresses that are
//! **both** present on an on-chain allowlist. Forwards cleared transfers to an
//! underlying SEP-41 token contract.
//!
//! ## Threat model note
//! This wrapper cannot stop a holder from calling the underlying token directly.
//! The underlying token must enforce wrapper-only transfers via `AUTH_REQUIRED`
//! and a custody design, otherwise this wrapper provides no security guarantee.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, token, Address, Env,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Token,
    Allowed(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum AllowlistError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    FromNotAllowed = 4,
    ToNotAllowed = 5,
    ContractPaused = 6,
}

#[contract]
pub struct AllowlistToken;

#[contractimpl]
impl AllowlistToken {
    /// Initialise the wrapper with an admin and the underlying token address.
    pub fn initialize(env: Env, admin: Address, token: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, AllowlistError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
    }

    fn require_admin(env: &Env) -> Address {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, AllowlistError::NotInitialized));
        admin.require_auth();
        admin
    }

    fn token_address(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Token)
            .unwrap_or_else(|| panic_with_error!(env, AllowlistError::NotInitialized))
    }

    /// Add an address to the allowlist.
    pub fn allowlist(env: Env, address: Address) {
        Self::require_admin(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Allowed(address), &true);
    }

    /// Remove an address from the allowlist.
    pub fn denylist(env: Env, address: Address) {
        Self::require_admin(&env);
        env.storage()
            .persistent()
            .remove(&DataKey::Allowed(address));
    }

    /// Pause the wrapper.
    pub fn pause(env: Env) {
        Self::require_admin(&env);
        pausable::pause(&env);
    }

    /// Unpause the wrapper.
    pub fn unpause(env: Env) {
        Self::require_admin(&env);
        pausable::unpause(&env);
    }

    /// Transfer admin rights.
    pub fn set_admin(env: Env, new_admin: Address) {
        Self::require_admin(&env);
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }

    /// Returns the current admin.
    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, AllowlistError::NotInitialized))
    }

    /// Returns the underlying token address.
    pub fn token(env: Env) -> Address {
        Self::token_address(&env)
    }

    /// Returns `true` if `address` is on the allowlist.
    pub fn is_allowed(env: Env, address: Address) -> bool {
        env.storage()
            .persistent()
            .get::<DataKey, bool>(&DataKey::Allowed(address))
            .unwrap_or(false)
    }

    /// Returns `true` if the wrapper is paused.
    pub fn paused(env: Env) -> bool {
        pausable::is_paused(&env)
    }

    /// Transfer `amount` from `from` to `to`.
    /// Both must be on the allowlist.
    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        if pausable::is_paused(&env) {
            panic_with_error!(&env, AllowlistError::ContractPaused);
        }
        if !env
            .storage()
            .persistent()
            .get::<DataKey, bool>(&DataKey::Allowed(from.clone()))
            .unwrap_or(false)
        {
            panic_with_error!(&env, AllowlistError::FromNotAllowed);
        }
        if !env
            .storage()
            .persistent()
            .get::<DataKey, bool>(&DataKey::Allowed(to.clone()))
            .unwrap_or(false)
        {
            panic_with_error!(&env, AllowlistError::ToNotAllowed);
        }
        from.require_auth();
        let token_addr = Self::token_address(&env);
        let token_client = token::Client::new(&env, &token_addr);
        token_client.transfer(&from, &to, &amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn setup() -> (Env, Address, Address, AllowlistTokenClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let token_admin = Address::generate(&env);
        let token_addr = env
            .register_stellar_asset_contract_v2(token_admin)
            .address();
        let contract_id = env.register_contract(None, AllowlistToken);
        let client = AllowlistTokenClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin, &token_addr);
        (env, admin, token_addr, client)
    }

    #[test]
    fn test_initialize() {
        let (_env, admin, token_addr, client) = setup();
        assert_eq!(client.admin(), admin);
        assert_eq!(client.token(), token_addr);
    }

    #[test]
    fn test_allowlist_and_is_allowed() {
        let (env, _admin, _token, client) = setup();
        let user = Address::generate(&env);
        assert!(!client.is_allowed(&user));
        client.allowlist(&user);
        assert!(client.is_allowed(&user));
    }

    #[test]
    fn test_denylist_removes_address() {
        let (env, _admin, _token, client) = setup();
        let user = Address::generate(&env);
        client.allowlist(&user);
        client.denylist(&user);
        assert!(!client.is_allowed(&user));
    }

    #[test]
    #[should_panic]
    fn test_transfer_fails_if_from_not_allowed() {
        let (env, _admin, _token, client) = setup();
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        client.allowlist(&to);
        client.transfer(&from, &to, &100);
    }

    #[test]
    #[should_panic]
    fn test_transfer_fails_if_to_not_allowed() {
        let (env, _admin, _token, client) = setup();
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        client.allowlist(&from);
        client.transfer(&from, &to, &100);
    }

    #[test]
    #[should_panic]
    fn test_transfer_fails_when_paused() {
        let (env, _admin, _token, client) = setup();
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        client.allowlist(&from);
        client.allowlist(&to);
        client.pause();
        client.transfer(&from, &to, &100);
    }

    #[test]
    fn test_pause_and_unpause() {
        let (_env, _admin, _token, client) = setup();
        assert!(!client.paused());
        client.pause();
        assert!(client.paused());
        client.unpause();
        assert!(!client.paused());
    }
}
