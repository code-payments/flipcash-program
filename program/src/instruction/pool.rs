use steel::*;
use flipcash_api::prelude::*;

pub fn process_initialize_pool(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let raw_args = InitializePoolIx::try_from_bytes(data)?;
    let args = raw_args.to_struct()?;

    let [
        authority_info,
        currency_info,
        target_mint_info,
        base_mint_info,
        pool_info,
        target_vault_info,
        base_vault_info,
        fee_target_info,
        fee_base_info,
        token_program_info,
        system_program_info,
        rent_sysvar_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    solana_program::msg!("Args: {:?}", args);

    check_signer(authority_info)?;
    check_mut(currency_info)?;
    check_mut(target_mint_info)?;
    check_mut(pool_info)?;
    check_mut(target_vault_info)?;
    check_mut(base_vault_info)?;

    check_program(token_program_info, &spl_token::id())?;
    check_program(system_program_info, &system_program::id())?;
    check_sysvar(rent_sysvar_info, &sysvar::rent::id())?;

    // Check mint and token accounts

    base_mint_info.as_mint()?;
    target_mint_info.as_mint()?;

    check_condition(
        target_mint_info.key.ne(base_mint_info.key),
        "Target and base mints must be different"
    )?;

    check_condition(
        fee_target_info.data_len() > 0,
        "Fee target account is not initialized"
    )?;

    check_condition(
        fee_base_info.data_len() > 0,
        "Fee base account is not initialized"
    )?;

    fee_target_info.as_token_account()?
        .assert(|t| t.mint().eq(target_mint_info.key))?;

    fee_base_info.as_token_account()?
        .assert(|t| t.mint().eq(base_mint_info.key))?;

    solana_program::msg!("Checking PDAs");

    check_uninitialized_pda(
        pool_info,
        &[ POOL, currency_info.key.as_ref() ],
        &flipcash_api::id()
    )?;

    check_uninitialized_pda(
        target_vault_info,
        &[ TREASURY, pool_info.key.as_ref(), target_mint_info.key.as_ref() ],
        &flipcash_api::id()
    )?;

    check_uninitialized_pda(
        base_vault_info,
        &[ TREASURY, pool_info.key.as_ref(), base_mint_info.key.as_ref() ],
        &flipcash_api::id()
    )?;

    solana_program::msg!("Getting currency config");

    let currency = currency_info.as_account_mut::<CurrencyConfig>(&flipcash_api::ID)?;

    check_condition(
        currency.authority.eq(authority_info.key),
        "Currency authority does not match"
    )?;

    check_condition(
        currency.mint.eq(target_mint_info.key),
        "Currency mint does not match"
    )?;

    check_condition(
        currency.current_supply < currency.max_supply,
        "Currency is already at max supply"
    )?;

    check_condition(
        args.supply <= currency.max_supply - currency.current_supply,
        "Requested supply exceeds max mint supply"
    )?;

    solana_program::msg!("Creating vaults");

    create_token_account(
        target_mint_info,
        target_vault_info,
        &[
            TREASURY,
            pool_info.key.as_ref(), 
            target_mint_info.key.as_ref(),
            &[args.vault_a_bump]
        ],
        authority_info,
        system_program_info,
        rent_sysvar_info,
    )?;

    create_token_account(
        base_mint_info,
        base_vault_info,
        &[
            TREASURY,
            pool_info.key.as_ref(), 
            base_mint_info.key.as_ref(),
            &[args.vault_b_bump]
        ],
        authority_info,
        system_program_info,
        rent_sysvar_info,
    )?;

    solana_program::msg!("Minting tokens");

    mint_to_signed_with_bump(
        target_mint_info, 
        target_vault_info, 
        target_mint_info, // mint_authority
        token_program_info, 
        args.supply,
        &[
             MINT,
             authority_info.key.as_ref(),
             currency.name.as_ref(), 
             currency.seed.as_ref(),
        ],
        currency.mint_bump
    )?;


    solana_program::msg!("Creating pool");

    // Create the liquidity pool account.
    create_program_account_with_bump::<LiquidityPool>(
        pool_info,
        system_program_info,
        authority_info,
        &flipcash_api::ID,
        &[
            POOL, 
            currency_info.key.as_ref()
        ],
        args.bump,
    )?;

    let pool = pool_info.as_account_mut::<LiquidityPool>(&flipcash_api::ID)?;
    let now = Clock::get()?.unix_timestamp;

    pool.authority = *authority_info.key;
    pool.currency = *currency_info.key;
    pool.mint_a = *target_mint_info.key;
    pool.mint_b = *base_mint_info.key;
    pool.vault_a = *target_vault_info.key;
    pool.vault_b = *base_vault_info.key;
    pool.fees_a = *fee_target_info.key;
    pool.fees_b = *fee_base_info.key;
    pool.buy_fee = args.buy_fee;
    pool.sell_fee = args.sell_fee;
    pool.purchase_cap = args.purchase_cap;
    pool.sale_cap = args.sale_cap;
    pool.curve = raw_args.curve;
    pool.supply_from_bonding = 0;
    pool.bump = args.bump;
    pool.vault_a_bump = args.vault_a_bump;
    pool.vault_b_bump = args.vault_b_bump;

    // Update the current supply
    currency.current_supply += args.supply;

    Ok(())
}

