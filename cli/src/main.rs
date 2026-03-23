mod keypair;
mod swap;

use clap::{Parser, Subcommand};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::path::PathBuf;
use std::str::FromStr;
use anyhow::Result;
use flipcash_api::prelude::*;
use flipcash_client::{get_currency_account, get_pool_account, program};
use keypair::{get_keypair_path, get_payer};
use swap::mints;

#[derive(Debug, Clone)]
pub enum Cluster {
    Localnet,
    Mainnet,
    Devnet,
    Testnet,
    Custom(String),
}

impl Cluster {
    pub fn rpc_url(&self) -> String {
        match self {
            Cluster::Localnet => "http://127.0.0.1:8899".to_string(),
            Cluster::Mainnet => "https://api.mainnet-beta.solana.com".to_string(),
            Cluster::Devnet => "https://api.devnet.solana.com".to_string(),
            Cluster::Testnet => "https://api.testnet.solana.com".to_string(),
            Cluster::Custom(url) => url.clone(),
        }
    }
}

impl FromStr for Cluster {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "l" => Ok(Cluster::Localnet),
            "m" => Ok(Cluster::Mainnet),
            "d" => Ok(Cluster::Devnet),
            "t" => Ok(Cluster::Testnet),
            s if s.starts_with("http://") || s.starts_with("https://") => Ok(Cluster::Custom(s.to_string())),
            _ => Err(format!(
                "Invalid cluster value: '{}'. Use l, m, d, t, or a valid RPC URL (http:// or https://)",
                s
            )),
        }
    }
}

#[derive(Parser)]
#[command(name = "flipcash-cli")]
#[command(about = "CLI for interacting with the Flipcash Solana program")]
struct Cli {
    #[arg(long, global = true, help = "Path to Solana keypair file (default: ~/.config/solana/id.json)")]
    keypair: Option<PathBuf>,

    #[arg(
        long,
        global = true,
        default_value = "l",
        help = "Solana cluster (l = localnet, m = mainnet, d = devnet, t = testnet, or a custom RPC URL)"
    )]
    cluster: Cluster,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Creates a new currency and its associated pool
    CreateCurrency {
        #[arg(long, help = "Name of the currency (max 32 characters)")]
        name: String,

        #[arg(long, help = "Symbol of the currency (max 8 characters)")]
        symbol: String,
    },

    /// Retrieves metadata for a currency and its pool
    GetCurrency {
        #[arg(long, help = "Currency mint address")]
        mint: Pubkey,
    },

    /// Buys tokens from the pool
    Buy {
        #[arg(long, help = "Currency mint address")]
        mint: Pubkey,

        #[arg(long, help = "Amount to buy (in base tokens, e.g., 100.50 USDF)")]
        amount: f64,
    },

    /// Sells tokens to the pool
    Sell {
        #[arg(long, help = "Currency mint address")]
        mint: Pubkey,

        #[arg(long, help = "Amount to sell (in tokens, e.g., 100.50)")]
        amount: f64,
    },

    /// Burns accumulated fees from the pool
    BurnFees {
        #[arg(long, help = "Currency mint address")]
        mint: Pubkey,
    },
    /// Swap SOL or USDC for USDF using Jupiter
    SwapToUsdf {
        #[arg(long, default_value = "10.0")] amount: f64,
        #[arg(long, default_value = "sol", value_parser = ["sol", "usdc"])] input: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = RpcClient::new(cli.cluster.rpc_url());
    let keypair_path = get_keypair_path(cli.keypair);
    let payer = get_payer(keypair_path)?;

    match cli.command {
        Commands::CreateCurrency { name, symbol } => {
            let (currency_sig, pool_sig, mint_pda, currency_pda, pool_pda) = program::initialize(
                &client,
                &payer,
                name.clone(),
                symbol.clone(),
                USDF_BASE_MINT,
            ).await?;
            println!("Currency created. Signature: {}", currency_sig);
            println!("Pool created. Signature: {}", pool_sig);
            println!("Currency Mint: {}", mint_pda);
            println!("Currency PDA: {}", currency_pda);
            println!("Pool PDA: {}", pool_pda);
        }

        Commands::GetCurrency { mint } => {
            let (currency_pda, _) = find_currency_pda(&mint);
            let (pool_pda, _) = find_pool_pda(&currency_pda);

            let (currency, _) = get_currency_account(&client, &currency_pda).await?;
            let name = from_name(&currency.name);
            let symbol = from_symbol(&currency.symbol);
            println!("Currency Metadata:");
            println!("  Authority: {}", currency.authority);
            println!("  Mint: {}", currency.mint);
            println!("  Name: {}", name);
            println!("  Symbol: {}", symbol);

            let (pool, _) = get_pool_account(&client, &pool_pda).await?;
            println!("\nPool Metadata:");
            println!("  Authority: {}", pool.authority);
            println!("  Currency: {}", pool.currency);
            println!("  Mint A (Target): {}", pool.mint_a);
            println!("  Mint B (Base): {}", pool.mint_b);
            println!("  Vault A: {}", pool.vault_a);
            println!("  Vault B: {}", pool.vault_b);
            println!("  Fees Accumulated: {}", pool.fees_accumulated);
            println!("  Sell Fee: {} bps ({}%)", pool.sell_fee, pool.sell_fee as f64 / 100.0);
        }

        Commands::Buy { mint, amount } => {
            let signature = program::buy(&client, &payer, mint, USDF_BASE_MINT, amount).await?;
            println!("Buy transaction successful. Signature: {}", signature);
        }

        Commands::Sell { mint, amount } => {
            let signature = program::sell(&client, &payer, mint, USDF_BASE_MINT, amount).await?;
            println!("Sell transaction successful. Signature: {}", signature);
        }

        Commands::BurnFees { mint } => {
            let signature = program::burn_fees(&client, &payer, mint, USDF_BASE_MINT).await?;
            println!("Burn fees successful. Signature: {}", signature);
        }
        Commands::SwapToUsdf { amount, input } => {
            let mint = match input.as_str() { "sol" => mints::SOL, "usdc" => mints::USDC, _ => mints::SOL };
            let lamports = if input == "sol" { (amount * 1e9) as u64 } else { (amount * 1e6) as u64 };
            let sig = swap::swap_to_usdf(&client, &payer, lamports, mint).await?;
            println!("Swap successful! Tx: {}", sig);
        }
    }
    Ok(())
}
