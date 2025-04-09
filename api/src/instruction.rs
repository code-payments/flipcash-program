use steel::*;
use crate::prelude::*;
use crate::curve::{ExponentialCurve, RawExponentialCurve};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum InstructionType {
    Unknown = 0,

    InitializeCurrencyIx,
    InitializePoolIx,

    BuyTokensIx,
    SellTokensIx,
}

instruction!(InstructionType, InitializeCurrencyIx);
instruction!(InstructionType, InitializePoolIx);
instruction!(InstructionType, BuyTokensIx);
instruction!(InstructionType, SellTokensIx);

#[derive(Debug)]
pub struct ParsedInitializeCurrencyIx {
    pub name: String,
    pub seed: [u8; 32],
    pub max_supply: u64,
    pub decimal_places: u8,

    pub bump: u8,
    pub mint_bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InitializeCurrencyIx {
    pub name: [u8; MAX_NAME_LEN],
    pub seed: [u8; 32],
    pub max_supply: [u8; 8],
    pub decimal_places: u8,

    pub bump: u8,
    pub mint_bump: u8,
    _padding: [u8; 5],
}

impl InitializeCurrencyIx {
    pub fn from_struct(parsed: ParsedInitializeCurrencyIx) -> Self {
        let name = to_name(&parsed.name);
        Self {
            name,
            seed: parsed.seed,
            max_supply: parsed.max_supply.to_le_bytes(),
            decimal_places: parsed.decimal_places,

            bump: parsed.bump,
            mint_bump: parsed.mint_bump,
            _padding: [0; 5],
        }
    }

    pub fn to_struct(&self) -> Result<ParsedInitializeCurrencyIx, std::io::Error> {
        let name = from_name(&self.name);
        Ok(ParsedInitializeCurrencyIx {
            name,
            seed: self.seed,
            max_supply: u64::from_le_bytes(self.max_supply),
            decimal_places: self.decimal_places,

            bump: self.bump,
            mint_bump: self.mint_bump,
        })
    }
}

#[derive(Debug)]
pub struct ParsedInitializePoolIx {
    pub supply: u64, // Number of tokens to mint to the pool
    pub curve: ExponentialCurve,
    pub purchase_cap: u64,
    pub sale_cap: u64,
    pub buy_fee: u32,
    pub sell_fee: u32,
    pub go_live_wait_time: i64,

    pub bump: u8,
    pub vault_a_bump: u8,
    pub vault_b_bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InitializePoolIx {
    pub supply: [u8; 8],
    pub curve: RawExponentialCurve,
    pub purchase_cap: [u8; 8],
    pub sale_cap: [u8; 8],
    pub buy_fee: [u8; 4],
    pub sell_fee: [u8; 4],
    pub go_live_wait_time: [u8; 8],

    pub bump: u8,
    pub vault_a_bump: u8,
    pub vault_b_bump: u8,
    _padding: [u8; 5],
}

impl InitializePoolIx {
    pub fn from_struct(parsed: ParsedInitializePoolIx) -> Self {
        Self {
            supply: parsed.supply.to_le_bytes(),
            curve: RawExponentialCurve::from_struct(parsed.curve),
            purchase_cap: parsed.purchase_cap.to_le_bytes(),
            sale_cap: parsed.sale_cap.to_le_bytes(),
            buy_fee: parsed.buy_fee.to_le_bytes(),
            sell_fee: parsed.sell_fee.to_le_bytes(),
            go_live_wait_time: parsed.go_live_wait_time.to_le_bytes(),

            bump: parsed.bump,
            vault_a_bump: parsed.vault_a_bump,
            vault_b_bump: parsed.vault_b_bump,
            _padding: [0; 5],
        }
    }

    pub fn to_struct(&self) -> Result<ParsedInitializePoolIx, std::io::Error> {
        Ok(ParsedInitializePoolIx {
            supply: u64::from_le_bytes(self.supply),
            curve: self.curve.to_struct()?,
            purchase_cap: u64::from_le_bytes(self.purchase_cap),
            sale_cap: u64::from_le_bytes(self.sale_cap),
            buy_fee: u32::from_le_bytes(self.buy_fee),
            sell_fee: u32::from_le_bytes(self.sell_fee),
            go_live_wait_time: i64::from_le_bytes(self.go_live_wait_time),

            bump: self.bump,
            vault_a_bump: self.vault_a_bump,
            vault_b_bump: self.vault_b_bump,
        })
    }
}

#[derive(Debug)]
pub struct ParsedBuyTokensIx {
    pub in_amount: u64,
    pub min_amount_out: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct BuyTokensIx {
    pub in_amount: [u8; 8],
    pub min_amount_out: [u8; 8],
}

impl BuyTokensIx {
    pub fn from_struct(parsed: ParsedBuyTokensIx) -> Self {
        Self {
            in_amount: parsed.in_amount.to_le_bytes(),
            min_amount_out: parsed.min_amount_out.to_le_bytes(),
        }
    }

    pub fn to_struct(&self) -> ParsedBuyTokensIx {
        ParsedBuyTokensIx {
            in_amount: u64::from_le_bytes(self.in_amount),
            min_amount_out: u64::from_le_bytes(self.min_amount_out),
        }
    }
}

#[derive(Debug)]
pub struct ParsedSellTokensIx {
    pub in_amount: u64,
    pub min_amount_out: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SellTokensIx {
    pub in_amount: [u8; 8],
    pub min_amount_out: [u8; 8],
}

impl SellTokensIx {
    pub fn from_struct(parsed: ParsedSellTokensIx) -> Self {
        Self {
            in_amount: parsed.in_amount.to_le_bytes(),
            min_amount_out: parsed.min_amount_out.to_le_bytes(),
        }
    }

    pub fn to_struct(&self) -> ParsedSellTokensIx {
        ParsedSellTokensIx {
            in_amount: u64::from_le_bytes(self.in_amount),
            min_amount_out: u64::from_le_bytes(self.min_amount_out),
        }
    }
}


