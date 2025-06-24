use steel::*;
use crate::consts::*;
use crate::utils::to_name;

pub fn find_mint_pda(authority: &Pubkey, name: &str, seed: &[u8; 32]) -> (Pubkey, u8) {
    let name = to_name(name);
    Pubkey::find_program_address(
        &[MINT, authority.as_ref(), name.as_ref(), seed],
        &crate::id(),
    )
}

pub fn find_currency_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[CURRENCY, mint.as_ref()],
        &crate::id(),
    )
}

pub fn find_pool_pda(currency: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[POOL, currency.as_ref()],
        &crate::id(),
    )
}

pub fn find_vault_pda(pool: &Pubkey, mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[TREASURY, pool.as_ref(), mint.as_ref()],
        &crate::id(),
    )
}

pub fn metadata_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[ METADATA, mpl_token_metadata::ID.as_ref(), mint.as_ref() ],
        &mpl_token_metadata::ID,
    )
}
