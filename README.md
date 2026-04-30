# GroupPay Stellar

**One-Line Description:** A decentralized group fund manager that allows student leaders to collect project contributions seamlessly using XLM or USDC while instantly tracking who has paid.

## Problem
A college student in Manila leading a group project has to pay upfront for printing materials, transportation, and supplies. Collecting each member's share manually takes time, requires tracking multiple digital wallet apps, and sometimes causes interpersonal conflict.

## Solution
GroupPay Stellar lets the project leader create an on-chain payment pool. Members send their exact share using Stellar XLM or USDC, and the Soroban smart contract instantly marks each member as paid on the blockchain. Once fully funded, the leader can withdraw the pooled funds in one click.

## Timeline
- **MVP Delivery:** Demo-able core contract mapping payments and tracking user status.
- **Future Enhancements:** Automated split-billing calculations and deadline enforcements.

## Stellar Features Used
- **Soroban Smart Contracts:** Tracks the ledger of who has paid and secures the pooled funds.
- **XLM / USDC Transfers:** Enables fast, low-cost micro-transactions perfect for student budgets.

## Vision and Purpose
To eliminate the friction of shared expenses in academic and micro-economies. GroupPay leverages blockchain to provide undisputed, transparent proof of payment for everyday coordination problems.

## Prerequisites
- Rust toolchain
- Soroban CLI (`v22.0.0` or compatible)

## How to Build
```bash
soroban contract build
