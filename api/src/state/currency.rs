use steel::*;
use super::AccountType;
use crate::state;

/// Currency configuration account that stores metadata for a custom currency.
/// PDA seeds: ["currency", mint_pubkey]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct CurrencyConfig {
    pub authority: Pubkey,              // Can set fees, etc.
    pub mint: Pubkey,                   // SPL Mint A (target)
    pub name: [u8; 32],                 // Currency name (target)
    pub symbol: [u8; 8],                // Currency symbol (target)
    pub seed: [u8; 32],                 // Seed for PDA (target)

    // Bump seeds for PDAs
    pub bump: u8,
    pub mint_bump: u8,                   // Mint bump seed

    _padding: [u8; 6],
}

state!(AccountType, CurrencyConfig);
