use anyhow::{anyhow, Result};
use brine_fp::UnsignedNumeric;
use jupiter_amm_interface::{
    AccountMap, Amm, AmmContext, KeyedAccount, Quote, QuoteParams, Swap, SwapAndAccountMetas,
    SwapMode, SwapParams,
};
use rust_decimal::Decimal;
use solana_program::{program_pack::Pack, pubkey::Pubkey};
use solana_sdk::instruction::AccountMeta;

use flipcash_api::prelude::{
    from_basis_points, from_numeric, to_numeric, DiscreteExponentialCurve, LiquidityPool,
    MAX_TOKEN_SUPPLY, QUARKS_PER_TOKEN, SCALED_MAX_CUMULATIVE_VALUE, TOKEN_DECIMALS, USDF_BASE_MINT,
};
use flipcash_api::pda::find_vault_pda;

/// In-memory snapshot of one Flipcash pool, kept fresh by Jupiter via
/// [`Amm::update`] and consulted by [`Amm::quote`].
#[derive(Clone, Debug)]
pub struct FlipcashAmm {
    /// Pool PDA — stable, used as Jupiter's `key` for this Amm instance.
    key: Pubkey,
    /// Decoded `LiquidityPool` from the pool account; refreshed each `update`
    /// because `fees_accumulated` changes on every sell.
    pool: LiquidityPool,
    /// Live circulating supply of the currency token (from mint_a). Drives the
    /// curve position used by `quote`.
    target_mint_supply: u64,
    /// Currency-token reserves remaining in the pool's vault_a.
    target_vault_amount: u64,
    /// USDF reserves in the pool's vault_b. The portion above
    /// `pool.fees_accumulated` is the live "value" on the curve.
    base_vault_amount: u64,
    /// Decimals for USDF (mint_b). Cached to convert between raw token units
    /// and the curve's `UnsignedNumeric` value units.
    base_mint_decimals: u8,
}

impl FlipcashAmm {
    /// Reconstruct the value the curve is currently sitting at and the
    /// implied tokens-sold supply, both in `UnsignedNumeric`. These two
    /// quantities feed every quote.
    fn curve_state(&self) -> Result<(UnsignedNumeric, UnsignedNumeric, UnsignedNumeric)> {
        let tokens_left_raw = self.target_vault_amount;
        let supply_from_bonding = MAX_TOKEN_SUPPLY
            .checked_mul(QUARKS_PER_TOKEN)
            .ok_or_else(|| anyhow!("supply overflow"))?
            .checked_sub(tokens_left_raw)
            .ok_or_else(|| anyhow!("vault_a exceeds max supply"))?;

        let value_left_raw = self
            .base_vault_amount
            .checked_sub(self.pool.fees_accumulated)
            .ok_or_else(|| anyhow!("fees_accumulated exceeds base vault balance"))?;

        let supply = to_numeric(supply_from_bonding, TOKEN_DECIMALS).map_err(map_pe)?;
        let tokens_left = to_numeric(tokens_left_raw, TOKEN_DECIMALS).map_err(map_pe)?;
        let value_left = to_numeric(value_left_raw, self.base_mint_decimals).map_err(map_pe)?;

        Ok((supply, tokens_left, value_left))
    }

    /// Buy quote: deposit `in_amount` USDF, receive currency tokens.
    /// Mirrors `buy_common` in `program/src/instruction/buy.rs`.
    fn quote_buy(&self, in_amount_raw: u64) -> Result<Quote> {
        let (supply, tokens_left, current_value) = self.curve_state()?;

        let in_amount = to_numeric(in_amount_raw, self.base_mint_decimals).map_err(map_pe)?;
        let uncapped_new_value = current_value
            .checked_add(&in_amount)
            .ok_or_else(|| anyhow!("value overflow"))?;
        let max_cumulative = UnsignedNumeric::from_scaled_u128(SCALED_MAX_CUMULATIVE_VALUE);
        let capped_new_value = if uncapped_new_value.greater_than(&max_cumulative) {
            max_cumulative
        } else {
            uncapped_new_value
        };
        let capped_in = if capped_new_value.greater_than(&current_value) {
            capped_new_value
                .checked_sub(&current_value)
                .ok_or_else(|| anyhow!("in_amount underflow"))?
        } else {
            UnsignedNumeric::zero()
        };

        let curve = DiscreteExponentialCurve::default();
        let zero = UnsignedNumeric::zero();
        let new_supply = curve
            .value_to_tokens(&zero, &capped_new_value)
            .ok_or_else(|| anyhow!("value_to_tokens failed"))?;
        let mut tokens_bought = new_supply
            .checked_sub(&supply)
            .ok_or_else(|| anyhow!("tokens_bought underflow"))?;
        if tokens_bought.greater_than(&tokens_left) {
            tokens_bought = tokens_left;
        }

        let actual_in_amount_raw = from_numeric(capped_in, self.base_mint_decimals).map_err(map_pe)?;
        let tokens_bought_raw = from_numeric(tokens_bought, TOKEN_DECIMALS).map_err(map_pe)?;

        Ok(Quote {
            in_amount: actual_in_amount_raw,
            out_amount: tokens_bought_raw,
            fee_amount: 0,
            fee_mint: USDF_BASE_MINT,
            fee_pct: Decimal::ZERO,
        })
    }

    /// Sell quote: burn `in_amount` currency tokens, receive USDF (minus fee).
    /// Mirrors `sell_common` in `program/src/instruction/sell.rs`.
    fn quote_sell(&self, in_amount_raw: u64) -> Result<Quote> {
        let (supply, _tokens_left, value_left) = self.curve_state()?;

        if in_amount_raw == 0 {
            return Ok(empty_sell_quote());
        }

        let in_amount = to_numeric(in_amount_raw, TOKEN_DECIMALS).map_err(map_pe)?;
        let new_supply = supply
            .checked_sub(&in_amount)
            .ok_or_else(|| anyhow!("in_amount exceeds bonded supply"))?;
        let after_fee_rate = from_basis_points(
            10_000u16
                .checked_sub(self.pool.sell_fee)
                .ok_or_else(|| anyhow!("sell_fee > 10000 bps"))?,
        )
        .map_err(map_pe)?;

        let curve = DiscreteExponentialCurve::default();
        let zero = UnsignedNumeric::zero();
        let new_value = curve
            .tokens_to_value(&zero, &new_supply)
            .ok_or_else(|| anyhow!("tokens_to_value failed"))?;

        let mut total_sell_value = if new_value.less_than(&value_left) {
            value_left
                .checked_sub(&new_value)
                .ok_or_else(|| anyhow!("sell_value underflow"))?
        } else {
            UnsignedNumeric::zero()
        };
        if total_sell_value.greater_than(&value_left) {
            total_sell_value = value_left.clone();
        }

        let sell_value_after_fee = total_sell_value
            .checked_mul(&after_fee_rate)
            .ok_or_else(|| anyhow!("fee multiplication overflow"))?;

        let total_sell_value_raw =
            from_numeric(total_sell_value, self.base_mint_decimals).map_err(map_pe)?;
        let (sell_value_after_fee_raw, fee_amount_raw) = if self.pool.sell_fee > 0 {
            let after = from_numeric(sell_value_after_fee, self.base_mint_decimals).map_err(map_pe)?;
            let fee = total_sell_value_raw
                .checked_sub(after)
                .ok_or_else(|| anyhow!("fee underflow"))?;
            (after, fee)
        } else {
            (total_sell_value_raw, 0)
        };

        let fee_pct = bps_to_decimal(self.pool.sell_fee);

        Ok(Quote {
            in_amount: in_amount_raw,
            out_amount: sell_value_after_fee_raw,
            fee_amount: fee_amount_raw,
            fee_mint: USDF_BASE_MINT,
            fee_pct,
        })
    }

    /// ExactOut buy quote: deliver at least `out_amount` currency tokens,
    /// derive the USDF input.
    ///
    /// Algebraic inversion of `value_to_tokens(0, current_value + V) - supply
    /// = X` gives `V = tokens_to_value(0, supply + X) - current_value`. But
    /// the curve's `value_to_tokens` is a fixed-point division, so the
    /// round-trip loses up to a couple token-quarks; on top of that,
    /// `from_numeric` truncates `V` to whole base-mint quarks. So we treat the
    /// algebraic answer as a candidate, forward-simulate via `quote_buy`, and
    /// bump the input by 1 quark at a time until the on-chain math actually
    /// delivers the requested output. Returns the forward-simulated Quote, so
    /// `out_amount` reflects what will actually land on chain (possibly a few
    /// token-quarks above the request).
    fn quote_buy_exact_out(&self, out_amount_raw: u64) -> Result<Quote> {
        let (supply, tokens_left, current_value) = self.curve_state()?;

        if out_amount_raw == 0 {
            return Ok(empty_buy_quote());
        }

        let out_amount = to_numeric(out_amount_raw, TOKEN_DECIMALS).map_err(map_pe)?;
        if out_amount.greater_than(&tokens_left) {
            return Err(anyhow!(
                "requested out_amount exceeds available pool tokens"
            ));
        }

        let new_supply = supply
            .checked_add(&out_amount)
            .ok_or_else(|| anyhow!("supply overflow"))?;

        let curve = DiscreteExponentialCurve::default();
        let zero = UnsignedNumeric::zero();
        let new_total_value = curve
            .tokens_to_value(&zero, &new_supply)
            .ok_or_else(|| anyhow!("tokens_to_value failed"))?;

        let max_cumulative = UnsignedNumeric::from_scaled_u128(SCALED_MAX_CUMULATIVE_VALUE);
        if new_total_value.greater_than(&max_cumulative) {
            return Err(anyhow!(
                "requested out_amount would exceed max cumulative curve value"
            ));
        }

        let in_amount = new_total_value
            .checked_sub(&current_value)
            .ok_or_else(|| anyhow!(
                "current_value > new_total_value — pool drift exceeds inversion limit"
            ))?;

        let candidate_in =
            from_numeric(in_amount, self.base_mint_decimals).map_err(map_pe)?;

        self.exact_out_converge(candidate_in, out_amount_raw, |amt| self.quote_buy(amt))
    }

    /// ExactOut sell quote: deliver exactly `out_amount` USDF (post-fee),
    /// derive the currency-token input.
    ///
    /// Forward: `out = (value_left - tokens_to_value(0, supply - in)) * after_fee_rate`.
    /// Inverted: divide by `after_fee_rate` to get total_sell_value, subtract
    /// from `value_left` to get the curve's target new_value, then call
    /// `value_to_tokens` to map that back to a supply, and the input is
    /// `supply - new_supply`.
    fn quote_sell_exact_out(&self, out_amount_raw: u64) -> Result<Quote> {
        let (supply, _tokens_left, value_left) = self.curve_state()?;

        if out_amount_raw == 0 {
            return Ok(empty_sell_quote());
        }

        let after_fee_rate = from_basis_points(
            10_000u16
                .checked_sub(self.pool.sell_fee)
                .ok_or_else(|| anyhow!("sell_fee > 10000 bps"))?,
        )
        .map_err(map_pe)?;

        let out_amount = to_numeric(out_amount_raw, self.base_mint_decimals).map_err(map_pe)?;

        // total_sell_value = out_amount / after_fee_rate (skip the divide if
        // there's no fee — multiplying by 1.0 round-trips lossy in fixed point).
        let total_sell_value = if self.pool.sell_fee > 0 {
            out_amount
                .checked_div(&after_fee_rate)
                .ok_or_else(|| anyhow!("fee inversion failed"))?
        } else {
            out_amount.clone()
        };

        if total_sell_value.greater_than(&value_left) {
            return Err(anyhow!(
                "requested out_amount exceeds pool USDF reserves after fee"
            ));
        }

        let new_value = value_left
            .checked_sub(&total_sell_value)
            .ok_or_else(|| anyhow!("new_value underflow"))?;

        let curve = DiscreteExponentialCurve::default();
        let zero = UnsignedNumeric::zero();
        let new_supply = curve
            .value_to_tokens(&zero, &new_value)
            .ok_or_else(|| anyhow!("value_to_tokens failed"))?;

        let in_amount = supply
            .checked_sub(&new_supply)
            .ok_or_else(|| anyhow!("in_amount underflow — bonded supply lower than target"))?;

        let candidate_in = from_numeric(in_amount, TOKEN_DECIMALS).map_err(map_pe)?;

        self.exact_out_converge(candidate_in, out_amount_raw, |amt| self.quote_sell(amt))
    }

    /// Walk `candidate_in` upward in 1-quark steps until forward-simulating
    /// the corresponding ExactIn quote produces at least `target_out`. The
    /// returned Quote reflects what on-chain math would actually deliver.
    ///
    /// The discrete curve's `value_to_tokens` and the fee-rate division both
    /// lose sub-quark precision, so the algebraic inverse alone may undershoot
    /// by a small handful of quarks. A handful of bumps closes the gap; if it
    /// somehow doesn't, we error rather than under-deliver.
    fn exact_out_converge<F>(
        &self,
        candidate_in: u64,
        target_out: u64,
        forward: F,
    ) -> Result<Quote>
    where
        F: Fn(u64) -> Result<Quote>,
    {
        const MAX_ITERATIONS: u32 = 16;
        let mut in_amount = candidate_in;
        let mut last_out: Option<u64> = None;
        for _ in 0..MAX_ITERATIONS {
            let q = forward(in_amount)?;
            if q.out_amount >= target_out {
                return Ok(q);
            }
            // If a 1-quark bump didn't move the output at all, the curve is
            // too flat to converge here — error out before looping forever.
            if last_out == Some(q.out_amount) && in_amount > candidate_in {
                return Err(anyhow!(
                    "ExactOut convergence stalled at out={} below target {}",
                    q.out_amount,
                    target_out
                ));
            }
            last_out = Some(q.out_amount);
            in_amount = in_amount
                .checked_add(1)
                .ok_or_else(|| anyhow!("in_amount overflow"))?;
        }
        Err(anyhow!(
            "ExactOut did not converge in {} iterations",
            MAX_ITERATIONS
        ))
    }
}

impl Amm for FlipcashAmm {
    fn from_keyed_account(
        keyed_account: &KeyedAccount,
        _amm_context: &AmmContext,
    ) -> Result<Self> {
        let pool = LiquidityPool::unpack(&keyed_account.account.data)
            .map_err(|e| anyhow!("failed to unpack LiquidityPool: {e}"))
            .copied()?;

        // Sanity-check the hardcoded base mint up front; if a pool's mint_b
        // ever drifts, every quote would be wrong.
        if pool.mint_b != USDF_BASE_MINT {
            return Err(anyhow!(
                "pool {} has unexpected base mint {}; expected USDF {}",
                keyed_account.key,
                pool.mint_b,
                USDF_BASE_MINT
            ));
        }

        Ok(Self {
            key: keyed_account.key,
            pool,
            target_mint_supply: 0,
            target_vault_amount: 0,
            base_vault_amount: 0,
            base_mint_decimals: 0,
        })
    }

    fn label(&self) -> String {
        "Flipcash".to_string()
    }

    fn program_id(&self) -> Pubkey {
        flipcash_api::ID
    }

    fn key(&self) -> Pubkey {
        self.key
    }

    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        vec![self.pool.mint_a, self.pool.mint_b]
    }

    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        vec![
            self.key,           // pool — fees_accumulated changes per sell
            self.pool.mint_a,   // currency mint — supply changes every trade
            self.pool.mint_b,   // base mint — for decimals (read once, cheap to refresh)
            self.pool.vault_a,  // currency reserves
            self.pool.vault_b,  // USDF reserves
        ]
    }

    fn update(&mut self, account_map: &AccountMap) -> Result<()> {
        let pool_account = account_map
            .get(&self.key)
            .ok_or_else(|| anyhow!("missing pool account in update"))?;
        self.pool = LiquidityPool::unpack(&pool_account.data)
            .map_err(|e| anyhow!("failed to unpack pool on update: {e}"))
            .copied()?;

        let mint_a = account_map
            .get(&self.pool.mint_a)
            .ok_or_else(|| anyhow!("missing mint_a in update"))?;
        let mint_a = spl_token::state::Mint::unpack(&mint_a.data)
            .map_err(|e| anyhow!("failed to unpack mint_a: {e}"))?;
        self.target_mint_supply = mint_a.supply;

        let mint_b = account_map
            .get(&self.pool.mint_b)
            .ok_or_else(|| anyhow!("missing mint_b in update"))?;
        let mint_b = spl_token::state::Mint::unpack(&mint_b.data)
            .map_err(|e| anyhow!("failed to unpack mint_b: {e}"))?;
        self.base_mint_decimals = mint_b.decimals;

        let vault_a = account_map
            .get(&self.pool.vault_a)
            .ok_or_else(|| anyhow!("missing vault_a in update"))?;
        let vault_a = spl_token::state::Account::unpack(&vault_a.data)
            .map_err(|e| anyhow!("failed to unpack vault_a: {e}"))?;
        self.target_vault_amount = vault_a.amount;

        let vault_b = account_map
            .get(&self.pool.vault_b)
            .ok_or_else(|| anyhow!("missing vault_b in update"))?;
        let vault_b = spl_token::state::Account::unpack(&vault_b.data)
            .map_err(|e| anyhow!("failed to unpack vault_b: {e}"))?;
        self.base_vault_amount = vault_b.amount;

        Ok(())
    }

    fn quote(&self, params: &QuoteParams) -> Result<Quote> {
        let is_buy =
            params.input_mint == self.pool.mint_b && params.output_mint == self.pool.mint_a;
        let is_sell =
            params.input_mint == self.pool.mint_a && params.output_mint == self.pool.mint_b;
        if !is_buy && !is_sell {
            return Err(anyhow!(
                "unsupported pair: {} -> {}",
                params.input_mint,
                params.output_mint
            ));
        }

        match (params.swap_mode, is_buy) {
            (SwapMode::ExactIn, true) => self.quote_buy(params.amount),
            (SwapMode::ExactIn, false) => self.quote_sell(params.amount),
            (SwapMode::ExactOut, true) => self.quote_buy_exact_out(params.amount),
            (SwapMode::ExactOut, false) => self.quote_sell_exact_out(params.amount),
        }
    }

    fn supports_exact_out(&self) -> bool {
        true
    }

    fn get_swap_and_account_metas(&self, params: &SwapParams) -> Result<SwapAndAccountMetas> {
        let target_mint = self.pool.mint_a;
        let base_mint = self.pool.mint_b;
        let (vault_a, _) = find_vault_pda(&self.key, &target_mint);
        let (vault_b, _) = find_vault_pda(&self.key, &base_mint);

        // Direction determines which side is mut on the pool account: sells
        // mutate `fees_accumulated`, buys don't (see buy_common/sell_common).
        let (pool_meta, user_target, user_base) =
            if params.source_mint == base_mint && params.destination_mint == target_mint {
                (
                    AccountMeta::new_readonly(self.key, false),
                    params.destination_token_account,
                    params.source_token_account,
                )
            } else if params.source_mint == target_mint && params.destination_mint == base_mint {
                (
                    AccountMeta::new(self.key, false),
                    params.source_token_account,
                    params.destination_token_account,
                )
            } else {
                return Err(anyhow!("unsupported swap direction"));
            };

        let account_metas = vec![
            AccountMeta::new(params.token_transfer_authority, true),
            pool_meta,
            AccountMeta::new_readonly(target_mint, false),
            AccountMeta::new_readonly(base_mint, false),
            AccountMeta::new(vault_a, false),
            AccountMeta::new(vault_b, false),
            AccountMeta::new(user_target, false),
            AccountMeta::new(user_base, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ];

        // NOTE: Jupiter's `Swap` enum lives in `jupiter-amm-interface`. A
        // dedicated `Swap::Flipcash` variant needs to land in their crate
        // before this can be routed end-to-end. Until then, the adapter
        // quotes correctly but the swap-instruction tag here is a placeholder
        // chosen for compatibility with the router's serializer.
        Ok(SwapAndAccountMetas {
            swap: Swap::TokenSwap,
            account_metas,
        })
    }

    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }
}

fn empty_sell_quote() -> Quote {
    Quote {
        in_amount: 0,
        out_amount: 0,
        fee_amount: 0,
        fee_mint: USDF_BASE_MINT,
        fee_pct: Decimal::ZERO,
    }
}

fn empty_buy_quote() -> Quote {
    Quote {
        in_amount: 0,
        out_amount: 0,
        fee_amount: 0,
        fee_mint: USDF_BASE_MINT,
        fee_pct: Decimal::ZERO,
    }
}

fn bps_to_decimal(bps: u16) -> Decimal {
    // 1 bps = 0.0001
    Decimal::new(bps as i64, 4)
}

fn map_pe(e: solana_program::program_error::ProgramError) -> anyhow::Error {
    anyhow!("flipcash math error: {e}")
}
