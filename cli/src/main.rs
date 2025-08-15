mod keypair;

use clap::{Parser, Subcommand};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signer::Signer};
use std::path::PathBuf;
use std::str::FromStr;
use anyhow::Result;
use flipcash_api::prelude::*;
use flipcash_client::{create_mint, create_ata, mint_to, get_currency_account, get_pool_account, program};
use keypair::{get_keypair_path, get_payer};

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
    /// Creates a new base mint (e.g., USDC) for testing, with an ATA and initial tokens
    CreateBaseMint {
        #[arg(long, default_value_t = 6, help = "Decimals for the base mint")]
        decimals: u8,

        #[arg(long, default_value_t = 1_000_000_000_000, help = "Initial amount of tokens to mint (in smallest units, e.g., 1_000_000_000_000 = 1,000,000 USDC for 6 decimals)")]
        initial_amount: u64,
    },

    /// Creates a new currency and its associated pool
    CreateCurrency {
        #[arg(long, help = "Name of the currency (max 32 characters)")]
        name: String,

        #[arg(long, help = "Symbol of the currency (max 8 characters)")]
        symbol: String,

        #[arg(long, help = "Base mint address (e.g., USDC mint)")]
        base_mint: Pubkey,
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

        #[arg(long, help = "Base mint address (e.g., USDC mint)")]
        base_mint: Pubkey,

        #[arg(long, help = "Amount to buy (in base tokens, e.g., 100.50 USDC)")]
        amount: f64,
    },

    /// Sells tokens to the pool
    Sell {
        #[arg(long, help = "Currency mint address")]
        mint: Pubkey,

        #[arg(long, help = "Base mint address (e.g., USDC mint)")]
        base_mint: Pubkey,

        #[arg(long, help = "Amount to sell (in tokens, e.g., 100.50)")]
        amount: f64,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = RpcClient::new(cli.cluster.rpc_url());
    let keypair_path = get_keypair_path(cli.keypair);
    let payer = get_payer(keypair_path)?;

    match cli.command {
        Commands::CreateBaseMint { decimals, initial_amount } => {
            // Create mint
            let (mint, mint_signature) = create_mint(&client, &payer, decimals).await?;
            println!("Base mint created. Mint: {}. Signature: {}", mint, mint_signature);

            // Create ATA
            let (ata, ata_signature) = create_ata(&client, &payer, &mint, &payer.pubkey(), None).await?;
            println!("Created ATA {}. Signature: {}", ata, ata_signature);

            // Mint tokens to ATA
            let mint_to_signature = mint_to(&client, &payer, &mint, &ata, initial_amount).await?;
            println!("Minted {} tokens to ATA {}. Signature: {}", initial_amount, ata, mint_to_signature);
        }

        Commands::CreateCurrency { name, symbol, base_mint } => {
            let (currency_sig, pool_sig, mint_pda, currency_pda, pool_pda) = program::initialize(
                &client,
                &payer,
                name.clone(),
                symbol.clone(),
                base_mint,
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
            println!("  Max Supply: {}", currency.max_supply);
            println!("  Current Supply: {}", currency.current_supply);
            println!("  Decimals: {}", currency.decimals_places);

            let (pool, _) = get_pool_account(&client, &pool_pda).await?;
            println!("\nPool Metadata:");
            println!("  Authority: {}", pool.authority);
            println!("  Currency: {}", pool.currency);
            println!("  Mint A (Target): {}", pool.mint_a);
            println!("  Mint B (Base): {}", pool.mint_b);
            println!("  Vault A: {}", pool.vault_a);
            println!("  Vault B: {}", pool.vault_b);
            println!("  Fees A: {}", pool.fees_a);
            println!("  Fees B: {}", pool.fees_b);
            println!("  Buy Fee: {} bps ({}%)", pool.buy_fee, pool.buy_fee as f64 / 100.0);
            println!("  Sell Fee: {} bps ({}%)", pool.sell_fee, pool.sell_fee as f64 / 100.0);
            println!("  Purchase Cap: {}", pool.purchase_cap);
            println!("  Sale Cap: {}", pool.sale_cap);
            println!("  Created Unix Time: {}", pool.created_unix_time);
            println!("  Go Live Unix Time: {}", pool.go_live_unix_time);
            println!("  Supply from Bonding: {}", pool.supply_from_bonding);
        }

        Commands::Buy { mint, base_mint, amount } => {
            let signature = program::buy(&client, &payer, mint, base_mint, amount).await?;
            println!("Buy transaction successful. Signature: {}", signature);
        }

        Commands::Sell { mint, base_mint, amount } => {
            let signature = program::sell(&client, &payer, mint, base_mint, amount).await?;
            println!("Sell transaction successful. Signature: {}", signature);
        }
    }

    Ok(())
}
