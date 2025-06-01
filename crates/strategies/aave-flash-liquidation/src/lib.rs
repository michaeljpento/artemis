pub mod strategy;
pub mod types;
pub mod bindings;
pub mod executor;
pub mod collector;

pub use strategy::AaveFlashLiquidationStrategy;
pub use executor::AaveFlashLiquidationExecutor;
pub use collector::AaveFlashLiquidationCollector;
pub use types::*;

use tracing;

use artemis_core::types::Strategy as ArtemisStrategy;
use async_trait::async_trait;
use ethers::prelude::{Middleware, Signer};

#[async_trait]
impl<M: Middleware + 'static, S: Signer + 'static> ArtemisStrategy<Vec<u8>, types::Action> for AaveFlashLiquidationStrategy<M, S> {
    async fn process_event(&mut self, event: Vec<u8>) -> Vec<types::Action> {
        <strategy::AaveFlashLiquidationStrategy<M, S> as types::LiquidationStrategy<M, S>>::process_event(self, event).await
    }

    async fn sync_state(&mut self) -> Result<(), anyhow::Error> {
        self.update_state().await.map_err(|e| anyhow::anyhow!("Failed to sync state: {}", e))
    }
}
