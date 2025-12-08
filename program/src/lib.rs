#![allow(unexpected_cfgs)]
use steel::*;
use flipcash_api::prelude::*;
use solana_security_txt::security_txt;

pub mod instruction;
use instruction::*;

security_txt! {
    name: "Flipcash Currency Creator Program",
    project_url: "https://flipcash.com",
    contacts: "email:security@flipcash.com",
    policy: "https://github.com/code-payments/flipcash-program/blob/main/SECURITY.md",
    source_code: "https://github.com/code-payments/flipcash-program",
    auditors: "Sec3"
}

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (ix, data) = parse_instruction(&flipcash_api::ID, program_id, data)?;

    match ix {
        InstructionType::Unknown => return Err(ProgramError::InvalidInstructionData),

        InstructionType::InitializeCurrencyIx => process_initialize_currency(accounts, data)?,
        InstructionType::InitializePoolIx => process_initialize_pool(accounts, data)?,
        InstructionType::InitializeMetadataIx => process_initialize_metadata(accounts, data)?,
        InstructionType::BuyTokensIx => process_buy_tokens(accounts, data)?,
        InstructionType::SellTokensIx => process_sell_tokens(accounts, data)?,
        InstructionType::BuyAndDepositIntoVmIx => process_buy_and_deposit_into_vm(accounts, data)?,
        InstructionType::SellAndDepositIntoVmIx => process_sell_and_deposit_into_vm(accounts, data)?,
        InstructionType::BurnFeesIx => process_burn_fees(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);

