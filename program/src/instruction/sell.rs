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

    let mint_a_decimals = target_mint_info.as_mint()?.decimals();
    let mint_b_decimals = base_mint_info.as_mint()?.decimals();

    let curve = ExponentialCurve::default();

    let supply_value = MAX_TOKEN_SUPPLY
        .checked_mul(QUARKS_PER_TOKEN)
        .ok_or(ProgramError::InvalidArgument)?
        .checked_sub(target_vault_info.as_token_account()?.amount())
        .ok_or(ProgramError::InvalidArgument)?
        .checked_sub(args.in_amount)
        .ok_or(ProgramError::InvalidArgument)?;
    let supply = to_numeric(supply_value, mint_a_decimals)?;
    let in_amount = to_numeric(args.in_amount, mint_a_decimals)?;
    let fee_rate = from_basis_points(pool.sell_fee)?;

    let total_tokens = curve.tokens_to_value(&supply, &in_amount)
        .ok_or(ProgramError::InvalidArgument)?;
    let fee_amount = total_tokens.checked_mul(&fee_rate)
        .ok_or(ProgramError::InvalidArgument)?;
    let tokens_after_fee = total_tokens.checked_sub(&fee_amount)
        .ok_or(ProgramError::InvalidArgument)?;

    solana_program::msg!("selling: {}", args.in_amount);
    solana_program::msg!("for: {}", total_tokens.to_string());
    solana_program::msg!("fee: {}", fee_amount.to_string());
    solana_program::msg!("tokens_after_fee: {}", tokens_after_fee.to_string());

    let total_tokens_raw = from_numeric(total_tokens.clone(), mint_b_decimals)?;
    let fee_amount_raw = from_numeric(fee_amount.clone(), mint_b_decimals)?;
    let tokens_after_fee_raw = from_numeric(tokens_after_fee.clone(), mint_b_decimals)?;

    check_condition(
        tokens_after_fee_raw >= args.min_amount_out,
        "Slippage exceeded"
    )?;

    solana_program::msg!("deposit tokens");

    transfer(
        seller_info,
        seller_target_ata_info,
        target_vault_info,
        token_program_info,
        args.in_amount,
    )?;

    solana_program::msg!("withdraw tokens");

    base_vault_info.as_token_account()?
        .assert(|t| t.amount() >= total_tokens_raw)?;

    transfer_signed_with_bump(
        base_vault_info,
        base_vault_info,
        seller_base_ata_info,
        token_program_info,
        tokens_after_fee_raw,
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
