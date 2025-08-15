pub const MINT: &[u8]           = b"mint";
pub const CURRENCY: &[u8]       = b"currency";
pub const POOL: &[u8]           = b"pool";
pub const TREASURY: &[u8]       = b"treasury";
pub const METADATA: &[u8]       = b"metadata";

pub const METADATA_URI: &str    = "https://fun.flipcash.com/{}/metadata.json";

pub const MAX_NAME_LEN: usize   = 32;
pub const MAX_SYMBOL_LEN: usize = 8;

pub const TOKEN_DECIMALS: u8    = 6; // Decimals for the new currency
pub const MAX_TOKEN_SUPPLY: u64 = 21_000_000;
pub const QUARKS_PER_TOKEN: u64 = 1_000_000;

// Constants for the default curve from $0.01 to $1_000_000 over 21_000_000 tokens
pub const CURVE_A: u128         = 11400_230149967394933471;
pub const CURVE_B: u128         = 0_000000877175273521;
pub const CURVE_C: u128         = CURVE_B;

