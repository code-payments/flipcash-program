mod currency;
mod pool;

pub use currency::*;
pub use pool::*;

use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum AccountType {
    Unknown = 0,
    CurrencyConfig,
    LiquidityPool,
}
