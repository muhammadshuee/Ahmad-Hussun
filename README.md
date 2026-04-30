# GroupPay 

## Project Description
GroupPay is a decentralized group fund manager that allows student leaders to collect project contributions seamlessly using XLM or USDC while instantly tracking who has paid. A college student in Manila leading a group project has to pay upfront for printing materials and transportation. Collecting each member's share manually takes time and causes conflict. GroupPay lets the project leader create an on-chain payment pool where members send their exact share, and the smart contract instantly marks each member as paid.

## Project Vision
To eliminate the friction of shared expenses in academic and micro-economies. GroupPay leverages blockchain to provide undisputed, transparent proof of payment for everyday coordination problems, preventing arguments and saving students time.

## Key Features
- **Smart Escrow Pool:** A Soroban contract securely holds funds until the project leader is ready to withdraw.
- **Instant Payment Tracking:** The contract acts as an immutable ledger, automatically tracking `has_paid` status for every member.
- **Micro-transaction Friendly:** Utilizes XLM / USDC for fast, near-zero fee transfers perfect for tight student budgets.

## Deployed Contract Details
- **Network:** Testnet
- **Contract ID:** CCOCHMZGL2BUY6NK6JQLMLXNA4BKFCQQWM5NIZJ4JM5HNC3VLPGCZPLU
- **Deployer Address:** GCSGI4ZRPWFZV3DZMHNRELFJXJ6YELBV3LDADK7AWUHSELBLPAU4UT42

#Contract link
#1
https://stellar.expert/explorer/testnet/tx/6ed712f8a2241afb7faa37f6dc54d8213b05bb567117ac5b6791518e93a3cb98

#2
https://lab.stellar.org/r/testnet/contract/CCOCHMZGL2BUY6NK6JQLMLXNA4BKFCQQWM5NIZJ4JM5HNC3VLPGCZPLU



## Future Scope
- **Automated Split-Billing:** Dynamic calculation of individual shares based on total receipts uploaded.
- **Deadline Enforcements:** Smart contract logic to apply penalties or lock out members who do not pay their share by a specific date.
- **Multi-Asset Support:** Allowing members to pay in different Stellar-issued stablecoins while settling in a unified asset for the leader.

## Prerequisites
- Rust toolchain
- Soroban CLI (`v22.0.0` or compatible)

## How to Build
```bash
soroban contract build

#![screenshot](screenshot.png)