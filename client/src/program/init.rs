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
use rand::Rng;

pub async fn initialize(
    client: &RpcClient,
    signer: &Keypair,
    name: String,
    symbol: String,
    base_mint: Pubkey, // USDC mint
) -> Result<(Signature, Signature, Pubkey, Pubkey, Pubkey)> {
    if name.len() > MAX_NAME_LEN {
        return Err(anyhow!("Name exceeds {} characters", MAX_NAME_LEN));
    }
    if symbol.len() > MAX_SYMBOL_LEN {
        return Err(anyhow!("Symbol exceeds {} characters", MAX_SYMBOL_LEN));
    }

    let authority = signer.pubkey();
    let seed: [u8; 32] = rand::thread_rng().gen(); // Random seed

    // Initialize currency
    let (mint_pda, _) = find_mint_pda(&authority, &name, &seed);
    let (currency_pda, _) = find_currency_pda(&mint_pda);

    let compute_budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(250_000);
    let create_currency_ix = build_initialize_currency_ix(
        authority,
        name,
        symbol,
        seed,
    );

    let blockhash_bytes = get_latest_blockhash(client).await?;
    let recent_blockhash = deserialize(&blockhash_bytes)?;
    let currency_tx = Transaction::new_signed_with_payer(
        &[compute_budget_ix.clone(), create_currency_ix],
        Some(&authority),
        &[signer],
        recent_blockhash,
    );

    println!("Initializing currency with PDA: {}", currency_pda);

    let currency_signature_bytes = send_and_confirm_transaction(client, &currency_tx)
        .await
        .map_err(|e| anyhow!("Failed to initialize currency: {}", e))?;
    let currency_signature: Signature = deserialize(&currency_signature_bytes)?;

    println!("Currency initialized with signature: {}", currency_signature);

    // Create fee ATAs
    let fee_mint_ata = spl_associated_token_account::get_associated_token_address(&authority, &mint_pda);
    let fee_base_ata = spl_associated_token_account::get_associated_token_address(&authority, &base_mint);

    let (_mint_ata, mint_ata_sig) = create_ata(client, signer, &mint_pda, &authority, None).await?;
    if mint_ata_sig != Signature::default() {
        println!("Created fee mint ATA: {}. Signature: {}", fee_mint_ata, mint_ata_sig);
    }

    let (_base_ata, base_ata_sig) = create_ata(client, signer, &base_mint, &authority, None).await?;
    if base_ata_sig != Signature::default() {
        println!("Created fee base ATA: {}. Signature: {}", fee_base_ata, base_ata_sig);
    }

    // Initialize pool
    let (pool_pda, _) = find_pool_pda(&currency_pda);
    let pool_ix = build_initialize_pool_ix(
        authority,
        currency_pda,
        mint_pda,
        base_mint,
        PURCHASE_CAP,
        SALE_CAP,
        BUY_FEE_BPS,
        SELL_FEE_BPS,
        fee_mint_ata,
        fee_base_ata,
    );

    let blockhash_bytes = get_latest_blockhash(client).await?;
    let recent_blockhash = deserialize(&blockhash_bytes)?;
    let pool_tx = Transaction::new_signed_with_payer(
        &[compute_budget_ix, pool_ix],
        Some(&authority),
        &[signer],
        recent_blockhash,
    );

    let pool_signature_bytes = send_and_confirm_transaction(client, &pool_tx)
        .await
        .map_err(|e| anyhow!("Failed to initialize pool: {}", e))?;
    let pool_signature: Signature = deserialize(&pool_signature_bytes)?;

    Ok((currency_signature, pool_signature, mint_pda, currency_pda, pool_pda))
}
