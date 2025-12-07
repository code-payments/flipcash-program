use steel::*;
use flipcash_api::prelude::*;

pub fn process_buy_tokens(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let raw_args = BuyTokensIx::try_from_bytes(data)?;
    let args = raw_args.to_struct();

    let [
        buyer_info,
        pool_info,
        currency_info,
        target_mint_info,
        base_mint_info,
        target_vault_info,
        base_vault_info,
        buyer_target_ata_info,
        buyer_base_ata_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    buyer_target_ata_info.as_token_account()?
        .assert(|t| t.owner().eq(buyer_info.key))?
        .assert(|t| t.mint().eq(target_mint_info.key))?;

    //solana_program::msg!("Args: {:?}", args);

    let tokens_after_fee_raw= buy_common(
        buyer_info,
        pool_info,
        currency_info,
        target_mint_info,
        base_mint_info,
        target_vault_info,
        base_vault_info,
        buyer_target_ata_info,
        buyer_base_ata_info,
        token_program_info,
        args.in_amount,
        args.min_amount_out,
    )?;

    let pool = pool_info.as_account::<LiquidityPool>(&flipcash_api::ID)?;
    transfer_signed_with_bump(
        target_vault_info,
        target_vault_info,
        buyer_target_ata_info,
        token_program_info,
        tokens_after_fee_raw,
        &[
            TREASURY,
            pool_info.key.as_ref(),
            target_mint_info.key.as_ref()
        ],
        pool.vault_a_bump,
    )?;

    Ok(())
}

pub fn process_buy_and_deposit_into_vm(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let raw_args = BuyAndDepositIntoVmIx::try_from_bytes(data)?;
    let args = raw_args.to_struct();

    let [
        buyer_info,
        pool_info,
        currency_info,
        target_mint_info,
        base_mint_info,
        target_vault_info,
        base_vault_info,
        buyer_base_ata_info,
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

    vm_omnibus_info.as_token_account()?
        .assert(|t| t.mint().eq(target_mint_info.key))?;

    let tokens_after_fee_raw= buy_common(
        buyer_info,
        pool_info,
        currency_info,
        target_mint_info,
        base_mint_info,
        target_vault_info,
        base_vault_info,
        vm_omnibus_info,
        buyer_base_ata_info,
        token_program_info,
        args.in_amount,
        args.min_amount_out,
    )?;

    let pool = pool_info.as_account::<LiquidityPool>(&flipcash_api::ID)?;
    deposit_into_vm(
        vm_authority_info,
        vm_info,
        vm_memory_info,
        target_vault_info,
        target_vault_info,
        vta_owner_info,
        vm_omnibus_info,
        args.vm_memory_index,
        tokens_after_fee_raw,
        &[
            TREASURY,
            pool_info.key.as_ref(),
            target_mint_info.key.as_ref(),
            &[pool.vault_a_bump]
        ],
        token_program_info,
        vm_program_info,
    )?;

    return Ok(())
}

// Buy ixn common utility that executes everything but transfering the bought
// tokens to the intended destination.
fn buy_common<'info>(
    buyer_info: &AccountInfo<'info>,
    pool_info: &AccountInfo<'info>,
    currency_info: &AccountInfo<'info>,
    target_mint_info: &AccountInfo<'info>,
    base_mint_info: &AccountInfo<'info>,
    target_vault_info: &AccountInfo<'info>,
    base_vault_info: &AccountInfo<'info>,
    buyer_target_ata_info: &AccountInfo<'info>,
    buyer_base_ata_info: &AccountInfo<'info>,
    token_program_info: &AccountInfo<'info>,
    in_amount_arg: u64,
    min_amount_out_arg: u64,
) -> Result<u64, ProgramError>{
    // Basic checks
    check_signer(buyer_info)?;
    check_mut(pool_info)?;
    check_mut(currency_info)?;
    check_mut(target_vault_info)?;
    check_mut(base_vault_info)?;
    check_mut(buyer_target_ata_info)?;
    check_mut(buyer_base_ata_info)?;
    check_program(token_program_info, &spl_token::id())?;

    let target_mint = target_mint_info.as_mint()?;
    let base_mint = base_mint_info.as_mint()?;
    let buyer_base_ata = buyer_base_ata_info.as_token_account()?;
    let target_vault = target_vault_info.as_token_account()?;
    let base_vault = base_vault_info.as_token_account()?;

    buyer_base_ata
        .assert(|t| t.owner().eq(buyer_info.key))?
        .assert(|t| t.mint().eq(base_mint_info.key))?;

    let pool = pool_info.as_account_mut::<LiquidityPool>(&flipcash_api::ID)?;

    check_condition(
        pool.mint_a == *target_mint_info.key && pool.mint_b == *base_mint_info.key,
        "Invalid mint accounts"
    )?;
    check_condition(
        pool.vault_a == *target_vault_info.key && pool.vault_b == *base_vault_info.key,
        "Invalid vault accounts"
    )?;

    let mint_a_decimals = target_mint.decimals();
    let mint_b_decimals = base_mint.decimals();

    let tokens_left_raw = target_vault.amount();
    let supply_from_bonding = MAX_TOKEN_SUPPLY
        .checked_mul(QUARKS_PER_TOKEN)
        .ok_or(ProgramError::InvalidArgument)?
        .checked_sub(tokens_left_raw)
        .ok_or(ProgramError::InvalidArgument)?;

    let current_value_raw = base_vault.amount();

    let mut in_amount_raw = in_amount_arg;
    if in_amount_raw == 0 {
        in_amount_raw = buyer_base_ata.amount();
    }

    let tokens_left = to_numeric(tokens_left_raw, mint_a_decimals)?;
    let supply = to_numeric(supply_from_bonding, mint_a_decimals)?;
    let current_value = to_numeric(current_value_raw, mint_b_decimals)?;

    let in_amount = to_numeric(in_amount_raw, mint_b_decimals)?;
    let uncapped_new_value = current_value
        .checked_add(&in_amount)
        .ok_or(ProgramError::InvalidArgument)?;
    let max_cumulative_value = UnsignedNumeric::from_scaled_u128(MAX_CUMULATIVE_VALUE);
    let capped_new_value = if uncapped_new_value.greater_than(&max_cumulative_value) {
        max_cumulative_value
    } else {
        uncapped_new_value
    };
    let capped_in_amount = capped_new_value
        .checked_sub(&current_value)
        .ok_or(ProgramError::InvalidArgument)?;

    let curve = DiscreteExponentialCurve::default();
    let zero = to_numeric(0, 0)?;
    let new_supply = curve.value_to_tokens(&zero, &capped_new_value)
        .ok_or(ProgramError::InvalidArgument)?;
    let mut tokens_bought = new_supply
        .checked_sub(&supply)
        .ok_or(ProgramError::InvalidArgument)?;
    if tokens_bought.greater_than(&tokens_left) {
        tokens_bought = tokens_left;
    }

    //solana_program::msg!("paying: ${}", capped_in_amount.to_string());
    //solana_program::msg!("for: {}", tokens_bought.to_string());

    let actual_in_amount_raw = from_numeric(capped_in_amount, mint_b_decimals)?;
    let tokens_bought_raw = from_numeric(tokens_bought, mint_a_decimals)?;

    check_condition(
        tokens_bought_raw > 0,
        "No tokens bought"
    )?;
    check_condition(
        tokens_bought_raw >= min_amount_out_arg,
        "Slippage exceeded"
    )?;

    transfer(
        buyer_info,
        buyer_base_ata_info,
        base_vault_info,
        token_program_info,
        actual_in_amount_raw,
    )?;

    Ok(tokens_bought_raw)
}
