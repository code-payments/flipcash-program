use solana_program::{
    program_pack::Pack,
    system_instruction, 
    rent::Rent, 
};
use steel::*;

use crate::utils::check_program;

pub const VM_PROGRAM_ID: Pubkey = Pubkey::new_from_array([13, 198, 40, 104, 167, 126, 85, 128, 122, 28, 213, 136, 201, 162, 154, 118, 107, 62, 226, 174, 205, 192, 125, 224, 140, 254, 145, 7, 221, 75, 49, 101]);

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


// todo: properly expose CPI in code-vm
pub fn deposit_into_vm<'info>(
    vm_authority: &AccountInfo<'info>,
    vm: &AccountInfo<'info>,
    vm_memory: &AccountInfo<'info>,
    source_authority: &AccountInfo<'info>,
    source_ata: &AccountInfo<'info>,
    vta_owner: &AccountInfo<'info>,
    vm_omnibus: &AccountInfo<'info>,
    account_index: u16,
    amount: u64,
    seeds: &[&[u8]],
    token_program: &AccountInfo<'info>,
    vm_program: &AccountInfo<'info>,
) -> ProgramResult {
    check_program(vm_program, &VM_PROGRAM_ID)?;

    let accounts = vec![
        AccountMeta::new(*vm_authority.key, true),
        AccountMeta::new(*vm.key, false),
        AccountMeta::new(*vm_memory.key, false),
        AccountMeta::new(*source_authority.key, true),
        AccountMeta::new(*source_ata.key, false),
        AccountMeta::new_readonly(*vta_owner.key, false),
        AccountMeta::new(*vm_omnibus.key, false),
        AccountMeta::new_readonly(*token_program.key, false),
    ];

    let mut data = vec![16];
    data.extend_from_slice(&account_index.to_le_bytes());
    data.extend_from_slice(&amount.to_le_bytes());

    solana_program::program::invoke_signed(
        &Instruction {
            program_id: *vm_program.key,
            accounts: accounts,
            data: data,
        },
        &[
            vm_authority.clone(),
            vm.clone(),
            vm_memory.clone(),
            source_authority.clone(),
            source_ata.clone(),
            vta_owner.clone(),
            vm_omnibus.clone(),
            token_program.clone(),
        ],
        &[seeds],
    )?;

    Ok(())
}
