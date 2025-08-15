# Constant Product AMM Contract

This repository implements a Constant Product Automated Market Maker (AMM) smart contract for the Solana blockchain using [Anchor](https://book.anchor-lang.com/). The contract allows users to create liquidity pools, deposit tokens, swap between tokens, and withdraw liquidity, following the `x*y=k` invariant.

## Features

- **Initialize Liquidity Pool:** Create a new pool with two tokens and initial liquidity.
- **Deposit:** Add liquidity to an existing pool and receive LP tokens.
- **Swap:** Swap between base and quote tokens with automatic fee deduction.
- **Withdraw:** Remove liquidity from the pool and redeem LP tokens for underlying assets.
- **Event Emission:** Emits events for pool initialization, deposits, swaps, and withdrawals.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) Version: rustc 1.88.0-nightly (3350c1eb3 2025-05-01)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) Version: solana-cli 2.2.12 (src:0315eb6a; feat:1522022101, client:Agave)
- [Anchor](https://solana.com/docs/intro/installation) Verison: anchor-cli 0.31.1
- [Node.js](https://nodejs.org/) Version: v23.11.0

### Build and Deploy

1. **Build the Program:**
   ```bash
   anchor build
   ```
1. **Run Tests**
   ```bash
   anchor test
   ```

### Usage

- Initialize Pool: Use the initializeLiquidity instruction to create a new pool.
- Deposit: Use the deposit instruction to add liquidity.
- Swap: Use the swapBaseIn instruction to swap tokens.
- Withdraw: Use the withdraw instruction to remove liquidity.

See [tests/amm.ts](tests/amm.ts) for example usage and integration tests.
