use anyhow::{Result, anyhow};
use solana_sdk::{
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
    pubkey::Pubkey,
    system_instruction,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use spl_token::instruction as token_instruction;
use spl_token::ID as TOKEN_PROGRAM_ID;

use crate::utils::{deserialize, get_latest_blockhash, send_and_confirm_transaction};


/// Creates a new SPL token mint.
pub async fn create_mint(
    client: &RpcClient,
    payer: &Keypair,
    decimals: u8,
) -> Result<(Pubkey, Signature)> {
    let mint = Keypair::new();
    let mint_pubkey = mint.pubkey();
    let payer_pk = payer.pubkey();

    const MINT_LEN: usize = 82;

    let mint_rent = client
        .get_minimum_balance_for_rent_exemption(MINT_LEN)
        .await?;

    let create_mint_ix = system_instruction::create_account(
        &payer_pk,
        &mint_pubkey,
        mint_rent,
        MINT_LEN as u64,
        &TOKEN_PROGRAM_ID,
    );

    let init_mint_ix = token_instruction::initialize_mint(
        &TOKEN_PROGRAM_ID,
        &mint_pubkey,
        &payer_pk,   // mint authority
        None,        // freeze authority
        decimals,
    )?;

    let blockhash_bytes = get_latest_blockhash(client).await?;
    let recent_blockhash = deserialize(&blockhash_bytes)?;
    let tx = Transaction::new_signed_with_payer(
        &[create_mint_ix, init_mint_ix],
        Some(&payer_pk),
        &[payer, &mint],
        recent_blockhash,
    );

    let signature_bytes = send_and_confirm_transaction(client, &tx)
        .await
        .map_err(|e| anyhow!("Failed to create mint: {}", e))?;
    let signature: Signature = deserialize(&signature_bytes)?;

    println!("Created mint {}. Signature: {}", mint_pubkey, signature);
    Ok((mint_pubkey, signature))
}


pub async fn mint_to(
    client: &RpcClient,
    payer: &Keypair,
    mint: &Pubkey,
    ata: &Pubkey,
    amount: u64,
) -> Result<Signature> {
    let payer_pk = payer.pubkey();

    // Check if mint is valid
    match client.get_account(mint).await {
        Ok(account) if account.owner == TOKEN_PROGRAM_ID => {
            // Mint is valid
        }
        Ok(account) => {
            return Err(anyhow!(
                "Mint {} is owned by {}, not the expected token program {}",
                mint,
                account.owner,
                TOKEN_PROGRAM_ID
            ));
        }
        Err(e) => {
            return Err(anyhow!("Failed to fetch mint {}: {}", mint, e));
        }
    }

    // Check if ATA is valid
    match client.get_account(ata).await {
        Ok(account) if account.owner == TOKEN_PROGRAM_ID => {
            // ATA is valid
        }
        Ok(account) => {
            return Err(anyhow!(
                "ATA {} is owned by {}, not the expected token program {}",
                ata,
                account.owner,
                TOKEN_PROGRAM_ID
            ));
        }
        Err(e) => {
            return Err(anyhow!("Failed to fetch ATA {}: {}", ata, e));
        }
    }

    let mint_to_ix = token_instruction::mint_to(
        &TOKEN_PROGRAM_ID,
        mint,
        ata,
        &payer_pk,
        &[],
        amount,
    )?;

    let blockhash_bytes = get_latest_blockhash(client).await?;
    let recent_blockhash = deserialize(&blockhash_bytes)?;
    let mint_to_tx = Transaction::new_signed_with_payer(
        &[ mint_to_ix, ],
        Some(&payer_pk),
        &[payer],
        recent_blockhash,
    );

    let mint_to_signature_bytes = send_and_confirm_transaction(client, &mint_to_tx)
        .await
        .map_err(|e| anyhow!("Failed to mint tokens to ATA: {}", e))?;
    let mint_to_signature: Signature = deserialize(&mint_to_signature_bytes)?;
    println!(
        "Minted {} tokens to ATA {}. Signature: {}",
        amount, ata, mint_to_signature
    );

    Ok(mint_to_signature)
}
