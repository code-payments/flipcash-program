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

    // Check mint and token accounts
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

    solana_program::msg!("Checking pool state");

    // Validate pool state
    let now = Clock::get()?.unix_timestamp;
    check_condition(
        now >= pool.go_live_unix_time,
        "Pool is not yet live"
    )?;
    check_condition(
        pool.mint_a == *target_mint_info.key && 
        pool.mint_b == *base_mint_info.key,
        "Invalid mint accounts"
    )?;
    check_condition(
        pool.vault_a == *target_vault_info.key && 
        pool.vault_b == *base_vault_info.key,
        "Invalid vault accounts"
    )?;

    solana_program::msg!("Checking sale cap");

    // Check sale cap against input amount (in target tokens)
    if pool.sale_cap > 0 {
        check_condition(
            args.in_amount <= pool.sale_cap,
            "Sale amount exceeds cap"
        )?;
    }

    let mint_a_decimals = target_mint_info.as_mint()?.decimals();
    let mint_b_decimals = base_mint_info.as_mint()?.decimals();

    // Get curve parameters
    let curve = pool.curve.to_struct()?;
    let supply = to_decimal(pool.supply_from_bonding - args.in_amount, mint_a_decimals);
    let fee = from_basis_points(pool.sell_fee);

    // Calculate fee from output USDC and amount after fee
    let in_amount = to_decimal(args.in_amount, mint_a_decimals);
    let total_tokens = curve.tokens_to_value(supply, in_amount);
    let fee_amount = total_tokens * fee;
    let tokens_after_fee = total_tokens - fee_amount;

    // Print out the amounts for debugging
    solana_program::msg!("selling: {}", args.in_amount);
    solana_program::msg!("for: {}", total_tokens);
    solana_program::msg!("fee: {}", fee_amount);
    solana_program::msg!("tokens_after_fee: {}", tokens_after_fee);

    // Tokens here would be USDC (base mint)
    let total_tokens = from_decimal(total_tokens, mint_b_decimals);
    let fee_amount = from_decimal(fee_amount, mint_b_decimals);
    let tokens_after_fee = from_decimal(tokens_after_fee, mint_b_decimals);

    // Check minimum amount out against amount after fee
    check_condition(
        tokens_after_fee >= args.min_amount_out,
        "Slippage exceeded"
    )?;

    solana_program::msg!("deposit tokens");

    // Transfer full token amount from seller to vault (using seller signature)
    transfer(
        seller_info,
        seller_target_ata_info,
        target_vault_info,
        token_program_info,
        args.in_amount,
    )?;

    solana_program::msg!("withdraw USDC");

    // Check if the vault has enough USDC to cover the sale
    base_vault_info.as_token_account()?
        .assert(|t| t.amount() >= total_tokens)?;

    // Transfer USDC after fee to seller (using PDA signature)
    transfer_signed_with_bump(
        base_vault_info,
        base_vault_info,
        seller_base_ata_info,
        token_program_info,
        tokens_after_fee,
        &[
            FLIPCASH, b"vault",
            pool_info.key.as_ref(),
            base_mint_info.key.as_ref()
        ],
        pool.vault_b_bump,
    )?;

    // Transfer fee to fee account if applicable
    if fee_amount > 0 {
        transfer_signed_with_bump(
            base_vault_info,
            base_vault_info,
            fee_base_info,
            token_program_info,
            fee_amount as u64,
            &[
                FLIPCASH, b"vault",
                pool_info.key.as_ref(),
                base_mint_info.key.as_ref()
            ],
            pool.vault_b_bump,
        )?;
    }

    // Update pool state
    pool.supply_from_bonding -= args.in_amount;

    solana_program::msg!("pool.supply_from_bonding: {}", pool.supply_from_bonding);
    solana_program::msg!("pool.supply_from_bonding: {}", to_decimal(pool.supply_from_bonding, mint_a_decimals));

    Ok(())
}
