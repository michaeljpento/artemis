use async_trait::async_trait;
use ethers::{
    prelude::{Address, Middleware, SignerMiddleware, U256, H256},
    signers::Signer,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use strum_macros::{Display, EnumString};

// Enums for strategy types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString, Serialize, Deserialize)]
pub enum StrategyType {
    Arbitrage,
    JitLiquidity,
    MEVShareBackrun,
}

// Enums for DEX types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DexType {
    UniswapV2,
    UniswapV3,
    Curve,
}

// Enums for flash loan providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FlashLoanProvider {
    Aave,
    Balancer,
    UniswapV3,
}

// Pool reserve data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolReserves {
    pub address: Address,
    pub token0: Address,
    pub token1: Address,
    pub reserve0: U256,
    pub reserve1: U256,
    pub fee: u32,          // Represented in basis points (e.g., 30 = 0.3%)
    pub dex_type: DexType,
}

// Swap data structure for arbitrage paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Swap {
    pub pool_address: Address,
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: U256,
    pub min_amount_out: U256,
    pub zero_for_one: bool,
    pub dex_type: DexType,
    // Curve-specific fields
    pub i: Option<i128>,
    pub j: Option<i128>,
    pub use_underlying: Option<bool>,
}

// Arbitrage path for flash loan arbitrage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitragePath {
    pub start_token: Address,
    pub borrow_amount: U256,
    pub swaps: Vec<Swap>,
    pub flash_loan_provider: FlashLoanProvider,
}

// JIT liquidity parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JITLiquidityParams {
    pub pool: Address,
    pub token0: Address,
    pub token1: Address,
    pub amount0: U256,
    pub amount1: U256,
    pub dex_type: DexType,
    pub min_fee_expected: U256,
    pub flash_loan_provider: FlashLoanProvider,
    // V3-specific fields
    pub fee: Option<u32>,
    pub tick_lower: Option<i32>,
    pub tick_upper: Option<i32>,
    pub token_id: Option<U256>,
}

// MEV-Share backrun parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackrunParams {
    pub target_tx: H256,
    pub backrun_data: Vec<u8>,
    pub expected_profit: f64,
}

// Strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub enabled_strategies: Vec<StrategyType>,
    pub flash_arb_executor: Address,
    pub jit_liquidity_provider: Address,
    pub tokens: Vec<Address>,
    pub min_profit_threshold: f64,      // Minimum profit in ETH to consider an opportunity
    pub gas_price_multiplier: f64,      // Multiplier for gas cost estimation
    pub max_slippage: f64,              // Maximum allowed slippage in percentage
    pub flash_loan_fee_multiplier: f64, // Multiplier to account for flash loan fees
    pub arbitrage: ArbitrageConfig,
    pub jit_liquidity: JITLiquidityConfig,
    pub mev_share: MEVShareConfig,
}

// Arbitrage-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArbitrageConfig {
    pub max_path_length: usize,
    pub min_profit_threshold: f64,      // Specific to arbitrage
    pub max_flash_loan_amount: U256,
    pub preferred_flash_loan_provider: FlashLoanProvider,
}

// JIT liquidity-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JITLiquidityConfig {
    pub min_fee_expected: f64,         // Minimum expected fee in ETH
    pub position_duration: u64,        // How long to hold the position in seconds
    pub preferred_flash_loan_provider: FlashLoanProvider,
}

// MEV-Share-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MEVShareConfig {
    pub backrun_enabled: bool,
    pub min_backrun_profit: f64,
}

// Strategy state
#[derive(Debug, Clone, Default)]
pub struct State {
    pub pools: HashMap<Address, PoolReserves>,
    pub token_prices: HashMap<Address, f64>,
    pub gas_price: U256,
    pub active_jit_positions: Vec<JITLiquidityParams>,
    pub historical_profits: HashMap<StrategyType, f64>,
}

// Actions that the strategy can take
#[derive(Debug, Clone)]
pub enum Action {
    ExecuteArbitrage { path: ArbitragePath, expected_profit: f64 },
    ExecuteJitLiquidity { params: JITLiquidityParams, expected_profit: f64 },
    ExecuteBackrun { params: BackrunParams },
    None,
}

// Strategy trait
#[async_trait]
pub trait Strategy<M: Middleware + 'static, S: Signer + 'static> {
    async fn process_event(&mut self, data: Vec<u8>) -> Vec<Action>;
    async fn update_state(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn get_state(&self) -> &State;
    fn get_config(&self) -> &Config;
}

// Helper type for middleware
pub type ClientWithSigner<M, S> = SignerMiddleware<Arc<M>, S>;