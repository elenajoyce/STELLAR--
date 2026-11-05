//! # Compliance Aggregator
//!
//! Batches denylist-gate and jurisdiction-flag checks into a single
//! cross-contract call. Returns an aggregated pass/fail result along with a
//! per-check breakdown so callers can see exactly which gate blocked a transfer.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Env, String,
    Vec,
};

// ── Result types ──────────────────────────────────────────────────────────────

/// Result of a single compliance check.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckResult {
    pub check_name: String,
    pub passed: bool,
}

/// Aggregated compliance evaluation result.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AggregateResult {
    /// `true` only if ALL individual checks passed.
    pub passed: bool,
    /// Per-check breakdown.
    pub checks: Vec<CheckResult>,
}

// ── Storage keys ──────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    DenylistGate,
    JurisdictionFlag,
    PermittedJurisdictions,
}

// ── Errors ────────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum AggregatorError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct ComplianceAggregator;

#[contractimpl]
impl ComplianceAggregator {
    pub fn initialize(
        env: Env,
        admin: Address,
        denylist_gate: Address,
        jurisdiction_flag: Address,
        permitted_jurisdictions: Vec<String>,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, AggregatorError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
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

    fn require_admin(env: &Env) -> Address {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, AggregatorError::NotInitialized));
        admin.require_auth();
        admin
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

    /// Evaluate all compliance checks for `address`.
    /// Returns an [`AggregateResult`] with per-check detail.
    pub fn evaluate(env: Env, address: Address) -> AggregateResult {
        let denylist_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::DenylistGate)
            .unwrap_or_else(|| panic_with_error!(&env, AggregatorError::NotInitialized));

        let juris_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::JurisdictionFlag)
            .unwrap_or_else(|| panic_with_error!(&env, AggregatorError::NotInitialized));

        let permitted: Vec<String> = env
            .storage()
            .instance()
            .get(&DataKey::PermittedJurisdictions)
            .unwrap_or_else(|| panic_with_error!(&env, AggregatorError::NotInitialized));

        let mut checks = Vec::new(&env);
        let mut all_passed = true;

        // Check 1: denylist
        let denylist_client = denylist_gate::DenylistGateClient::new(&env, &denylist_addr);
        let denylist_passed = !denylist_client.is_denied(&address);
        if !denylist_passed {
            all_passed = false;
        }
        checks.push_back(CheckResult {
            check_name: String::from_str(&env, "denylist"),
            passed: denylist_passed,
        });

        // Check 2: jurisdiction
        let juris_client = jurisdiction_flag::JurisdictionFlagClient::new(&env, &juris_addr);
        let juris_passed = if juris_client.has_jurisdiction(&address) {
            juris_client.is_permitted_jurisdiction(&address, &permitted)
        } else {
            false
        };
        if !juris_passed {
            all_passed = false;
        }
        checks.push_back(CheckResult {
            check_name: String::from_str(&env, "jurisdiction"),
            passed: juris_passed,
        });

        AggregateResult {
            passed: all_passed,
            checks,
        }
    }

    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, AggregatorError::NotInitialized))
    }

    pub fn denylist_gate(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::DenylistGate)
            .unwrap_or_else(|| panic_with_error!(&env, AggregatorError::NotInitialized))
    }

    pub fn jurisdiction_flag(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::JurisdictionFlag)
            .unwrap_or_else(|| panic_with_error!(&env, AggregatorError::NotInitialized))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use denylist_gate::DenylistGate;
    use jurisdiction_flag::JurisdictionFlag;
    use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

    fn setup() -> (Env, Address, ComplianceAggregatorClient<'static>) {
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

        let contract_id = env.register_contract(None, ComplianceAggregator);
        let client = ComplianceAggregatorClient::new(&env, &contract_id);
        client.initialize(&admin, &dl_id, &jf_id, &permitted);

        (env, admin, client)
    }

    #[test]
    fn test_evaluate_passes_clean_address() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);

        // Set jurisdiction for user
        let juris_addr = client.jurisdiction_flag();
        jurisdiction_flag::JurisdictionFlagClient::new(&env, &juris_addr)
            .set_jurisdiction(&user, &String::from_str(&env, "US"));

        let result = client.evaluate(&user);
        assert!(result.passed);
        assert_eq!(result.checks.len(), 2);
    }

    #[test]
    fn test_evaluate_fails_denied_address() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);

        // Deny user
        let dl_addr = client.denylist_gate();
        denylist_gate::DenylistGateClient::new(&env, &dl_addr).deny(&user);

        // Set valid jurisdiction
        let juris_addr = client.jurisdiction_flag();
        jurisdiction_flag::JurisdictionFlagClient::new(&env, &juris_addr)
            .set_jurisdiction(&user, &String::from_str(&env, "US"));

        let result = client.evaluate(&user);
        assert!(!result.passed);
    }

    #[test]
    fn test_evaluate_fails_bad_jurisdiction() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);

        let juris_addr = client.jurisdiction_flag();
        jurisdiction_flag::JurisdictionFlagClient::new(&env, &juris_addr)
            .set_jurisdiction(&user, &String::from_str(&env, "CN"));

        let result = client.evaluate(&user);
        assert!(!result.passed);
    }
}
