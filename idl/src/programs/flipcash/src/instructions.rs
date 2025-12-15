use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::state::*;

#[derive(Accounts)]
pub struct InitializeCurrency<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    #[account(mut)]
    pub currency: Account<'info, CurrencyConfig>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub currency: Account<'info, CurrencyConfig>,
    #[account(mut)]
    pub currency_mint: AccountInfo<'info>,
    pub base_mint: AccountInfo<'info>,
    #[account(mut)]
    pub pool: Account<'info, LiquidityPool>,
    #[account(mut)]
    pub currency_vault: AccountInfo<'info>,
    #[account(mut)]
    pub base_vault: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeMetadata<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub currency: Account<'info, CurrencyConfig>,
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    #[account(mut)]
    pub metadata: AccountInfo<'info>,
    pub metadata_program: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct BuyTokens<'info> {
    pub buyer: Signer<'info>,
    pub pool: Account<'info, LiquidityPool>,
    pub currency_mint: AccountInfo<'info>,
    pub base_mint: AccountInfo<'info>,
    #[account(mut)]
    pub currency_vault: AccountInfo<'info>,
    #[account(mut)]
    pub base_vault: AccountInfo<'info>,
    #[account(mut)]
    pub buyer_currency_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub buyer_base_token_account: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct SellTokens<'info> {
    pub seller: Signer<'info>,
    #[account(mut)]
    pub pool: Account<'info, LiquidityPool>,
    pub currency_mint: AccountInfo<'info>,
    pub base_mint: AccountInfo<'info>,
    #[account(mut)]
    pub currency_vault: AccountInfo<'info>,
    #[account(mut)]
    pub base_vault: AccountInfo<'info>,
    #[account(mut)]
    pub seller_currency_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub seller_base_token_account: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BuyAndDepositIntoVm<'info> {
    pub buyer: Signer<'info>,
    pub pool: Account<'info, LiquidityPool>,
    pub currency_mint: AccountInfo<'info>,
    pub base_mint: AccountInfo<'info>,
    #[account(mut)]
    pub currency_vault: AccountInfo<'info>,
    #[account(mut)]
    pub base_vault: AccountInfo<'info>,
    #[account(mut)]
    pub buyer_base_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub vm_authority: AccountInfo<'info>,
    #[account(mut)]
    pub vm: AccountInfo<'info>,
    #[account(mut)]
    pub vm_memory: AccountInfo<'info>,
    #[account(mut)]
    pub vm_omnibus: AccountInfo<'info>,
    pub vta_owner: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub vm_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct SellAndDepositIntoVm<'info> {
    pub seller: Signer<'info>,
    #[account(mut)]
    pub pool: Account<'info, LiquidityPool>,
    pub currency_mint: AccountInfo<'info>,
    pub base_mint: AccountInfo<'info>,
    #[account(mut)]
    pub currency_vault: AccountInfo<'info>,
    #[account(mut)]
    pub base_vault: AccountInfo<'info>,
    #[account(mut)]
    pub seller_currency_token_account: AccountInfo<'info>,
    #[account(mut)]
    pub vm_authority: AccountInfo<'info>,
    #[account(mut)]
    pub vm: AccountInfo<'info>,
    #[account(mut)]
    pub vm_memory: AccountInfo<'info>,
    #[account(mut)]
    pub vm_omnibus: AccountInfo<'info>,
    pub vta_owner: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub vm_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct BurnFees<'info> {
    pub payer: Signer<'info>,
    #[account(mut)]
    pub pool: Account<'info, LiquidityPool>,
    #[account(mut)]
    pub base_mint: AccountInfo<'info>,
    #[account(mut)]
    pub base_vault: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}
