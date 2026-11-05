//! # Audit Log
//!
//! An on-chain, append-only audit trail for compliance events emitted by other
//! stellarwarden primitives. Each entry records the event kind, the address
//! involved, and the ledger sequence number.

#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Env, String,
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEntry {
    pub ledger: u32,
    pub address: Address,
    pub event: String,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Count,
    Entry(u64),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum AuditLogError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    EntryNotFound = 4,
}

#[contract]
pub struct AuditLog;

#[contractimpl]
impl AuditLog {
    /// Initialise with an admin address.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic_with_error!(&env, AuditLogError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Count, &0u64);
    }

    fn require_admin(env: &Env) -> Address {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, AuditLogError::NotInitialized));
        admin.require_auth();
        admin
    }

    fn count_inner(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::Count)
            .unwrap_or(0u64)
    }

    /// Append a compliance event to the audit log.
    pub fn record(env: Env, address: Address, event: String) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, AuditLogError::NotInitialized));
        admin.require_auth();

        let count = Self::count_inner(&env);
        let entry = AuditEntry {
            ledger: env.ledger().sequence(),
            address,
            event,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Entry(count), &entry);
        env.storage().instance().set(&DataKey::Count, &(count + 1));
    }

    /// Returns the total number of audit entries.
    pub fn count(env: Env) -> u64 {
        Self::count_inner(&env)
    }

    /// Returns the nth audit entry (0-indexed).
    pub fn get_entry(env: Env, index: u64) -> AuditEntry {
        env.storage()
            .persistent()
            .get::<DataKey, AuditEntry>(&DataKey::Entry(index))
            .unwrap_or_else(|| panic_with_error!(&env, AuditLogError::EntryNotFound))
    }

    /// Returns the current admin.
    pub fn admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, AuditLogError::NotInitialized))
    }

    /// Transfer admin.
    pub fn set_admin(env: Env, new_admin: Address) {
        Self::require_admin(&env);
        env.storage().instance().set(&DataKey::Admin, &new_admin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    fn setup() -> (Env, Address, AuditLogClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register_contract(None, AuditLog);
        let client = AuditLogClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, admin, client)
    }

    #[test]
    fn test_initialize() {
        let (_env, _admin, client) = setup();
        assert_eq!(client.count(), 0);
    }

    #[test]
    fn test_record_and_retrieve() {
        let (env, _admin, client) = setup();
        let user = Address::generate(&env);
        let event = String::from_str(&env, "TRANSFER_BLOCKED");
        client.record(&user, &event);
        assert_eq!(client.count(), 1);
        let entry = client.get_entry(&0);
        assert_eq!(entry.address, user);
        assert_eq!(entry.event, event);
    }

    #[test]
    fn test_multiple_entries() {
        let (env, _admin, client) = setup();
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        client.record(&user1, &String::from_str(&env, "DENIED"));
        client.record(&user2, &String::from_str(&env, "ALLOWLISTED"));
        assert_eq!(client.count(), 2);
        assert_eq!(client.get_entry(&0).address, user1);
        assert_eq!(client.get_entry(&1).address, user2);
    }

    #[test]
    #[should_panic]
    fn test_get_entry_out_of_bounds_panics() {
        let (_env, _admin, client) = setup();
        client.get_entry(&99);
    }
}
