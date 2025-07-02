use anyhow::{Result, anyhow};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, account::Account};
use flipcash_api::state::{LiquidityPool, CurrencyConfig, };
use crate::utils::{deserialize, get_account};

pub async fn get_currency_account(client: &RpcClient, address: &Pubkey) -> Result<(CurrencyConfig, Pubkey)> {
    let account_bytes = get_account(client, address).await?;
    let account: Account = deserialize(&account_bytes)?;
    let account = CurrencyConfig::unpack(&account.data)
        .map_err(|e| anyhow!("Failed to unpack currency config account: {}", e))
        .copied()?;
    Ok((account, *address))
}

pub async fn get_pool_account(
    client: &RpcClient,
    address: &Pubkey,
) -> Result<(LiquidityPool, Pubkey)> {
    let account_bytes = get_account(client, address).await?;
    let account: Account = deserialize(&account_bytes)?;
    let account = LiquidityPool::unpack(&account.data)
        .map_err(|e| anyhow!("Failed to unpack liquidity pool account: {}", e))
        .copied()?;
    Ok((account, *address))
}
