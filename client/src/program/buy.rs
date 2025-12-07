use anyhow::{Result, anyhow};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
    pubkey::Pubkey,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use flipcash_api::prelude::*;

use crate::consts::*;
use crate::utils::*;

pub async fn buy(
    client: &RpcClient,
    signer: &Keypair,
    mint: Pubkey,
    base_mint: Pubkey,
    amount: f64, // Amount in USDC
) -> Result<Signature> {

    let buyer = signer.pubkey();
    let (currency_pda, _) = find_currency_pda(&mint);
    let (pool_pda, _) = find_pool_pda(&currency_pda);

    let buyer_target_ata = spl_associated_token_account::get_associated_token_address(&buyer, &mint);
    let buyer_base_ata = spl_associated_token_account::get_associated_token_address(&buyer, &base_mint);

    // Create buyer ATAs
    let (_target_ata, target_ata_sig) = create_ata(client, signer, &mint, &buyer, None).await?;
    if target_ata_sig != Signature::default() {
        println!("Created buyer target ATA: {}. Signature: {}", buyer_target_ata, target_ata_sig);
    }

    let (_base_ata, base_ata_sig) = create_ata(client, signer, &base_mint, &buyer, None).await?;
    if base_ata_sig != Signature::default() {
        println!("Created buyer base ATA: {}. Signature: {}", buyer_base_ata, base_ata_sig);
    }

    // Convert amount (in USDC) to token amount
    let in_amount = (amount * 10f64.powi(DECIMAL_PLACES as i32)) as u64;
    let min_amount_out = 0; // Allow any output amount for simplicity

    let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(100_000);
    let buy_ix = build_buy_tokens_ix(
        buyer,
        pool_pda,
        mint,
        base_mint,
        in_amount,
        min_amount_out,
        buyer_target_ata,
        buyer_base_ata,
    );

    let blockhash_bytes = get_latest_blockhash(client).await?;
    let recent_blockhash = deserialize(&blockhash_bytes)?;
    let tx = Transaction::new_signed_with_payer(
        &[compute_budget_ix, buy_ix],
        Some(&buyer),
        &[signer],
        recent_blockhash,
    );

    let signature_bytes = send_and_confirm_transaction(client, &tx)
        .await
        .map_err(|e| anyhow!("Failed to buy tokens: {}", e))?;
    let signature: Signature = deserialize(&signature_bytes)?;

    Ok(signature)
}
