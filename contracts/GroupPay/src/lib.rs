#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    token, Address, Env, Symbol, Map,
};

// ── Storage Keys ──────────────────────────────────────────────────────────────
// GROUP       → GroupConfig              (pool metadata)
// CONTRIBS    → Map<Address, i128>       (per-member contribution amounts)
// RELEASED    → bool                     (payout guard — prevents double-release)

const GROUP_KEY:    Symbol = symbol_short!("GROUP");
const CONTRIBS_KEY: Symbol = symbol_short!("CONTRIBS");
const RELEASED_KEY: Symbol = symbol_short!("RELEASED");

// ── Data Types ────────────────────────────────────────────────────────────────

/// Metadata stored on-chain when the organizer creates a payment group.
#[contracttype]
#[derive(Clone, Debug)]
pub struct GroupConfig {
    pub organizer:   Address, // wallet that created the group
    pub recipient:   Address, // wallet that will receive the pooled funds
    pub token:       Address, // XLM or USDC token contract address
    pub target:      i128,    // total amount required (in stroops / smallest unit)
    pub deadline:    u64,     // Unix timestamp after which refund is permitted
    pub description: Symbol,  // short label, e.g. "OfficeRent"
}

/// Error codes returned by contract functions.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum GroupPayError {
    AlreadyInitialized = 1, // create_group called twice
    NotInitialized     = 2, // contract not yet initialised
    DeadlinePassed     = 3, // contribution attempt after deadline
    DeadlineNotPassed  = 4, // refund attempt before deadline
    TargetAlreadyMet   = 5, // contribution attempt when pool is full
    AlreadyReleased    = 6, // action not allowed after payout
    TargetNotMet       = 7, // release attempt before pool is full
    InvalidAmount      = 8, // zero or negative amount supplied
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct GroupPayContract;

#[contractimpl]
impl GroupPayContract {

    // ── create_group ──────────────────────────────────────────────────────────
    /// Initialises a new payment pool. Callable only once per contract instance.
    /// Stores the GroupConfig and an empty contributions map in instance storage.
    pub fn create_group(
        env:         Env,
        organizer:   Address,
        recipient:   Address,
        token:       Address,
        target:      i128,
        deadline:    u64,
        description: Symbol,
    ) -> Result<(), GroupPayError> {
        // Prevent re-initialisation
        if env.storage().instance().has(&GROUP_KEY) {
            return Err(GroupPayError::AlreadyInitialized);
        }
        if target <= 0 {
            return Err(GroupPayError::InvalidAmount);
        }
        // Organizer must authorise the creation
        organizer.require_auth();

        let config = GroupConfig {
            organizer,
            recipient,
            token,
            target,
            deadline,
            description,
        };

        // Persist config and initialise empty contribution map
        env.storage().instance().set(&GROUP_KEY,    &config);
        let empty: Map<Address, i128> = Map::new(&env);
        env.storage().instance().set(&CONTRIBS_KEY, &empty);
        env.storage().instance().set(&RELEASED_KEY, &false);

        // Emit event so front-ends can index the new pool
        env.events().publish(
            (symbol_short!("group"), symbol_short!("created")),
            config.target,
        );
        Ok(())
    }

    // ── contribute ────────────────────────────────────────────────────────────
    /// A group member transfers `amount` tokens into this contract's escrow.
    /// Returns the new cumulative total collected.
    /// Rejects contributions after the deadline or once the target is already met.
    pub fn contribute(
        env:    Env,
        member: Address,
        amount: i128,
    ) -> Result<i128, GroupPayError> {
        member.require_auth();

        let config: GroupConfig = env
            .storage().instance().get(&GROUP_KEY)
            .ok_or(GroupPayError::NotInitialized)?;

        let released: bool = env
            .storage().instance().get(&RELEASED_KEY)
            .unwrap_or(false);

        if released {
            return Err(GroupPayError::AlreadyReleased);
        }
        if amount <= 0 {
            return Err(GroupPayError::InvalidAmount);
        }

        // Reject contributions after the deadline
        let now = env.ledger().timestamp();
        if now > config.deadline {
            return Err(GroupPayError::DeadlinePassed);
        }

        // Reject if pool is already fully funded
        let total = Self::total_collected(&env);
        if total >= config.target {
            return Err(GroupPayError::TargetAlreadyMet);
        }

        // Pull tokens from the member's wallet into this contract (escrow)
        let token_client = token::Client::new(&env, &config.token);
        token_client.transfer(
            &member,
            &env.current_contract_address(),
            &amount,
        );

        // Record the member's cumulative contribution
        let mut contribs: Map<Address, i128> =
            env.storage().instance().get(&CONTRIBS_KEY).unwrap();
        let prev = contribs.get(member.clone()).unwrap_or(0);
        contribs.set(member.clone(), prev + amount);
        env.storage().instance().set(&CONTRIBS_KEY, &contribs);

        let new_total = total + amount;

        // Emit event: (member, amount_contributed, running_total)
        env.events().publish(
            (symbol_short!("contrib"), member),
            (amount, new_total),
        );
        Ok(new_total)
    }

    // ── release_payment ───────────────────────────────────────────────────────
    /// Sends the entire escrowed pool to the recipient once the target is met.
    /// Sets the RELEASED flag before transfer as a re-entrancy guard.
    pub fn release_payment(env: Env) -> Result<i128, GroupPayError> {
        let config: GroupConfig = env
            .storage().instance().get(&GROUP_KEY)
            .ok_or(GroupPayError::NotInitialized)?;

        let released: bool = env
            .storage().instance().get(&RELEASED_KEY)
            .unwrap_or(false);

        if released {
            return Err(GroupPayError::AlreadyReleased);
        }

        let total = Self::total_collected(&env);
        if total < config.target {
            return Err(GroupPayError::TargetNotMet);
        }

        // Set released BEFORE transfer to prevent re-entrancy
        env.storage().instance().set(&RELEASED_KEY, &true);

        let token_client = token::Client::new(&env, &config.token);
        token_client.transfer(
            &env.current_contract_address(),
            &config.recipient,
            &total,
        );

        env.events().publish(
            (symbol_short!("released"), config.recipient),
            total,
        );
        Ok(total)
    }

    // ── refund ────────────────────────────────────────────────────────────────
    /// After deadline, if the target was NOT met, each contributor can reclaim
    /// their own contribution. Idempotent — a second call returns 0.
    pub fn refund(env: Env, member: Address) -> Result<i128, GroupPayError> {
        member.require_auth();

        let config: GroupConfig = env
            .storage().instance().get(&GROUP_KEY)
            .ok_or(GroupPayError::NotInitialized)?;

        let released: bool = env
            .storage().instance().get(&RELEASED_KEY)
            .unwrap_or(false);

        if released {
            return Err(GroupPayError::AlreadyReleased);
        }

        // Refund only available after the deadline has passed
        let now = env.ledger().timestamp();
        if now <= config.deadline {
            return Err(GroupPayError::DeadlineNotPassed);
        }

        // If the target was met, no refunds — use release_payment instead
        let total = Self::total_collected(&env);
        if total >= config.target {
            return Err(GroupPayError::TargetAlreadyMet);
        }

        let mut contribs: Map<Address, i128> =
            env.storage().instance().get(&CONTRIBS_KEY).unwrap();
        let owed = contribs.get(member.clone()).unwrap_or(0);
        if owed == 0 {
            return Ok(0); // nothing to refund — idempotent
        }

        // Zero out the member's balance BEFORE transfer (re-entrancy guard)
        contribs.set(member.clone(), 0_i128);
        env.storage().instance().set(&CONTRIBS_KEY, &contribs);

        let token_client = token::Client::new(&env, &config.token);
        token_client.transfer(
            &env.current_contract_address(),
            &member,
            &owed,
        );

        env.events().publish(
            (symbol_short!("refund"), member),
            owed,
        );
        Ok(owed)
    }

    // ── get_status ────────────────────────────────────────────────────────────
    /// Read-only view returning (total_collected, target, is_released).
    /// Used by the front-end to render the contribution progress bar.
    pub fn get_status(env: Env) -> Result<(i128, i128, bool), GroupPayError> {
        let config: GroupConfig = env
            .storage().instance().get(&GROUP_KEY)
            .ok_or(GroupPayError::NotInitialized)?;
        let released: bool = env
            .storage().instance().get(&RELEASED_KEY)
            .unwrap_or(false);
        Ok((Self::total_collected(&env), config.target, released))
    }

    // ── internal helper ───────────────────────────────────────────────────────
    /// Sums all values in the contributions map to get the running total.
    fn total_collected(env: &Env) -> i128 {
        let contribs: Map<Address, i128> =
            env.storage().instance()
                .get(&CONTRIBS_KEY)
                .unwrap_or_else(|| Map::new(env));
        contribs.values().iter().sum()
    }
}

// Include test module when building with test feature
#[cfg(test)]
mod test;