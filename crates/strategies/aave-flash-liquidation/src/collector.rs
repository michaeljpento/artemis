use crate::types::{Config, LiquidationTarget};
use artemis_core::types::{Collector, CollectorStream};
use async_trait::async_trait;
use ethers::{
    prelude::{Address, Filter, Middleware, U256},
    types::{H256, TransactionRequest},
    types::transaction::eip2718::TypedTransaction,
};
use futures::stream::{self, StreamExt};
use serde_json::json;
use std::ops::{Mul, Div};
use std::sync::Arc;
use tokio_stream::wrappers::IntervalStream;
use tracing::{error, info, warn};

pub struct AaveFlashLiquidationCollector<M: Middleware + 'static> {
    client: Arc<M>,
    config: Config,
    aave_pool: Address,
    aave_oracle: Address,
    block_interval: u64,
}

impl<M: Middleware + 'static> AaveFlashLiquidationCollector<M> {
    pub fn new(
        client: Arc<M>,
        config: Config,
        aave_pool: Address,
        aave_oracle: Address,
        block_interval: u64,
    ) -> Self {
        Self {
            client,
            config,
            aave_pool,
            aave_oracle,
            block_interval,
        }
    }

    async fn collect_liquidation_events(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let current_block = self.client.get_block_number().await?;
        
        let liquidation_filter = Filter::new()
            .address(self.aave_pool)
            .topic0(H256::from_slice(&[
                0x56, 0x86, 0x4c, 0x5d, 0x0c, 0x47, 0x13, 0x7c,
                0x15, 0x2e, 0x7e, 0x17, 0x73, 0x0a, 0x50, 0x2e,
                0x8c, 0x8c, 0x80, 0x9a, 0x79, 0x70, 0x42, 0x6d,
                0xb1, 0x04, 0x83, 0x7c, 0x2d, 0x8d, 0x64, 0xa9,
            ]))
            .from_block(current_block - 10)
            .to_block(current_block);

        let logs = self.client.get_logs(&liquidation_filter).await?;
        
        if !logs.is_empty() {
            info!("Found {} liquidation events in recent blocks", logs.len());
            
            let event_data = json!({
                "type": "liquidation_events",
                "events": logs.len(),
                "block": current_block.as_u64()
            });
            
            Ok(serde_json::to_vec(&event_data)?)
        } else {
            let block_data = json!({
                "type": "block",
                "block_number": current_block.as_u64(),
                "timestamp": chrono::Utc::now().timestamp()
            });
            
            Ok(serde_json::to_vec(&block_data)?)
        }
    }

    async fn monitor_user_health_factors(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut liquidation_opportunities = Vec::new();
        
        for &asset in &self.config.monitored_assets {
            if let Some(users) = self.get_users_with_positions(asset).await {
                for user in users {
                    if let Some(target) = self.check_liquidation_opportunity(user, asset).await {
                        liquidation_opportunities.push(target);
                    }
                }
            }
        }

        if !liquidation_opportunities.is_empty() {
            info!("Found {} liquidation opportunities", liquidation_opportunities.len());
            
            let opportunity_data = json!({
                "type": "liquidation_opportunity",
                "opportunities": liquidation_opportunities,
                "count": liquidation_opportunities.len()
            });
            
            Ok(serde_json::to_vec(&opportunity_data)?)
        } else {
            let status_data = json!({
                "type": "health_check",
                "monitored_assets": self.config.monitored_assets.len(),
                "timestamp": chrono::Utc::now().timestamp()
            });
            
            Ok(serde_json::to_vec(&status_data)?)
        }
    }

    async fn get_users_with_positions(&self, _asset: Address) -> Option<Vec<Address>> {
        None
    }

    async fn check_liquidation_opportunity(&self, user: Address, debt_asset: Address) -> Option<LiquidationTarget> {
        let health_factor = self.get_user_health_factor(user).await?;
        
        if health_factor >= U256::from(10).pow(18.into()) {
            return None;
        }

        let user_data = self.get_user_account_data(user).await?;
        let collateral_asset = self.config.monitored_assets.first().copied()?;
        
        let liquidation_bonus = self.get_liquidation_bonus(collateral_asset).await?;
        let max_debt_to_cover = user_data.total_debt_eth
            .mul(U256::from(5000))
            .div(U256::from(10000))
            .min(self.config.max_liquidation_amount);

        let expected_profit = self.calculate_liquidation_profit(
            collateral_asset,
            debt_asset,
            max_debt_to_cover,
            liquidation_bonus,
        ).await?;

        Some(LiquidationTarget {
            user,
            collateral_asset,
            debt_asset,
            debt_to_cover: max_debt_to_cover,
            health_factor,
            liquidation_bonus,
            expected_profit,
            gas_cost_estimate: U256::from(500_000),
            receive_a_token: false,
        })
    }

    async fn get_user_health_factor(&self, user: Address) -> Option<U256> {
        let tx_req = TransactionRequest::new()
            .to(self.aave_pool)
            .data(hex::decode("bf92857c").unwrap())
            .data(ethers::abi::encode(&[ethers::abi::Token::Address(user)]));
        let tx = TypedTransaction::Legacy(tx_req);
            
        match self.client.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 192 {
                    Some(U256::from_big_endian(&result[160..192]))
                } else {
                    None
                }
            }
            Err(e) => {
                warn!("Failed to get health factor for user {}: {}", user, e);
                None
            }
        }
    }

    async fn get_user_account_data(&self, user: Address) -> Option<crate::types::AaveUserData> {
        let tx_req = TransactionRequest::new()
            .to(self.aave_pool)
            .data(hex::decode("bf92857c").unwrap())
            .data(ethers::abi::encode(&[ethers::abi::Token::Address(user)]));
        let tx = TypedTransaction::Legacy(tx_req);
            
        match self.client.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 192 {
                    Some(crate::types::AaveUserData {
                        total_collateral_eth: U256::from_big_endian(&result[0..32]),
                        total_debt_eth: U256::from_big_endian(&result[32..64]),
                        available_borrows_eth: U256::from_big_endian(&result[64..96]),
                        current_liquidation_threshold: U256::from_big_endian(&result[96..128]),
                        ltv: U256::from_big_endian(&result[128..160]),
                        health_factor: U256::from_big_endian(&result[160..192]),
                    })
                } else {
                    None
                }
            }
            Err(e) => {
                warn!("Failed to get user account data for {}: {}", user, e);
                None
            }
        }
    }

    async fn get_liquidation_bonus(&self, asset: Address) -> Option<U256> {
        let tx_req = TransactionRequest::new()
            .to(self.aave_pool)
            .data(hex::decode("c44b11f7").unwrap())
            .data(ethers::abi::encode(&[ethers::abi::Token::Address(asset)]));
        let tx = TypedTransaction::Legacy(tx_req);
            
        match self.client.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    let config = U256::from_big_endian(&result[0..32]);
                    let liquidation_bonus = (config >> 16) & U256::from(0xFFFF);
                    Some(liquidation_bonus)
                } else {
                    None
                }
            }
            Err(e) => {
                warn!("Failed to get liquidation bonus for asset {}: {}", asset, e);
                None
            }
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
        
        match ethers::utils::format_units(profit_wei, 18) {
            Ok(profit_str) => profit_str.parse::<f64>().ok(),
            Err(_) => None,
        }
    }

    async fn get_asset_price(&self, asset: Address) -> Option<U256> {
        let tx_req = TransactionRequest::new()
            .to(self.aave_oracle)
            .data(hex::decode("b3596f07").unwrap())
            .data(ethers::abi::encode(&[ethers::abi::Token::Address(asset)]));
        let tx = TypedTransaction::Legacy(tx_req);
            
        match self.client.call(&tx, None).await {
            Ok(result) => {
                if result.len() >= 32 {
                    Some(U256::from_big_endian(&result[0..32]))
                } else {
                    None
                }
            }
            Err(e) => {
                warn!("Failed to get asset price for {}: {}", asset, e);
                None
            }
        }
    }
}

#[async_trait]
impl<M: Middleware + 'static> Collector<Vec<u8>> for AaveFlashLiquidationCollector<M> {
    async fn get_event_stream(&self) -> Result<CollectorStream<'_, Vec<u8>>, anyhow::Error> {
        let interval = tokio::time::interval(tokio::time::Duration::from_secs(self.block_interval));
        let interval_stream = IntervalStream::new(interval);
        
        let event_stream = interval_stream.then(move |_| async move {
            match self.collect_liquidation_events().await {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to collect liquidation events: {}", e);
                    serde_json::to_vec(&json!({
                        "type": "error",
                        "message": e.to_string()
                    })).unwrap_or_default()
                }
            }
        });

        let health_interval = tokio::time::interval(tokio::time::Duration::from_secs(self.block_interval * 2));
        let health_stream = IntervalStream::new(health_interval).then(move |_| async move {
            match self.monitor_user_health_factors().await {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to monitor health factors: {}", e);
                    serde_json::to_vec(&json!({
                        "type": "error",
                        "message": e.to_string()
                    })).unwrap_or_default()
                }
            }
        });

        let combined_stream = stream::select(event_stream, health_stream);
        
        Ok(Box::pin(combined_stream))
    }
}
