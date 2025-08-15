#![cfg(test)]

pub mod utils;
use utils::*;

use flipcash_api::prelude::*;
use solana_sdk::{signer::Signer, transaction::Transaction};

fn as_token(val: u64, decimals: u8) -> u64 {
    val.checked_mul(10u64.pow(decimals as u32))
        .expect("Overflow in as_token")
}

struct TestCurrency {
    name: String,
    symbol: String,
    seed: [u8; 32],
}

struct TestPool {
    purchase_cap: u64,
    sale_cap: u64,
    buy_fee: u32,
    sell_fee: u32,
}

#[test]
fn run_integration() {
    // TODO: take fee only on sell: 1%, no fee on buy

    let mut svm = setup_svm();

    let payer = create_payer(&mut svm);
    let payer_pk = payer.pubkey();

    let usdc_decimals = 9;
    let darksky_decimals = 6;

    let usdc = create_mint(&mut svm, &payer, &payer_pk, usdc_decimals);

    let purchase_cap = to_numeric(as_token(5000, usdc_decimals), usdc_decimals).unwrap();
    let sale_cap = to_numeric(as_token(1000, darksky_decimals), darksky_decimals).unwrap();
    let buy_fee = to_basis_points(&to_numeric(5, 4).unwrap()).unwrap();
    let sell_fee = to_basis_points(&to_numeric(5, 4).unwrap()).unwrap();

    let currency = TestCurrency {
        name: "dark-sky".to_string(),
        symbol: "DSKY".to_string(),
        seed: [0u8; 32],
    };

    let (mint_pda, mint_bump) = find_mint_pda(&payer_pk, &currency.name, &currency.seed);
    let (currency_pda, currency_bump) = find_currency_pda(&mint_pda);

    let blockhash = svm.latest_blockhash();
    let ix = build_initialize_currency_ix(
        payer_pk,
        currency.name.clone(),
        currency.symbol.clone(),
        currency.seed,
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[&payer], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    let account = svm.get_account(&currency_pda).unwrap();
    let account = CurrencyConfig::unpack(&account.data).unwrap();

    assert_eq!(account.authority, payer_pk);
    assert_eq!(account.mint, mint_pda);
    assert_eq!(account.name, to_name(&currency.name));
    assert_eq!(account.seed, currency.seed);
    assert_eq!(account.bump, currency_bump);
    assert_eq!(account.mint_bump, mint_bump);

    let pool = TestPool {
        purchase_cap: from_numeric(purchase_cap, usdc_decimals).unwrap(),
        sale_cap: from_numeric(sale_cap, darksky_decimals).unwrap(),
        buy_fee,
        sell_fee,
    };

    let (pool_pda, pool_bump) = find_pool_pda(&currency_pda);
    let (vault_a_pda, vault_a_bump) = find_vault_pda(&pool_pda, &mint_pda);
    let (vault_b_pda, vault_b_bump) = find_vault_pda(&pool_pda, &usdc);

    let fee_usdc_ata = create_ata(&mut svm, &payer, &usdc, &payer_pk);
    let fee_mint_ata = create_ata(&mut svm, &payer, &mint_pda, &payer_pk);

    let blockhash = svm.latest_blockhash();
    let ix = build_initialize_pool_ix(
        payer_pk,
        currency_pda,
        mint_pda,
        usdc,
        pool.purchase_cap,
        pool.sale_cap,
        pool.buy_fee,
        pool.sell_fee,
        fee_mint_ata,
        fee_usdc_ata,
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[&payer], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    let account = svm.get_account(&pool_pda).unwrap();
    let account = LiquidityPool::unpack(&account.data).unwrap();

    assert_eq!(account.authority, payer_pk);
    assert_eq!(account.currency, currency_pda);
    assert_eq!(account.mint_a, mint_pda);
    assert_eq!(account.mint_b, usdc);
    assert_eq!(account.vault_a, vault_a_pda);
    assert_eq!(account.vault_b, vault_b_pda);
    assert_eq!(account.fees_a, fee_mint_ata);
    assert_eq!(account.fees_b, fee_usdc_ata);
    assert_eq!(account.buy_fee, pool.buy_fee);
    assert_eq!(account.sell_fee, pool.sell_fee);
    assert_eq!(account.purchase_cap, pool.purchase_cap);
    assert_eq!(account.sale_cap, pool.sale_cap);
    assert_eq!(account.bump, pool_bump);
    assert_eq!(account.vault_a_bump, vault_a_bump);
    assert_eq!(account.vault_b_bump, vault_b_bump);

    assert_eq!(get_ata_balance(&svm, &vault_a_pda), as_token(MAX_TOKEN_SUPPLY, TOKEN_DECIMALS));
    assert_eq!(get_ata_balance(&svm, &vault_b_pda), 0);

    let user = create_payer(&mut svm);
    let user_pk = user.pubkey();

    let user_mint_ata = create_ata(&mut svm, &payer, &mint_pda, &user_pk);
    let user_usdc_ata = create_ata(&mut svm, &payer, &usdc, &user_pk);

    let mint_amt = as_token(5000, usdc_decimals);
    let res = mint_to(&mut svm, &user, &usdc, &payer, &user_usdc_ata, mint_amt);
    assert!(res.is_ok());

    assert_eq!(get_ata_balance(&svm, &user_mint_ata), 0);
    assert_eq!(get_ata_balance(&svm, &user_usdc_ata), mint_amt);

    // BUY
    let buy_amount = as_token(2306, usdc_decimals);
    let buy_ix = build_buy_tokens_ix(
        user_pk,
        pool_pda,
        currency_pda,
        mint_pda,
        usdc,
        buy_amount,
        0,
        user_mint_ata,
        user_usdc_ata,
        fee_mint_ata,
        fee_usdc_ata,
    );
    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[buy_ix], Some(&user_pk), &[&user], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    let user_mint_balance = get_ata_balance(&svm, &user_mint_ata);
    let vault_a_balance = get_ata_balance(&svm, &vault_a_pda);
    let vault_b_balance = get_ata_balance(&svm, &vault_b_pda);
    let fee_mint_balance = get_ata_balance(&svm, &fee_mint_ata);
    let user_usdc_after_buy = get_ata_balance(&svm, &user_usdc_ata);

    assert!(user_mint_balance > 0, "User should have received some tokens");
    assert!(vault_a_balance > 0, "Vault A should have been debited");
    assert!(vault_b_balance > 0, "Vault B should have received funds");
    assert!(fee_mint_balance > 0, "Fee in mint should have been collected");

    // SELL
    let sell_amount = as_token(25, darksky_decimals);
    let sell_ix = build_sell_tokens_ix(
        user_pk,
        pool_pda,
        currency_pda,
        mint_pda,
        usdc,
        sell_amount,
        0,
        user_mint_ata,
        user_usdc_ata,
        fee_mint_ata,
        fee_usdc_ata,
    );
    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[sell_ix], Some(&user_pk), &[&user], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    let user_usdc_after_sell = get_ata_balance(&svm, &user_usdc_ata);
    let vault_a_after_sell = get_ata_balance(&svm, &vault_a_pda);
    let fee_usdc_balance = get_ata_balance(&svm, &fee_usdc_ata);

    assert!(
        user_usdc_after_sell > user_usdc_after_buy,
        "User should have received USDC from sale"
    );
    assert!(
        vault_a_after_sell > vault_a_balance,
        "Vault A should have received tokens back"
    );
    assert!(
        fee_usdc_balance > 0,
        "USDC fee account should have received fee"
    );

}
