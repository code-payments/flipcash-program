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
        fee_target_info,
        _fee_base_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    solana_program::msg!("Args: {:?}", args);

    // Basic checks
    check_signer(buyer_info)?;
    check_mut(pool_info)?;
    check_mut(currency_info)?;
    check_mut(target_vault_info)?;
    check_mut(base_vault_info)?;
    check_mut(buyer_target_ata_info)?;
    check_mut(buyer_base_ata_info)?;
    check_program(token_program_info, &spl_token::id())?;

    // Check mint and token accounts
    target_mint_info.as_mint()?;
    base_mint_info.as_mint()?;

    buyer_target_ata_info.as_token_account()?
        .assert(|t| t.owner().eq(buyer_info.key))?
        .assert(|t| t.mint().eq(target_mint_info.key))?;

    buyer_base_ata_info.as_token_account()?
        .assert(|t| t.owner().eq(buyer_info.key))?
        .assert(|t| t.mint().eq(base_mint_info.key))?;

    let pool = pool_info.as_account_mut::<LiquidityPool>(&flipcash_api::ID)?;

    check_condition(
        pool.fees_a.eq(fee_target_info.key),
        "Royalties account does not match"
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

    solana_program::msg!("Checking purchase cap");

    // Check purchase cap against input amount (in base tokens/USDC)
    if pool.purchase_cap > 0 {
        check_condition(
            args.in_amount <= pool.purchase_cap,
            "Purchase amount exceeds cap"
        )?;
    }

    let mint_a_decimals = target_mint_info.as_mint()?.decimals();
    let mint_b_decimals = base_mint_info.as_mint()?.decimals();

    // Get curve parameters
    let curve = pool.curve.to_struct()?;
    let supply = pool.supply_from_bonding as f64;
    let fee = from_basis_points(pool.buy_fee);

    // Calculate total tokens to receive before fee
    let in_amount = to_decimal(args.in_amount, mint_b_decimals);
    let total_tokens = curve.value_to_tokens(supply, in_amount);
    let fee_amount = total_tokens * fee;
    let tokens_after_fee = total_tokens - fee_amount;

    // print out the tokens, fees, etc.
    solana_program::msg!("paying: ${}", in_amount);
    solana_program::msg!("for: {}", total_tokens);
    solana_program::msg!("fee: {}", fee_amount);
    solana_program::msg!("tokens_after_fee: {}", tokens_after_fee);

    let total_tokens = from_decimal(total_tokens, mint_a_decimals);
    let fee_amount = from_decimal(fee_amount, mint_a_decimals);
    let tokens_after_fee = from_decimal(tokens_after_fee, mint_a_decimals);

    // Check minimum amount out against amount after fee
    check_condition(
        tokens_after_fee >= args.min_amount_out,
        "Slippage exceeded"
    )?;

    solana_program::msg!("deposit tokens");

    // Transfer full USDC amount from buyer to vault (using buyer signature)
    transfer(
        buyer_info,
        buyer_base_ata_info,
        base_vault_info,
        token_program_info,
        args.in_amount,
    )?;

    solana_program::msg!("withdraw tokens");

    // Check if the vault has enough tokens to cover the purchase
    target_vault_info.as_token_account()?
        .assert(|t| t.amount() > 0)?;

    // Transfer tokens after fee to buyer (using PDA signature)
    transfer_signed_with_bump(
        target_vault_info,
        target_vault_info,
        buyer_target_ata_info,
        token_program_info,
        tokens_after_fee,
        &[
            FLIPCASH, b"vault",
            pool_info.key.as_ref(),
            target_mint_info.key.as_ref()
        ],
        pool.vault_a_bump,
    )?;

    // Transfer fee to fee account if applicable
    if fee_amount > 0 {
        transfer_signed_with_bump(
            target_vault_info,
            target_vault_info,
            fee_target_info,
            token_program_info,
            fee_amount,
            &[
                FLIPCASH, b"vault",
                pool_info.key.as_ref(),
                target_mint_info.key.as_ref()
            ],
            pool.vault_a_bump,
        )?;
    }

    // Update pool state
    pool.supply_from_bonding += total_tokens;

    solana_program::msg!("pool.supply_from_bonding: {}", pool.supply_from_bonding);
    solana_program::msg!("pool.supply_from_bonding: {}", to_decimal(pool.supply_from_bonding, mint_a_decimals));

    Ok(())
}

