use brine_fp::UnsignedNumeric;
use steel::*;
use flipcash_api::prelude::*;

pub fn process_sell_tokens(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let raw_args = SellTokensIx::try_from_bytes(data)?;
    let args = raw_args.to_struct();

    let [
        seller_info,
        pool_info,
        target_mint_info,
        base_mint_info,
        target_vault_info,
        base_vault_info,
        seller_target_info,
        seller_base_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    //solana_program::msg!("Args: {:?}", args);

    let pool = pool_info.as_account::<LiquidityPool>(&flipcash_api::ID)?;

    seller_base_info.as_token_account()?
        .assert(|t| t.owner().eq(seller_info.key))?
        .assert(|t| t.mint().eq(base_mint_info.key))?;

    let value_after_fee_raw= sell_common(
        seller_info,
        pool_info,
        target_mint_info,
        base_mint_info,
        target_vault_info,
        base_vault_info,
        seller_target_info,
        seller_base_info,
        token_program_info,
        pool,
        args.in_amount,
        args.min_amount_out,
    )?;

    transfer_signed_with_bump(
        base_vault_info,
        base_vault_info,
        seller_base_info,
        token_program_info,
        value_after_fee_raw,
        &[
            TREASURY,
            pool_info.key.as_ref(),
            base_mint_info.key.as_ref()
        ],
        pool.vault_b_bump,
    )?;

    Ok(())
}

pub fn process_sell_and_deposit_into_vm(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let raw_args = SellAndDepositIntoVmIx::try_from_bytes(data)?;
    let args = raw_args.to_struct();

    let [
        seller_info,
        pool_info,
        target_mint_info,
        base_mint_info,
        target_vault_info,
        base_vault_info,
        seller_target_info,
        vm_authority_info,
        vm_info,
        vm_memory_info,
        vm_omnibus_info,
        vta_owner_info,
        token_program_info,
        vm_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    //solana_program::msg!("Args: {:?}", args);

    check_mut(vm_authority_info)?;
    check_mut(vm_info)?;
    check_mut(vm_memory_info)?;
    check_program(vm_program_info, &VM_PROGRAM_ID)?;

    let pool = pool_info.as_account::<LiquidityPool>(&flipcash_api::ID)?;

    vm_omnibus_info.as_token_account()?
        .assert(|t| t.mint().eq(base_mint_info.key))?;

    let value_after_fee_raw= sell_common(
        seller_info,
        pool_info,
        target_mint_info,
        base_mint_info,
        target_vault_info,
        base_vault_info,
        seller_target_info,
        vm_omnibus_info,
        token_program_info,
        pool,
        args.in_amount,
        args.min_amount_out,
    )?;

    deposit_into_vm(
        vm_authority_info,
        vm_info,
        vm_memory_info,
        base_vault_info,
        base_vault_info,
        vta_owner_info,
        vm_omnibus_info,
        args.vm_memory_index,
        value_after_fee_raw,
        &[
            TREASURY,
            pool_info.key.as_ref(),
            base_mint_info.key.as_ref(),
            &[pool.vault_b_bump]
        ],
        token_program_info,
        vm_program_info,
    )?;

    Ok(())
}

// Sell ixn common utility that executes everything but transfering the value
// received for selling tokens to the intended destination.
fn sell_common<'info>(
    seller_info: &AccountInfo<'info>,
    pool_info: &AccountInfo<'info>,
    target_mint_info: &AccountInfo<'info>,
    base_mint_info: &AccountInfo<'info>,
    target_vault_info: &AccountInfo<'info>,
    base_vault_info: &AccountInfo<'info>,
    seller_target_info: &AccountInfo<'info>,
    seller_base_info: &AccountInfo<'info>,
    token_program_info: &AccountInfo<'info>,
    pool: &LiquidityPool,
    in_amount_arg: u64,
    min_amount_out_arg: u64,
) -> Result<u64, ProgramError>{
    // Basic checks
    check_signer(seller_info)?;
    check_mut(base_mint_info)?;
    check_mut(target_vault_info)?;
    check_mut(base_vault_info)?;
    check_mut(seller_target_info)?;
    check_mut(seller_base_info)?;
    check_program(token_program_info, &spl_token::id())?;

    let base_mint = base_mint_info.as_mint()?;
    let seller_target = seller_target_info.as_token_account()?;
    let target_vault = target_vault_info.as_token_account()?;
    let base_vault = base_vault_info.as_token_account()?;

    seller_target
        .assert(|t| t.owner().eq(seller_info.key))?
        .assert(|t| t.mint().eq(target_mint_info.key))?;

    check_condition(
        pool.mint_a == *target_mint_info.key && pool.mint_b == *base_mint_info.key,
        "Invalid mint accounts"
    )?;
    check_condition(
        pool.vault_a == *target_vault_info.key && pool.vault_b == *base_vault_info.key,
        "Invalid vault accounts"
    )?;

    let mint_a_decimals = TOKEN_DECIMALS;
    let mint_b_decimals = base_mint.decimals();

    let tokens_left_raw = target_vault.amount();
    let supply_from_bonding = MAX_TOKEN_SUPPLY
        .checked_mul(QUARKS_PER_TOKEN)
        .ok_or(ProgramError::InvalidArgument)?
        .checked_sub(tokens_left_raw)
        .ok_or(ProgramError::InvalidArgument)?;

    let value_left_raw = base_vault.amount();

    let mut in_amount_raw = in_amount_arg;
    if in_amount_raw == 0 {
        in_amount_raw = seller_target.amount();
    }

    let in_amount = to_numeric(in_amount_raw, mint_a_decimals)?;
    let new_supply = to_numeric(supply_from_bonding, mint_a_decimals)?
        .checked_sub(&in_amount)
        .unwrap();
    let value_left = to_numeric(value_left_raw, mint_b_decimals)?;
    let fee_rate = from_basis_points(pool.sell_fee)?;

    let curve = DiscreteExponentialCurve::default();
    let zero = UnsignedNumeric::zero();
    let new_value = curve.tokens_to_value(&zero, &new_supply)
        .ok_or(ProgramError::InvalidArgument)?;

    let mut total_sell_value = value_left
        .checked_sub(&new_value)
        .ok_or(ProgramError::InvalidArgument)?;
    if total_sell_value.greater_than(&value_left) {
        total_sell_value = value_left
    }

    let fee_amount = total_sell_value
        .checked_mul(&fee_rate)
        .ok_or(ProgramError::InvalidArgument)?;
    let sell_value_after_fee = total_sell_value
        .checked_sub(&fee_amount)
        .ok_or(ProgramError::InvalidArgument)?;

    //solana_program::msg!("selling: {}", in_amount.to_string());
    //solana_program::msg!("for: ${}", total_sell_value.to_string());
    //solana_program::msg!("fee: ${}", fee_amount.to_string());
    //solana_program::msg!("value_after_fee: ${}", sell_value_after_fee.to_string());

    let fee_amount_raw = from_numeric(fee_amount, mint_b_decimals)?;
    let sell_value_after_fee_raw = from_numeric(sell_value_after_fee, mint_b_decimals)?;

    check_condition(
        sell_value_after_fee_raw > 0,
        "No value received"
    )?;
    if pool.sell_fee > 0 {
        check_condition(
            fee_amount_raw > 0,
            "No fees generated"
        )?;
    }
    check_condition(
        sell_value_after_fee_raw >= min_amount_out_arg,
        "Slippage exceeded"
    )?;

    transfer(
        seller_info,
        seller_target_info,
        target_vault_info,
        token_program_info,
        in_amount_raw,
    )?;

    if fee_amount_raw > 0 {
        burn_signed_with_bump(
            base_vault_info,
            base_mint_info,
            base_vault_info,
            token_program_info,
            fee_amount_raw,
            &[
                TREASURY,
                pool_info.key.as_ref(),
                base_mint_info.key.as_ref()
            ],
            pool.vault_b_bump,
        )?;
    }

    Ok(sell_value_after_fee_raw)
}