use steel::*;
use super::AccountType;
use crate::state;
use crate::consts::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct CurrencyConfig {
    pub authority: Pubkey,              // Can set fees, etc.
    pub mint: Pubkey,                   // SPL Mint A (target)
    pub name: [u8; MAX_NAME_LEN],       // Currency name (target)
    pub symbol: [u8; MAX_SYMBOL_LEN],   // Currency symbol (target)
    pub seed: [u8; 32],                 // Seed for PDA (target)

    // Bump seeds for PDAs
    pub bump: u8,
    pub mint_bump: u8,                   // Mint bump seed

    _padding: [u8; 6],
    //_buffer: [u8; 256],                // Padding (future use)
}

state!(AccountType, CurrencyConfig);
