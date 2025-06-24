use steel::*;
use solana_program::msg;
use crate::consts::*;
use brine_fp::UnsignedNumeric;

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

pub fn to_symbol(val: &str) -> [u8; MAX_SYMBOL_LEN] {
    assert!(val.len() <= MAX_SYMBOL_LEN, "symbol too long");

    let mut symbol_bytes = [0u8; MAX_SYMBOL_LEN];
    symbol_bytes[..val.len()].copy_from_slice(val.as_bytes());
    symbol_bytes
}

pub fn from_symbol(val: &[u8]) -> String {
    let mut symbol_bytes = val.to_vec();
    symbol_bytes.retain(|&x| x != 0);
    String::from_utf8(symbol_bytes).unwrap()
}

/// Convert token amount to a UnsignedNumeric value (e.g., 10_000_000 with 6 decimals -> 10.0 UnsignedNumeric)
pub fn to_numeric(amount: u64, decimal_places: u8) -> Result<UnsignedNumeric, ProgramError> {
    if decimal_places > 18 {
        return Err(ProgramError::InvalidArgument);
    }

    let scale = 10u64.checked_pow(decimal_places as u32)
        .ok_or(ProgramError::InvalidArgument)?;

    let amount_scaled = UnsignedNumeric::new(amount.into())
        .ok_or(ProgramError::InvalidArgument)?;

    let divisor = UnsignedNumeric::new(scale.into())
        .ok_or(ProgramError::InvalidArgument)?;

    amount_scaled.checked_div(&divisor)
        .ok_or(ProgramError::InvalidArgument)
}

/// Convert UnsignedNumeric into a token amount value (e.g., 10.0 UnsignedNumeric with 6 decimals -> 10_000_000)
pub fn from_numeric(value: UnsignedNumeric, decimal_places: u8) -> Result<u64, ProgramError> {
    if decimal_places > 18 {
        return Err(ProgramError::InvalidArgument);
    }

    let scale = 10u64.checked_pow(decimal_places as u32)
        .ok_or(ProgramError::InvalidArgument)?;

    let multiplier = UnsignedNumeric::new(scale.into())
        .ok_or(ProgramError::InvalidArgument)?;

    let result = value.checked_mul(&multiplier)
        .and_then(|r| r.to_imprecise())
        .ok_or(ProgramError::InvalidArgument)?;

    u64::try_from(result).map_err(|_| ProgramError::InvalidArgument)
}

/// Converts basis points (e.g. 123) into an UnsignedNumeric (e.g. 0.0123)
pub fn from_basis_points(bps: u32) -> Result<UnsignedNumeric, ProgramError> {
    let value = UnsignedNumeric::new(bps.into())
        .ok_or(ProgramError::InvalidArgument)?;
    let divisor = UnsignedNumeric::new(10_000)
        .ok_or(ProgramError::InvalidArgument)?;
    value.checked_div(&divisor).ok_or(ProgramError::InvalidArgument)
}

/// Converts an UnsignedNumeric (e.g. 0.0123) into basis points (e.g. 123)
pub fn to_basis_points(numeric: &UnsignedNumeric) -> Result<u32, ProgramError> {
    let multiplier = UnsignedNumeric::new(10_000)
        .ok_or(ProgramError::InvalidArgument)?;
    let bps = numeric.checked_mul(&multiplier)
        .and_then(|r| r.to_imprecise())
        .ok_or(ProgramError::InvalidArgument)?;

    u32::try_from(bps).map_err(|_| ProgramError::InvalidArgument)
}

#[cfg(test)]
mod tests {
    use super::*;
    use brine_fp::InnerUint;

    #[test]
    fn test_to_name() {
        let name = "TestName";
        let name_bytes = to_name(name);
        assert_eq!(name_bytes[..name.len()], *name.as_bytes());
    }

    #[test]
    fn test_from_name() {
        let name_bytes = [84, 101, 115, 116, 78, 97, 109, 101, 0, 0];
        let name = from_name(&name_bytes);
        assert_eq!(name, "TestName");
    }

    #[test]
    fn test_to_symbol() {
        let symbol = "TST";
        let symbol_bytes = to_symbol(symbol);
        assert_eq!(symbol_bytes[..symbol.len()], *symbol.as_bytes());
    }

    #[test]
    fn test_from_symbol() {
        let symbol_bytes = [84, 83, 84, 0, 0, 0, 0, 0];
        let symbol = from_symbol(&symbol_bytes);
        assert_eq!(symbol, "TST");
    }

    #[test]
    fn test_to_numeric_simple() {
        // 10_000_000 with 6 decimals = 10.0
        let result = to_numeric(10_000_000, 6).unwrap();
        assert_eq!(result.to_string(), "10.000000000000000000");
    }

    #[test]
    fn test_from_numeric_simple() {
        // 10.0 with 6 decimals = 10_000_000
        let numeric = UnsignedNumeric::new(10).unwrap();
        let result = from_numeric(numeric, 6).unwrap();
        assert_eq!(result, 10_000_000);
    }

    #[test]
    fn test_to_numeric_with_decimals() {
        // 123456 with 3 decimals = 123.456
        let result = to_numeric(123_456, 3).unwrap();
        assert_eq!(result.to_string(), "123.456000000000000000");
    }

    #[test]
    fn test_from_numeric_with_decimals() {
        // 123.456 with 3 decimals = 123456
        let base = UnsignedNumeric::new(123).unwrap();
        let decimal = UnsignedNumeric::from_scaled_u128(456_000_000_000_000_000);
        let value = base.checked_add(&decimal).unwrap();
        let result = from_numeric(value, 3).unwrap();
        assert_eq!(result, 123_456);
    }

    #[test]
    fn test_to_numeric_zero() {
        let result = to_numeric(0, 6).unwrap();
        assert_eq!(result.to_string(), "0.000000000000000000");
    }

    #[test]
    fn test_from_numeric_zero() {
        let result = from_numeric(UnsignedNumeric::zero(), 6).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_roundtrip_decimal_6() {
        // amount = 1234567890 with 6 decimals => numeric => back to amount
        let amount = 1_234_567_890;
        let decimal_places = 6;
        let numeric = to_numeric(amount, decimal_places).unwrap();
        let result = from_numeric(numeric, decimal_places).unwrap();
        assert_eq!(result, amount);
    }

    #[test]
    fn test_roundtrip_decimal_9() {
        let amount = 123_456_789;
        let decimal_places = 9;
        let numeric = to_numeric(amount, decimal_places).unwrap();
        let result = from_numeric(numeric, decimal_places).unwrap();
        assert_eq!(result, amount);
    }

    #[test]
    fn test_decimal_overflow_rejected() {
        // decimal_places > 18 should panic
        let result = to_numeric(1, 19);
        assert!(result.is_err());

        let result = from_numeric(UnsignedNumeric::new(1).unwrap(), 19);
        assert!(result.is_err());
    }

    #[test]
    fn test_basis_points_roundtrip() {
        let percent = UnsignedNumeric::new(123).unwrap() // 123% = 1.23
            .checked_div(&UnsignedNumeric::new(100).unwrap())
            .unwrap();
        let bps = to_basis_points(&percent).unwrap();
        assert_eq!(bps, 12_300);

        let back = from_basis_points(bps).unwrap();
        assert!(percent.almost_eq(&back, InnerUint::from(1_000_000_000)));
    }
}

