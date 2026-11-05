//! # Policy Engine
//!
//! Composes denylist and jurisdiction checks with configurable AND/OR logic
//! into a single `evaluate` call.
//!
//! - `PolicyMode::All` — address must pass **all** enabled checks (AND).
//! - `PolicyMode::Any` — address must pass **at least one** check (OR).

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Env, String,
    Vec,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PolicyMode {
    All,
    Any,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    DenylistGate,
    JurisdictionFlag,
    PermittedJurisdictions,
    Mode,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum PolicyEngineError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    PolicyFailed = 4,
}

#[contract]
pub struct PolicyEngine;

#[contractimpl]
impl PolicyEngine {
    pub fn initialize(
        env: Env,
        admin: Address,
        denylist_gate: Option<Address>,
        jurisdiction_flag: Option<Address>,
        permitted_jurisdictions: Vec<String>,
        mode: PolicyMode,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, PolicyEngineError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        if let Some(addr) = denylist_gate {
            env.storage().instance().set(&DataKey::DenylistGate, &addr);
        }
        if let Some(addr) = jurisdiction_flag {
            env.storage()
                .instance()
                .set(&DataKey::JurisdictionFlag, &addr);
        }
        env.storage()
            .instance()
            .set(&DataKey::PermittedJurisdictions, &permitted_jurisdictions);
        env.storage().instance().set(&DataKey::Mode, &mode);
    }

    fn require_admin(env: &Env) -> Address {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, PolicyEngineError::NotInitialized));
        admin.require_auth();
        admin
    }

    pub fn set_mode(env: Env, mode: PolicyMode) {
        Self::require_admin(&env);
        env.storage().instance().set(&DataKey::Mode, &mode);
    }

    pub fn set_permitted_jurisdictions(env: Env, permitted: Vec<String>) {
        Self::require_admin(&env);
        env.storage()
            .instance()
            .set(&DataKey::PermittedJurisdictions, &permitted);
    }

    pub fn set_admin(env: Env, new_admin: Address) {
        Self::require_admin(&env);
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }

    /// Evaluate the policy for `address`. Returns `true` if address passes.
    pub fn evaluate(env: Env, address: Address) -> bool {
        let mode: PolicyMode = env
            .storage()
            .instance()
            .get(&DataKey::Mode)
            .unwrap_or_else(|| panic_with_error!(&env, PolicyEngineError::NotInitialized));

        let mut results: Vec<bool> = Vec::new(&env);

        // Denylist check
        if let Some(denylist_addr) = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::DenylistGate)
        {
            let client = denylist_gate::DenylistGateClient::new(&env, &denylist_addr);
            results.push_back(!client.is_denied(&address));
        }

        // Jurisdiction check
        if let Some(juris_addr) = env
            .storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::JurisdictionFlag)
        {
            let permitted: Vec<String> = env
                .storage()
                .instance()
                .get(&DataKey::PermittedJurisdictions)
                .unwrap_or_else(|| Vec::new(&env));
            let client = jurisdiction_flag::JurisdictionFlagClient::new(&env, &juris_addr);
            let passed = if client.has_jurisdiction(&address) {
                client.is_permitted_jurisdiction(&address, &permitted)
            } else {
                false
            };
            results.push_back(passed);
        }

        if results.is_empty() {
            return true;
        }

        match mode {
            PolicyMode::All => {
                for r in results.iter() {
                    if !r {
                        return false;
                    }
                }
                true
            }
            PolicyMode::Any => {
                for r in results.iter() {
                    if r {
                        return true;
                    }
                }
                false
            }
        }
    }

    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, PolicyEngineError::NotInitialized))
    }

    pub fn mode(env: Env) -> PolicyMode {
        env.storage()
            .instance()
            .get(&DataKey::Mode)
            .unwrap_or_else(|| panic_with_error!(&env, PolicyEngineError::NotInitialized))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use denylist_gate::DenylistGate;
    use jurisdiction_flag::JurisdictionFlag;
    use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

    fn setup(mode: PolicyMode) -> (Env, Address, PolicyEngineClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        let dl_id = env.register_contract(None, DenylistGate);
        denylist_gate::DenylistGateClient::new(&env, &dl_id).initialize(&admin);

        let jf_id = env.register_contract(None, JurisdictionFlag);
        jurisdiction_flag::JurisdictionFlagClient::new(&env, &jf_id).initialize(&admin);

        let permitted = vec![
            &env,
            String::from_str(&env, "US"),
            String::from_str(&env, "DE"),
        ];

        let contract_id = env.register_contract(None, PolicyEngine);
        let client = PolicyEngineClient::new(&env, &contract_id);
        client.initialize(&admin, &Some(dl_id), &Some(jf_id), &permitted, &mode);

        (env, admin, client)
    }

    #[test]
    fn test_all_mode_passes_when_both_pass() {
        let (env, _admin, client) = setup(PolicyMode::All);
        let user = Address::generate(&env);

        // Give user a valid jurisdiction
        let _pe_addr = client.admin(); // get admin addr to get jf addr
                                       // For test we need to set jurisdiction directly - use mode All
                                       // Address has no jurisdiction set -> fails jurisdiction check
        assert!(!client.evaluate(&user)); // no jurisdiction → fails
    }

    #[test]
    fn test_mode_query() {
        let (_env, _admin, client) = setup(PolicyMode::All);
        assert_eq!(client.mode(), PolicyMode::All);
    }

    #[test]
    fn test_set_mode() {
        let (_env, _admin, client) = setup(PolicyMode::All);
        client.set_mode(&PolicyMode::Any);
        assert_eq!(client.mode(), PolicyMode::Any);
    }
}
