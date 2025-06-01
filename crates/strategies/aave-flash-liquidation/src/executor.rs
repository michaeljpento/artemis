use crate::types::{Action, LiquidationPath};
use artemis_core::types::Executor;
use async_trait::async_trait;
use ethers::{
    prelude::{Address, Middleware, SignerMiddleware, U256},
    signers::Signer,
    types::TransactionRequest,
};
use alloy_sol_types::SolCall;
use std::sync::Arc;
use tracing::{error, info, warn};

pub struct AaveFlashLiquidationExecutor<M: Middleware + 'static, S: Signer + 'static> {
    client: Arc<SignerMiddleware<Arc<M>, S>>,
    liquidator_contract: Address,
}

impl<M: Middleware + 'static, S: Signer + 'static> AaveFlashLiquidationExecutor<M, S> {
    pub fn new(client: Arc<SignerMiddleware<Arc<M>, S>>, liquidator_contract: Address) -> Self {
        Self {
            client,
            liquidator_contract,
        }
    }

    async fn execute_flash_liquidation(&self, path: &LiquidationPath) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Executing flash liquidation for user {} with expected profit: {} ETH",
            path.target.user, path.expected_profit_eth
        );

        let gas_price = self.client.get_gas_price().await?;
        
        if gas_price > path.max_gas_price {
            warn!("Gas price {} exceeds maximum {}, skipping liquidation", gas_price, path.max_gas_price);
            return Err("Gas price too high".into());
        }

        let tx_request = if path.use_flashbots {
            self.build_flashbots_transaction(path).await?
        } else {
            self.build_standard_transaction(path).await?
        };

        match self.client.send_transaction(tx_request, None).await {
            Ok(pending_tx) => {
                info!("Flash liquidation transaction submitted: {:?}", pending_tx.tx_hash());
                
                match pending_tx.await {
                    Ok(Some(receipt)) => {
                        info!("Flash liquidation confirmed in block {}", receipt.block_number.unwrap_or_default());
                        Ok(())
                    }
                    Ok(None) => {
                        error!("Flash liquidation transaction failed - no receipt");
                        Err("Transaction failed".into())
                    }
                    Err(e) => {
                        error!("Flash liquidation transaction error: {}", e);
                        Err(Box::new(e))
                    }
                }
            }
            Err(e) => {
                error!("Failed to submit flash liquidation transaction: {}", e);
                Err(Box::new(e))
            }
        }
    }

    async fn build_standard_transaction(&self, path: &LiquidationPath) -> Result<TransactionRequest, Box<dyn std::error::Error + Send + Sync>> {
        let function_data = self.encode_flash_liquidation_call(path)?;
        
        Ok(TransactionRequest::new()
            .to(self.liquidator_contract)
            .data(function_data)
            .gas_price(path.max_gas_price)
            .gas(U256::from(500_000)))
    }

    async fn build_flashbots_transaction(&self, path: &LiquidationPath) -> Result<TransactionRequest, Box<dyn std::error::Error + Send + Sync>> {
        let function_data = self.encode_protected_liquidation_call(path)?;
        
        Ok(TransactionRequest::new()
            .to(self.liquidator_contract)
            .data(function_data)
            .gas_price(path.max_gas_price)
            .gas(U256::from(600_000)))
    }

    fn encode_flash_liquidation_call(&self, path: &LiquidationPath) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::bindings::AaveV3FlashLiquidator::flashLiquidateCall;
        use alloy_primitives::{Address as AlloyAddress, U256 as AlloyU256};
        
        let call = flashLiquidateCall {
            collateralAsset: AlloyAddress::from_slice(&path.target.collateral_asset.0),
            debtAsset: AlloyAddress::from_slice(&path.target.debt_asset.0),
            user: AlloyAddress::from_slice(&path.target.user.0),
            debtToCover: AlloyU256::from_limbs(path.target.debt_to_cover.0),
            receiveAToken: path.target.receive_a_token,
        };
        
        Ok(call.abi_encode())
    }

    fn encode_protected_liquidation_call(&self, path: &LiquidationPath) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        use crate::bindings::AaveV3FlashLiquidator::submitProtectedLiquidationCall;
        use alloy_primitives::{Address as AlloyAddress, U256 as AlloyU256, Bytes};
        
        let call = submitProtectedLiquidationCall {
            collateralAsset: AlloyAddress::from_slice(&path.target.collateral_asset.0),
            debtAsset: AlloyAddress::from_slice(&path.target.debt_asset.0),
            user: AlloyAddress::from_slice(&path.target.user.0),
            debtToCover: AlloyU256::from_limbs(path.target.debt_to_cover.0),
            receiveAToken: path.target.receive_a_token,
            flashbotsData: Bytes::new(),
        };
        
        Ok(call.abi_encode())
    }
}

#[async_trait]
impl<M: Middleware + 'static, S: Signer + 'static> Executor<Action> for AaveFlashLiquidationExecutor<M, S> {
    async fn execute(&self, action: Action) -> Result<(), anyhow::Error> {
        match action {
            Action::ExecuteLiquidation { path, expected_profit } => {
                info!("Executing liquidation with expected profit: {} ETH", expected_profit);
                self.execute_flash_liquidation(&path).await.map_err(|e| anyhow::anyhow!("Flash liquidation failed: {}", e))
            }
            Action::UpdatePrices { assets } => {
                info!("Price update requested for {} assets", assets.len());
                Ok(())
            }
            Action::TriggerCircuitBreaker { reason } => {
                warn!("Circuit breaker triggered: {}", reason);
                Ok(())
            }
            Action::None => {
                Ok(())
            }
        }
    }
}
