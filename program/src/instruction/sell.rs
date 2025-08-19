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
        _fee_target_info,
        fee_base_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    solana_program::msg!("Args: {:?}", args);

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

    let value_left_raw = base_vault_info.as_token_account()?.amount();

    let mint_a_decimals = target_mint_info.as_mint()?.decimals();
    let mint_b_decimals = base_mint_info.as_mint()?.decimals();

    let curve = ExponentialCurve::default();
    let value_left = to_numeric(value_left_raw, mint_b_decimals)?;
    let in_amount = to_numeric(args.in_amount, mint_a_decimals)?;
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

    solana_program::msg!("selling: {}", args.in_amount);
    solana_program::msg!("for: {}", total_value.to_string());
    solana_program::msg!("fee: {}", fee_amount.to_string());
    solana_program::msg!("value_after_fee: {}", value_after_fee.to_string());

    let fee_amount_raw = from_numeric(fee_amount.clone(), mint_b_decimals)?;
    let value_after_fee_raw = from_numeric(value_after_fee.clone(), mint_b_decimals)?;

    check_condition(
        value_after_fee_raw >= args.min_amount_out,
        "Slippage exceeded"
    )?;

    transfer(
        seller_info,
        seller_target_ata_info,
        target_vault_info,
        token_program_info,
        args.in_amount,
    )?;

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

    Ok(())
}
