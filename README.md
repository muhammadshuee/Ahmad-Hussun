# GroupPay Stellar 💸

> Decentralized group bill-splitting and escrow — split costs fairly, pay atomically, on Stellar.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Built on Stellar](https://img.shields.io/badge/Built%20on-Stellar-blue)](https://stellar.org)
[![Soroban](https://img.shields.io/badge/Smart%20Contracts-Soroban-purple)](https://soroban.stellar.org)

---

## Project Description

GroupPay Stellar is a decentralized group payment and escrow application built on the Stellar blockchain using Soroban smart contracts. It allows any group — freelancers, students, or small teams — to pool money together toward a shared goal (rent, event fees, supplier payments) without trusting a single person to hold the funds. Contributions are locked in an on-chain escrow, automatically released to the recipient when the target is met, or refunded to each contributor if the deadline passes unfunded. Built for Southeast Asia where group payments are common but informal, costly, and unreliable.

---

## Project Vision

GroupPay Stellar aims to become the go-to trustless payment layer for informal group finance across Southeast Asia. Today, millions of freelancers, student organizations, and small businesses in the Philippines, Indonesia, and Vietnam split costs manually — chasing payments across GCash, Maya, and bank transfers with no enforcement, no transparency, and no safety net. Our vision is a world where any group can create a payment pool in 30 seconds, contribute from any wallet, and trust that the money moves exactly as agreed — automatically, on-chain, for fractions of a cent. Long-term, GroupPay Stellar will integrate local fiat on-ramps (GCash → USDC via MoneyGram Ramps), AI-powered bill-split suggestions, and recurring payment schedules, making blockchain-native group finance as simple as sending a link.

---

## 🧩 Problem

A group of 5 freelancers in Manila sharing an office rental cannot easily collect
contributions from each other before paying the landlord. One person fronts the full
monthly cost, then manually chases four colleagues across different e-wallets (GCash,
Maya, PayMaya) — often waiting days while missing the due date, paying late fees, or
absorbing the entire cost alone.

The organizer bears **100% of the financial risk** and the social friction of repeated
reminders, with zero on-chain enforcement or transparent audit trail.

---

## ✅ Solution

GroupPay Stellar uses a **Soroban smart contract** to create an on-chain escrow pool:

1. The organizer defines the **target amount**, **deadline**, and **recipient wallet**
2. Each group member calls `contribute()` to lock their XLM or USDC into the contract
3. When the full target is reached, `release_payment()` sends the entire pool **atomically** to the recipient — no intermediary, no trust required
4. If the deadline passes without hitting the target, any contributor calls `refund()` to recover their own funds in full

**Why Stellar?** 3–5 second finality and sub-cent fees make micropayment-level contributions practical. Soroban provides programmable escrow with on-chain events for real-time front-end updates.

---

## ⭐ Stellar Features Used

| Feature | Role |
|---|---|
| **Soroban Smart Contracts** | Core escrow: `create_group`, `contribute`, `release_payment`, `refund`, `get_status` |
| **XLM / USDC Token Interface** | Actual money movement — members lock tokens into contract, released atomically |
| **On-chain Events** | Front-end indexes contributions and releases in real-time without a database |
| **Trustlines** | USDC trustline required per member wallet before contributing stablecoin |
| **Testnet / Futurenet** | Development and hackathon demo environment |

---

## 🎯 Target Users

| Segment | Description |
|---|---|
| **Freelancers** | Remote workers in Metro Manila / Cebu splitting co-working space, team dinners, or software subscriptions |
| **University Students** | Org treasurers in PH / ID / VN collecting event fees, trip deposits, graduation payments from 10–50 members |
| **SME Teams** | Small businesses in Jakarta or Ho Chi Minh City splitting supplier orders or trade-fair booth fees |
| **Event Organizers** | Anyone coordinating group purchases: concert tickets, catered meals, venue deposits |

---

## 🏗️ Contract Architecture

```
src/
├── lib.rs      ← Soroban contract (all business logic)
└── test.rs     ← 3-test suite (happy path, edge case, state verification)
Cargo.toml
README.md
```

### Public Functions

| Function | Description |
|---|---|
| `create_group(organizer, recipient, token, target, deadline, description)` | Initialise the payment pool (once per contract instance) |
| `contribute(member, amount)` | Lock tokens into escrow, returns running total |
| `release_payment()` | Atomically pay out pool to recipient when target is met |
| `refund(member)` | Reclaim individual contribution after deadline if target not met |
| `get_status()` | Read-only: returns `(total_collected, target, is_released)` |

---

## 🔄 MVP Transaction Flow

```
Organizer                  Contract                  Member(s)
   │                          │                          │
   │── create_group() ───────▶│                          │
   │                          │ stores GroupConfig       │
   │                          │ emits group:created      │
   │                          │                          │
   │                          │◀──── contribute() ───────│
   │                          │ pulls tokens into escrow │
   │                          │ emits contrib event      │
   │                          │                          │
   │── release_payment() ────▶│                          │
   │                          │ transfers full pool      │
   │                          │ to recipient             │
   │                          │ emits released event     │
```

---

## 📅 Suggested MVP Timeline

| Day | Deliverable |
|---|---|
| **Day 1** | Soroban contract — `create_group`, `contribute`, `release_payment`, unit tests passing |
| **Day 2** | Soroban contract — `refund`, `get_status`, full 3-test suite, testnet deploy |
| **Day 3** | Next.js front-end — create group form, real-time progress bar, Freighter wallet integration |
| **Day 4** | End-to-end demo polish — QR invite link, Stellar Explorer link, pitch preparation |

---

## 🛠️ Prerequisites

```bash
# Rust toolchain
curl https://sh.rustup.rs -sSf | sh
rustup target add wasm32-unknown-unknown

# Soroban CLI v22
cargo install --locked soroban-cli --version 22.0.0

# Verify
soroban --version
```

---

## 🔨 Build

```bash
soroban contract build
# → target/wasm32-unknown-unknown/release/grouppay_stellar.wasm
```

---

## 🧪 Test

```bash
cargo test
```

Expected output:
```
running 3 tests
test tests::test_happy_path_create_contribute_release ... ok
test tests::test_contribute_rejected_after_release ... ok
test tests::test_state_reflects_contributions ... ok

test result: ok. 3 passed; 0 failed
```

---

## 🚀 Deploy to Testnet

### 1. Generate and fund a test identity

```bash
soroban keys generate --global alice --network testnet
soroban keys fund alice --network testnet
```

### 2. Deploy the contract

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/grouppay_stellar.wasm \
  --source alice \
  --network testnet
# Outputs: CONTRACT_ID — save this!
```

---

## 🖥️ CLI Invocations

### `create_group`

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --source alice \
  --network testnet \
  -- create_group \
     --organizer GAABC...ORGANIZER \
     --recipient GBPAY...RECIPIENT \
     --token CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC \
     --target 5000000 \
     --deadline 1800000000 \
     --description OfficeRent
```

### `contribute`

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --source bob \
  --network testnet \
  -- contribute \
     --member GBOB...MEMBER \
     --amount 1000000
```

### `release_payment`

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --source alice \
  --network testnet \
  -- release_payment
```

### `refund`

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --source bob \
  --network testnet \
  -- refund \
     --member GBOB...MEMBER
```

### `get_status`

```bash
soroban contract invoke \
  --id CONTRACT_ID \
  --source alice \
  --network testnet \
  -- get_status
# Returns: (total_collected, target, is_released)
# e.g.    → [500000, 1000000, false]
```

---

## 🌐 References

- Stellar Bootcamp 2026: https://github.com/armlynobinguar/Stellar-Bootcamp-2026
- Example Full-Stack (community-treasury): https://github.com/armlynobinguar/community-treasury
- Soroban Docs: https://soroban.stellar.org/docs
- Stellar Testnet Faucet: https://friendbot.stellar.org

---

## 🗺️ Roadmap (Post-Hackathon)

- [ ] GCash / Maya on-ramp via MoneyGram Ramps (PHP → USDC)
- [ ] AI-powered bill-split suggestion (paste receipt image → suggested amounts)
- [ ] Freighter deep-link QR code for mobile tap-to-pay
- [ ] Offline contribution queuing with IndexedDB sync
- [ ] Multi-token support (any Stellar asset)
- [ ] Recurring group payments (monthly rent, subscriptions)

---

## 📄 License

MIT License — Copyright (c) 2026 GroupPay Stellar Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.