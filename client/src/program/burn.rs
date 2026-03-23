use anyhow::{Result, anyhow};
use solana_sdk::{
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
    pubkey::Pubkey,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use flipcash_api::prelude::*;

use crate::utils::*;

pub async fn burn_fees(
    client: &RpcClient,
    signer: &Keypair,
    mint: Pubkey,
    base_mint: Pubkey,
) -> Result<Signature> {
    let authority = signer.pubkey();
    let (currency_pda, _) = find_currency_pda(&mint);
    let (pool_pda, _) = find_pool_pda(&currency_pda);

    let burn_fees_ix = build_burn_fees_ix(authority, pool_pda, base_mint);

    let blockhash_bytes = get_latest_blockhash(client).await?;
    let recent_blockhash = deserialize(&blockhash_bytes)?;
    let tx = Transaction::new_signed_with_payer(
        &[burn_fees_ix],
        Some(&authority),
        &[signer],
        recent_blockhash,
    );

    let signature_bytes = send_and_confirm_transaction(client, &tx)
        .await
        .map_err(|e| anyhow!("Failed to burn fees: {}", e))?;
    let signature: Signature = deserialize(&signature_bytes)?;

    Ok(signature)
}
