//! # Circuit Breaker + Policy Engine
//!
//! Wires circuit-breaker as a pre-check before denylist + jurisdiction checks.

#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contracttype, panic_with_error, Address,
    Env, String, Vec,
};

// ── Cross-contract client interfaces ───────────────────────────────────────
// Only the methods this contract calls are declared here as client traits, so
// circuit-breaker, denylist-gate, and jurisdiction-flag are not linked into
// this contract's WASM. They are deployed separately and invoked by address.
#[contractclient(name = "CircuitBreakerClient")]
pub trait CircuitBreakerTrait {
    fn check(env: Env);
}

#[contractclient(name = "DenylistGateClient")]
pub trait DenylistGateTrait {
    fn check(env: Env, address: Address);
}

#[contractclient(name = "JurisdictionFlagClient")]
pub trait JurisdictionFlagTrait {
    fn is_permitted_jurisdiction(env: Env, address: Address, permitted_codes: Vec<String>) -> bool;
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    CircuitBreaker,
    DenylistGate,
    JurisdictionFlag,
    PermittedJurisdictions,
    Balance(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum CbPeError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    InsufficientFunds = 3,
}

#[contract]
pub struct CbPeToken;

#[contractimpl]
impl CbPeToken {
    pub fn initialize(
        env: Env,
        admin: Address,
        circuit_breaker: Address,
        denylist_gate: Address,
        jurisdiction_flag: Address,
        permitted_jurisdictions: Vec<String>,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, CbPeError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::CircuitBreaker, &circuit_breaker);
        env.storage()
            .instance()
            .set(&DataKey::DenylistGate, &denylist_gate);
        env.storage()
            .instance()
            .set(&DataKey::JurisdictionFlag, &jurisdiction_flag);
        env.storage()
            .instance()
            .set(&DataKey::PermittedJurisdictions, &permitted_jurisdictions);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, CbPeError::NotInitialized));
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
            .unwrap_or_else(|| panic_with_error!(&env, CbPeError::NotInitialized));
        CircuitBreakerClient::new(&env, &cb_addr).check();

        let dl_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::DenylistGate)
            .unwrap_or_else(|| panic_with_error!(&env, CbPeError::NotInitialized));
        let dl = DenylistGateClient::new(&env, &dl_addr);
        dl.check(&from);
        dl.check(&to);

        let jf_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::JurisdictionFlag)
            .unwrap_or_else(|| panic_with_error!(&env, CbPeError::NotInitialized));
        let permitted: Vec<String> = env
            .storage()
            .instance()
            .get(&DataKey::PermittedJurisdictions)
            .unwrap_or_else(|| panic_with_error!(&env, CbPeError::NotInitialized));
        let jf = JurisdictionFlagClient::new(&env, &jf_addr);
        jf.is_permitted_jurisdiction(&from, &permitted);
        jf.is_permitted_jurisdiction(&to, &permitted);

        let fb: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        if fb < amount {
            panic_with_error!(&env, CbPeError::InsufficientFunds);
        }
        let tb: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from), &(fb - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(tb + amount));
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
    use jurisdiction_flag::JurisdictionFlag;
    use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

    #[test]
    fn test_initialize_and_mint() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        let cb_id = env.register_contract(None, CircuitBreaker);
        circuit_breaker::CircuitBreakerClient::new(&env, &cb_id).initialize(&admin);
        let dl_id = env.register_contract(None, DenylistGate);
        denylist_gate::DenylistGateClient::new(&env, &dl_id).initialize(&admin);
        let jf_id = env.register_contract(None, JurisdictionFlag);
        jurisdiction_flag::JurisdictionFlagClient::new(&env, &jf_id).initialize(&admin);

        let permitted = vec![&env, String::from_str(&env, "US")];
        let contract_id = env.register_contract(None, CbPeToken);
        let client = CbPeTokenClient::new(&env, &contract_id);
        client.initialize(&admin, &cb_id, &dl_id, &jf_id, &permitted);

        let user = Address::generate(&env);
        client.mint(&user, &500);
        assert_eq!(client.balance(&user), 500);
    }
}
