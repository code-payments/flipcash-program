//! Jupiter Amm adapter for the Flipcash bonding-curve program.
//!
//! Each Flipcash currency has its own `LiquidityPool` PDA, an SPL mint for the
//! currency token (mint_a, "target") and a hardcoded base mint of USDF (mint_b).
//! Buys deposit USDF and mint currency tokens against an exponential bonding
//! curve; sells return USDF (with a `sell_fee` bps cut) and burn currency tokens.
//!
//! The quoting math here is a 1:1 mirror of the on-chain `buy_common` /
//! `sell_common` paths in `program/src/instruction/{buy,sell}.rs` — they share
//! the same `DiscreteExponentialCurve`, `to_numeric`/`from_numeric`, and
//! `from_basis_points` helpers from `flipcash-api`.

mod amm;

pub use amm::FlipcashAmm;
