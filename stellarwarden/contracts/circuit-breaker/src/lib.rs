//! # Circuit Breaker
//!
//! An admin-controlled emergency freeze switch. Other contracts call
//! [`CircuitBreakerClient::check`] before allowing a transfer. When the breaker
//! is tripped (frozen), `check` panics and blocks the transfer.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Env,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Frozen,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum CircuitBreakerError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    CircuitTripped = 4,
}

#[contract]
pub struct CircuitBreaker;

#[contractimpl]
impl CircuitBreaker {
    /// Initialise with an admin address. Breaker starts in the active state.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, CircuitBreakerError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Frozen, &false);
    }

    fn require_admin(env: &Env) -> Address {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, CircuitBreakerError::NotInitialized));
        admin.require_auth();
        admin
    }

    /// Trip the circuit breaker — all subsequent `check` calls will panic.
    pub fn trip(env: Env) {
        Self::require_admin(&env);
        env.storage().instance().set(&DataKey::Frozen, &true);
    }

    /// Reset the circuit breaker — `check` calls will pass again.
    pub fn reset(env: Env) {
        Self::require_admin(&env);
        env.storage().instance().set(&DataKey::Frozen, &false);
    }

    /// Transfer admin rights.
    pub fn set_admin(env: Env, new_admin: Address) {
        Self::require_admin(&env);
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }

    /// Returns `true` if the breaker is currently tripped.
    pub fn is_frozen(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Frozen)
            .unwrap_or(false)
    }

    /// Returns the current admin.
    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, CircuitBreakerError::NotInitialized))
    }

    /// Panics with `CircuitTripped` if the breaker is frozen.
    pub fn check(env: Env) {
        let frozen: bool = env
            .storage()
            .instance()
            .get(&DataKey::Frozen)
            .unwrap_or(false);
        if frozen {
            panic_with_error!(&env, CircuitBreakerError::CircuitTripped);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn setup() -> (Env, Address, CircuitBreakerClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, CircuitBreaker);
        let client = CircuitBreakerClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, admin, client)
    }

    #[test]
    fn test_starts_not_frozen() {
        let (_env, _admin, client) = setup();
        assert!(!client.is_frozen());
    }

    #[test]
    fn test_check_passes_when_active() {
        let (_env, _admin, client) = setup();
        client.check();
    }

    #[test]
    fn test_trip_freezes() {
        let (_env, _admin, client) = setup();
        client.trip();
        assert!(client.is_frozen());
    }

    #[test]
    #[should_panic]
    fn test_check_panics_when_frozen() {
        let (_env, _admin, client) = setup();
        client.trip();
        client.check();
    }

    #[test]
    fn test_reset_unfreezes() {
        let (_env, _admin, client) = setup();
        client.trip();
        client.reset();
        assert!(!client.is_frozen());
        client.check();
    }

    #[test]
    fn test_set_admin() {
        let (env, _admin, client) = setup();
        let new_admin = Address::generate(&env);
        client.set_admin(&new_admin);
        assert_eq!(client.admin(), new_admin);
    }
}
