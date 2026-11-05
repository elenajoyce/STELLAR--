//! # Pausable
//!
//! Shared pause/unpause helper library composed into other stellarwarden contracts
//! at compile time. Not a deployable contract — no `#[contract]` macro or WASM exports.
//!
//! ## Usage
//! ```ignore
//! use pausable::{pause, unpause, is_paused, require_not_paused};
//!
//! // In your contract's transfer function:
//! require_not_paused(&env);
//! ```

#![no_std]

use soroban_sdk::{contracterror, Env, Symbol};

const PAUSED_KEY: &str = "PAUSED";

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum PausableError {
    ContractPaused = 1,
}

/// Returns `true` if the contract is currently paused.
pub fn is_paused(env: &Env) -> bool {
    let key = Symbol::new(env, PAUSED_KEY);
    env.storage()
        .instance()
        .get::<Symbol, bool>(&key)
        .unwrap_or(false)
}

/// Sets the contract's paused state to `true`.
pub fn pause(env: &Env) {
    let key = Symbol::new(env, PAUSED_KEY);
    env.storage().instance().set(&key, &true);
}

/// Sets the contract's paused state to `false`.
pub fn unpause(env: &Env) {
    let key = Symbol::new(env, PAUSED_KEY);
    env.storage().instance().set(&key, &false);
}

/// Panics with `ContractPaused` if the contract is currently paused.
pub fn require_not_paused(env: &Env) {
    if is_paused(env) {
        panic!("ContractPaused");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{contract, contractimpl, Env};

    // Minimal wrapper contract so pausable functions can access storage in tests.
    #[contract]
    struct TestContract;

    #[contractimpl]
    impl TestContract {
        pub fn check_not_paused(env: Env) -> bool {
            !is_paused(&env)
        }
        pub fn do_pause(env: Env) {
            pause(&env);
        }
        pub fn do_unpause(env: Env) {
            unpause(&env);
        }
        pub fn check_paused(env: Env) -> bool {
            is_paused(&env)
        }
        pub fn require_not_paused_call(env: Env) {
            require_not_paused(&env);
        }
    }

    #[test]
    fn test_initially_not_paused() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, TestContract);
        let client = TestContractClient::new(&env, &id);
        assert!(client.check_not_paused());
    }

    #[test]
    fn test_pause_and_unpause() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, TestContract);
        let client = TestContractClient::new(&env, &id);
        client.do_pause();
        assert!(client.check_paused());
        client.do_unpause();
        assert!(!client.check_paused());
    }

    #[test]
    #[should_panic]
    fn test_require_not_paused_panics_when_paused() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, TestContract);
        let client = TestContractClient::new(&env, &id);
        client.do_pause();
        client.require_not_paused_call();
    }
}
