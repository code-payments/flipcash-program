# BBB Bounty Hunter Guide

## Quick Start - Get USDF

### Option 1: Flipcash CLI (Recommended)
```bash
cargo run --bin flipcash -- swap-to-usdf --amount 100 --input sol
```

### Option 2: Jupiter UI
Visit https://jup.ag and swap SOL/USDC for USDF (`5AMAA9JV9H97YYVxx8F6FsCMmTwXSuTTQneiup4RYAUQ`)

### Option 3: Node.js Script
```bash
npm install @solana/web3.js
node scripts/get-usdf.js --amount 100
```

## USDF Info
- **Mint:** `5AMAA9JV9H97YYVxx8F6FsCMmTwXSuTTQneiup4RYAUQ`
- **Recommended for testing:** 100-500 USDF
- **First price boundary:** ~$11,376 USDF

## Testing Commands
```bash
# Create currency
./target/release/flipcash --cluster m create-currency --name "Test" --symbol "TST"

# Buy tokens
./target/release/flipcash --cluster m buy --mint <MINT> --amount 100

# Sell tokens  
./target/release/flipcash --cluster m sell --mint <MINT> --amount 50

# Get info
./target/release/flipcash --cluster m get-currency --mint <MINT>
```

## Contract Addresses
- **Program:** `ccJYP5gjZqcEHaphcxAZvkxCrnTVfYMjyhSYkpQtf8Z`
- **USDF:** `5AMAA9JV9H97YYVxx8F6FsCMmTwXSuTTQneiup4RYAUQ`

## Bonding Curve
- Start: $0.01 | End: $1,000,000
- Supply: 21M tokens
- See `api/src/curve.rs`

Good luck! 🌶️
