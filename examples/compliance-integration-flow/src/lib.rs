//! # Compliance Integration Flow
//!
//! Combines circuit breaker, jurisdiction checks, and audit logging.

#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contracttype, panic_with_error, Address,
    Env, String, Vec,
};

// ── Cross-contract client interfaces ───────────────────────────────────────
// Only the methods this contract calls are declared here as client traits, so
// circuit-breaker, jurisdiction-flag, and audit-log are not linked into this
// contract's WASM. They are deployed separately and invoked by address.
#[contractclient(name = "CircuitBreakerClient")]
pub trait CircuitBreakerTrait {
    fn check(env: Env);
}

#[contractclient(name = "JurisdictionFlagClient")]
pub trait JurisdictionFlagTrait {
    fn is_permitted_jurisdiction(env: Env, address: Address, permitted_codes: Vec<String>) -> bool;
}

#[contractclient(name = "AuditLogClient")]
pub trait AuditLogTrait {
    fn record(env: Env, address: Address, event: String);
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    CircuitBreaker,
    JurisdictionFlag,
    AuditLog,
    PermittedJurisdictions,
    Balance(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum CifError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    InsufficientFunds = 3,
}

#[contract]
pub struct ComplianceIntegrationFlow;

#[contractimpl]
impl ComplianceIntegrationFlow {
    pub fn initialize(
        env: Env,
        admin: Address,
        circuit_breaker: Address,
        jurisdiction_flag: Address,
        audit_log: Address,
        permitted_jurisdictions: Vec<String>,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, CifError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::CircuitBreaker, &circuit_breaker);
        env.storage()
            .instance()
            .set(&DataKey::JurisdictionFlag, &jurisdiction_flag);
        env.storage().instance().set(&DataKey::AuditLog, &audit_log);
        env.storage()
            .instance()
            .set(&DataKey::PermittedJurisdictions, &permitted_jurisdictions);
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, CifError::NotInitialized));
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
            .unwrap_or_else(|| panic_with_error!(&env, CifError::NotInitialized));
        CircuitBreakerClient::new(&env, &cb_addr).check();

        let jf_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::JurisdictionFlag)
            .unwrap_or_else(|| panic_with_error!(&env, CifError::NotInitialized));
        let permitted: Vec<String> = env
            .storage()
            .instance()
            .get(&DataKey::PermittedJurisdictions)
            .unwrap_or_else(|| panic_with_error!(&env, CifError::NotInitialized));
        let jf = JurisdictionFlagClient::new(&env, &jf_addr);
        jf.is_permitted_jurisdiction(&from, &permitted);
        jf.is_permitted_jurisdiction(&to, &permitted);

        let fb: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        if fb < amount {
            panic_with_error!(&env, CifError::InsufficientFunds);
        }
        let tb: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(to.clone()))
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &(fb - amount));
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &(tb + amount));

        let al_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::AuditLog)
            .unwrap_or_else(|| panic_with_error!(&env, CifError::NotInitialized));
        AuditLogClient::new(&env, &al_addr)
            .record(&from, &String::from_str(&env, "TRANSFER_COMPLETED"));
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
    use audit_log::AuditLog;
    use circuit_breaker::CircuitBreaker;
    use jurisdiction_flag::JurisdictionFlag;
    use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

    #[test]
    fn test_initialize_and_mint() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        let cb_id = env.register_contract(None, CircuitBreaker);
        circuit_breaker::CircuitBreakerClient::new(&env, &cb_id).initialize(&admin);
        let jf_id = env.register_contract(None, JurisdictionFlag);
        jurisdiction_flag::JurisdictionFlagClient::new(&env, &jf_id).initialize(&admin);
        let al_id = env.register_contract(None, AuditLog);
        audit_log::AuditLogClient::new(&env, &al_id).initialize(&admin);

        let permitted = vec![&env, String::from_str(&env, "US")];
        let cid = env.register_contract(None, ComplianceIntegrationFlow);
        let client = ComplianceIntegrationFlowClient::new(&env, &cid);
        client.initialize(&admin, &cb_id, &jf_id, &al_id, &permitted);

        let user = Address::generate(&env);
        client.mint(&user, &1000);
        assert_eq!(client.balance(&user), 1000);
    }
}
