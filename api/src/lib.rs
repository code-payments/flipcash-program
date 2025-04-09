#![allow(unexpected_cfgs)]

pub mod consts;
pub mod curve;
pub mod instruction;
pub mod state;
pub mod pda;
pub mod cpis;
pub mod utils;
pub mod math;

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
    pub use crate::math::*;

    #[cfg(not(target_os = "solana"))]
    pub use crate::sdk::*;
}

use steel::*;

declare_id!("fcSsgMo1FsEhDhTVCt9jALYuEU9o8YFfEQNrxqwiaV8"); 
