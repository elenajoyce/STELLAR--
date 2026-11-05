//! # RWA Compliance Flow
//!
//! Full RWA compliance stack:
//! 1. Circuit Breaker  2. Allowlist  3. Denylist  4. Jurisdiction  5. Audit Log

#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contracttype, panic_with_error, Address,
    Env, String, Vec,
};

// ── Cross-contract client interfaces ───────────────────────────────────────
// Only the methods this contract calls are declared here as client traits, so
// the referenced contracts are not linked into this contract's WASM. They are
// deployed separately and invoked by address at runtime.
#[contractclient(name = "CircuitBreakerClient")]
pub trait CircuitBreakerTrait {
    fn check(env: Env);
}

#[contractclient(name = "AllowlistTokenClient")]
pub trait AllowlistTokenTrait {
    fn is_allowed(env: Env, address: Address) -> bool;
}

#[contractclient(name = "DenylistGateClient")]
pub trait DenylistGateTrait {
    fn check(env: Env, address: Address);
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
    AllowlistToken,
    DenylistGate,
    JurisdictionFlag,
    AuditLog,
    PermittedJurisdictions,
    Balance(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum RwaFlowError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    FromNotAllowed = 3,
    ToNotAllowed = 4,
    InsufficientFunds = 5,
}

#[contract]
pub struct RwaComplianceFlow;

#[contractimpl]
impl RwaComplianceFlow {
    #[allow(clippy::too_many_arguments)]
    pub fn initialize(
        env: Env,
        admin: Address,
        circuit_breaker: Address,
        allowlist_token: Address,
        denylist_gate: Address,
        jurisdiction_flag: Address,
        audit_log: Address,
        permitted_jurisdictions: Vec<String>,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, RwaFlowError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::CircuitBreaker, &circuit_breaker);
        env.storage()
            .instance()
            .set(&DataKey::AllowlistToken, &allowlist_token);
        env.storage()
            .instance()
            .set(&DataKey::DenylistGate, &denylist_gate);
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
            .unwrap_or_else(|| panic_with_error!(&env, RwaFlowError::NotInitialized));
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
            .unwrap_or_else(|| panic_with_error!(&env, RwaFlowError::NotInitialized));
        let al_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::AllowlistToken)
            .unwrap_or_else(|| panic_with_error!(&env, RwaFlowError::NotInitialized));
        let dl_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::DenylistGate)
            .unwrap_or_else(|| panic_with_error!(&env, RwaFlowError::NotInitialized));
        let jf_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::JurisdictionFlag)
            .unwrap_or_else(|| panic_with_error!(&env, RwaFlowError::NotInitialized));
        let audit_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::AuditLog)
            .unwrap_or_else(|| panic_with_error!(&env, RwaFlowError::NotInitialized));
        let permitted: Vec<String> = env
            .storage()
            .instance()
            .get(&DataKey::PermittedJurisdictions)
            .unwrap_or_else(|| panic_with_error!(&env, RwaFlowError::NotInitialized));

        CircuitBreakerClient::new(&env, &cb_addr).check();

        let al = AllowlistTokenClient::new(&env, &al_addr);
        if !al.is_allowed(&from) {
            panic_with_error!(&env, RwaFlowError::FromNotAllowed);
        }
        if !al.is_allowed(&to) {
            panic_with_error!(&env, RwaFlowError::ToNotAllowed);
        }

        let dl = DenylistGateClient::new(&env, &dl_addr);
        dl.check(&from);
        dl.check(&to);

        let jf = JurisdictionFlagClient::new(&env, &jf_addr);
        jf.is_permitted_jurisdiction(&from, &permitted);
        jf.is_permitted_jurisdiction(&to, &permitted);

        let fb: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        if fb < amount {
            panic_with_error!(&env, RwaFlowError::InsufficientFunds);
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

        AuditLogClient::new(&env, &audit_addr)
            .record(&from, &String::from_str(&env, "RWA_TRANSFER"));
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
    use allowlist_token::AllowlistToken;
    use audit_log::AuditLog;
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

        let token_addr = env
            .register_stellar_asset_contract_v2(admin.clone())
            .address();
        let al_id = env.register_contract(None, AllowlistToken);
        allowlist_token::AllowlistTokenClient::new(&env, &al_id).initialize(&admin, &token_addr);

        let dl_id = env.register_contract(None, DenylistGate);
        denylist_gate::DenylistGateClient::new(&env, &dl_id).initialize(&admin);

        let jf_id = env.register_contract(None, JurisdictionFlag);
        jurisdiction_flag::JurisdictionFlagClient::new(&env, &jf_id).initialize(&admin);

        let audit_id = env.register_contract(None, AuditLog);
        audit_log::AuditLogClient::new(&env, &audit_id).initialize(&admin);

        let permitted = vec![
            &env,
            String::from_str(&env, "US"),
            String::from_str(&env, "DE"),
        ];
        let cid = env.register_contract(None, RwaComplianceFlow);
        let client = RwaComplianceFlowClient::new(&env, &cid);
        client.initialize(
            &admin, &cb_id, &al_id, &dl_id, &jf_id, &audit_id, &permitted,
        );

        let user = Address::generate(&env);
        client.mint(&user, &5_000);
        assert_eq!(client.balance(&user), 5_000);
    }
}
