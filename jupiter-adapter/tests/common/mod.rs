//! Shared scaffolding for parity tests. `tests/common/mod.rs` (rather than
//! `tests/common.rs`) keeps Cargo from compiling this as its own test binary.

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;

use ahash::RandomState;
use litesvm::LiteSVM;
use litesvm_token::{
    spl_token::{
        self,
        state::{Account as SplAccount, Mint},
    },
    CreateAssociatedTokenAccount, MintTo,
};
use solana_sdk::{
    account::Account, program_option::COption, program_pack::Pack, pubkey::Pubkey,
    signature::Keypair, signer::Signer, transaction::Transaction,
};

use flipcash_api::prelude::*;
use flipcash_jupiter_adapter::FlipcashAmm;
use jupiter_amm_interface::{AccountMap, Amm, AmmContext, KeyedAccount};

fn so_at(rel_path: &str) -> Vec<u8> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(rel_path);
    std::fs::read(&path)
        .unwrap_or_else(|_| panic!("missing {} — run `make build` first", path.display()))
}

fn create_mint_at(svm: &mut LiteSVM, address: &Pubkey, owner: &Pubkey, decimals: u8) {
    let mint = Mint {
        mint_authority: COption::Some(*owner),
        supply: 0,
        decimals,
        is_initialized: true,
        freeze_authority: COption::None,
    };
    let mut data = vec![0u8; Mint::LEN];
    Mint::pack(mint, &mut data).unwrap();
    svm.set_account(
        *address,
        Account {
            lamports: 1_000_000_000,
            data,
            owner: spl_token::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

pub fn account_map(svm: &LiteSVM, keys: &[Pubkey]) -> AccountMap {
    let mut map: AccountMap = HashMap::with_hasher(RandomState::default());
    for k in keys {
        map.insert(*k, svm.get_account(k).expect("account missing in svm"));
    }
    map
}

pub fn refresh_amm(svm: &LiteSVM, pool: Pubkey) -> FlipcashAmm {
    let keyed = KeyedAccount {
        key: pool,
        account: svm.get_account(&pool).unwrap(),
        params: None,
    };
    let mut amm = FlipcashAmm::from_keyed_account(&keyed, &AmmContext::default()).unwrap();
    let map = account_map(svm, &amm.get_accounts_to_update());
    amm.update(&map).unwrap();
    amm
}

fn token_balance(svm: &LiteSVM, ata: &Pubkey) -> u64 {
    SplAccount::unpack(&svm.get_account(ata).unwrap().data)
        .unwrap()
        .amount
}

/// One pool in litesvm + a user with USDF, ready to buy/sell against the
/// curve. Each test gets its own fresh harness.
pub struct PoolHarness {
    pub svm: LiteSVM,
    pub authority: Keypair,
    pub user: Keypair,
    pub user_target: Pubkey,
    pub user_base: Pubkey,
    pub mint_pda: Pubkey,
    pub pool_pda: Pubkey,
    pub usdf: Pubkey,
}

impl PoolHarness {
    pub fn new(sell_fee_bps: u16, user_usdf: u64) -> Self {
        let mut svm = LiteSVM::new();
        svm.add_program(
            mpl_token_metadata::ID,
            &so_at("../target/deploy/metadata.so"),
        );
        svm.add_program(flipcash_api::ID, &so_at("../target/deploy/flipcash.so"));

        let authority = Keypair::new();
        svm.airdrop(&authority.pubkey(), 1_000_000_000).unwrap();

        let usdf = USDF_BASE_MINT;
        create_mint_at(&mut svm, &usdf, &authority.pubkey(), 6);

        let name = "harness".to_string();
        let symbol = "HRNS".to_string();
        let seed = [0u8; 32];
        let (mint_pda, _) = find_mint_pda(&authority.pubkey(), &name, &seed);
        let (currency_pda, _) = find_currency_pda(&mint_pda);
        let (pool_pda, _) = find_pool_pda(&currency_pda);

        let bh = svm.latest_blockhash();
        let ix = build_initialize_currency_ix(authority.pubkey(), name, symbol, seed);
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&authority.pubkey()),
            &[&authority],
            bh,
        );
        svm.send_transaction(tx).unwrap();

        let bh = svm.latest_blockhash();
        let ix = build_initialize_pool_ix(
            authority.pubkey(),
            currency_pda,
            mint_pda,
            usdf,
            sell_fee_bps,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&authority.pubkey()),
            &[&authority],
            bh,
        );
        svm.send_transaction(tx).unwrap();

        let user = Keypair::new();
        svm.airdrop(&user.pubkey(), 1_000_000_000).unwrap();
        let user_target = CreateAssociatedTokenAccount::new(&mut svm, &authority, &mint_pda)
            .owner(&user.pubkey())
            .send()
            .unwrap();
        let user_base = CreateAssociatedTokenAccount::new(&mut svm, &authority, &usdf)
            .owner(&user.pubkey())
            .send()
            .unwrap();

        if user_usdf > 0 {
            MintTo::new(&mut svm, &authority, &usdf, &user_base, user_usdf)
                .owner(&authority)
                .send()
                .unwrap();
        }

        Self {
            svm,
            authority,
            user,
            user_target,
            user_base,
            mint_pda,
            pool_pda,
            usdf,
        }
    }

    pub fn buy(&mut self, in_amount: u64) {
        let bh = self.svm.latest_blockhash();
        let ix = build_buy_tokens_ix(
            self.user.pubkey(),
            self.pool_pda,
            self.mint_pda,
            self.usdf,
            in_amount,
            0,
            self.user_target,
            self.user_base,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.user.pubkey()),
            &[&self.user],
            bh,
        );
        self.svm.send_transaction(tx).unwrap();
    }

    pub fn sell(&mut self, in_amount: u64) {
        let bh = self.svm.latest_blockhash();
        let ix = build_sell_tokens_ix(
            self.user.pubkey(),
            self.pool_pda,
            self.mint_pda,
            self.usdf,
            in_amount,
            0,
            self.user_target,
            self.user_base,
        );
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&self.user.pubkey()),
            &[&self.user],
            bh,
        );
        self.svm.send_transaction(tx).unwrap();
    }

    pub fn amm(&self) -> FlipcashAmm {
        refresh_amm(&self.svm, self.pool_pda)
    }
    pub fn target_balance(&self) -> u64 {
        token_balance(&self.svm, &self.user_target)
    }
    pub fn base_balance(&self) -> u64 {
        token_balance(&self.svm, &self.user_base)
    }
    pub fn fees_accumulated(&self) -> u64 {
        let acc = self.svm.get_account(&self.pool_pda).unwrap();
        LiquidityPool::unpack(&acc.data).unwrap().fees_accumulated
    }
}
