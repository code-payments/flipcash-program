use anchor_lang::prelude::*;
use crate::consts::*;

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct InitializeCurrencyArgs {
    pub name: [u8; MAX_NAME_LEN],
    pub symbol: [u8; MAX_SYMBOL_LEN],
    pub seed: [u8; 32],
    pub bump: u8,
    pub mint_bump: u8,
    _padding: [u8; 6],
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct InitializePoolArgs {
    pub sell_fee: u16,
    pub bump: u8,
    pub vault_a_bump: u8,
    pub vault_b_bump: u8,
    _padding: [u8; 1],
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct BuyTokensArgs {
    pub in_amount: u64,
    pub min_amount_out: u64,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct SellTokensArgs {
    pub in_amount: u64,
    pub min_amount_out: u64,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct BuyAndDepositIntoVmArgs {
    pub in_amount: u64,
    pub min_amount_out: u64,
    pub vm_memory_index: u16,
}

#[repr(C)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub struct SellAndDepositIntoVmArgs {
    pub in_amount: u64,
    pub min_amount_out: u64,
    pub vm_memory_index: u16,
}
