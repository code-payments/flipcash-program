# Flipcash Reserve Contract
![license][license-image]
![version][version-image]

[version-image]: https://img.shields.io/badge/version-1.0.0-blue.svg?style=flat
[license-image]: https://img.shields.io/badge/license-MIT-blue.svg?style=flat


## The Flipcash Currency Creator

Flipcash is the only platform for creating digital currencies that are immediately used as real money. The moment a currency is created it can be sent, spent, or handed to someone as simply as physical cash. Every currency has guaranteed liquidity from day one, which is managed by the Reserve Contract.

## The Reserve Contract

Every Flipcash currency is governed by the Reserve Contract, an on-chain contract that manages each currency’s supply and liquidity.

When a new currency is created, 21 million coins are minted and deposited into the Reserve Contract. Each currency has a fixed supply of 21 million coins, so there will never be more. The Reserve Contract then sells coins on a predefined pricing curve, accepting payment in USDF, a fully backed 1:1 USD stablecoin managed in partnership with Coinbase. 

The first coin sells for $0.01. With every $11,400 of coins purchased, the price per coin increases by roughly one penny until the 21 millionth coin sells for $1 million.

The Reserve Contract custodies the supply of each currency that hasn’t yet been sold. It also custodies the USDF received as payment for each coin, using that USDF to also buy coins on the same pricing curve. In doing so, the Reserve Contract acts as a guaranteed buyer, ensuring continuous liquidity without relying on market makers, order books, or liquidity providers, all in a fully trustless manner.

## Audits

| Network | Address | Audited By | Status | Audit Report |
| --- | --- | --- | --- | --- |
| Mainnet | [ccJYP5...pQtf8Z](https://explorer.solana.com/address/ccJYP5gjZqcEHaphcxAZvkxCrnTVfYMjyhSYkpQtf8Z) | [Sec3](https://www.sec3.dev/) | Completed | [Report](https://github.com/code-payments/flipcash-program/blob/main/docs/audit_final.pdf) |

## Features

The Reserve Contract provides the following core features:

- **Currency Initialization:** Creates a new SPL Token mint for a custom currency with Metaplex metadata
- **Pool Creation:** Creates a liquidity pool linked to the currency, backed by a base mint. The pool manages two vaults (one for the currency, one for the base), and sell fee rates (in basis points)
- **Trading (Buy/Sell):** Allows users to buy currency tokens by depositing base tokens or sell currency tokens for base tokens. Fees are applied on sells, and the pool uses a deterministic pricing model via a discrete bonding curve logic found in `flipcash_api`
- **Metadata Retrieval:** Exposes account data for currencies and pools, including authorities, mints, vaults, and fees

## CLI

The Flipcash CLI is a command-line interface tool built in Rust for interacting with the Flipcash program. Flipcash provides a Solana-based protocol for creating custom currencies backed by a base mint (e.g. a stablecoin like USDF), managing liquidity pools, and facilitating buy/sell operations on those currencies. The CLI allows users to create test mints, initialize currencies and pools, retrieve metadata, and perform trades.

The CLI supports various Solana clusters (localnet, mainnet, devnet, testnet, or custom RPC URLs) and requires a Solana keypair for signing transactions.


## Installation

To build and install the CLI:
1. Ensure you have Rust and Cargo installed
2. Clone the repository (assuming it's in a repo) or use the provided source code
3. Run `cargo build --release` to compile the binary
4. The executable will be available at `target/release/flipcash-cli`

You may need to install Solana CLI tools separately for keypair management.

## Options

These options are available for all commands and can be specified before the subcommand.

- `--keypair <PATH>`: Path to the Solana keypair file (JSON format). Default: `~/.config/solana/id.json`
- `--cluster <VALUE>`: Solana cluster to connect to. Options:
  - `l`: Localnet (`http://127.0.0.1:8899`)
  - `m`: Mainnet (`https://api.mainnet-beta.solana.com`)
  - `d`: Devnet (`https://api.devnet.solana.com`)
  - `t`: Testnet (`https://api.testnet.solana.com`)
  - Custom RPC URL (e.g. `https://my-custom-rpc.com`)
  Default: `l` (localnet)

Example usage:
```
flipcash-cli --keypair /path/to/keypair.json --cluster d create-currency --name "MyToken" --symbol "MTK" --base-mint <PUBKEY>
```

## Commands

### create-base-mint

Creates a new base mint (e.g. a test USDF-like token) for testing purposes. This includes:
- Creating the mint account
- Creating an Associated Token Account (ATA) for the fee-payer
- Minting an initial amount of tokens to the ATA

**Usage:**
```
flipcash-cli create-base-mint [OPTIONS]
```

**Options:**
- `--decimals <U8>`: Number of decimal places for the mint. Default: 6
- `--initial-amount <U64>`: Initial amount of tokens to mint (in smallest units, e.g. 1_000_000_000_000 for 1,000,000 tokens with 6 decimals). Default: 1_000_000_000_000

**Output:**
- Prints the mint address and transaction signatures for creation, ATA, and minting

**Functionality in Flipcash Program:**
- This is a utility command for testing. It uses SPL Token program functions to create a mint, not directly interacting with the core logic of the Flipcash program

### create-currency

Creates a new currency mint and its associated liquidity pool on the Flipcash program.

**Usage:**
```
flipcash-cli create-currency --name <STRING> --symbol <STRING> --base-mint <PUBKEY>
```

**Options:**
- `--name <STRING>`: Name of the currency (max 32 characters). Required
- `--symbol <STRING>`: Symbol of the currency (max 8 characters). Required
- `--base-mint <PUBKEY>`: Public key of the base mint (e.g. USDF mint). Required

**Output:**
- Prints transaction signatures for currency and pool creation
- Prints addresses for the currency mint PDA, currency PDA, and pool PDA

**Functionality in Flipcash Program:**
- Calls the `initialize` instruction on the Flipcash program
- Creates a currency account with metadata (authority, mint, name, symbol)
- Creates a pool account linked to the currency, including vaults for the target currency and base mint, fee structures (sell fees in basis points), and other metadata
- Creates a Metaplex metadata account for on-chain token metadata
- PDAs (Program-Derived Addresses) are used for deterministic account addresses

### get-currency

Retrieves metadata for a given currency mint and its associated pool.

**Usage:**
```
flipcash-cli get-currency --mint <PUBKEY>
```

**Options:**
- `--mint <PUBKEY>`: Public key of the currency mint. Required

**Output:**
- Currency Metadata: Authority, Mint, Name, Symbol
- Pool Metadata: Authority, Currency, Mint A (Target), Mint B (Base), Vault A, Vault B, Fees Accumulated, Sell Fee (bps and %)

**Functionality in Flipcash Program:**
- Derives the currency PDA and pool PDA from the mint
- Fetches and deserializes the currency and pool accounts from the blockchain
- Displays on-chain data, including fees accumulated and fee rates (e.g. sell_fee in basis points, where 100 bps = 1%)

### buy

Buys tokens from the pool using base tokens (e.g. spend USDF to buy the custom currency)

**Usage:**
```
flipcash-cli buy --mint <PUBKEY> --base-mint <PUBKEY> --amount <F64>
```

**Options:**
- `--mint <PUBKEY>`: Public key of the currency mint to buy. Required
- `--base-mint <PUBKEY>`: Public key of the base mint (e.g. USDF). Required
- `--amount <F64>`: Amount of base tokens to spend (e.g. 100.50 USDF). Required

**Output:**
- Prints the transaction signature if successful

**Functionality in Flipcash Program:**
- Calls the `buy` instruction on the Flipcash program.
- Transfers base tokens from the user's ATA to the pool's vault.
- Mints and transfers the equivalent amount of currency tokens to the user

### sell

Sells tokens to the pool in exchange for base tokens (e.g. sell custom currency for USDF).

**Usage:**
```
flipcash-cli sell --mint <PUBKEY> --base-mint <PUBKEY> --amount <F64>
```

**Options:**
- `--mint <PUBKEY>`: Public key of the currency mint to sell. Required
- `--base-mint <PUBKEY>`: Public key of the base mint (e.g. USDF). Required
- `--amount <F64>`: Amount of currency tokens to sell (e.g. 100.50). Required

**Output:**
- Prints the transaction signature if successful

**Functionality in Flipcash Program:**
- Calls the `sell` instruction on the Flipcash program.
- Transfers currency tokens from the user's ATA to the pool's vault (possibly burning them)
- Transfers the equivalent amount of base tokens to the user
- Applies sell fees as configured in the pool

### burn-fees

Burns base tokens (e.g. USDF) accumulated for sell fees

**Usage:**
```
flipcash-cli burn-fees --mint <PUBKEY> --base-mint <PUBKEY>
```

**Options:**
- `--mint <PUBKEY>`: Public key of the currency mint to sell. Required
- `--base-mint <PUBKEY>`: Public key of the base mint (e.g. USDF). Required

**Output:**
- Prints the transaction signature if successful.

**Functionality in Flipcash Program:**
- Calls the `burn_fees` instruction on the Flipcash program
- Burns base tokens from the pool's vault
- Resets fees accumulated to zero

## Examples

1. Create a test base mint on localnet:
   ```
   flipcash-cli create-base-mint --decimals 6 --initial-amount 1000000000000
   ```

2. Create a new currency:
   ```
   flipcash-cli create-currency --name "FlipToken" --symbol "FLIP" --base-mint <USDF_MINT_PUBKEY>
   ```

3. Get currency details:
   ```
   flipcash-cli get-currency --mint <CURRENCY_MINT_PUBKEY>
   ```

4. Buy 100 base units:
   ```
   flipcash-cli buy --mint <CURRENCY_MINT_PUBKEY> --base-mint <USDF_MINT_PUBKEY> --amount 100.0
   ```

5. Sell 50 tokens:
   ```
   flipcash-cli sell --mint <CURRENCY_MINT_PUBKEY> --base-mint <USDF_MINT_PUBKEY> --amount 50.0
   ```

6. Burn fees:
   ```
   flipcash-cli burn-fees --mint <CURRENCY_MINT_PUBKEY> --base-mint <USDF_MINT_PUBKEY>
   ```
