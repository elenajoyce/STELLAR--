//! # Multisig-Governed Allowlist
//!
//! Demonstrates using `multisig-admin` as the admin of `allowlist-token`.
//! Every allowlist change requires M-of-N signer approval.

#![no_std]

use soroban_sdk::{contract, contractimpl, Env};

/// Documentation + test harness — no deployable contract needed.
#[contract]
pub struct MultisigGovernedAllowlist;

#[contractimpl]
impl MultisigGovernedAllowlist {
    pub fn noop(_env: Env) {}
}

#[cfg(test)]
mod tests {
    use allowlist_token::{AllowlistToken, AllowlistTokenClient};
    use multisig_admin::{MultisigAdmin, MultisigAdminClient};
    use soroban_sdk::{testutils::Address as _, vec, Address, Env};

    #[test]
    fn test_multisig_governed_allowlist_wiring() {
        let env = Env::default();
        env.mock_all_auths();

        let s1 = Address::generate(&env);
        let s2 = Address::generate(&env);
        let s3 = Address::generate(&env);
        let signers = vec![&env, s1.clone(), s2.clone(), s3.clone()];

        let ms_id = env.register_contract(None, MultisigAdmin);
        let ms_client = MultisigAdminClient::new(&env, &ms_id);
        ms_client.initialize(&signers, &2u32);

        let token_addr = env.register_stellar_asset_contract_v2(s1.clone()).address();

        let al_id = env.register_contract(None, AllowlistToken);
        let al_client = AllowlistTokenClient::new(&env, &al_id);
        al_client.initialize(&ms_id, &token_addr);

        assert_eq!(al_client.admin(), ms_id);

        let user = Address::generate(&env);
        al_client.allowlist(&user);
        assert!(al_client.is_allowed(&user));

        al_client.denylist(&user);
        assert!(!al_client.is_allowed(&user));
    }

    #[test]
    fn test_multisig_threshold_and_execute() {
        let env = Env::default();
        env.mock_all_auths();

        let s1 = Address::generate(&env);
        let s2 = Address::generate(&env);
        let signers = vec![&env, s1.clone(), s2.clone()];

        let ms_id = env.register_contract(None, MultisigAdmin);
        let ms_client = MultisigAdminClient::new(&env, &ms_id);
        ms_client.initialize(&signers, &2u32);

        assert_eq!(ms_client.threshold(), 2);
        ms_client.approve(&s1, &0);
        ms_client.approve(&s2, &0);
        let new_nonce = ms_client.execute();
        assert_eq!(new_nonce, 1);
    }
}
