use anchor_lang::prelude::*;
use crate::consts::*;

#[account]
#[repr(C, align(8))]
#[derive(Copy, Debug, PartialEq)]
pub struct CurrencyConfig {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub name: [u8; MAX_NAME_LEN],
    pub symbol: [u8; MAX_SYMBOL_LEN],
    pub seed: [u8; 32],
    pub bump: u8,
    pub mint_bump: u8,
    pub padding: [u8; 6],
}

#[account]
#[repr(C, align(8))]
#[derive(Copy, Debug, PartialEq)]
pub struct LiquidityPool {
    pub authority: Pubkey,
    pub currency: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub fees_accumulated: u64,
    pub sell_fee: u16,
    pub bump: u8,
    pub vault_a_bump: u8,
    pub vault_b_bump: u8,
    pub padding: [u8; 3],
}
