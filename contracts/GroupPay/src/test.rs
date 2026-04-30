#[cfg(test)]
mod tests {
    use crate::{GroupPayContract, GroupPayContractClient, GroupPayError};
    use soroban_sdk::{
        testutils::{Address as _, Ledger, LedgerInfo},
        token::{Client as TokenClient, StellarAssetClient},
        Address, Env, Symbol,
    };

    // ── Shared test helper ────────────────────────────────────────────────────
    /// Spins up an Env, registers the contract, mints test tokens to two members,
    /// and sets the ledger timestamp to a known value (1_000_000).
    fn setup_env() -> (
        Env,
        Address, // contract_id
        Address, // token_id
        Address, // organizer
        Address, // recipient
        Address, // member1
        Address, // member2
    ) {
        let env = Env::default();
        env.mock_all_auths();

        // Register the GroupPay contract
        let contract_id = env.register_contract(None, GroupPayContract);

        // Create a Stellar Asset (simulates XLM / USDC)
        let token_admin = Address::generate(&env);
        let token_id = env
            .register_stellar_asset_contract_v2(token_admin.clone())
            .address();
        let asset_client = StellarAssetClient::new(&env, &token_id);

        let organizer = Address::generate(&env);
        let recipient = Address::generate(&env);
        let member1   = Address::generate(&env);
        let member2   = Address::generate(&env);

        // Mint tokens — member1 gets 600, member2 gets 400 (total = target of 1000)
        asset_client.mint(&member1, &600_i128);
        asset_client.mint(&member2, &400_i128);

        // Pin ledger timestamp so deadline checks are deterministic
        env.ledger().set(LedgerInfo {
            timestamp: 1_000_000,
            ..env.ledger().get()
        });

        (env, contract_id, token_id, organizer, recipient, member1, member2)
    }

    // ════════════════════════════════════════════════════════════════════════
    // TEST 1 — Happy Path
    // Full end-to-end: create group → two members contribute → organizer
    // releases payment → recipient receives the full pool.
    // ════════════════════════════════════════════════════════════════════════
    #[test]
    fn test_happy_path_create_contribute_release() {
        let (env, contract_id, token_id, organizer, recipient, member1, member2) =
            setup_env();

        let client = GroupPayContractClient::new(&env, &contract_id);

        // ── 1. Create group with target = 1000, deadline = 2_000_000 ─────────
        client.create_group(
            &organizer,
            &recipient,
            &token_id,
            &1000_i128,
            &2_000_000_u64,
            &Symbol::new(&env, "OfficeRent"),
        );

        // ── 2. member1 contributes 600 ────────────────────────────────────────
        let total_after_1 = client.contribute(&member1, &600_i128);
        assert_eq!(total_after_1, 600_i128, "Running total should be 600 after member1");

        // ── 3. member2 contributes 400 — pool is now full ─────────────────────
        let total_after_2 = client.contribute(&member2, &400_i128);
        assert_eq!(total_after_2, 1000_i128, "Running total should be 1000 after member2");

        // ── 4. Release payment to recipient ───────────────────────────────────
        let released_amount = client.release_payment();
        assert_eq!(released_amount, 1000_i128, "Released amount should equal target");

        // ── 5. Verify recipient's token balance ───────────────────────────────
        let token_client = TokenClient::new(&env, &token_id);
        assert_eq!(
            token_client.balance(&recipient),
            1000_i128,
            "Recipient should hold the full 1000 tokens"
        );

        // ── 6. Contract escrow should now be empty ────────────────────────────
        assert_eq!(
            token_client.balance(&contract_id),
            0_i128,
            "Contract escrow should be drained after release"
        );
    }

    // ════════════════════════════════════════════════════════════════════════
    // TEST 2 — Edge Case
    // A contribution attempt AFTER the payment has already been released
    // must be rejected with AlreadyReleased.
    // ════════════════════════════════════════════════════════════════════════
    #[test]
    fn test_contribute_rejected_after_release() {
        let (env, contract_id, token_id, organizer, recipient, member1, _member2) =
            setup_env();

        // Mint extra tokens for a late contributor
        let late_member   = Address::generate(&env);
        let asset_client  = StellarAssetClient::new(&env, &token_id);
        asset_client.mint(&late_member, &500_i128);

        let client = GroupPayContractClient::new(&env, &contract_id);

        // Create group and fully fund it with member1 alone
        client.create_group(
            &organizer,
            &recipient,
            &token_id,
            &600_i128,          // target = 600 so member1's balance covers it
            &2_000_000_u64,
            &Symbol::new(&env, "TeamDinner"),
        );

        client.contribute(&member1, &600_i128);
        client.release_payment(); // pool is now released

        // Late member tries to contribute AFTER release — must fail
        let result = client.try_contribute(&late_member, &500_i128);
        assert!(
            result.is_err(),
            "Contribution after release should return an error"
        );
    }

    // ════════════════════════════════════════════════════════════════════════
    // TEST 3 — State Verification
    // After two partial contributions, contract storage must reflect:
    //   • correct total collected
    //   • correct target
    //   • is_released = false
    //   • contract escrow holds the right token balance
    //   • contributing wallets have been debited
    // ════════════════════════════════════════════════════════════════════════
    #[test]
    fn test_state_reflects_contributions() {
        let (env, contract_id, token_id, organizer, recipient, member1, member2) =
            setup_env();

        // Only give each member a partial amount so target is NOT yet reached
        // member1 has 600, member2 has 400 but we only contribute 300 + 200 = 500
        let client = GroupPayContractClient::new(&env, &contract_id);

        client.create_group(
            &organizer,
            &recipient,
            &token_id,
            &1000_i128,         // target
            &2_000_000_u64,
            &Symbol::new(&env, "Retreat"),
        );

        client.contribute(&member1, &300_i128);
        client.contribute(&member2, &200_i128);

        // ── get_status should show 500 / 1000, not yet released ───────────────
        let (collected, target, released) = client.get_status();
        assert_eq!(collected, 500_i128,   "Total collected should be 500");
        assert_eq!(target,    1000_i128,  "Target should remain 1000");
        assert!(!released,                "Pool should not be released yet");

        // ── Token balances: contract escrow holds 500 ─────────────────────────
        let token_client = TokenClient::new(&env, &token_id);
        assert_eq!(
            token_client.balance(&contract_id),
            500_i128,
            "Contract escrow should hold 500 tokens"
        );

        // ── Members' wallets are correctly debited ────────────────────────────
        assert_eq!(
            token_client.balance(&member1),
            300_i128, // started with 600, contributed 300
            "member1 should have 300 remaining"
        );
        assert_eq!(
            token_client.balance(&member2),
            200_i128, // started with 400, contributed 200
            "member2 should have 200 remaining"
        );
    }
}