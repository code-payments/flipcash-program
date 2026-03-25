use steel::*;
use flipcash_api::prelude::*;

pub fn process_burn_fees(accounts: &[AccountInfo], _data: &[u8]) -> ProgramResult {
    let [
        authority_info,
        pool_info,
        base_mint_info,
        base_vault_info,
        token_program_info,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Basic checks
    check_signer(authority_info)?;
    check_mut(pool_info)?;
    check_mut(base_mint_info)?;
    check_mut(base_vault_info)?;
    check_program(token_program_info, &spl_token::id())?;

    let pool = pool_info.as_account_mut::<LiquidityPool>(&flipcash_api::ID)?;

    // Verify authority is the pool authority
    check_condition(
        pool.authority == *authority_info.key,
        "Invalid authority: must be pool authority"
    )?;

    // Validate accounts match the pool
    check_condition(
        pool.mint_b == *base_mint_info.key,
        "Invalid base mint"
    )?;
    check_condition(
        pool.vault_b == *base_vault_info.key,
        "Invalid base vault"
    )?;

    let fees_to_burn = pool.fees_accumulated;

    // Only burn if there are fees to burn
    check_condition(
        fees_to_burn > 0,
        "No fees to burn"
    )?;

    // Burn fees from the base vault
    burn_signed_with_bump(
        base_vault_info,
        base_mint_info,
        base_vault_info,
        token_program_info,
        fees_to_burn,
        &[
            TREASURY,
            pool_info.key.as_ref(),
            base_mint_info.key.as_ref(),
        ],
        pool.vault_b_bump,
    )?;

    // Reset fees_accumulated to 0
    pool.fees_accumulated = 0;

    Ok(())
}
