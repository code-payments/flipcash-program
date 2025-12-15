use anchor_lang::prelude::*;

mod consts;
mod types;
mod state;
mod args;
mod instructions;

use consts::*;
use args::*;
use instructions::*;

declare_id!("ccJYP5gjZqcEHaphcxAZvkxCrnTVfYMjyhSYkpQtf8Z");

#[program]
pub mod flipcash {
    use super::*;

    pub fn initialize_currency(_ctx: Context<InitializeCurrency>, _data: InitializeCurrencyArgs) -> Result<()> {
        Ok(())
    }

    pub fn initialize_pool(_ctx: Context<InitializePool>, _data: InitializePoolArgs) -> Result<()> {
        Ok(())
    }

    pub fn initialize_metadata(_ctx: Context<InitializeMetadata>) -> Result<()> {
        Ok(())
    }

    pub fn buy_tokens(_ctx: Context<BuyTokens>, _data: BuyTokensArgs) -> Result<()> {
        Ok(())
    }

    pub fn sell_tokens(_ctx: Context<SellTokens>, _data: SellTokensArgs) -> Result<()> {
        Ok(())
    }

    pub fn buy_and_deposit_into_vm(_ctx: Context<BuyAndDepositIntoVm>, _data: BuyAndDepositIntoVmArgs) -> Result<()> {
        Ok(())
    }

    pub fn sell_and_deposit_into_vm(_ctx: Context<SellAndDepositIntoVm>, _data: SellAndDepositIntoVmArgs) -> Result<()> {
        Ok(())
    }

    pub fn burn_fees(_ctx: Context<BurnFees>) -> Result<()> {
        Ok(())
    }
}
