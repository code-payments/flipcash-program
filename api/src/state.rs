use steel::*;
use crate::consts::*;
use crate::curve::RawExponentialCurve;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum AccountType {
    Unknown = 0,
    CurrencyConfig,
    LiquidityPool,
}

account!(AccountType, CurrencyConfig);
account!(AccountType, LiquidityPool);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct CurrencyConfig {
    pub authority: Pubkey,        // Can set fees, etc.
    pub creator: Pubkey,          // Creator of the target mint and pool
    pub mint: Pubkey,             // SPL Mint A (target)
    pub name: [u8; MAX_NAME_LEN], // Currency name (target)
    pub seed: [u8; 32],           // Seed for PDA (target)

    // Config
    pub max_supply: u64,
    pub current_supply: u64,
    pub decimals_places: u8,

    // Bump seeds for PDAs
    pub bump: u8,
    pub mint_bump: u8,            // Mint bump seed

    _padding: [u8; 5],
    //_buffer: [u8; 256],         // Padding (future use)
}

impl CurrencyConfig {
    pub const fn get_size() -> usize {
        8 + std::mem::size_of::<Self>()
    }

    pub fn unpack(data: &[u8]) -> Result<&Self, ProgramError> {
        let data = &data[..Self::get_size()];
        Self::try_from_bytes(data)
    }

    pub fn unpack_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        let data = &mut data[..Self::get_size()];
        Self::try_from_bytes_mut(data)
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct LiquidityPool {
    pub authority: Pubkey,        // Can set fees, etc.
    pub currency: Pubkey,         // Currency for this pool

    // SPL token accounts
    pub mint_a: Pubkey,           // SPL Mint A (target)
    pub mint_b: Pubkey,           // SPL Mint B (base, probably USDC)
    pub vault_a: Pubkey,          // Vault A (target)
    pub vault_b: Pubkey,          // Vault B (base)                                  
    pub fees_a: Pubkey,           // Fees destination (target)
    pub fees_b: Pubkey,           // Fees destination (base, probably USDC account)

    // Fee percentages
    pub buy_fee: u32,             // Basis points (0.5% = 50)
    pub sell_fee: u32,            // Basis points (0.5% = 50)

    // Config
    pub created_unix_time: i64,
    pub go_live_unix_time: i64,
    pub purchase_cap: u64,        // Maximum purchase amount (in base tokens, 0 is no limit)
    pub sale_cap: u64,            // Maximum sale amount (in target tokens, 0 is no limit)

    // Bonding curve parameters
    pub curve: RawExponentialCurve,
    pub supply_from_bonding: u64,

    // Bump seeds for PDAs
    pub bump: u8,
    pub currency_bump: u8,
    pub vault_a_bump: u8,
    pub vault_b_bump: u8,

    _padding: [u8; 4],
    //_buffer: [u8; 256],         // Padding (future use)
}

impl LiquidityPool {
    pub const fn get_size() -> usize {
        8 + std::mem::size_of::<Self>()
    }

    pub fn unpack(data: &[u8]) -> Result<&Self, ProgramError> {
        let data = &data[..Self::get_size()];
        Self::try_from_bytes(data)
    }

    pub fn unpack_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        let data = &mut data[..Self::get_size()];
        Self::try_from_bytes_mut(data)
    }
}

