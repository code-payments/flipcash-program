use steel::*;
use super::AccountType;
use crate::state;
use crate::consts::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct CurrencyConfig {
    pub authority: Pubkey,              // Can set fees, etc.
    pub creator: Pubkey,                // Creator of the target mint and pool
    pub mint: Pubkey,                   // SPL Mint A (target)
    pub name: [u8; MAX_NAME_LEN],       // Currency name (target)
    pub symbol: [u8; MAX_SYMBOL_LEN],   // Currency symbol (target)
    pub seed: [u8; 32],                 // Seed for PDA (target)

    // Config
    pub max_supply: u64,
    pub current_supply: u64,
    pub decimals_places: u8,

    // Bump seeds for PDAs
    pub bump: u8,
    pub mint_bump: u8,                   // Mint bump seed

    _padding: [u8; 5],
    //_buffer: [u8; 256],                // Padding (future use)
}

state!(AccountType, CurrencyConfig);
