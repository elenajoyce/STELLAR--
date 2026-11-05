//! # Denylist Gate SEP-41
//!
//! SEP-41 anchor token that gates every transfer through the denylist.

#![no_std]

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contracttype, panic_with_error, token,
    Address, Env,
};

// ── Cross-contract client interfaces ───────────────────────────────────────
// Only the methods this contract calls are declared here as client traits, so
// denylist-gate is not linked into this contract's WASM. It is deployed
// separately and invoked by address at runtime.
#[contractclient(name = "DenylistGateClient")]
pub trait DenylistGateTrait {
    fn check(env: Env, address: Address);
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    DenylistGate,
    UnderlyingToken,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Sep41DenylistError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
}

#[contract]
pub struct DenylistGateSep41;

#[contractimpl]
impl DenylistGateSep41 {
    pub fn initialize(env: Env, admin: Address, denylist_gate: Address, underlying_token: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, Sep41DenylistError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::DenylistGate, &denylist_gate);
        env.storage()
            .instance()
            .set(&DataKey::UnderlyingToken, &underlying_token);
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        Self::check_denylist(&env, &from);
        Self::check_denylist(&env, &to);
        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::UnderlyingToken)
            .unwrap_or_else(|| panic_with_error!(&env, Sep41DenylistError::NotInitialized));
        token::Client::new(&env, &token_addr).transfer(&from, &to, &amount);
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();
        Self::check_denylist(&env, &from);
        Self::check_denylist(&env, &to);
        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::UnderlyingToken)
            .unwrap_or_else(|| panic_with_error!(&env, Sep41DenylistError::NotInitialized));
        token::Client::new(&env, &token_addr).transfer_from(&spender, &from, &to, &amount);
    }

    fn check_denylist(env: &Env, address: &Address) {
        let dl_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::DenylistGate)
            .unwrap_or_else(|| panic_with_error!(env, Sep41DenylistError::NotInitialized));
        DenylistGateClient::new(env, &dl_addr).check(address);
    }

    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, Sep41DenylistError::NotInitialized))
    }

    pub fn denylist_gate(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::DenylistGate)
            .unwrap_or_else(|| panic_with_error!(&env, Sep41DenylistError::NotInitialized))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use denylist_gate::DenylistGate;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        let dl_id = env.register_contract(None, DenylistGate);
        denylist_gate::DenylistGateClient::new(&env, &dl_id).initialize(&admin);

        let token_addr = env
            .register_stellar_asset_contract_v2(admin.clone())
            .address();

        let cid = env.register_contract(None, DenylistGateSep41);
        let client = DenylistGateSep41Client::new(&env, &cid);
        client.initialize(&admin, &dl_id, &token_addr);

        assert_eq!(client.admin(), admin);
        assert_eq!(client.denylist_gate(), dl_id);
    }
}
