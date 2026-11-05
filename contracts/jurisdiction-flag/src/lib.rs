//! # Jurisdiction Flag
//!
//! Attaches an issuer-controlled jurisdiction code (e.g. ISO 3166-1 alpha-2) to
//! an address. Other contracts call [`JurisdictionFlagClient::is_permitted_jurisdiction`]
//! to check whether an address's jurisdiction is in a permitted list.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Env, String,
    Vec,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Jurisdiction(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum JurisdictionError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    NoJurisdictionSet = 4,
    JurisdictionDenied = 5,
    ContractPaused = 6,
}

#[contract]
pub struct JurisdictionFlag;

#[contractimpl]
impl JurisdictionFlag {
    /// Initialise with an admin/issuer address.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, JurisdictionError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    fn require_admin(env: &Env) -> Address {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, JurisdictionError::NotInitialized));
        admin.require_auth();
        admin
    }

    /// Set the jurisdiction code for an address.
    pub fn set_jurisdiction(env: Env, address: Address, code: String) {
        Self::require_admin(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Jurisdiction(address), &code);
    }

    /// Remove the jurisdiction flag from an address.
    pub fn remove_jurisdiction(env: Env, address: Address) {
        Self::require_admin(&env);
        env.storage()
            .persistent()
            .remove(&DataKey::Jurisdiction(address));
    }

    /// Pause the contract.
    pub fn pause(env: Env) {
        Self::require_admin(&env);
        pausable::pause(&env);
    }

    /// Unpause the contract.
    pub fn unpause(env: Env) {
        Self::require_admin(&env);
        pausable::unpause(&env);
    }

    /// Transfer admin rights.
    pub fn set_admin(env: Env, new_admin: Address) {
        Self::require_admin(&env);
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }

    /// Returns the current admin.
    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, JurisdictionError::NotInitialized))
    }

    /// Returns the jurisdiction code for an address.
    pub fn get_jurisdiction(env: Env, address: Address) -> String {
        env.storage()
            .persistent()
            .get::<DataKey, String>(&DataKey::Jurisdiction(address))
            .unwrap_or_else(|| panic_with_error!(&env, JurisdictionError::NoJurisdictionSet))
    }

    /// Returns `true` if the address has a jurisdiction set.
    pub fn has_jurisdiction(env: Env, address: Address) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Jurisdiction(address))
    }

    /// Returns `true` if the address's jurisdiction code is in `permitted_codes`.
    pub fn is_permitted_jurisdiction(
        env: Env,
        address: Address,
        permitted_codes: Vec<String>,
    ) -> bool {
        if pausable::is_paused(&env) {
            panic_with_error!(&env, JurisdictionError::ContractPaused);
        }
        let code: String = env
            .storage()
            .persistent()
            .get::<DataKey, String>(&DataKey::Jurisdiction(address))
            .unwrap_or_else(|| panic_with_error!(&env, JurisdictionError::NoJurisdictionSet));

        for permitted in permitted_codes.iter() {
            if permitted == code {
                return true;
            }
        }
        false
    }

    /// Returns `true` if the contract is paused.
    pub fn paused(env: Env) -> bool {
        pausable::is_paused(&env)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, vec, Address, Env, String};

    fn setup() -> (Env, Address, JurisdictionFlagClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, JurisdictionFlag);
        let client = JurisdictionFlagClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, admin, client)
    }

    #[test]
    fn test_set_and_get_jurisdiction() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);
        let code = String::from_str(&env, "US");
        client.set_jurisdiction(&user, &code);
        assert_eq!(client.get_jurisdiction(&user), code);
    }

    #[test]
    fn test_is_permitted_jurisdiction_true() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);
        client.set_jurisdiction(&user, &String::from_str(&env, "DE"));
        let permitted = vec![
            &env,
            String::from_str(&env, "US"),
            String::from_str(&env, "DE"),
            String::from_str(&env, "GB"),
        ];
        assert!(client.is_permitted_jurisdiction(&user, &permitted));
    }

    #[test]
    fn test_is_permitted_jurisdiction_false() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);
        client.set_jurisdiction(&user, &String::from_str(&env, "CN"));
        let permitted = vec![
            &env,
            String::from_str(&env, "US"),
            String::from_str(&env, "DE"),
        ];
        assert!(!client.is_permitted_jurisdiction(&user, &permitted));
    }

    #[test]
    fn test_remove_jurisdiction() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);
        client.set_jurisdiction(&user, &String::from_str(&env, "US"));
        client.remove_jurisdiction(&user);
        assert!(!client.has_jurisdiction(&user));
    }

    #[test]
    #[should_panic]
    fn test_paused_is_permitted_panics() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);
        client.set_jurisdiction(&user, &String::from_str(&env, "US"));
        client.pause();
        let permitted = vec![&env, String::from_str(&env, "US")];
        client.is_permitted_jurisdiction(&user, &permitted);
    }
}
