//! # Denylist Gate
//!
//! A standalone denylist contract. Other contracts call
//! [`DenylistGateClient::check`] via cross-contract invocation before executing
//! a transfer. If the address is on the denylist the call panics.
//!
//! ## Roles
//! - **Admin**: initialised once via `initialize`. Can add/remove addresses and
//!   pause/unpause the gate.
//!
//! ## Storage layout
//! | Key | Type | Meaning |
//! |-----|------|---------|
//! | `ADMIN` | `Address` | contract administrator |
//! | `DENY:<addr>` | `bool` | address is denied |
//! | `PAUSED` | `bool` | gate is paused |

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Env,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Denied(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum DenylistError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    AddressDenied = 4,
    ContractPaused = 5,
}

#[contract]
pub struct DenylistGate;

#[contractimpl]
impl DenylistGate {
    /// Initialise the gate with an admin address. Can only be called once.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, DenylistError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    fn require_admin(env: &Env) -> Address {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, DenylistError::NotInitialized));
        admin.require_auth();
        admin
    }

    /// Add an address to the denylist.
    pub fn deny(env: Env, address: Address) {
        Self::require_admin(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Denied(address), &true);
    }

    /// Remove an address from the denylist.
    pub fn allow(env: Env, address: Address) {
        Self::require_admin(&env);
        env.storage().persistent().remove(&DataKey::Denied(address));
    }

    /// Pause the gate.
    pub fn pause(env: Env) {
        Self::require_admin(&env);
        pausable::pause(&env);
    }

    /// Unpause the gate.
    pub fn unpause(env: Env) {
        Self::require_admin(&env);
        pausable::unpause(&env);
    }

    /// Transfer admin rights.
    pub fn set_admin(env: Env, new_admin: Address) {
        Self::require_admin(&env);
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }

    /// Returns the current admin address.
    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, DenylistError::NotInitialized))
    }

    /// Returns `true` if `address` is on the denylist.
    pub fn is_denied(env: Env, address: Address) -> bool {
        env.storage()
            .persistent()
            .get(&DataKey::Denied(address))
            .unwrap_or(false)
    }

    /// Returns `true` if the gate is paused.
    pub fn paused(env: Env) -> bool {
        pausable::is_paused(&env)
    }

    /// Called by other contracts before executing a transfer.
    /// Panics if the gate is paused or if `address` is on the denylist.
    pub fn check(env: Env, address: Address) {
        if pausable::is_paused(&env) {
            panic_with_error!(&env, DenylistError::ContractPaused);
        }
        if env
            .storage()
            .persistent()
            .get::<DataKey, bool>(&DataKey::Denied(address.clone()))
            .unwrap_or(false)
        {
            panic_with_error!(&env, DenylistError::AddressDenied);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn setup() -> (Env, Address, DenylistGateClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, DenylistGate);
        let client = DenylistGateClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, admin, client)
    }

    #[test]
    fn test_initialize() {
        let (_env, admin, client) = setup();
        assert_eq!(client.admin(), admin);
    }

    #[test]
    #[should_panic]
    fn test_double_initialize_panics() {
        let (env, _admin, client) = setup();
        let other = Address::generate(&env);
        client.initialize(&other);
    }

    #[test]
    fn test_deny_sets_flag() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);
        client.deny(&user);
        assert!(client.is_denied(&user));
    }

    #[test]
    fn test_allow_removes_flag() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);
        client.deny(&user);
        client.allow(&user);
        assert!(!client.is_denied(&user));
    }

    #[test]
    fn test_check_passes_clean_address() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);
        client.check(&user);
    }

    #[test]
    #[should_panic]
    fn test_check_panics_for_denied_address() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);
        client.deny(&user);
        client.check(&user);
    }

    #[test]
    #[should_panic]
    fn test_check_panics_when_paused() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);
        client.pause();
        client.check(&user);
    }

    #[test]
    fn test_unpause_resumes_checks() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);
        client.pause();
        client.unpause();
        client.check(&user);
    }

    #[test]
    fn test_set_admin() {
        let (env, _admin, client) = setup();
        let new_admin = Address::generate(&env);
        client.set_admin(&new_admin);
        assert_eq!(client.admin(), new_admin);
    }
}
