use steel::*;
use flipcash_api::prelude::*;

pub fn process_initialize_currency(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let raw_args = InitializeCurrencyIx::try_from_bytes(data)?;
    let args = raw_args.to_struct()?;

    let [
        authority_info,
        creator_info,
        mint_info,
        currency_info,

        token_program_info,
        system_program_info,
        rent_sysvar_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    solana_program::msg!("Args: {:?}", args);

    check_signer(authority_info)?;
    check_mut(mint_info)?;
    check_mut(currency_info)?;

    check_program(token_program_info, &spl_token::id())?;
    check_program(system_program_info, &system_program::id())?;
    check_sysvar(rent_sysvar_info, &sysvar::rent::id())?;

    check_uninitialized_pda(
        mint_info,
        &[ 
            FLIPCASH, b"mint", 
            authority_info.key.as_ref(), 
            raw_args.name.as_ref(), 
            args.seed.as_ref() 
        ],
        &flipcash_api::id()
    )?;

    check_uninitialized_pda(
        currency_info,
        &[ FLIPCASH, b"currency", mint_info.key.as_ref() ],
        &flipcash_api::id()
    )?;

    // Create the mint account.
    create_mint_account(
        mint_info,
        mint_info.key,  // mint_authority
        None,           // freeze_authority
        args.decimal_places,
        &[
             FLIPCASH, b"mint", 
             authority_info.key.as_ref(),
             raw_args.name.as_ref(), 
             args.seed.as_ref(),
             &[args.mint_bump]
        ],
        authority_info,
        system_program_info,
        rent_sysvar_info,
    )?;

    // Create the currency account.
    create_program_account_with_bump::<CurrencyConfig>(
        currency_info,
        system_program_info,
        authority_info,
        &flipcash_api::ID,
        &[
            FLIPCASH, b"currency", 
            mint_info.key.as_ref()
        ],
        args.bump,
    )?;

    let currency = currency_info.as_account_mut::<CurrencyConfig>(&flipcash_api::ID)?;

    currency.authority = *authority_info.key;
    currency.creator = *creator_info.key;
    currency.mint = *mint_info.key;
    currency.name = raw_args.name;
    currency.seed = args.seed;
    currency.max_supply = args.max_supply;
    currency.current_supply = 0;
    currency.decimals_places = args.decimal_places;
    currency.bump = args.bump;
    currency.mint_bump = args.mint_bump;

    Ok(())
}

