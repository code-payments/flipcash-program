use solana_program::{
    program_pack::Pack,
    system_instruction, 
    rent::Rent, 
};
use steel::*;

pub fn create_token_account<'info>(
    mint: &AccountInfo<'info>,
    target: &AccountInfo<'info>,
    seeds: &[&[u8]],
    payer: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    rent_sysvar: &AccountInfo<'info>,
) -> ProgramResult {
    // Check if the token account is already initialized
    if !target.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Calculate minimum balance for rent exemption
    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(spl_token::state::Account::LEN);

    // Create the account with system program
    solana_program::program::invoke_signed(
        &system_instruction::create_account(
            payer.key,
            target.key,
            required_lamports,
            spl_token::state::Account::LEN as u64,
            &spl_token::id(),
        ),
        &[
            payer.clone(),
            target.clone(),
            system_program.clone(),
        ],
        &[seeds],
    )?;

    // Initialize the PDA.
    solana_program::program::invoke_signed(
        &spl_token::instruction::initialize_account(
            &spl_token::id(),
            target.key,
            mint.key,
            target.key,
        ).unwrap(),
        &[
            target.clone(),
            mint.clone(),
            target.clone(),
            rent_sysvar.clone(),
        ],
        &[seeds],
    )
}

pub fn create_mint_account<'info>(
    mint: &AccountInfo<'info>,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
    seeds: &[&[u8]],
    payer: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    rent_sysvar: &AccountInfo<'info>,
) -> ProgramResult {
    // Check if the mint account is already initialized
    if !mint.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Calculate minimum balance for rent exemption
    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(spl_token::state::Mint::LEN);

    // Create the account with system program
    solana_program::program::invoke_signed(
        &system_instruction::create_account(
            payer.key,
            mint.key,
            required_lamports,
            spl_token::state::Mint::LEN as u64,
            &spl_token::id(),
        ),
        &[
            payer.clone(),
            mint.clone(),
            system_program.clone(),
        ],
        &[seeds],
    )?;

    // Initialize the mint
    solana_program::program::invoke_signed(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            mint.key,
            mint_authority,
            freeze_authority,
            decimals,
        )?,
        &[
            mint.clone(),
            rent_sysvar.clone(),
        ],
        &[seeds],
    )?;

    Ok(())
}

