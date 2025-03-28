use steel::*;
use solana_program::msg;
use crate::consts::*;

pub fn check_condition(condition: bool, message: &str) -> ProgramResult {
    if !condition {
        msg!("Failed condition: {}", message);
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

pub fn check_signer(account: &AccountInfo) -> ProgramResult {
    account.is_signer()?.is_writable()?;
    Ok(())
}

pub fn check_mut(account: &AccountInfo) -> ProgramResult {
    account.is_writable()?;
    Ok(())
}

pub fn check_uninitialized_pda(account: &AccountInfo, seeds: &[&[u8]], program_id: &Pubkey) -> ProgramResult {
    if !account.owner.eq(&system_program::ID) {
        return Err(ProgramError::InvalidAccountData);
    }

    account.is_empty()?.is_writable()?.has_seeds(seeds, program_id)?;
    Ok(())
}

pub fn check_seeds(account: &AccountInfo, seeds: &[&[u8]], program_id: &Pubkey) -> ProgramResult {
    account.has_seeds(seeds, program_id)?;
    Ok(())
}

pub fn check_program(account: &AccountInfo, program_id: &Pubkey) -> ProgramResult {
    account.is_program(program_id)?;
    Ok(())
}

pub fn check_sysvar(account: &AccountInfo, sysvar_id: &Pubkey) -> ProgramResult {
    account.is_sysvar(sysvar_id)?;
    Ok(())
}

pub fn to_name(val: &str) -> [u8; MAX_NAME_LEN] {
    assert!(val.len() <= MAX_NAME_LEN, "name too long");

    let mut name_bytes = [0u8; MAX_NAME_LEN];
    name_bytes[..val.len()].copy_from_slice(val.as_bytes());
    name_bytes
}

pub fn from_name(val: &[u8]) -> String {
    let mut name_bytes = val.to_vec();
    name_bytes.retain(|&x| x != 0);
    String::from_utf8(name_bytes).unwrap()
}

/// Convert to f64 whole value (e.g., 10_000_000 with 6 decimals -> 10.0)
pub fn to_decimal(amount: u64, decimal_places: u8) -> f64 {
    amount as f64 / 10f64.powi(decimal_places as i32)
}

/// Create from f64 whole token value (e.g., 10.0 with 6 decimals -> 10_000_000)
pub fn from_decimal(value: f64, decimal_places: u8) -> u64 {
    (value * 10f64.powi(decimal_places as i32)) as u64
}

pub fn to_basis_points(value: f64) -> u32 {
    (value * 10_000.0) as u32
}

pub fn from_basis_points(value: u32) -> f64 {
    value as f64 / 10_000.0
}
