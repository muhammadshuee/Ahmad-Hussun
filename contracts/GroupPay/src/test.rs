#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};
use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::token::StellarAssetClient;

/// Helper function to create a mock token for testing
fn create_token_contract<'a>(env: &Env, admin: &Address) -> (Address, TokenClient<'a>, StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract(admin.clone());
    (
        contract_address.clone(),
        TokenClient::new(env, &contract_address),
        StellarAssetClient::new(env, &contract_address),
    )
}

#[test]
fn test_happy_path_payment_and_withdrawal() {
    let env = Env::default();
    env.mock_all_auths(); // Mock signatures for testing

    let admin = Address::generate(&env);
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);

    // Setup mock token
    let (token_id, token_client, token_admin_client) = create_token_contract(&env, &admin);
    token_admin_client.mint(&member1, &500);
    token_admin_client.mint(&member2, &500);

    // Register our GroupPay contract
    let contract_id = env.register_contract(None, GroupPayContract);
    let client = GroupPayContractClient::new(&env, &contract_id);

    // 1. Leader initializes the contract
    client.initialize(&admin, &token_id);

    // 2. Members pay their share
    client.pay_share(&member1, &100);
    client.pay_share(&member2, &100);

    // 3. Verify they are marked as paid
    assert_eq!(client.has_paid(&member1), true);
    assert_eq!(client.has_paid(&member2), true);

    // 4. Leader withdraws funds
    client.withdraw();

    // 5. Verify leader received the 200 total tokens
    assert_eq!(token_client.balance(&admin), 200);
}

#[test]
#[should_panic(expected = "Contract is already initialized")]
fn test_edge_case_double_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let token_id = Address::generate(&env);

    let contract_id = env.register_contract(None, GroupPayContract);
    let client = GroupPayContractClient::new(&env, &contract_id);

    // First init succeeds
    client.initialize(&admin, &token_id);
    
    // Second init should fail and panic
    client.initialize(&admin, &token_id); 
}

#[test]
fn test_state_verification_unpaid_member() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let token_id = Address::generate(&env);
    let lazy_member = Address::generate(&env); // A member who hasn't paid yet

    let contract_id = env.register_contract(None, GroupPayContract);
    let client = GroupPayContractClient::new(&env, &contract_id);

    client.initialize(&admin, &token_id);

    // Verify state: Contract should correctly report false for a user who hasn't paid
    assert_eq!(client.has_paid(&lazy_member), false);
}