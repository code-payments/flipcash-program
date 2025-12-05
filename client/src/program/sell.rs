use anyhow::{Result, anyhow};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
    pubkey::Pubkey,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use flipcash_api::prelude::*;

use crate::utils::*;

pub async fn sell(
    client: &RpcClient,
    signer: &Keypair,
    mint: Pubkey,
    base_mint: Pubkey,
    amount: f64, // Amount in tokens
) -> Result<Signature> {

    let seller = signer.pubkey();
    let (currency_pda, _) = find_currency_pda(&mint);
    let (pool_pda, _) = find_pool_pda(&currency_pda);

    let seller_target_ata = spl_associated_token_account::get_associated_token_address(&seller, &mint);
    let seller_base_ata = spl_associated_token_account::get_associated_token_address(&seller, &base_mint);

    // Create seller ATAs
    let (_target_ata, target_ata_sig) = create_ata(client, signer, &mint, &seller, None).await?;
    if target_ata_sig != Signature::default() {
        println!("Created seller target ATA: {}. Signature: {}", seller_target_ata, target_ata_sig);
    }

    let (_base_ata, base_ata_sig) = create_ata(client, signer, &base_mint, &seller, None).await?;
    if base_ata_sig != Signature::default() {
        println!("Created seller base ATA: {}. Signature: {}", seller_base_ata, base_ata_sig);
    }

    // Convert amount (in tokens) to token amount
    let in_amount = (amount * 10f64.powi(TOKEN_DECIMALS as i32)) as u64;
    let min_amount_out = 0; // Allow any output amount for simplicity

    let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(250_000);
    let sell_ix = build_sell_tokens_ix(
        seller,
        pool_pda,
        currency_pda,
        mint,
        base_mint,
        in_amount,
        min_amount_out,
        seller_target_ata,
        seller_base_ata,
    );

    let blockhash_bytes = get_latest_blockhash(client).await?;
    let recent_blockhash = deserialize(&blockhash_bytes)?;
    let tx = Transaction::new_signed_with_payer(
        &[compute_budget_ix, sell_ix],
        Some(&seller),
        &[signer],
        recent_blockhash,
    );

    let signature_bytes = send_and_confirm_transaction(client, &tx)
        .await
        .map_err(|e| anyhow!("Failed to sell tokens: {}", e))?;
    let signature: Signature = deserialize(&signature_bytes)?;

    Ok(signature)
}
