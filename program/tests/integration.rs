#![cfg(test)]

pub mod utils;
use rand::{Rng, RngCore};
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
    sell_fee: u16,
}

#[test]
fn run_integration() {
    let mut svm = setup_svm();

    let payer = create_payer(&mut svm);
    let payer_pk = payer.pubkey();

    let usdc_decimals = 6;
    let darksky_decimals = TOKEN_DECIMALS;

    let usdc = create_mint(&mut svm, &payer, &payer_pk, usdc_decimals);

    let sell_fee = to_basis_points(&to_numeric(1, 2).unwrap()).unwrap();

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
        sell_fee,
    };

    let (pool_pda, pool_bump) = find_pool_pda(&currency_pda);
    let (vault_a_pda, vault_a_bump) = find_vault_pda(&pool_pda, &mint_pda);
    let (vault_b_pda, vault_b_bump) = find_vault_pda(&pool_pda, &usdc);

    let blockhash = svm.latest_blockhash();
    let ix = build_initialize_pool_ix(
        payer_pk,
        currency_pda,
        mint_pda,
        usdc,
        pool.sell_fee,
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
    assert_eq!(account.fees_accumulated, 0);
    assert_eq!(account.sell_fee, pool.sell_fee);
    assert_eq!(account.bump, pool_bump);
    assert_eq!(account.vault_a_bump, vault_a_bump);
    assert_eq!(account.vault_b_bump, vault_b_bump);

    let darksky_total_supply = as_token(MAX_TOKEN_SUPPLY, TOKEN_DECIMALS);

    assert_eq!(get_ata_balance(&svm, &vault_a_pda), darksky_total_supply);
    assert_eq!(get_ata_balance(&svm, &vault_b_pda), 0);

    let blockhash = svm.latest_blockhash();
    let ix = build_initialize_metadata_ix(
        payer_pk,
        currency_pda,
        mint_pda,
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[&payer], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

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
        mint_pda,
        usdc,
        buy_amount,
        0,
        user_mint_ata,
        user_usdc_ata,
    );
    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[buy_ix], Some(&user_pk), &[&user], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    let user_mint_balance = get_ata_balance(&svm, &user_mint_ata);
    let vault_a_balance = get_ata_balance(&svm, &vault_a_pda);
    let vault_b_balance = get_ata_balance(&svm, &vault_b_pda);
    let user_usdc_after_buy = get_ata_balance(&svm, &user_usdc_ata);

    assert!(vault_a_balance < darksky_total_supply, "Vault A should have been debited");
    assert!(user_mint_balance == darksky_total_supply - vault_a_balance, "User should have received some tokens");
    assert!(vault_b_balance > 0, "Vault B should have received funds");
    assert!(mint_amt - user_usdc_after_buy - vault_b_balance == 0, "No USDC fees should have been burned");

    let account = svm.get_account(&pool_pda).unwrap();
    let account = LiquidityPool::unpack(&account.data).unwrap();
    assert_eq!(account.fees_accumulated, 0, "No fees should have been accumulated on buy");

    // SELL
    let sell_amount = as_token(25, darksky_decimals);
    let sell_ix = build_sell_tokens_ix(
        user_pk,
        pool_pda,
        mint_pda,
        usdc,
        sell_amount,
        0,
        user_mint_ata,
        user_usdc_ata,
    );
    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[sell_ix], Some(&user_pk), &[&user], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    let user_usdc_after_sell = get_ata_balance(&svm, &user_usdc_ata);
    let vault_a_after_sell = get_ata_balance(&svm, &vault_a_pda);
    let vault_b_after_sell = get_ata_balance(&svm, &vault_b_pda);

    assert!(user_usdc_after_sell > user_usdc_after_buy, "User should have received USDC from sale");
    assert!(vault_a_after_sell > vault_a_balance, "Vault A should have received tokens back");
    assert!(mint_amt == user_usdc_after_sell + vault_b_after_sell, "USDC fee should have been burned");

    let account = svm.get_account(&pool_pda).unwrap();
    let account = LiquidityPool::unpack(&account.data).unwrap();
    assert!(account.fees_accumulated > 0, "Fees should have been accumulated on sell");

    let fees_before_burn = account.fees_accumulated;
    let vault_b_balance_before_burn = get_ata_balance(&svm, &vault_b_pda);

    // BURN FEES
    let random_payer = create_payer(&mut svm);
    let random_payer_pk = random_payer.pubkey();

    let burn_ix = build_burn_fees_ix(
        random_payer_pk,
        pool_pda,
        usdc,
    );
    let blockhash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[burn_ix], Some(&random_payer_pk), &[&random_payer], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok(), "Burn fees should succeed");

    let vault_b_balance_after_burn = get_ata_balance(&svm, &vault_b_pda);
    assert_eq!(vault_b_balance_before_burn - vault_b_balance_after_burn, fees_before_burn, "Vault B should have decreased by fees_accumulated");

    let account = svm.get_account(&pool_pda).unwrap();
    let account = LiquidityPool::unpack(&account.data).unwrap();
    assert_eq!(account.fees_accumulated, 0, "Fees should be reset to 0 after burn");
}

#[test]
#[ignore]
fn run_buy_and_sell_simulation_up_and_down_curve() {
    let mut svm = setup_svm();

    let payer = create_payer(&mut svm);
    let payer_pk = payer.pubkey();

    let usdc_decimals = 6;

    let usdc = create_mint(&mut svm, &payer, &payer_pk, usdc_decimals);

    let sell_fee = to_basis_points(&to_numeric(0, 2).unwrap()).unwrap();

    let currency = TestCurrency {
        name: "dark-sky".to_string(),
        symbol: "DSKY".to_string(),
        seed: [0u8; 32],
    };

    let (mint_pda, _) = find_mint_pda(&payer_pk, &currency.name, &currency.seed);
    let (currency_pda, _) = find_currency_pda(&mint_pda);

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

    let pool = TestPool {
        sell_fee,
    };

    let (pool_pda, _) = find_pool_pda(&currency_pda);
    let (vault_a_pda, _) = find_vault_pda(&pool_pda, &mint_pda);
    let (vault_b_pda, _) = find_vault_pda(&pool_pda, &usdc);

    let blockhash = svm.latest_blockhash();
    let ix = build_initialize_pool_ix(
        payer_pk,
        currency_pda,
        mint_pda,
        usdc,
        pool.sell_fee,
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[&payer], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    let user = create_payer(&mut svm);
    let user_pk = user.pubkey();

    let user_mint_ata = create_ata(&mut svm, &payer, &mint_pda, &user_pk);
    let user_usdc_ata = create_ata(&mut svm, &payer, &usdc, &user_pk);

    let mint_amt = as_token(2_000_000_000_000, usdc_decimals);
    let res = mint_to(&mut svm, &user, &usdc, &payer, &user_usdc_ata, mint_amt);
    assert!(res.is_ok());

    let iterations = 10_000;
    let buy_amount_per_iteration = get_ata_balance(&svm, &user_usdc_ata) / iterations;
    for i in 0..iterations {
        let mut buy_amount = buy_amount_per_iteration - i;
        if i == iterations - 1 {
            buy_amount = get_ata_balance(&svm, &user_usdc_ata);
        }
        let buy_ix = build_buy_tokens_ix(
            user_pk,
            pool_pda,
            mint_pda,
            usdc,
            buy_amount,
            0,
            user_mint_ata,
            user_usdc_ata,
        );
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new_signed_with_payer(&[buy_ix], Some(&user_pk), &[&user], blockhash);
        let res = send_tx(&mut svm, tx);
        assert!(res.is_ok());

        let user_mint_balance = get_ata_balance(&svm, &user_mint_ata);
        let user_usdc_balance = get_ata_balance(&svm, &user_usdc_ata);
        let vault_mint_balance = get_ata_balance(&svm, &vault_a_pda);
        let vault_usdc_balance = get_ata_balance(&svm, &vault_b_pda);
        println!("User DSKY balance: {:?}", user_mint_balance);
        println!("Vault DSKY balance: {:?}", vault_mint_balance);
        println!("User USDC balance: {:?}", user_usdc_balance);
        println!("Vault USDC balance: {:?}", vault_usdc_balance);

        if user_mint_balance == as_token(MAX_TOKEN_SUPPLY, TOKEN_DECIMALS) {
            break;
        }
    }

    let user_mint_balance = get_ata_balance(&svm, &user_mint_ata);
    let user_usdc_balance = get_ata_balance(&svm, &user_usdc_ata);
    let vault_a_balance = get_ata_balance(&svm, &vault_a_pda);
    let vault_b_balance = get_ata_balance(&svm, &vault_b_pda);
    assert!(user_mint_balance == as_token(MAX_TOKEN_SUPPLY, TOKEN_DECIMALS), "User should have all DSKY");
    assert!(user_usdc_balance > 0, "User should have some USDC left");
    assert!(vault_a_balance == 0, "Vault A should have no DSKY");
    assert!(vault_b_balance == 1_139_973_004_315_032_343, "Vault B should have the cumulative USDC to buy all tokens");

    let iterations = 12_345;
    let sell_amount_per_iteration = get_ata_balance(&svm, &user_mint_ata) / iterations;
    for i in 0..iterations {
        let mut sell_amount = sell_amount_per_iteration - i;
        if i == iterations - 1 {
            sell_amount = get_ata_balance(&svm, &user_mint_ata);
        }
        let sell_ix = build_sell_tokens_ix(
            user_pk,
            pool_pda,
            mint_pda,
            usdc,
            sell_amount,
            0,
            user_mint_ata,
            user_usdc_ata,
        );
        let blockhash = svm.latest_blockhash();
        let tx = Transaction::new_signed_with_payer(&[sell_ix], Some(&user_pk), &[&user], blockhash);
        let res = send_tx(&mut svm, tx);
        assert!(res.is_ok());

        let user_mint_balance = get_ata_balance(&svm, &user_mint_ata);
        let user_usdc_balance = get_ata_balance(&svm, &user_usdc_ata);
        let vault_mint_balance = get_ata_balance(&svm, &vault_a_pda);
        let vault_usdc_balance = get_ata_balance(&svm, &vault_b_pda);
        println!("User DSKY balance: {:?}", user_mint_balance);
        println!("Vault DSKY balance: {:?}", vault_mint_balance);
        println!("User USDC balance: {:?}", user_usdc_balance);
        println!("Vault USDC balance: {:?}", vault_usdc_balance);
    }

    let user_mint_balance = get_ata_balance(&svm, &user_mint_ata);
    let user_usdc_balance = get_ata_balance(&svm, &user_usdc_ata);
    let vault_a_balance = get_ata_balance(&svm, &vault_a_pda);
    let vault_b_balance = get_ata_balance(&svm, &vault_b_pda);
    assert!(user_mint_balance == 0, "User should have no DSKY");
    assert!(user_usdc_balance == mint_amt, "User should have all USDC");
    assert!(vault_a_balance == as_token(MAX_TOKEN_SUPPLY, TOKEN_DECIMALS), "Vault A should have all DSKY");
    assert!(vault_b_balance == 0, "Vault B should have no USDC");
}

#[test]
#[ignore]
fn run_buy_and_sell_simulation_random() {
    let mut svm = setup_svm();

    let payer = create_payer(&mut svm);
    let payer_pk = payer.pubkey();

    let usdc_decimals = 6;

    let usdc = create_mint(&mut svm, &payer, &payer_pk, usdc_decimals);

    let sell_fee = to_basis_points(&to_numeric(0, 2).unwrap()).unwrap();

    let currency = TestCurrency {
        name: "dark-sky".to_string(),
        symbol: "DSKY".to_string(),
        seed: [0u8; 32],
    };

    let (mint_pda, _) = find_mint_pda(&payer_pk, &currency.name, &currency.seed);
    let (currency_pda, _) = find_currency_pda(&mint_pda);

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

    let pool = TestPool {
        sell_fee,
    };

    let (pool_pda, _) = find_pool_pda(&currency_pda);
    let (vault_a_pda, _) = find_vault_pda(&pool_pda, &mint_pda);
    let (vault_b_pda, _) = find_vault_pda(&pool_pda, &usdc);

    let blockhash = svm.latest_blockhash();
    let ix = build_initialize_pool_ix(
        payer_pk,
        currency_pda,
        mint_pda,
        usdc,
        pool.sell_fee,
    );
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&payer_pk), &[&payer], blockhash);
    let res = send_tx(&mut svm, tx);
    assert!(res.is_ok());

    let user = create_payer(&mut svm);
    let user_pk = user.pubkey();

    let user_mint_ata = create_ata(&mut svm, &payer, &mint_pda, &user_pk);
    let user_usdc_ata = create_ata(&mut svm, &payer, &usdc, &user_pk);

    let mint_amt = as_token(1_139_973_004_315, usdc_decimals);
    let res = mint_to(&mut svm, &user, &usdc, &payer, &user_usdc_ata, mint_amt);
    assert!(res.is_ok());

    let mut max_supply_difference = 0;
    let mut max_usdc_locked_difference = 0;
    for i in 0..100_000 {
        if i == 0 || rand::thread_rng().gen_bool(0.5) {
            let buy_amount = rand::thread_rng().next_u64() % as_token(1_000_000_000, usdc_decimals); // Buy up to $1b
            let buy_ix = build_buy_tokens_ix(
                user_pk,
                pool_pda,
                mint_pda,
                usdc,
                buy_amount,
                0,
                user_mint_ata,
                user_usdc_ata,
            );
            let blockhash = svm.latest_blockhash();
            let tx = Transaction::new_signed_with_payer(&[buy_ix], Some(&user_pk), &[&user], blockhash);
            let res = send_tx(&mut svm, tx);
            assert!(res.is_ok());
        } else {
            let sell_amount = get_ata_balance(&svm, &user_mint_ata) / (i % 100 + 2); // Sell up to half
            let sell_ix = build_sell_tokens_ix(
                user_pk,
                pool_pda,
                mint_pda,
                usdc,
                sell_amount,
                0,
                user_mint_ata,
                user_usdc_ata,
            );
            let blockhash = svm.latest_blockhash();
            let tx = Transaction::new_signed_with_payer(&[sell_ix], Some(&user_pk), &[&user], blockhash);
            let res = send_tx(&mut svm, tx);
            assert!(res.is_ok());
        }

        let user_mint_balance = get_ata_balance(&svm, &user_mint_ata);
        let user_usdc_balance = get_ata_balance(&svm, &user_usdc_ata);
        let vault_mint_balance = get_ata_balance(&svm, &vault_a_pda);
        let vault_usdc_balance = get_ata_balance(&svm, &vault_b_pda);
        println!("User DSKY balance: {:?}", user_mint_balance);
        println!("Vault DSKY balance: {:?}", vault_mint_balance);
        println!("User USDC balance: {:?}", user_usdc_balance);
        println!("Vault USDC balance: {:?}", vault_usdc_balance);

        let mut difference;
        let curve = DiscreteExponentialCurve::default();
        let zero_supply = to_numeric(0, TOKEN_DECIMALS).unwrap();
        let usdc_buy_amount = to_numeric(vault_usdc_balance, usdc_decimals).unwrap();
        let expected_token_supply = curve.value_to_tokens(&zero_supply, &usdc_buy_amount).unwrap();
        let expected_quark_supply = from_numeric(expected_token_supply, TOKEN_DECIMALS).unwrap();
        if expected_quark_supply > user_mint_balance {
            difference = expected_quark_supply - user_mint_balance;
        } else {
            difference = user_mint_balance - expected_quark_supply;
        }
        if difference > max_supply_difference {
            max_supply_difference = difference
        }
        println!("DSKY supply difference from expectation: {:?}", difference);
        println!("Max DSKY supply difference from expectation so far: {:?}", max_supply_difference);

        let current_supply = to_numeric(user_mint_balance, TOKEN_DECIMALS).unwrap();
        let expected_locked_usdc = curve.tokens_to_value(&zero_supply, &current_supply).unwrap();
        let expected_locked_usdc_quarks = from_numeric(expected_locked_usdc, usdc_decimals).unwrap();
        if expected_locked_usdc_quarks > vault_usdc_balance {
            difference = expected_locked_usdc_quarks - vault_usdc_balance;
        } else {
            difference = vault_usdc_balance - expected_locked_usdc_quarks;
        }
        if difference > max_usdc_locked_difference {
            max_usdc_locked_difference = difference
        }
        println!("USDC locked difference from expectation: {:?}", difference);
        println!("Max USDC locked difference from expectation so far: {:?}", max_usdc_locked_difference);
    }

    println!("Max DSKY supply difference from expectation: {:?}", max_supply_difference);
    println!("Max USDC locked difference from expectation: {:?}", max_usdc_locked_difference); 

    assert!(max_supply_difference < 500_000, "Significant imprecision detected");
    assert!(max_usdc_locked_difference < 10, "Significant imprecision detected");
}
