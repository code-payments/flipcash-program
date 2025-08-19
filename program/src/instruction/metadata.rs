use steel::*;
use flipcash_api::prelude::*;

pub fn process_initialize_metadata(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    let raw_args = InitializeMetadataIx::try_from_bytes(data)?;
    let args = raw_args.to_struct()?;

    let [
        authority_info,
        currency_info,
        mint_info,
        metadata_info,

        metadata_program_info,
        system_program_info,
        rent_sysvar_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    solana_program::msg!("Args: {:?}", args);

    check_signer(authority_info)?;
    check_mut(metadata_info)?;

    check_program(metadata_program_info, &mpl_token_metadata::ID)?;
    check_program(system_program_info, &system_program::id())?;
    check_sysvar(rent_sysvar_info, &sysvar::rent::id())?;

    let currency = currency_info.as_account_mut::<CurrencyConfig>(&flipcash_api::ID)?;

    check_condition(
        currency.authority.eq(authority_info.key),
        "Currency authority does not match"
    )?;

    check_condition(
        currency.mint.eq(mint_info.key),
        "Currency mint does not match"
    )?;

    let (metadata_address, _metadata_bump) = metadata_pda(mint_info.key);

    metadata_info
        .is_empty()?
        .is_writable()?
        .has_address(&metadata_address)?;

    let uri = METADATA_URI.replace("{}", &mint_info.key.to_string());

    // Initialize mint metadata.
    mpl_token_metadata::instructions::CreateMetadataAccountV3Cpi {
        __program: metadata_program_info,
        metadata: metadata_info,
        mint: mint_info,
        mint_authority: mint_info,
        payer: authority_info,
        update_authority: (authority_info, true),
        system_program: system_program_info,
        rent: Some(rent_sysvar_info),
        __args: mpl_token_metadata::instructions::CreateMetadataAccountV3InstructionArgs {
            data: mpl_token_metadata::types::DataV2 {
                name: from_name(currency.name.as_ref()),
                symbol: from_symbol(currency.symbol.as_ref()),
                uri: uri.to_string(),
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            is_mutable: true,
            collection_details: None,
        },
    }
    .invoke_signed(
        &[&[
             MINT, 
             authority_info.key.as_ref(),
             currency.name.as_ref(), 
             currency.seed.as_ref(),
             &[currency.mint_bump]
        ]],
    )?;

    Ok(())
}