//! # Denylist Gate Consumer
//!
//! A minimal reference token demonstrating the cross-contract calling pattern
//! for [`denylist-gate`] and [`circuit-breaker`] before executing a transfer.

#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contracttype, panic_with_error, Address,
    Env,
};

// ── Cross-contract client interfaces ───────────────────────────────────────
// Only the methods this contract calls are declared here as client traits, so
// circuit-breaker and denylist-gate are not linked into this contract's WASM.
// They are deployed separately and invoked by address at runtime.
#[contractclient(name = "CircuitBreakerClient")]
pub trait CircuitBreakerTrait {
    fn check(env: Env);
}

#[contractclient(name = "DenylistGateClient")]
pub trait DenylistGateTrait {
    fn check(env: Env, address: Address);
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    DenylistGate,
    CircuitBreaker,
    Balance(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ConsumerError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    InsufficientFunds = 3,
}

#[contract]
pub struct DenylistGateConsumer;

#[contractimpl]
impl DenylistGateConsumer {
    pub fn initialize(env: Env, admin: Address, denylist_gate: Address, circuit_breaker: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, ConsumerError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::DenylistGate, &denylist_gate);
        env.storage()
            .instance()
            .set(&DataKey::CircuitBreaker, &circuit_breaker);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, ConsumerError::NotInitialized));
        admin.require_auth();
        let bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(bal + amount));
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        let cb_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::CircuitBreaker)
            .unwrap_or_else(|| panic_with_error!(&env, ConsumerError::NotInitialized));
        CircuitBreakerClient::new(&env, &cb_addr).check();

        let dl_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::DenylistGate)
            .unwrap_or_else(|| panic_with_error!(&env, ConsumerError::NotInitialized));
        let dl_client = DenylistGateClient::new(&env, &dl_addr);
        dl_client.check(&from);
        dl_client.check(&to);

        let from_bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        if from_bal < amount {
            panic_with_error!(&env, ConsumerError::InsufficientFunds);
        }
        let to_bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(from_bal - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(to_bal + amount));
    }

    pub fn balance(env: Env, address: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(address))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use circuit_breaker::CircuitBreaker;
    use denylist_gate::DenylistGate;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    fn setup() -> (Env, DenylistGateConsumerClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        let dl_id = env.register_contract(None, DenylistGate);
        denylist_gate::DenylistGateClient::new(&env, &dl_id).initialize(&admin);

        let cb_id = env.register_contract(None, CircuitBreaker);
        circuit_breaker::CircuitBreakerClient::new(&env, &cb_id).initialize(&admin);

        let contract_id = env.register_contract(None, DenylistGateConsumer);
        let client = DenylistGateConsumerClient::new(&env, &contract_id);
        client.initialize(&admin, &dl_id, &cb_id);
        (env, client)
    }

    #[test]
    fn test_transfer_succeeds() {
        let (env, client) = setup();
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        client.mint(&from, &1000);
        client.transfer(&from, &to, &500);
        assert_eq!(client.balance(&from), 500);
        assert_eq!(client.balance(&to), 500);
    }

    #[test]
    #[should_panic]
    fn test_transfer_blocked_insufficient_funds() {
        let (env, client) = setup();
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        client.mint(&from, &100);
        client.transfer(&from, &to, &500);
    }
}
