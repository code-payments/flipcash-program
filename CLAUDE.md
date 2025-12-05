# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Flipcash is a Solana program for creating custom currencies with automated market-making via an exponential bonding curve. Each currency has its own SPL token mint and liquidity pool backed by a base mint (typically USDC or similar stablecoin).

**Program ID:** `ccZR1qzNyMaHDB47PkqDZVpNdimji7wJf65zyfGR3FJ`

## Workspace Structure

This is a Rust workspace with four main crates:

- **`api/`** - Core types, state definitions, instructions, and bonding curve logic. Contains the program's data structures (`CurrencyConfig`, `LiquidityPool`), PDA derivation functions, and the exponential curve mathematics
- **`program/`** - On-chain Solana program implementation. Handles instruction processing for currency/pool initialization, buy/sell operations, and metadata creation
- **`client/`** - Off-chain Rust client library for building and sending transactions to the program
- **`cli/`** - Command-line interface tool (`flipcash`) for interacting with the program

## Build & Test Commands

### Building
```bash
# Build the Solana program (requires metadata.so dependency)
make build

# Download metadata.so dependency (required before building)
make metadata
```

### Testing
```bash
# Run program tests using cargo test-sbf
make test
```

### Local Development
```bash
# Clean ledger, build, and start local validator with program deployed
make local

# Just start validator with mainnet metadata cloned
make validator
```

### Documentation
```bash
# Generate and open workspace documentation
make docs
```

## Program Architecture

### PDA Derivation Hierarchy

PDAs are derived in a specific hierarchy:

1. **Mint PDA**: `["mint", authority, name_bytes, seed]` - The SPL token mint for the currency
2. **Currency PDA**: `["currency", mint_pda]` - Stores currency metadata and configuration
3. **Pool PDA**: `["pool", currency_pda]` - Manages the liquidity pool for this currency
4. **Vault PDAs**: `["treasury", pool_pda, mint]` - Token vaults for the pool (one for currency, one for base)
5. **Metadata PDA**: Uses Metaplex standard derivation for token metadata

See `api/src/pda.rs` for the exact derivation functions.

### Core State Accounts

**CurrencyConfig** (`api/src/state/currency.rs`):
- Stores authority, mint address, name (32 bytes max), symbol (8 bytes max), and random seed
- Created by `InitializeCurrencyIx` instruction

**LiquidityPool** (`api/src/state/pool.rs`):
- Links currency to base mint with two vaults (mint_a/target and mint_b/base)
- Stores `sell_fee` in basis points (e.g., 50 = 0.5%)
- Created by `InitializePoolIx` instruction

### Bonding Curve Mechanism

The pricing uses an exponential curve defined in `api/src/curve.rs`:

**Price function**: `R'(S) = a * b * e^(c * S)`

**Default parameters** (defined in `api/src/consts.rs`):
- Starting price: $0.01 (at supply = 0)
- Ending price: $1,000,000 (at supply = 21,000,000 tokens)
- Constants: `CURVE_A`, `CURVE_B`, `CURVE_C` are pre-calculated fixed-point values

**Key curve functions**:
- `spot_price_at_supply()` - Current price at a given supply level
- `tokens_to_value_from_current_supply()` - Cost to buy X tokens (integrates the curve)
- `tokens_to_value_from_current_value()` - Value received when selling X tokens
- `value_to_tokens()` - How many tokens you get for Y value

Uses `brine-fp` library for fixed-point arithmetic to avoid floating point on-chain.

### Instructions

All instructions are defined in `api/src/instruction.rs`:

1. **InitializeCurrencyIx** - Creates currency mint and config account
2. **InitializePoolIx** - Sets up liquidity pool with vaults and fee configuration
3. **InitializeMetadataIx** - Creates Metaplex metadata for the token
4. **BuyTokensIx** - Buy currency tokens by depositing base tokens
5. **SellTokensIx** - Sell currency tokens for base tokens (fees applied)
6. **BuyAndDepositIntoVmIx** - Buy tokens and deposit into VM omnibus account
7. **SellAndDepositIntoVmIx** - Sell tokens and deposit proceeds into VM omnibus account

The instruction processor in `program/src/lib.rs` routes to individual handlers in `program/src/instruction/`.

### Testing Infrastructure

Tests use `litesvm` for fast local Solana VM simulation without needing a validator.

Test utilities in `program/tests/utils/`:
- `svm.rs` - VM setup and transaction submission
- `token.rs` - SPL token helpers (create mint, ATAs, etc.)
- `print.rs` - Debug printing utilities

Main integration test: `program/tests/integration.rs`

## CLI Usage

The CLI binary is named `flipcash` (defined in `cli/Cargo.toml`).

**Global options**:
- `--keypair <PATH>` - Defaults to `~/.config/solana/id.json`
- `--cluster <VALUE>` - `l` (localnet), `m` (mainnet), `d` (devnet), `t` (testnet), or custom URL

**Common workflow**:
```bash
# 1. Create test base mint (e.g., fake USDC)
cargo run --bin flipcash -- create-base-mint --decimals 6 --initial-amount 1000000000000

# 2. Create currency
cargo run --bin flipcash -- create-currency --name "FlipToken" --symbol "FLIP" --base-mint <BASE_MINT_PUBKEY>

# 3. Buy tokens
cargo run --bin flipcash -- buy --mint <CURRENCY_MINT> --base-mint <BASE_MINT> --amount 100.0

# 4. Sell tokens
cargo run --bin flipcash -- sell --mint <CURRENCY_MINT> --base-mint <BASE_MINT> --amount 50.0

# 5. Get currency info
cargo run --bin flipcash -- get-currency --mint <CURRENCY_MINT>
```

## Important Constants

Defined in `api/src/consts.rs`:

- `TOKEN_DECIMALS: u8 = 10` - All currencies use 10 decimals
- `MAX_TOKEN_SUPPLY: u64 = 21_000_000` - Maximum supply per currency
- `QUARKS_PER_TOKEN: u64 = 10_000_000_000` - Smallest unit (10^10)
- `MAX_NAME_LEN: usize = 32` - Maximum currency name length
- `MAX_SYMBOL_LEN: usize = 8` - Maximum symbol length
- `METADATA_URI` - Template for token metadata JSON URL

## Dependencies

Key external dependencies:
- **steel** (v4.0.0) - Solana program framework with helpful macros and utilities
- **spl-token** - SPL Token program bindings
- **mpl-token-metadata** - Metaplex metadata program bindings
- **brine-fp** - Fixed-point arithmetic library for bonding curve calculations
- **solana-sdk** (v2.1) - Solana SDK for client operations
- **litesvm** (v0.5.0) - Fast local SVM for testing

## Solana Version

All Solana crates pinned to version `2.1`. The program uses `cargo build-sbf` and `cargo test-sbf` commands (Solana BPF toolchain).
