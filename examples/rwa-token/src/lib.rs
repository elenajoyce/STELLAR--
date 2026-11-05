//! # RWA Token
//!
//! Reference RWA (Real-World Asset) token composing allowlist, denylist, and
//! jurisdiction checks in a single transfer path.

#![no_std]

use allowlist_token::AllowlistTokenClient;
use denylist_gate::DenylistGateClient;
use jurisdiction_flag::JurisdictionFlagClient;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Env, String,
    Vec,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    AllowlistToken,
    DenylistGate,
    JurisdictionFlag,
    PermittedJurisdictions,
    Balance(Address),
    Name,
    Symbol,
    Decimals,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum RwaTokenError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    FromNotAllowed = 4,
    ToNotAllowed = 5,
    FromDenied = 6,
    ToDenied = 7,
    FromJurisdictionFail = 8,
    ToJurisdictionFail = 9,
    InsufficientFunds = 10,
}

#[contract]
pub struct RwaToken;

#[contractimpl]
impl RwaToken {
    #[allow(clippy::too_many_arguments)]
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
        decimals: u32,
        allowlist_token: Address,
        denylist_gate: Address,
        jurisdiction_flag: Address,
        permitted_jurisdictions: Vec<String>,
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, RwaTokenError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Name, &name);
        env.storage().instance().set(&DataKey::Symbol, &symbol);
        env.storage().instance().set(&DataKey::Decimals, &decimals);
        env.storage()
            .instance()
            .set(&DataKey::AllowlistToken, &allowlist_token);
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
            .unwrap_or_else(|| panic_with_error!(env, RwaTokenError::NotInitialized));
        admin.require_auth();
        admin
    }

    pub fn mint(env: Env, to: Address, amount: i128) {
        Self::require_admin(&env);
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

        let al_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::AllowlistToken)
            .unwrap_or_else(|| panic_with_error!(&env, RwaTokenError::NotInitialized));
        let dl_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::DenylistGate)
            .unwrap_or_else(|| panic_with_error!(&env, RwaTokenError::NotInitialized));
        let jf_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::JurisdictionFlag)
            .unwrap_or_else(|| panic_with_error!(&env, RwaTokenError::NotInitialized));
        let permitted: Vec<String> = env
            .storage()
            .instance()
            .get(&DataKey::PermittedJurisdictions)
            .unwrap_or_else(|| panic_with_error!(&env, RwaTokenError::NotInitialized));

        let al_client = AllowlistTokenClient::new(&env, &al_addr);
        let dl_client = DenylistGateClient::new(&env, &dl_addr);
        let jf_client = JurisdictionFlagClient::new(&env, &jf_addr);

        if !al_client.is_allowed(&from) {
            panic_with_error!(&env, RwaTokenError::FromNotAllowed);
        }
        if !al_client.is_allowed(&to) {
            panic_with_error!(&env, RwaTokenError::ToNotAllowed);
        }
        if dl_client.is_denied(&from) {
            panic_with_error!(&env, RwaTokenError::FromDenied);
        }
        if dl_client.is_denied(&to) {
            panic_with_error!(&env, RwaTokenError::ToDenied);
        }
        if !jf_client.is_permitted_jurisdiction(&from, &permitted) {
            panic_with_error!(&env, RwaTokenError::FromJurisdictionFail);
        }
        if !jf_client.is_permitted_jurisdiction(&to, &permitted) {
            panic_with_error!(&env, RwaTokenError::ToJurisdictionFail);
        }

        let from_bal: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Balance(from.clone()))
            .unwrap_or(0);
        if from_bal < amount {
            panic_with_error!(&env, RwaTokenError::InsufficientFunds);
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

    pub fn name(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Name)
            .unwrap_or_else(|| panic_with_error!(&env, RwaTokenError::NotInitialized))
    }

    pub fn symbol(env: Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::Symbol)
            .unwrap_or_else(|| panic_with_error!(&env, RwaTokenError::NotInitialized))
    }

    pub fn decimals(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::Decimals)
            .unwrap_or_else(|| panic_with_error!(&env, RwaTokenError::NotInitialized))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use allowlist_token::AllowlistToken;
    use denylist_gate::DenylistGate;
    use jurisdiction_flag::JurisdictionFlag;
    use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

    fn setup() -> (Env, Address, RwaTokenClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        let token_addr = env
            .register_stellar_asset_contract_v2(admin.clone())
            .address();

        let al_id = env.register_contract(None, AllowlistToken);
        AllowlistTokenClient::new(&env, &al_id).initialize(&admin, &token_addr);

        let dl_id = env.register_contract(None, DenylistGate);
        DenylistGateClient::new(&env, &dl_id).initialize(&admin);

        let jf_id = env.register_contract(None, JurisdictionFlag);
        JurisdictionFlagClient::new(&env, &jf_id).initialize(&admin);

        let permitted = vec![
            &env,
            String::from_str(&env, "US"),
            String::from_str(&env, "DE"),
        ];

        let rwa_id = env.register_contract(None, RwaToken);
        let client = RwaTokenClient::new(&env, &rwa_id);
        client.initialize(
            &admin,
            &String::from_str(&env, "RWA Bond Token"),
            &String::from_str(&env, "RWA"),
            &7u32,
            &al_id,
            &dl_id,
            &jf_id,
            &permitted,
        );
        (env, admin, client)
    }

    #[test]
    fn test_initialize() {
        let (env, _admin, client) = setup();
        assert_eq!(client.name(), String::from_str(&env, "RWA Bond Token"));
        assert_eq!(client.symbol(), String::from_str(&env, "RWA"));
        assert_eq!(client.decimals(), 7u32);
    }

    #[test]
    fn test_mint() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);
        client.mint(&user, &1_000_000);
        assert_eq!(client.balance(&user), 1_000_000);
    }
}
