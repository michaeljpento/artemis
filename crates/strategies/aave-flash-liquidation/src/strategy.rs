use crate::types::*;
use async_trait::async_trait;
use ethers::{
    prelude::{Address, Middleware, Signer, U256},
    utils::format_units,
    contract::Contract,
};
use std::{sync::Arc, ops::{Mul, Div}};
use tracing::{debug, warn, error};

pub struct AaveFlashLiquidationStrategy<M: Middleware + 'static, S: Signer + 'static> {
    pub client: Arc<ClientWithSigner<M, S>>,
    pub config: Config,
    pub state: State,
    pub liquidator_contract: Contract<ClientWithSigner<M, S>>,
    pub aave_pool: Contract<ClientWithSigner<M, S>>,
    pub aave_oracle: Contract<ClientWithSigner<M, S>>,
}

impl<M: Middleware + 'static, S: Signer + 'static> AaveFlashLiquidationStrategy<M, S> {
    pub fn new(
        client: Arc<ClientWithSigner<M, S>>,
        config: Config,
        liquidator_abi: ethers::abi::Abi,
        pool_abi: ethers::abi::Abi,
        oracle_abi: ethers::abi::Abi,
    ) -> Self {
        let liquidator_contract = Contract::new(
            config.liquidator_contract,
            liquidator_abi,
            client.clone(),
        );
        
        let aave_pool = Contract::new(
            config.aave_pool,
            pool_abi,
            client.clone(),
        );
        
        let aave_oracle = Contract::new(
            config.aave_oracle,
            oracle_abi,
            client.clone(),
        );

        Self {
            client,
            config,
            state: State::default(),
            liquidator_contract,
            aave_pool,
            aave_oracle,
        }
    }

    async fn process_block_event(&mut self) -> Vec<Action> {
        let mut actions = Vec::new();

        if self.state.circuit_breaker_triggered {
            debug!("Circuit breaker is active, skipping liquidation opportunities");
            return actions;
        }

        for strategy_type in &self.config.enabled_strategies {
            match strategy_type {
                LiquidationStrategyType::FlashLoanLiquidation => {
                    if let Some(action) = self.find_flash_loan_liquidation_opportunities().await {
                        actions.push(action);
                    }
                }
                LiquidationStrategyType::DirectLiquidation => {
                    if let Some(action) = self.find_direct_liquidation_opportunities().await {
                        actions.push(action);
                    }
                }
                LiquidationStrategyType::MEVProtectedLiquidation => {
                    if let Some(action) = self.find_mev_protected_liquidation_opportunities().await {
                        actions.push(action);
                    }
                }
            }
        }

        actions
    }

    async fn find_flash_loan_liquidation_opportunities(&self) -> Option<Action> {
        let liquidation_targets = self.find_liquidation_opportunities().await;
        
        if liquidation_targets.is_empty() {
            return None;
        }

        let mut most_profitable_path = None;
        let mut highest_profit = 0.0;

        for target in liquidation_targets {
            if let Some(profit) = self.calculate_profit(&target).await {
                if profit > highest_profit && profit > self.config.min_profit_threshold {
                    if let Some(path) = self.create_liquidation_path(&target, profit).await {
                        highest_profit = profit;
                        most_profitable_path = Some(path);
                    }
                }
            }
        }

        most_profitable_path.map(|path| Action::ExecuteLiquidation {
            path,
            expected_profit: highest_profit,
        })
    }

    async fn find_direct_liquidation_opportunities(&self) -> Option<Action> {
        None
    }

    async fn find_mev_protected_liquidation_opportunities(&self) -> Option<Action> {
        if !self.config.mev_protection_enabled {
            return None;
        }

        let liquidation_targets = self.find_liquidation_opportunities().await;
        
        for target in liquidation_targets {
            if let Some(profit) = self.calculate_profit(&target).await {
                if profit > self.config.min_profit_threshold {
                    if let Some(mut path) = self.create_liquidation_path(&target, profit).await {
                        path.use_flashbots = true;
                        return Some(Action::ExecuteLiquidation {
                            path,
                            expected_profit: profit,
                        });
                    }
                }
            }
        }

        None
    }

    async fn create_liquidation_path(&self, target: &LiquidationTarget, expected_profit: f64) -> Option<LiquidationPath> {
        let flash_loan = FlashLoanParameters {
            asset: target.debt_asset,
            amount: target.debt_to_cover,
            provider: self.config.flash_loan_config.preferred_provider,
            fee_rate: self.get_flash_loan_fee_rate(&self.config.flash_loan_config.preferred_provider),
        };

        let swap_routes = self.calculate_optimal_swap_routes(
            target.collateral_asset,
            target.debt_asset,
            target.liquidation_bonus,
        ).await?;

        Some(LiquidationPath {
            target: target.clone(),
            flash_loan,
            swap_routes,
            expected_profit_eth: expected_profit,
            max_gas_price: self.config.max_gas_price,
            use_flashbots: self.config.flashbots_enabled,
        })
    }

    async fn calculate_optimal_swap_routes(
        &self,
        collateral_asset: Address,
        debt_asset: Address,
        liquidation_bonus: U256,
    ) -> Option<Vec<SwapRoute>> {
        let mut routes = Vec::new();

        for dex_type in &self.config.supported_dexes {
            if let Some(route) = self.find_best_route_for_dex(
                collateral_asset,
                debt_asset,
                liquidation_bonus,
                *dex_type,
            ).await {
                routes.push(route);
            }
        }

        if routes.is_empty() {
            None
        } else {
            routes.sort_by(|a, b| b.min_amount_out.cmp(&a.min_amount_out));
            Some(vec![routes[0].clone()])
        }
    }

    async fn find_best_route_for_dex(
        &self,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
        dex_type: DexType,
    ) -> Option<SwapRoute> {
        match dex_type {
            DexType::UniswapV2 => {
                self.find_uniswap_v2_route(token_in, token_out, amount_in).await
            }
            DexType::UniswapV3 => {
                self.find_uniswap_v3_route(token_in, token_out, amount_in).await
            }
            DexType::Curve => {
                self.find_curve_route(token_in, token_out, amount_in).await
            }
            DexType::Balancer => {
                self.find_balancer_route(token_in, token_out, amount_in).await
            }
        }
    }

    async fn find_uniswap_v2_route(
        &self,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
    ) -> Option<SwapRoute> {
        let dex_config = self.config.dex_configs.get(&DexType::UniswapV2)?;
        
        Some(SwapRoute {
            dex_type: DexType::UniswapV2,
            token_in,
            token_out,
            amount_in,
            min_amount_out: amount_in.mul(U256::from(95)).div(U256::from(100)),
            pool_address: dex_config.router_address,
            path: vec![format!("{:?}", token_in), format!("{:?}", token_out)],
            fee: None,
        })
    }

    async fn find_uniswap_v3_route(
        &self,
        token_in: Address,
        token_out: Address,
        amount_in: U256,
    ) -> Option<SwapRoute> {
        let dex_config = self.config.dex_configs.get(&DexType::UniswapV3)?;
        
        Some(SwapRoute {
            dex_type: DexType::UniswapV3,
            token_in,
            token_out,
            amount_in,
            min_amount_out: amount_in.mul(U256::from(97)).div(U256::from(100)),
            pool_address: dex_config.router_address,
            path: vec![format!("{:?}", token_in), format!("{:?}", token_out)],
            fee: Some(3000u32),
        })
    }

    async fn estimate_uniswap_v3_output(
        &self,
        _token_in: Address,
        _token_out: Address,
        _amount_in: U256,
        _fee: u32,
    ) -> Option<U256> {
        None
    }

    async fn find_curve_route(
        &self,
        _token_in: Address,
        _token_out: Address,
        _amount_in: U256,
    ) -> Option<SwapRoute> {
        None
    }

    async fn find_balancer_route(
        &self,
        _token_in: Address,
        _token_out: Address,
        _amount_in: U256,
    ) -> Option<SwapRoute> {
        None
    }

    fn get_flash_loan_fee_rate(&self, provider: &FlashLoanProvider) -> U256 {
        self.config.flash_loan_config.providers
            .get(provider)
            .map(|config| config.fee_rate)
            .unwrap_or(U256::from(9))
    }

    async fn get_user_health_factor(&self, user: Address) -> Option<U256> {
        match self.aave_pool
            .method::<_, (U256, U256, U256, U256, U256, U256)>("getUserAccountData", user)
            .unwrap()
            .call()
            .await
        {
            Ok((_, _, _, _, _, health_factor)) => Some(health_factor),
            Err(e) => {
                warn!("Failed to get health factor for user {}: {}", user, e);
                None
            }
        }
    }

    async fn is_user_liquidatable(&self, user: Address) -> bool {
        if let Some(health_factor) = self.get_user_health_factor(user).await {
            health_factor < U256::from(10).pow(18.into())
        } else {
            false
        }
    }

    async fn get_liquidation_bonus(&self, asset: Address) -> Option<U256> {
        match self.aave_pool
            .method::<_, U256>("getConfiguration", asset)
            .unwrap()
            .call()
            .await
        {
            Ok(config) => {
                let liquidation_bonus = (config >> 16) & U256::from(0xFFFF);
                Some(liquidation_bonus)
            }
            Err(e) => {
                warn!("Failed to get liquidation bonus for asset {}: {}", asset, e);
                None
            }
        }
    }

    async fn calculate_expected_profit(
        &self,
        collateral_asset: Address,
        debt_asset: Address,
        _user: Address,
        debt_to_cover: U256,
    ) -> Option<U256> {
        let liquidation_bonus = self.get_liquidation_bonus(collateral_asset).await?;
        let collateral_price = self.get_asset_price(collateral_asset).await?;
        let debt_price = self.get_asset_price(debt_asset).await?;
        
        let max_liquidation_amount = debt_to_cover.min(
            debt_to_cover.mul(U256::from(5000)).div(U256::from(10000))
        );
        
        let collateral_amount = max_liquidation_amount
            .mul(debt_price)
            .div(collateral_price)
            .mul(liquidation_bonus)
            .div(U256::from(10000));
        
        let profit_wei = collateral_amount
            .mul(collateral_price)
            .div(U256::from(10).pow(18.into()))
            .saturating_sub(max_liquidation_amount.mul(debt_price).div(U256::from(10).pow(18.into())));
        
        let gas_cost = U256::from((self.estimate_gas_cost().await * 1e18) as u64);
        
        if profit_wei > gas_cost {
            Some(profit_wei.saturating_sub(gas_cost))
        } else {
            None
        }
    }

    async fn get_asset_price(&self, asset: Address) -> Option<U256> {
        match self.aave_oracle
            .method::<_, U256>("getAssetPrice", asset)
            .unwrap()
            .call()
            .await
        {
            Ok(price) => Some(price),
            Err(e) => {
                warn!("Failed to get asset price for {}: {}", asset, e);
                None
            }
        }
    }

    async fn estimate_gas_cost(&self) -> f64 {
        let gas_price_gwei = format_units(self.state.gas_price, 9)
            .unwrap_or_else(|_| "20.0".to_string())
            .parse::<f64>()
            .unwrap_or(20.0);

        let estimated_gas_units = 500_000.0;
        let gas_cost_eth = (gas_price_gwei * estimated_gas_units) / 1e9;
        
        gas_cost_eth * self.config.gas_price_multiplier
    }
}

#[async_trait]
impl<M: Middleware + 'static, S: Signer + 'static> LiquidationStrategy<M, S> for AaveFlashLiquidationStrategy<M, S> {
    async fn process_event(&mut self, data: Vec<u8>) -> Vec<Action> {
        if data.is_empty() {
            return self.process_block_event().await;
        }

        match serde_json::from_slice::<serde_json::Value>(&data) {
            Ok(event) => {
                if let Some(event_type) = event.get("type").and_then(|v| v.as_str()) {
                    match event_type {
                        "block" => self.process_block_event().await,
                        "liquidation_opportunity" => {
                            if let Ok(target) = serde_json::from_value::<LiquidationTarget>(event) {
                                if let Some(profit) = self.calculate_profit(&target).await {
                                    if profit > self.config.min_profit_threshold {
                                        if let Some(path) = self.create_liquidation_path(&target, profit).await {
                                            return vec![Action::ExecuteLiquidation {
                                                path,
                                                expected_profit: profit,
                                            }];
                                        }
                                    }
                                }
                            }
                            vec![]
                        }
                        _ => vec![],
                    }
                } else {
                    vec![]
                }
            }
            Err(_) => self.process_block_event().await,
        }
    }

    async fn update_state(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let current_block = self.client.get_block_number().await?;
        self.state.last_update_block = current_block.as_u64();

        let gas_price = self.client.get_gas_price().await?;
        self.state.gas_price = gas_price;

        self.update_asset_prices().await?;

        Ok(())
    }

    async fn find_liquidation_opportunities(&self) -> Vec<LiquidationTarget> {
        let mut targets = Vec::new();

        for &asset in &self.config.monitored_assets {
            if let Some(users) = self.get_users_with_asset_debt(asset).await {
                for user in users {
                    if let Some(target) = self.create_liquidation_target(user, asset).await {
                        targets.push(target);
                    }
                }
            }
        }

        targets
    }

    async fn calculate_profit(&self, target: &LiquidationTarget) -> Option<f64> {
        let expected_profit_wei = self.calculate_expected_profit(
            target.collateral_asset,
            target.debt_asset,
            target.user,
            target.debt_to_cover,
        ).await?;

        let profit_eth = format_units(expected_profit_wei, 18)
            .unwrap_or_else(|_| "0.0".to_string())
            .parse::<f64>()
            .unwrap_or(0.0);

        let gas_cost = self.estimate_gas_cost().await;
        let flash_loan_fee = self.calculate_flash_loan_fee(target.debt_to_cover, target.debt_asset).await;

        let net_profit = profit_eth - gas_cost - flash_loan_fee;

        if net_profit > 0.0 {
            Some(net_profit)
        } else {
            None
        }
    }

    fn get_state(&self) -> &State {
        &self.state
    }

    fn get_config(&self) -> &Config {
        &self.config
    }
}

impl<M: Middleware + 'static, S: Signer + 'static> AaveFlashLiquidationStrategy<M, S> {
    async fn update_asset_prices(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let assets: Vec<Address> = self.config.monitored_assets.clone();
        
        match self.aave_oracle
            .method::<_, Vec<U256>>("getAssetsPrices", assets.clone())
            .unwrap()
            .call()
            .await
        {
            Ok(prices) => {
                for (asset, price) in assets.iter().zip(prices.iter()) {
                    self.state.asset_prices.insert(*asset, *price);
                }
                Ok(())
            }
            Err(e) => {
                error!("Failed to update asset prices: {}", e);
                Err(Box::new(e))
            }
        }
    }

    async fn get_users_with_asset_debt(&self, _asset: Address) -> Option<Vec<Address>> {
        None
    }

    async fn create_liquidation_target(&self, user: Address, debt_asset: Address) -> Option<LiquidationTarget> {
        if !self.is_user_liquidatable(user).await {
            return None;
        }

        let health_factor = self.get_user_health_factor(user).await?;
        
        if health_factor >= self.config.health_factor_threshold {
            return None;
        }

        let _user_data = self.get_user_account_data(user).await?;
        let collateral_asset = self.find_best_collateral_asset(user).await?;
        let liquidation_bonus = self.get_liquidation_bonus(collateral_asset).await?;
        
        let max_debt_to_cover = self.calculate_max_liquidation_amount(user, debt_asset).await?;
        let debt_to_cover = max_debt_to_cover.min(self.config.max_liquidation_amount);
        
        let gas_cost_estimate = self.estimate_gas_cost().await;
        let expected_profit = self.calculate_liquidation_profit(
            collateral_asset,
            debt_asset,
            debt_to_cover,
            liquidation_bonus
        ).await?;

        Some(LiquidationTarget {
            user,
            collateral_asset,
            debt_asset,
            debt_to_cover,
            health_factor,
            liquidation_bonus,
            expected_profit,
            gas_cost_estimate: U256::from((gas_cost_estimate * 1e18) as u64),
            receive_a_token: false,
        })
    }

    async fn get_user_account_data(&self, user: Address) -> Option<AaveUserData> {
        match self.aave_pool
            .method::<_, (U256, U256, U256, U256, U256, U256)>("getUserAccountData", user)
            .unwrap()
            .call()
            .await
        {
            Ok((total_collateral_eth, total_debt_eth, available_borrows_eth, 
                current_liquidation_threshold, ltv, health_factor)) => {
                Some(AaveUserData {
                    total_collateral_eth,
                    total_debt_eth,
                    available_borrows_eth,
                    current_liquidation_threshold,
                    ltv,
                    health_factor,
                })
            }
            Err(e) => {
                warn!("Failed to get user account data for {}: {}", user, e);
                None
            }
        }
    }

    async fn find_best_collateral_asset(&self, _user: Address) -> Option<Address> {
        self.config.monitored_assets.first().copied()
    }

    async fn calculate_max_liquidation_amount(&self, user: Address, _debt_asset: Address) -> Option<U256> {
        if let Some(user_data) = self.get_user_account_data(user).await {
            let max_liquidation = user_data.total_debt_eth
                .mul(U256::from(5000))
                .div(U256::from(10000));
            Some(max_liquidation.min(self.config.max_liquidation_amount))
        } else {
            None
        }
    }

    async fn calculate_liquidation_profit(
        &self,
        collateral_asset: Address,
        debt_asset: Address,
        debt_to_cover: U256,
        liquidation_bonus: U256,
    ) -> Option<f64> {
        let collateral_price = self.get_asset_price(collateral_asset).await?;
        let debt_price = self.get_asset_price(debt_asset).await?;
        
        let collateral_amount = debt_to_cover
            .mul(debt_price)
            .div(collateral_price)
            .mul(liquidation_bonus)
            .div(U256::from(10000));
        
        let profit_wei = collateral_amount
            .mul(collateral_price)
            .div(U256::from(10).pow(18.into()))
            .saturating_sub(debt_to_cover.mul(debt_price).div(U256::from(10).pow(18.into())));
        
        match format_units(profit_wei, 18) {
            Ok(profit_str) => profit_str.parse::<f64>().ok(),
            Err(_) => None,
        }
    }

    async fn calculate_flash_loan_fee(&self, amount: U256, _asset: Address) -> f64 {
        let fee_rate = self.get_flash_loan_fee_rate(&self.config.flash_loan_config.preferred_provider);
        let fee_wei = amount.mul(fee_rate).div(U256::from(10000));
        
        format_units(fee_wei, 18)
            .unwrap_or_else(|_| "0.0".to_string())
            .parse::<f64>()
            .unwrap_or(0.0)
    }
}
