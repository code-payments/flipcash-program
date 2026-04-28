//! Parity tests: the adapter's quote must match the program's actual output
//! token-for-token across a range of pool configurations and trade sizes.
//!
//! Requires `target/deploy/flipcash.so` — run `make build` first.

#![cfg(test)]

mod common;

use common::PoolHarness;
use flipcash_api::prelude::*;
use jupiter_amm_interface::{
    Amm, FeeMode, Quote, QuoteParams, SwapAndAccountMetas, SwapMode, SwapParams,
};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signer::Signer, transaction::Transaction};

fn buy_quote(h: &PoolHarness, amount: u64) -> Quote {
    h.amm()
        .quote(&QuoteParams {
            amount,
            input_mint: h.usdf,
            output_mint: h.mint_pda,
            swap_mode: SwapMode::ExactIn,
            fee_mode: FeeMode::Normal,
        })
        .unwrap()
}

fn sell_quote(h: &PoolHarness, amount: u64) -> Quote {
    h.amm()
        .quote(&QuoteParams {
            amount,
            input_mint: h.mint_pda,
            output_mint: h.usdf,
            swap_mode: SwapMode::ExactIn,
            fee_mode: FeeMode::Normal,
        })
        .unwrap()
}

/// Quote a buy, execute it, and assert exact balance-delta parity.
fn assert_buy_parity(h: &mut PoolHarness, in_amount: u64) {
    let quote = buy_quote(h, in_amount);
    let target_before = h.target_balance();
    let base_before = h.base_balance();
    h.buy(in_amount);
    let tokens_received = h.target_balance() - target_before;
    let usdf_spent = base_before - h.base_balance();
    assert_eq!(quote.out_amount, tokens_received, "buy out_amount");
    assert_eq!(quote.in_amount, usdf_spent, "buy in_amount");
    assert_eq!(quote.fee_amount, 0, "buys are fee-free");
}

/// Quote a sell, execute it, and assert exact balance-delta and fee parity.
fn assert_sell_parity(h: &mut PoolHarness, in_amount: u64) {
    let quote = sell_quote(h, in_amount);
    let target_before = h.target_balance();
    let base_before = h.base_balance();
    let fees_before = h.fees_accumulated();
    h.sell(in_amount);
    let tokens_sold = target_before - h.target_balance();
    let usdf_received = h.base_balance() - base_before;
    let fee_delta = h.fees_accumulated() - fees_before;
    assert_eq!(quote.in_amount, tokens_sold, "sell in_amount");
    assert_eq!(quote.out_amount, usdf_received, "sell out_amount");
    assert_eq!(quote.fee_amount, fee_delta, "sell fee_amount");
}

#[test]
fn buy_at_supply_zero() {
    let mut h = PoolHarness::new(100, 50_000_000_000);
    assert_buy_parity(&mut h, 1_000_000_000);
}

#[test]
fn buy_small_within_first_step() {
    let mut h = PoolHarness::new(100, 50_000_000_000);
    // $0.50 — small enough to stay inside the first 100-token step.
    assert_buy_parity(&mut h, 500_000);
}

#[test]
fn buy_cross_many_steps_after_priming() {
    let mut h = PoolHarness::new(100, 200_000_000_000);
    h.buy(20_000_000_000);
    assert_buy_parity(&mut h, 50_000_000_000);
}

#[test]
fn sell_partial_position() {
    let mut h = PoolHarness::new(100, 50_000_000_000);
    h.buy(10_000_000_000);
    let half = h.target_balance() / 2;
    assert_sell_parity(&mut h, half);
}

#[test]
fn sell_full_position() {
    let mut h = PoolHarness::new(100, 50_000_000_000);
    h.buy(10_000_000_000);
    let all = h.target_balance();
    assert_sell_parity(&mut h, all);
}

// NOTE: variable-fee tests (e.g. 0 or 500 bps) aren't included — the program
// enforces sell_fee == 100 bps at pool init. If that constraint relaxes,
// add cases here.

/// Repeated alternating trades — confirms the adapter stays in sync as the
/// pool's `fees_accumulated`, vault balances, and mint supply all evolve.
#[test]
fn alternating_buy_sell_stays_in_sync() {
    let mut h = PoolHarness::new(100, 100_000_000_000);
    for i in 0..5 {
        let buy_amount = 2_000_000_000u64 * (i as u64 + 1);
        assert_buy_parity(&mut h, buy_amount);
        let sell_amount = h.target_balance() / 4;
        assert_sell_parity(&mut h, sell_amount);
    }
}

/// Validates that `get_swap_and_account_metas` returns the exact account
/// ordering & flags the program expects, by feeding the metas into a real
/// transaction (with the program's own ix data) and asserting it lands.
#[test]
fn swap_instruction_metas_match_sdk_and_execute() {
    let mut h = PoolHarness::new(100, 50_000_000_000);
    h.buy(1_000_000_000); // mid-curve

    let amm = h.amm();
    let buy_in = 2_000_000_000u64;
    let jupiter_program_id = Pubkey::default();

    let SwapAndAccountMetas { account_metas, .. } = amm
        .get_swap_and_account_metas(&SwapParams {
            swap_mode: SwapMode::ExactIn,
            in_amount: buy_in,
            out_amount: 0,
            source_mint: h.usdf,
            destination_mint: h.mint_pda,
            source_token_account: h.user_base,
            destination_token_account: h.user_target,
            token_transfer_authority: h.user.pubkey(),
            user: h.user.pubkey(),
            payer: h.user.pubkey(),
            quote_mint_to_referrer: None,
            jupiter_program_id: &jupiter_program_id,
            missing_dynamic_accounts_as_default: false,
        })
        .unwrap();

    // Adapter's metas must be byte-identical to the SDK builder's. If this
    // ever drifts, Jupiter would route an instruction the program rejects.
    let reference = build_buy_tokens_ix(
        h.user.pubkey(),
        h.pool_pda,
        h.mint_pda,
        h.usdf,
        buy_in,
        0,
        h.user_target,
        h.user_base,
    );
    assert_eq!(account_metas.len(), reference.accounts.len());
    for (i, (got, expected)) in account_metas.iter().zip(reference.accounts.iter()).enumerate() {
        assert_eq!(got.pubkey, expected.pubkey, "meta[{i}] pubkey");
        assert_eq!(got.is_signer, expected.is_signer, "meta[{i}] is_signer");
        assert_eq!(got.is_writable, expected.is_writable, "meta[{i}] is_writable");
    }

    // Belt-and-suspenders: actually execute the buy via the adapter's metas.
    let ix = Instruction {
        program_id: flipcash_api::ID,
        accounts: account_metas,
        data: BuyTokensIx::from_struct(ParsedBuyTokensIx {
            in_amount: buy_in,
            min_amount_out: 0,
        })
        .to_bytes(),
    };
    let target_before = h.target_balance();
    let bh = h.svm.latest_blockhash();
    let tx =
        Transaction::new_signed_with_payer(&[ix], Some(&h.user.pubkey()), &[&h.user], bh);
    h.svm.send_transaction(tx).unwrap();
    assert!(
        h.target_balance() > target_before,
        "buy via adapter metas should succeed"
    );
}

// ---- ExactOut parity ---------------------------------------------------
//
// For each direction, we ask the adapter "what input gives me exactly Y out?",
// then run the program in its native ExactIn mode with that input and check
// the actual output equals Y. The discrete curve's `tokens_to_value` and
// `value_to_tokens` are algebraic inverses on step boundaries, and inputs are
// quantized via `from_numeric`, so in practice the round-trip is exact.

fn buy_exact_out_quote(h: &PoolHarness, out_amount: u64) -> Quote {
    h.amm()
        .quote(&QuoteParams {
            amount: out_amount,
            input_mint: h.usdf,
            output_mint: h.mint_pda,
            swap_mode: SwapMode::ExactOut,
            fee_mode: FeeMode::Normal,
        })
        .unwrap()
}

fn sell_exact_out_quote(h: &PoolHarness, out_amount: u64) -> Quote {
    h.amm()
        .quote(&QuoteParams {
            amount: out_amount,
            input_mint: h.mint_pda,
            output_mint: h.usdf,
            swap_mode: SwapMode::ExactOut,
            fee_mode: FeeMode::Normal,
        })
        .unwrap()
}

/// ExactOut buy from a fresh pool: candidate inverse + forward-sim convergence
/// must deliver at least the target, and on-chain execution must match the
/// quote exactly (forward-sim is the on-chain math).
#[test]
fn buy_exact_out_at_supply_zero() {
    let mut h = PoolHarness::new(100, 50_000_000_000);
    let target_out = 100_000_000_000u64; // 10 tokens (10 decimals)
    let quote = buy_exact_out_quote(&h, target_out);
    assert!(
        quote.out_amount >= target_out,
        "quote {} must meet target {}",
        quote.out_amount,
        target_out
    );

    let target_before = h.target_balance();
    let base_before = h.base_balance();
    h.buy(quote.in_amount);
    let actual_received = h.target_balance() - target_before;
    let actual_spent = base_before - h.base_balance();

    assert_eq!(actual_spent, quote.in_amount, "buy_exact_out in_amount");
    assert_eq!(actual_received, quote.out_amount, "quote matches actual");
    assert!(actual_received >= target_out);
}

#[test]
fn buy_exact_out_after_priming() {
    let mut h = PoolHarness::new(100, 100_000_000_000);
    h.buy(5_000_000_000);
    let target_out = h.target_balance() / 4;
    let quote = buy_exact_out_quote(&h, target_out);
    assert!(quote.out_amount >= target_out);

    let target_before = h.target_balance();
    let base_before = h.base_balance();
    h.buy(quote.in_amount);

    assert_eq!(base_before - h.base_balance(), quote.in_amount);
    assert_eq!(h.target_balance() - target_before, quote.out_amount);
    assert!(h.target_balance() - target_before >= target_out);
}

#[test]
fn sell_exact_out_after_priming() {
    let mut h = PoolHarness::new(100, 50_000_000_000);
    h.buy(10_000_000_000);
    let target_usdf = 1_000_000u64; // $1
    let quote = sell_exact_out_quote(&h, target_usdf);
    assert!(quote.out_amount >= target_usdf);

    let target_before = h.target_balance();
    let base_before = h.base_balance();
    let fees_before = h.fees_accumulated();
    h.sell(quote.in_amount);
    let actual_sold = target_before - h.target_balance();
    let actual_received = h.base_balance() - base_before;
    let fee_delta = h.fees_accumulated() - fees_before;

    assert_eq!(actual_sold, quote.in_amount, "sell_exact_out in_amount");
    assert_eq!(actual_received, quote.out_amount, "quote matches actual");
    assert!(actual_received >= target_usdf);
    assert_eq!(fee_delta, quote.fee_amount, "sell_exact_out fee_amount");
}

/// ExactIn → ExactOut → ExactIn cycle: quote a buy, take its `out_amount`,
/// quote ExactOut for that target, and confirm the inverted `in_amount`
/// matches the original input.
#[test]
fn exact_in_out_roundtrip_buy() {
    let mut h = PoolHarness::new(100, 50_000_000_000);
    h.buy(2_000_000_000);

    let amm = h.amm();
    let original_in = 5_000_000_000u64;
    let exact_in_quote = amm
        .quote(&QuoteParams {
            amount: original_in,
            input_mint: h.usdf,
            output_mint: h.mint_pda,
            swap_mode: SwapMode::ExactIn,
            fee_mode: FeeMode::Normal,
        })
        .unwrap();

    let exact_out_quote = amm
        .quote(&QuoteParams {
            amount: exact_in_quote.out_amount,
            input_mint: h.usdf,
            output_mint: h.mint_pda,
            swap_mode: SwapMode::ExactOut,
            fee_mode: FeeMode::Normal,
        })
        .unwrap();

    // ExactOut converges by bumping the input upward, so its in_amount may
    // exceed the original ExactIn input by a small handful of quarks; the
    // resulting out_amount is whatever the on-chain math actually produces.
    assert!(exact_out_quote.in_amount >= exact_in_quote.in_amount);
    assert!(
        exact_out_quote.in_amount - exact_in_quote.in_amount <= 4,
        "round-trip in_amount drift: in={} -> out_quote in={}",
        exact_in_quote.in_amount,
        exact_out_quote.in_amount
    );
    assert!(exact_out_quote.out_amount >= exact_in_quote.out_amount);
}

#[test]
fn buy_exact_out_errors_when_exceeding_pool_supply() {
    let h = PoolHarness::new(100, 50_000_000_000);
    let amm = h.amm();
    let too_many = (MAX_TOKEN_SUPPLY as u64) * QUARKS_PER_TOKEN + 1;
    let result = amm.quote(&QuoteParams {
        amount: too_many,
        input_mint: h.usdf,
        output_mint: h.mint_pda,
        swap_mode: SwapMode::ExactOut,
        fee_mode: FeeMode::Normal,
    });
    assert!(result.is_err(), "should error when out_amount > tokens_left");
}

#[test]
fn sell_exact_out_errors_when_exceeding_reserves() {
    let mut h = PoolHarness::new(100, 50_000_000_000);
    h.buy(1_000_000_000); // small priming buy → tiny vault_b
    let amm = h.amm();
    // Ask for more USDF than the pool has.
    let too_much = 100_000_000_000u64;
    let result = amm.quote(&QuoteParams {
        amount: too_much,
        input_mint: h.mint_pda,
        output_mint: h.usdf,
        swap_mode: SwapMode::ExactOut,
        fee_mode: FeeMode::Normal,
    });
    assert!(result.is_err(), "should error when out exceeds reserves");
}

/// Sells mutate `pool.fees_accumulated`, buys don't. Verify that
/// `get_swap_and_account_metas` flags the pool account correctly per direction
/// — getting this wrong would either fail (pool not writable on sell) or
/// over-permission accounts (security smell on buy).
#[test]
fn swap_instruction_pool_writability_per_direction() {
    let mut h = PoolHarness::new(100, 50_000_000_000);
    h.buy(1_000_000_000);

    let amm = h.amm();
    let jupiter_program_id = Pubkey::default();
    let make_params = |source_mint, destination_mint, src_ata, dst_ata| SwapParams {
        swap_mode: SwapMode::ExactIn,
        in_amount: 1_000,
        out_amount: 0,
        source_mint,
        destination_mint,
        source_token_account: src_ata,
        destination_token_account: dst_ata,
        token_transfer_authority: h.user.pubkey(),
        user: h.user.pubkey(),
        payer: h.user.pubkey(),
        quote_mint_to_referrer: None,
        jupiter_program_id: &jupiter_program_id,
        missing_dynamic_accounts_as_default: false,
    };

    // Buy: pool is read-only.
    let SwapAndAccountMetas { account_metas, .. } = amm
        .get_swap_and_account_metas(&make_params(
            h.usdf,
            h.mint_pda,
            h.user_base,
            h.user_target,
        ))
        .unwrap();
    let pool_meta = account_metas.iter().find(|m| m.pubkey == h.pool_pda).unwrap();
    assert!(!pool_meta.is_writable, "pool should be read-only on buy");

    // Sell: pool is writable.
    let SwapAndAccountMetas { account_metas, .. } = amm
        .get_swap_and_account_metas(&make_params(
            h.mint_pda,
            h.usdf,
            h.user_target,
            h.user_base,
        ))
        .unwrap();
    let pool_meta = account_metas.iter().find(|m| m.pubkey == h.pool_pda).unwrap();
    assert!(pool_meta.is_writable, "pool should be writable on sell");
}
