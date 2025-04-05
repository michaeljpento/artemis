use ethers::core::types::{Address, U256};
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Configuration for the multi-strategy system
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Address of the flash loan executor contract
    pub flash_executor_address: Address,
    /// Address of the JIT liquidity provider contract
    pub jit_provider_address: Address,
    /// Minimum profit threshold in ETH
    pub min_profit_threshold: f64,
    /// Maximum gas price willing to pay (in gwei)
    pub max_gas_price: u64,
    /// Timeout for transaction submission
    pub submission_timeout: u64,
    /// Whether to enable arbitrage strategy
    pub enable_arbitrage: bool,
    /// Whether to enable JIT liquidity strategy
    pub enable_jit: bool,
    /// Whether to enable backrunning strategy
    pub enable_backrunning: bool,
    /// List of tokens to monitor for opportunities
    pub monitored_tokens: Vec<Address>,
    /// DEX pools to monitor
    pub monitored_pools: Vec<PoolConfig>,
}

/// Configuration for a DEX pool
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoolConfig {
    /// Address of the pool
    pub address: Address,
    /// Type of pool (V2, V3, etc.)
    pub pool_type: PoolType,
    /// The tokens in the pool
    pub tokens: [Address; 2],
    /// Optional fee tier (for V3 pools)
    pub fee_tier: Option<u32>,
}

/// Type of DEX pool
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum PoolType {
    UniswapV2,
    UniswapV3,
    SushiSwap,
    Curve,
}

/// In-memory state for the strategy
#[derive(Debug)]
pub struct State {
    /// Token price cache
    pub token_prices: HashMap<Address, f64>,
    /// Pool reserve cache
    pub pool_reserves: HashMap<Address, PoolReserves>,
    /// Tracked transactions
    pub tracked_txs: HashMap<H256, TrackedTransaction>,
    /// Last price update timestamp
    pub last_price_update: SystemTime,
}

impl Default for State {
    fn default() -> Self {
        Self {
            token_prices: HashMap::new(),
            pool_reserves: HashMap::new(),
            tracked_txs: HashMap::new(),
            last_price_update: SystemTime::now(),
        }
    }
}

/// Pool reserves information
#[derive(Debug, Clone)]
pub struct PoolReserves {
    /// Address of token0
    pub token0: Address,
    /// Address of token1
    pub token1: Address,
    /// Reserve of token0
    pub reserve0: U256,
    /// Reserve of token1
    pub reserve1: U256,
    /// Last updated timestamp
    pub last_updated: SystemTime,
    /// Pool type
    pub pool_type: PoolType,
}

/// Transaction being tracked
#[derive(Debug, Clone)]
pub struct TrackedTransaction {
    /// Hash of transaction
    pub tx_hash: H256,
    /// Time transaction was sent
    pub sent_at: SystemTime,
    /// Profit expectation
    pub expected_profit: f64,
    /// Type of opportunity
    pub opportunity_type: OpportunityType,
}

/// Price update information
#[derive(Debug, Clone)]
pub struct PriceUpdate {
    /// Token that was updated
    pub token: Address,
    /// New price in ETH
    pub price: f64,
}

/// Type of opportunity
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpportunityType {
    /// Cross-DEX arbitrage
    Arbitrage,
    /// JIT liquidity provision
    JitLiquidity,
    /// MEV-Share backrunning
    Backrunning,
}

/// Metrics for the strategy
#[derive(Debug, Default)]
pub struct Metrics {
    /// Total profit earned
    pub total_profit: f64,
    /// Number of successful transactions
    pub successful_txs: u64,
    /// Number of failed transactions
    pub failed_txs: u64,
    /// Number of arbitrage opportunities found
    pub arbitrage_opportunities: u64,
    /// Number of JIT opportunities found
    pub jit_opportunities: u64,
    /// Number of backrunning opportunities found
    pub backrunning_opportunities: u64,
    /// Average gas price paid
    pub avg_gas_price: f64,
}

/// Action to take based on an opportunity
#[derive(Debug, Clone)]
pub enum Action {
    /// Execute a cross-DEX arbitrage
    ExecuteArbitrage {
        /// Path of the arbitrage
        path: ArbitragePath,
        /// Expected profit
        expected_profit: f64,
    },
    /// Execute JIT liquidity provision
    ExecuteJitLiquidity {
        /// Pool to provide liquidity to
        pool: Address,
        /// Amounts to provide
        amounts: [U256; 2],
        /// Expected profit
        expected_profit: f64,
    },
    /// Execute a backrun on MEV-Share
    ExecuteBackrun {
        /// Original transaction to backrun
        target_tx: H256,
        /// Backrun data
        backrun_data: Vec<u8>,
        /// Expected profit
        expected_profit: f64,
    },
}

/// Path for an arbitrage opportunity
#[derive(Debug, Clone)]
pub struct ArbitragePath {
    /// Start token
    pub start_token: Address,
    /// Amount to borrow
    pub borrow_amount: U256,
    /// Sequence of swaps to execute
    pub swaps: Vec<Swap>,
}

/// A single swap in an arbitrage path
#[derive(Debug, Clone)]
pub struct Swap {
    /// Pool to swap on
    pub pool: Address,
    /// Type of pool
    pub pool_type: PoolType,
    /// Direction of swap (0->1 or 1->0)
    pub zero_for_one: bool,
    /// Expected swap amount (input)
    pub amount_in: U256,
    /// Expected output amount
    pub expected_out: U256,
}