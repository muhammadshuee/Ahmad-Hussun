#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, token, Address, Env};

// We define the keys used to store our data on the blockchain.
#[contracttype]
pub enum DataKey {
    Admin,             // The group leader
    Token,             // The asset being used (e.g., XLM or USDC)
    TotalCollected,    // Running total of collected funds
    HasPaid(Address),  // Maps a member's address to a boolean indicating if they paid
}

#[contract]
pub struct GroupPayContract;

#[contractimpl]
impl GroupPayContract {
    /// Initializes the group pay contract.
    /// `admin`: The group leader organizing the payment.
    /// `token`: The token to be used for payments (e.g., XLM/USDC contract address).
    pub fn initialize(env: Env, admin: Address, token: Address) {
        // Prevent re-initialization
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract is already initialized");
        }
        
        // The admin must authorize this initialization
        admin.require_auth();

        // Save the setup state
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::TotalCollected, &0i128);
    }

    /// A group member pays their share into the smart contract pool.
    pub fn pay_share(env: Env, member: Address, amount: i128) {
        member.require_auth(); // Ensure the member signed the transaction

        if amount <= 0 {
            panic!("Payment amount must be greater than zero");
        }

        // Get the token client to perform the transfer
        let token_id: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_id);

        // Transfer tokens from the member's wallet to this smart contract
        token_client.transfer(&member, &env.current_contract_address(), &amount);

        // Mark this specific member as having paid their share
        env.storage().persistent().set(&DataKey::HasPaid(member.clone()), &true);

        // Update the total collected amount in the pool
        let mut collected: i128 = env.storage().instance().get(&DataKey::TotalCollected).unwrap_or(0);
        collected += amount;
        env.storage().instance().set(&DataKey::TotalCollected, &collected);
    }

    /// Read-only function to check if a specific member has paid.
    pub fn has_paid(env: Env, member: Address) -> bool {
        env.storage().persistent().get(&DataKey::HasPaid(member)).unwrap_or(false)
    }

    /// The group leader withdraws all collected funds to pay for the project.
    pub fn withdraw(env: Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth(); // Only the leader can withdraw

        let token_id: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_id);

        let collected: i128 = env.storage().instance().get(&DataKey::TotalCollected).unwrap_or(0);

        if collected > 0 {
            // Transfer all collected funds from the contract to the leader
            token_client.transfer(&env.current_contract_address(), &admin, &collected);
            
            // Reset the collected balance to 0 after withdrawal
            env.storage().instance().set(&DataKey::TotalCollected, &0i128);
        } else {
            panic!("No funds available to withdraw");
        }
    }
}