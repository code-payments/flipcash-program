use steel::*;
use super::AccountType;
use crate::state;

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

    // Config
    pub purchase_cap: u64,        // Maximum purchase amount (in base tokens, 0 is no limit)
    pub sale_cap: u64,            // Maximum sale amount (in target tokens, 0 is no limit)

    // Fee percentages
    pub buy_fee: u16,             // Basis points (0.5% = 50)
    pub sell_fee: u16,            // Basis points (0.5% = 50)

    // Bump seeds for PDAs
    pub bump: u8,
    pub vault_a_bump: u8,
    pub vault_b_bump: u8,

    _padding: [u8; 1],
    //_buffer: [u8; 256],         // Padding (future use)
}

state!(AccountType, LiquidityPool);
