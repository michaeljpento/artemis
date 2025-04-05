//! A multi-strategy MEV system implementing arbitrage, JIT liquidity, and backrunning
//! strategies using flash loans for capital efficiency.

/// Strategy implementation module
pub mod strategy;

/// Type definitions
pub mod types;

#[cfg(test)]
mod tests;

pub use strategy::{Event, MultiStrategy};
pub use types::{Action, Config};