use shank::ShankInstruction;
use steel::*;
use crate::prelude::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive, ShankInstruction)]
pub enum InstructionType {
    Unknown = 0,

    #[account(0, signer, writable, name = "authority", desc = "Authority that owns the currency")]
    #[account(1, writable, name = "mint", desc = "SPL token mint PDA")]
    #[account(2, writable, name = "currency", desc = "Currency config PDA")]
    #[account(3, name = "token_program", desc = "SPL token program")]
    #[account(4, name = "system_program", desc = "System program")]
    #[account(5, name = "rent_sysvar", desc = "Rent sysvar")]
    InitializeCurrencyIx,

    #[account(0, signer, name = "authority", desc = "Authority that owns the currency")]
    #[account(1, name = "currency", desc = "Currency config account")]
    #[account(2, writable, name = "currency_mint", desc = "Currency SPL token mint")]
    #[account(3, name = "base_mint", desc = "Base token mint (eg. USDF or USDC)")]
    #[account(4, writable, name = "pool", desc = "Liquidity pool PDA")]
    #[account(5, writable, name = "currency_vault", desc = "Token vault PDA for currency tokens")]
    #[account(6, writable, name = "base_vault", desc = "Token vault PDA for base tokens")]
    #[account(7, name = "token_program", desc = "SPL token program")]
    #[account(8, name = "system_program", desc = "System program")]
    #[account(9, name = "rent_sysvar", desc = "Rent sysvar")]
    InitializePoolIx,

    #[account(0, signer, name = "authority", desc = "Authority that owns the currency")]
    #[account(1, name = "currency", desc = "Currency config account")]
    #[account(2, writable, name = "mint", desc = "Currency SPL token mint")]
    #[account(3, writable, name = "metadata", desc = "Metaplex metadata PDA")]
    #[account(4, name = "metadata_program", desc = "Metaplex token metadata program")]
    #[account(5, name = "token_program", desc = "SPL token program")]
    #[account(6, name = "system_program", desc = "System program")]
    #[account(7, name = "rent_sysvar", desc = "Rent sysvar")]
    InitializeMetadataIx,

    #[account(0, signer, name = "buyer", desc = "Buyer wallet")]
    #[account(1, name = "pool", desc = "Liquidity pool account")]
    #[account(2, name = "currency_mint", desc = "Currency SPL token mint")]
    #[account(3, name = "base_mint", desc = "Base token mint")]
    #[account(4, writable, name = "currency_vault", desc = "Currency token vault")]
    #[account(5, writable, name = "base_vault", desc = "Base token vault")]
    #[account(6, writable, name = "buyer_currency_token_account", desc = "Buyer's currency token account")]
    #[account(7, writable, name = "buyer_base_token_account", desc = "Buyer's base token account")]
    #[account(8, name = "token_program", desc = "SPL token program")]
    BuyTokensIx,

    #[account(0, signer, name = "seller", desc = "Seller wallet")]
    #[account(1, writable, name = "pool", desc = "Liquidity pool account")]
    #[account(2, name = "currency_mint", desc = "Currency SPL token mint")]
    #[account(3, name = "base_mint", desc = "Base token mint")]
    #[account(4, writable, name = "currency_vault", desc = "Currency token vault")]
    #[account(5, writable, name = "base_vault", desc = "Base token vault")]
    #[account(6, writable, name = "seller_currency_token_account", desc = "Seller's currency token account")]
    #[account(7, writable, name = "seller_base_token_account", desc = "Seller's base token account")]
    #[account(8, name = "token_program", desc = "SPL token program")]
    SellTokensIx,

    #[account(0, signer, name = "buyer", desc = "Buyer wallet")]
    #[account(1, name = "pool", desc = "Liquidity pool account")]
    #[account(2, name = "currency_mint", desc = "Currency SPL token mint")]
    #[account(3, name = "base_mint", desc = "Base token mint")]
    #[account(4, writable, name = "currency_vault", desc = "Currency token vault")]
    #[account(5, writable, name = "base_vault", desc = "Base token vault")]
    #[account(6, writable, name = "buyer_base_token_account", desc = "Buyer's base token account")]
    #[account(7, writable, name = "vm_authority", desc = "VM authority account")]
    #[account(8, writable, name = "vm", desc = "VM account")]
    #[account(9, writable, name = "vm_memory", desc = "VM memory account")]
    #[account(10, name = "vm_omnibus", desc = "VM omnibus account")]
    #[account(11, name = "vta_owner", desc = "Virtual token account owner")]
    #[account(12, name = "token_program", desc = "SPL token program")]
    #[account(13, name = "vm_program", desc = "OCP VM program")]
    BuyAndDepositIntoVmIx,

    #[account(0, signer, name = "seller", desc = "Seller wallet")]
    #[account(1, writable, name = "pool", desc = "Liquidity pool account")]
    #[account(2, name = "currency_mint", desc = "Currency SPL token mint")]
    #[account(3, name = "base_mint", desc = "Base token mint")]
    #[account(4, writable, name = "currency_vault", desc = "Currency token vault")]
    #[account(5, writable, name = "base_vault", desc = "Base token vault")]
    #[account(6, writable, name = "seller_currency_token_account", desc = "Seller's currency token account")]
    #[account(7, writable, name = "vm_authority", desc = "VM authority account")]
    #[account(8, writable, name = "vm", desc = "VM account")]
    #[account(9, writable, name = "vm_memory", desc = "VM memory account")]
    #[account(10, name = "vm_omnibus", desc = "VM omnibus token account")]
    #[account(11, name = "vta_owner", desc = "Virtual token account owner")]
    #[account(12, name = "token_program", desc = "SPL token program")]
    #[account(13, name = "vm_program", desc = "OCP VM program")]
    SellAndDepositIntoVmIx,

    #[account(0, signer, name = "payer", desc = "Transaction payer")]
    #[account(1, writable, name = "pool", desc = "Liquidity pool account")]
    #[account(2, writable, name = "base_mint", desc = "Base token mint")]
    #[account(3, writable, name = "base_vault", desc = "Base token vault")]
    #[account(4, name = "token_program", desc = "SPL token program")]
    BurnFeesIx,
}

instruction!(InstructionType, InitializeCurrencyIx);
instruction!(InstructionType, InitializePoolIx);
instruction!(InstructionType, InitializeMetadataIx);
instruction!(InstructionType, BuyTokensIx);
instruction!(InstructionType, SellTokensIx);
instruction!(InstructionType, BuyAndDepositIntoVmIx);
instruction!(InstructionType, SellAndDepositIntoVmIx);
instruction!(InstructionType, BurnFeesIx);

#[derive(Debug)]
pub struct ParsedInitializeCurrencyIx {
    pub name: String,
    pub symbol: String,
    pub seed: [u8; 32],

    pub bump: u8,
    pub mint_bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InitializeCurrencyIx {
    pub name: [u8; MAX_NAME_LEN],
    pub symbol: [u8; MAX_SYMBOL_LEN],
    pub seed: [u8; 32],

    pub bump: u8,
    pub mint_bump: u8,
    _padding: [u8; 6],
}

impl InitializeCurrencyIx {
    pub fn from_struct(parsed: ParsedInitializeCurrencyIx) -> Self {
        let name = to_name(&parsed.name);
        let symbol = to_symbol(&parsed.symbol);

        Self {
            name,
            symbol,
            seed: parsed.seed,

            bump: parsed.bump,
            mint_bump: parsed.mint_bump,
            _padding: [0; 6],
        }
    }

    pub fn to_struct(&self) -> Result<ParsedInitializeCurrencyIx, std::io::Error> {
        let name = from_name(&self.name);
        let symbol = from_symbol(&self.symbol);

        Ok(ParsedInitializeCurrencyIx {
            name,
            symbol,

            seed: self.seed,

            bump: self.bump,
            mint_bump: self.mint_bump,
        })
    }
}

#[derive(Debug)]
pub struct ParsedInitializePoolIx {
    pub sell_fee: u16,

    pub bump: u8,
    pub vault_a_bump: u8,
    pub vault_b_bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InitializePoolIx {
    pub sell_fee: [u8; 2],

    pub bump: u8,
    pub vault_a_bump: u8,
    pub vault_b_bump: u8,
    _padding: [u8; 1],
}

impl InitializePoolIx {
    pub fn from_struct(parsed: ParsedInitializePoolIx) -> Self {
        Self {
            sell_fee: parsed.sell_fee.to_le_bytes(),

            bump: parsed.bump,
            vault_a_bump: parsed.vault_a_bump,
            vault_b_bump: parsed.vault_b_bump,
            _padding: [0; 1],
        }
    }

    pub fn to_struct(&self) -> Result<ParsedInitializePoolIx, std::io::Error> {
        Ok(ParsedInitializePoolIx {
            sell_fee: u16::from_le_bytes(self.sell_fee),

            bump: self.bump,
            vault_a_bump: self.vault_a_bump,
            vault_b_bump: self.vault_b_bump,
        })
    }
}

#[derive(Debug)]
pub struct ParsedInitializeMetadataIx {
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InitializeMetadataIx {
}

impl InitializeMetadataIx {
    pub fn from_struct(_parsed: ParsedInitializeMetadataIx) -> Self {
        Self {
        }
    }

    pub fn to_struct(&self) -> Result<ParsedInitializeMetadataIx, std::io::Error> {
        Ok(ParsedInitializeMetadataIx {
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

#[derive(Debug)]
pub struct ParsedBuyAndDepositIntoVmIx {
    pub in_amount: u64,
    pub min_amount_out: u64,
    pub vm_memory_index: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct BuyAndDepositIntoVmIx {
    pub in_amount: [u8; 8],
    pub min_amount_out: [u8; 8],
    pub vm_memory_index: [u8; 2],
}

impl BuyAndDepositIntoVmIx {
    pub fn from_struct(parsed: ParsedBuyAndDepositIntoVmIx) -> Self {
        Self {
            in_amount: parsed.in_amount.to_le_bytes(),
            min_amount_out: parsed.min_amount_out.to_le_bytes(),
            vm_memory_index: parsed.vm_memory_index.to_le_bytes(),
        }
    }

    pub fn to_struct(&self) -> ParsedBuyAndDepositIntoVmIx {
        ParsedBuyAndDepositIntoVmIx {
            in_amount: u64::from_le_bytes(self.in_amount),
            min_amount_out: u64::from_le_bytes(self.min_amount_out),
            vm_memory_index: u16::from_le_bytes(self.vm_memory_index),
        }
    }
}

#[derive(Debug)]
pub struct ParsedSellAndDepositIntoVmIx {
    pub in_amount: u64,
    pub min_amount_out: u64,
    pub vm_memory_index: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SellAndDepositIntoVmIx {
    pub in_amount: [u8; 8],
    pub min_amount_out: [u8; 8],
    pub vm_memory_index: [u8; 2],
}

impl SellAndDepositIntoVmIx {
    pub fn from_struct(parsed: ParsedSellAndDepositIntoVmIx) -> Self {
        Self {
            in_amount: parsed.in_amount.to_le_bytes(),
            min_amount_out: parsed.min_amount_out.to_le_bytes(),
            vm_memory_index: parsed.vm_memory_index.to_le_bytes(),
        }
    }

    pub fn to_struct(&self) -> ParsedSellAndDepositIntoVmIx {
        ParsedSellAndDepositIntoVmIx {
            in_amount: u64::from_le_bytes(self.in_amount),
            min_amount_out: u64::from_le_bytes(self.min_amount_out),
            vm_memory_index: u16::from_le_bytes(self.vm_memory_index),
        }
    }
}

#[derive(Debug)]
pub struct ParsedBurnFeesIx {
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct BurnFeesIx {
}

impl BurnFeesIx {
    pub fn from_struct(_parsed: ParsedBurnFeesIx) -> Self {
        Self {
        }
    }

    pub fn to_struct(&self) -> ParsedBurnFeesIx {
        ParsedBurnFeesIx {
        }
    }
}
