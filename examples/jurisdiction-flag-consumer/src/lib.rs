//! # Jurisdiction Flag Consumer
//!
//! Token enforcing jurisdiction-based transfer restrictions.

#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contracttype, panic_with_error, Address,
    Env, String, Vec,
};

// ── Cross-contract client interfaces ───────────────────────────────────────
// Only the methods this contract calls are declared here as client traits, so
// jurisdiction-flag is not linked into this contract's WASM. It is deployed
// separately and invoked by address at runtime.
#[contractclient(name = "JurisdictionFlagClient")]
pub trait JurisdictionFlagTrait {
    fn is_permitted_jurisdiction(env: Env, address: Address, permitted_codes: Vec<String>) -> bool;
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    JurisdictionFlag,
    PermittedJurisdictions,
    Balance(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum JfcError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    InsufficientFunds = 3,
    JurisdictionDenied = 4,
}

#[contract]
pub struct JurisdictionFlagConsumer;

#[contractimpl]
impl JurisdictionFlagConsumer {
    pub fn initialize(
        env: Env,
        admin: Address,
        jurisdiction_flag: Address,
        permitted_jurisdictions: Vec<String>,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, JfcError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
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
            .unwrap_or_else(|| panic_with_error!(&env, JfcError::NotInitialized));
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

        let jf_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::JurisdictionFlag)
            .unwrap_or_else(|| panic_with_error!(&env, JfcError::NotInitialized));
        let permitted: Vec<String> = env
            .storage()
            .instance()
            .get(&DataKey::PermittedJurisdictions)
            .unwrap_or_else(|| panic_with_error!(&env, JfcError::NotInitialized));
        let jf = JurisdictionFlagClient::new(&env, &jf_addr);

        if !jf.is_permitted_jurisdiction(&from, &permitted) {
            panic_with_error!(&env, JfcError::JurisdictionDenied);
        }
        if !jf.is_permitted_jurisdiction(&to, &permitted) {
            panic_with_error!(&env, JfcError::JurisdictionDenied);
        }

        let fb: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        if fb < amount {
            panic_with_error!(&env, JfcError::InsufficientFunds);
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
    use jurisdiction_flag::JurisdictionFlag;
    use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

    #[test]
    fn test_initialize_and_mint() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        let jf_id = env.register_contract(None, JurisdictionFlag);
        jurisdiction_flag::JurisdictionFlagClient::new(&env, &jf_id).initialize(&admin);

        let permitted = vec![
            &env,
            String::from_str(&env, "US"),
            String::from_str(&env, "GB"),
        ];
        let cid = env.register_contract(None, JurisdictionFlagConsumer);
        let client = JurisdictionFlagConsumerClient::new(&env, &cid);
        client.initialize(&admin, &jf_id, &permitted);

        let user = Address::generate(&env);
        client.mint(&user, &750);
        assert_eq!(client.balance(&user), 750);
    }
}
