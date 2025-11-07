use steel::*;
use flipcash_api::prelude::*;

pub fn process_sell_tokens(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let raw_args = SellTokensIx::try_from_bytes(data)?;
    let args = raw_args.to_struct();

    let [
        seller_info,
        pool_info,
        currency_info,
        target_mint_info,
        base_mint_info,
        target_vault_info,
        base_vault_info,
        seller_target_ata_info,
        seller_base_ata_info,
        fee_target_info,
        fee_base_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    solana_program::msg!("Args: {:?}", args);

    let value_after_fee_raw= sell_common(
        seller_info,
        pool_info,
        currency_info,
        target_mint_info,
        base_mint_info,
        target_vault_info,
        base_vault_info,
        seller_target_ata_info,
        seller_base_ata_info,
        fee_target_info,
        fee_base_info,
        token_program_info,
        args.in_amount,
        args.min_amount_out,
    )?;

    let pool = pool_info.as_account::<LiquidityPool>(&flipcash_api::ID)?;
    transfer_signed_with_bump(
        base_vault_info,
        base_vault_info,
        seller_base_ata_info,
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
        currency_info,
        target_mint_info,
        base_mint_info,
        target_vault_info,
        base_vault_info,
        seller_target_ata_info,
        fee_target_info,
        fee_base_info,
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

    solana_program::msg!("Args: {:?}", args);

    check_mut(vm_authority_info)?;
    check_mut(vm_info)?;
    check_mut(vm_memory_info)?;
    check_program(vm_program_info, &VM_PROGRAM_ID)?;

    let value_after_fee_raw= sell_common(
        seller_info,
        pool_info,
        currency_info,
        target_mint_info,
        base_mint_info,
        target_vault_info,
        base_vault_info,
        seller_target_ata_info,
        vm_omnibus_info,
        fee_target_info,
        fee_base_info,
        token_program_info,
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
            base_mint_info.key.as_ref()
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
    currency_info: &AccountInfo<'info>,
    target_mint_info: &AccountInfo<'info>,
    base_mint_info: &AccountInfo<'info>,
    target_vault_info: &AccountInfo<'info>,
    base_vault_info: &AccountInfo<'info>,
    seller_target_ata_info: &AccountInfo<'info>,
    seller_base_ata_info: &AccountInfo<'info>,
    _fee_target_info: &AccountInfo<'info>,
    fee_base_info: &AccountInfo<'info>,
    token_program_info: &AccountInfo<'info>,
    in_amount_arg: u64,
    min_amount_out_arg: u64,
) -> Result<u64, ProgramError>{
    // Basic checks
    check_signer(seller_info)?;
    check_mut(pool_info)?;
    check_mut(currency_info)?;
    check_mut(target_vault_info)?;
    check_mut(base_vault_info)?;
    check_mut(seller_target_ata_info)?;
    check_mut(seller_base_ata_info)?;
    check_program(token_program_info, &spl_token::id())?;

    target_mint_info.as_mint()?;
    base_mint_info.as_mint()?;

    seller_target_ata_info.as_token_account()?
        .assert(|t| t.owner().eq(seller_info.key))?
        .assert(|t| t.mint().eq(target_mint_info.key))?;

    seller_base_ata_info.as_token_account()?
        .assert(|t| t.owner().eq(seller_info.key))?
        .assert(|t| t.mint().eq(base_mint_info.key))?;

    let pool = pool_info.as_account_mut::<LiquidityPool>(&flipcash_api::ID)?;

    check_condition(
        pool.fees_b.eq(fee_base_info.key),
        "Royalties base account does not match"
    )?;

    check_condition(
        pool.mint_a == *target_mint_info.key && pool.mint_b == *base_mint_info.key,
        "Invalid mint accounts"
    )?;
    check_condition(
        pool.vault_a == *target_vault_info.key && pool.vault_b == *base_vault_info.key,
        "Invalid vault accounts"
    )?;

    let mint_a_decimals = target_mint_info.as_mint()?.decimals();
    let mint_b_decimals = base_mint_info.as_mint()?.decimals();

    let value_left_raw = base_vault_info.as_token_account()?.amount();

    let mut in_amount_raw = in_amount_arg;
    if in_amount_raw == 0 {
        in_amount_raw = seller_target_ata_info.as_token_account()?.amount();
    }

    let curve = ExponentialCurve::default();
    let value_left = to_numeric(value_left_raw, mint_b_decimals)?;
    let in_amount = to_numeric(in_amount_raw, mint_a_decimals)?;
    let fee_rate = from_basis_points(pool.sell_fee)?;

    let mut total_value = curve.tokens_to_value_from_current_value(&value_left, &in_amount)
        .ok_or(ProgramError::InvalidArgument)?;
    if value_left.less_than(&total_value) {
        total_value = value_left
    }
    let fee_amount = total_value.checked_mul(&fee_rate)
        .ok_or(ProgramError::InvalidArgument)?;
    let value_after_fee = total_value.checked_sub(&fee_amount)
        .ok_or(ProgramError::InvalidArgument)?;

    solana_program::msg!("selling: {}", in_amount.to_string());
    solana_program::msg!("for: {}", total_value.to_string());
    solana_program::msg!("fee: {}", fee_amount.to_string());
    solana_program::msg!("value_after_fee: {}", value_after_fee.to_string());

    let fee_amount_raw = from_numeric(fee_amount.clone(), mint_b_decimals)?;
    let value_after_fee_raw = from_numeric(value_after_fee.clone(), mint_b_decimals)?;

    check_condition(
        value_after_fee_raw > 0,
        "No value received"
    )?;
    if pool.sell_fee > 0 {
        check_condition(
            fee_amount_raw > 0,
            "No fees generated"
        )?;
    }
    check_condition(
        value_after_fee_raw >= min_amount_out_arg,
        "Slippage exceeded"
    )?;

    transfer(
        seller_info,
        seller_target_ata_info,
        target_vault_info,
        token_program_info,
        in_amount_raw,
    )?;

    if fee_amount_raw > 0 {
        transfer_signed_with_bump(
            base_vault_info,
            base_vault_info,
            fee_base_info,
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

    Ok(value_after_fee_raw)
}