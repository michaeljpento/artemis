use async_trait::async_trait;
use ethers::{
    prelude::{Address, Middleware, SignerMiddleware, U256, H256},
    signers::Signer,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString, Serialize, Deserialize)]
pub enum LiquidationStrategyType {
    FlashLoanLiquidation,
    DirectLiquidation,
    MEVProtectedLiquidation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FlashLoanProvider {
    AaveV3,
    Balancer,
    UniswapV3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DexType {
    UniswapV2,
    UniswapV3,
    Curve,
    Balancer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidationTarget {
    pub user: Address,
    pub collateral_asset: Address,
    pub debt_asset: Address,
    pub debt_to_cover: U256,
    pub health_factor: U256,
    pub liquidation_bonus: U256,
    pub expected_profit: f64,
    pub gas_cost_estimate: U256,
    pub receive_a_token: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRoute {
    pub token_in: Address,
    pub token_out: Address,
    pub amount_in: U256,
    pub min_amount_out: U256,
    pub dex_type: DexType,
    pub pool_address: Address,
    pub fee: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashLoanParameters {
    pub asset: Address,
    pub amount: U256,
    pub provider: FlashLoanProvider,
    pub fee_rate: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidationPath {
    pub target: LiquidationTarget,
    pub flash_loan: FlashLoanParameters,
    pub swap_routes: Vec<SwapRoute>,
    pub expected_profit_eth: f64,
    pub max_gas_price: U256,
    pub use_flashbots: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub enabled_strategies: Vec<LiquidationStrategyType>,
    pub liquidator_contract: Address,
    pub aave_pool: Address,
    pub aave_oracle: Address,
    pub min_profit_threshold: f64,
    pub max_gas_price: U256,
    pub gas_price_multiplier: f64,
    pub max_slippage: f64,
    pub health_factor_threshold: U256,
    pub max_liquidation_amount: U256,
    pub flashbots_enabled: bool,
    pub mev_protection_enabled: bool,
    pub circuit_breaker_enabled: bool,
    pub monitored_assets: Vec<Address>,
    pub supported_dexes: Vec<DexType>,
    pub flash_loan_config: FlashLoanConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlashLoanConfig {
    pub preferred_provider: FlashLoanProvider,
    pub max_flash_loan_amount: U256,
    pub fee_multiplier: f64,
    pub providers: HashMap<FlashLoanProvider, ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub contract_address: Address,
    pub fee_rate: U256,
    pub max_amount: U256,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub liquidation_targets: HashMap<Address, LiquidationTarget>,
    pub asset_prices: HashMap<Address, U256>,
    pub gas_price: U256,
    pub total_profits: f64,
    pub successful_liquidations: u64,
    pub failed_liquidations: u64,
    pub circuit_breaker_triggered: bool,
    pub last_update_block: u64,
}

#[derive(Debug, Clone)]
pub enum Action {
    ExecuteLiquidation {
        path: LiquidationPath,
        expected_profit: f64,
    },
    UpdatePrices {
        assets: Vec<Address>,
    },
    TriggerCircuitBreaker {
        reason: String,
    },
    None,
}

#[async_trait]
pub trait LiquidationStrategy<M: Middleware + 'static, S: Signer + 'static> {
    async fn process_event(&mut self, data: Vec<u8>) -> Vec<Action>;
    async fn update_state(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn find_liquidation_opportunities(&self) -> Vec<LiquidationTarget>;
    async fn calculate_profit(&self, target: &LiquidationTarget) -> Option<f64>;
    fn get_state(&self) -> &State;
    fn get_config(&self) -> &Config;
}

pub type ClientWithSigner<M, S> = SignerMiddleware<Arc<M>, S>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AaveUserData {
    pub total_collateral_eth: U256,
    pub total_debt_eth: U256,
    pub available_borrows_eth: U256,
    pub current_liquidation_threshold: U256,
    pub ltv: U256,
    pub health_factor: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveData {
    pub configuration: U256,
    pub liquidity_index: U256,
    pub variable_borrow_index: U256,
    pub current_liquidity_rate: U256,
    pub current_variable_borrow_rate: U256,
    pub current_stable_borrow_rate: U256,
    pub last_update_timestamp: u64,
    pub a_token_address: Address,
    pub stable_debt_token_address: Address,
    pub variable_debt_token_address: Address,
    pub interest_rate_strategy_address: Address,
    pub id: u8,
}
