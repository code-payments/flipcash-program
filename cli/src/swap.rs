use anyhow::{Result, Context};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::{Keypair, Signature}, transaction::Transaction, instruction::Instruction};
use serde::Deserialize;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct JupiterQuote {
    #[serde(rename = "inputMint")] pub input_mint: String,
    #[serde(rename = "outputMint")] pub output_mint: String,
    #[serde(rename = "inAmount")] pub in_amount: String,
    #[serde(rename = "outAmount")] pub out_amount: String,
    #[serde(rename = "routePlan")] pub route_plan: Vec<RoutePlan>,
}

#[derive(Debug, Deserialize)]
pub struct RoutePlan {
    #[serde(rename = "swapInfo")] pub swap_info: SwapInfo,
}

#[derive(Debug, Deserialize)]
pub struct SwapInfo { pub label: Option<String> }

#[derive(Debug, Deserialize)]
pub struct JupiterSwapResponse {
    #[serde(rename = "swapInstruction")] pub swap_instruction: JupiterSwapInstruction,
}

#[derive(Debug, Deserialize)]
pub struct JupiterSwapInstruction {
    #[serde(rename = "programId")] pub program_id: String,
    #[serde(rename = "data")] pub data: String,
    #[serde(rename = "accounts")] pub accounts: Vec<SwapAccountMeta>,
}

#[derive(Debug, Deserialize)]
pub struct SwapAccountMeta {
    #[serde(rename = "pubkey")] pub pubkey: String,
    #[serde(rename = "isSigner")] pub is_signer: bool,
    #[serde(rename = "isWritable")] pub is_writable: bool,
}

pub mod mints {
    pub const USDF: &str = "5AMAA9JV9H97YYVxx8F6FsCMmTwXSuTTQneiup4RYAUQ";
    pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    pub const SOL: &str = "So11111111111111111111111111111111111111112";
}

pub async fn get_quote(input_mint: &str, output_mint: &str, amount: u64, slippage_bps: u16) -> Result<JupiterQuote> {
    let url = format!("https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}", input_mint, output_mint, amount, slippage_bps);
    let client = reqwest::Client::new();
    let response = client.get(&url).send().await.context("Jupiter quote request failed")?;
    if !response.status().is_success() { anyhow::bail!("Jupiter API error: {}", response.status()); }
    Ok(response.json().await?)
}

pub async fn get_swap_instructions(user_public_key: &str, quote: &JupiterQuote, wrap_sol: bool) -> Result<JupiterSwapResponse> {
    let client = reqwest::Client::new();
    let body = serde_json::json!({"quote": quote, "userPublicKey": user_public_key, "wrapAndUnwrapSol": wrap_sol, "dynamicComputeUnitLimit": true, "prioritizationFeeLamports": "auto"});
    let response = client.post("https://quote-api.jup.ag/v6/swap-instructions").json(&body).send().await.context("Jupiter swap request failed")?;
    if !response.status().is_success() { anyhow::bail!("Jupiter API error"); }
    Ok(response.json().await?)
}

pub async fn swap_to_usdf(rpc_client: &RpcClient, payer: &Keypair, amount_lamports: u64, input_mint: &str) -> Result<Signature> {
    use base64::{Engine, prelude::BASE64_STANDARD};
    let quote = get_quote(input_mint, mints::USDF, amount_lamports, 50).await?;
    println!("Quote: {} {} -> {} USDF", quote.in_amount, input_mint, quote.out_amount);
    let swap_resp = get_swap_instructions(&payer.pubkey().to_string(), &quote, input_mint == mints::SOL).await?;
    let data = BASE64_STANDARD.decode(&swap_resp.swap_instruction.data)?;
    let program_id = Pubkey::from_str(&swap_resp.swap_instruction.program_id)?;
    let accounts = swap_resp.swap_instruction.accounts.iter().map(|a| solana_sdk::instruction::AccountMeta { pubkey: Pubkey::from_str(&a.pubkey).unwrap(), is_signer: a.is_signer, is_writable: a.is_writable }).collect();
    let ix = Instruction { program_id, accounts, data };
    let blockhash = rpc_client.get_latest_blockhash().await?;
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    tx.sign(&[payer], blockhash);
    Ok(rpc_client.send_and_confirm_transaction(&tx).await?)
}
