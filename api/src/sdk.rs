use steel::*;
use crate::prelude::*;

pub fn build_initialize_currency_ix(
    authority: Pubkey,
    name: String,
    symbol: String,
    seed: [u8; 32],
) -> Instruction {
    let (mint_pda, mint_bump) = find_mint_pda(&authority, &name, &seed);
    let (currency_pda, currency_bump) = find_currency_pda(&mint_pda);

    println!("mint_pda: {}, bump: {} (target)", mint_pda, mint_bump);

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(authority, true),
            AccountMeta::new(mint_pda, false),
            AccountMeta::new(currency_pda, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: InitializeCurrencyIx::from_struct(
            ParsedInitializeCurrencyIx {
                name,
                symbol,
                seed,
                bump: currency_bump,
                mint_bump,
            }
        ).to_bytes(),
    }
}

pub fn build_initialize_pool_ix(
    authority: Pubkey,
    currency: Pubkey,
    target_mint: Pubkey,
    base_mint: Pubkey,    // Probably USDC

    buy_fee: u16,
    sell_fee: u16,

    fee_target : Pubkey,
    fee_base: Pubkey,
    ) -> Instruction {

    let (pool_pda, pool_bump) = find_pool_pda(&currency);
    let (vault_a_pda, vault_a_bump) = find_vault_pda(&pool_pda, &target_mint);
    let (vault_b_pda, vault_b_bump) = find_vault_pda(&pool_pda, &base_mint);

    println!("pool_pda: {}, bump: {}", pool_pda, pool_bump);
    println!("vault_a_pda: {}, bump: {} (target)", vault_a_pda, vault_a_bump);
    println!("vault_b_pda: {}, bump: {} (base)", vault_b_pda, vault_b_bump);

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(authority, true),
            AccountMeta::new(currency, false),
            AccountMeta::new(target_mint, false),
            AccountMeta::new_readonly(base_mint, false),
            AccountMeta::new(pool_pda, false),
            AccountMeta::new(vault_a_pda, false),
            AccountMeta::new(vault_b_pda, false),
            AccountMeta::new_readonly(fee_target, false),
            AccountMeta::new_readonly(fee_base, false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: InitializePoolIx::from_struct(
            ParsedInitializePoolIx {
                buy_fee,
                sell_fee,
                bump: pool_bump,
                vault_a_bump,
                vault_b_bump,
            }
        ).to_bytes(),
    }
}

pub fn build_initialize_metadata_ix(
    authority: Pubkey,
    currency: Pubkey,
    mint: Pubkey,
    ) -> Instruction {

    let (metadata_pda, metadata_bump) = metadata_pda(&mint);

    println!("metadata_pda: {}, bump: {}", metadata_pda, metadata_bump);

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(authority, true),
            AccountMeta::new_readonly(currency, false),
            AccountMeta::new_readonly(mint, false),
            AccountMeta::new(metadata_pda, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(mpl_token_metadata::ID, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: InitializeMetadataIx::from_struct(
            ParsedInitializeMetadataIx {}
        ).to_bytes(),
    }
}

pub fn build_buy_tokens_ix(
    buyer: Pubkey,
    pool: Pubkey,
    currency: Pubkey,
    target_mint: Pubkey,
    base_mint: Pubkey,
    in_amount: u64,
    min_amount_out: u64,
    buyer_target_ata: Pubkey,
    buyer_base_ata: Pubkey,
    fee_target: Pubkey,
    fee_base: Pubkey,
) -> Instruction {
    let (vault_a_pda, vault_a_bump) = find_vault_pda(&pool, &target_mint);
    let (vault_b_pda, vault_b_bump) = find_vault_pda(&pool, &base_mint);

    println!("vault_a_pda: {}, bump: {} (target)", vault_a_pda, vault_a_bump);
    println!("vault_b_pda: {}, bump: {} (base)", vault_b_pda, vault_b_bump);

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(buyer, true),
            AccountMeta::new(pool, false),
            AccountMeta::new(currency, false),
            AccountMeta::new(target_mint, false),
            AccountMeta::new_readonly(base_mint, false),
            AccountMeta::new(vault_a_pda, false),
            AccountMeta::new(vault_b_pda, false),
            AccountMeta::new(buyer_target_ata, false),
            AccountMeta::new(buyer_base_ata, false),
            AccountMeta::new(fee_target, false),
            AccountMeta::new_readonly(fee_base, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: BuyTokensIx::from_struct(ParsedBuyTokensIx {
            in_amount,
            min_amount_out,
        }).to_bytes(),
    }
}

pub fn build_sell_tokens_ix(
    seller: Pubkey,
    pool: Pubkey,
    currency: Pubkey,
    target_mint: Pubkey,
    base_mint: Pubkey,
    in_amount: u64,
    min_amount_out: u64,
    seller_target_ata: Pubkey,
    seller_base_ata: Pubkey,
    fee_target: Pubkey,
    fee_base: Pubkey,
) -> Instruction {
    let (vault_a_pda, vault_a_bump) = find_vault_pda(&pool, &target_mint);
    let (vault_b_pda, vault_b_bump) = find_vault_pda(&pool, &base_mint);

    println!("vault_a_pda: {}, bump: {} (target)", vault_a_pda, vault_a_bump);
    println!("vault_b_pda: {}, bump: {} (base)", vault_b_pda, vault_b_bump);

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(seller, true),
            AccountMeta::new(pool, false),
            AccountMeta::new(currency, false),
            AccountMeta::new(target_mint, false),
            AccountMeta::new_readonly(base_mint, false),
            AccountMeta::new(vault_a_pda, false),
            AccountMeta::new(vault_b_pda, false),
            AccountMeta::new(seller_target_ata, false),
            AccountMeta::new(seller_base_ata, false),
            AccountMeta::new_readonly(fee_target, false),
            AccountMeta::new(fee_base, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: SellTokensIx::from_struct(ParsedSellTokensIx {
            in_amount,
            min_amount_out,
        }).to_bytes(),
    }
}
