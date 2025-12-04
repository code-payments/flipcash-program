#![allow(unexpected_cfgs)]

pub mod consts;
pub mod curve;
pub mod instruction;
pub mod state;
pub mod pda;
pub mod cpis;
pub mod utils;
pub mod event;
mod macros;

#[cfg(not(target_os = "solana"))]
pub mod sdk;

pub mod prelude {
    pub use crate::consts::*;
    pub use crate::curve::*;
    pub use crate::instruction::*;
    pub use crate::state::*; 
    pub use crate::pda::*;
    pub use crate::cpis::*;
    pub use crate::utils::*;

    #[cfg(not(target_os = "solana"))]
    pub use crate::sdk::*;
}

use steel::*;

declare_id!("ccZR1qzNyMaHDB47PkqDZVpNdimji7wJf65zyfGR3FJ");
