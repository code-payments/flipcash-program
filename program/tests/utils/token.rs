#![cfg(test)]
use solana_sdk::{pubkey::Pubkey, signature::Keypair, account::Account as SolanaAccount};
use litesvm::{types::FailedTransactionMetadata, LiteSVM};
use litesvm_token::{CreateAssociatedTokenAccount, MintTo, spl_token::{self, state::{Account, Mint}}, get_spl_account};
use solana_sdk::program_pack::Pack;

pub fn create_mint_at(svm: &mut LiteSVM, address: &Pubkey, owner_pk: &Pubkey, decimals: u8) {
    let mint = Mint {
        mint_authority: solana_sdk::program_option::COption::Some(*owner_pk),
        supply: 0,
        decimals,
        is_initialized: true,
        freeze_authority: solana_sdk::program_option::COption::None,
    };
    let mut data = vec![0u8; Mint::LEN];
    Mint::pack(mint, &mut data).unwrap();
    let account = SolanaAccount {
        lamports: 1_000_000_000,
        data,
        owner: spl_token::ID,
        executable: false,
        rent_epoch: 0,
    };
    svm.set_account(*address, account).unwrap();
}

pub fn create_ata(svm: &mut LiteSVM, payer_kp: &Keypair, mint_pk: &Pubkey, owner_pk: &Pubkey) -> Pubkey {
    CreateAssociatedTokenAccount::new(svm, payer_kp, mint_pk)
        .owner(owner_pk)
        .send()
        .unwrap()
}

pub fn get_ata_balance(svm: &LiteSVM, ata: &Pubkey) -> u64 {
    let info:Account = get_spl_account(svm, &ata).unwrap();
    info.amount
}

pub fn mint_to(svm: &mut LiteSVM,
        payer: &Keypair,
        mint: &Pubkey,
        mint_owner: &Keypair,
        destination: &Pubkey,
        amount: u64,
) -> Result<(), FailedTransactionMetadata> {
    MintTo::new(svm, payer, mint, destination, amount)
        .owner(mint_owner)
        .send()
}
