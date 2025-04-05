pub mod strategy;
pub mod types;

pub use strategy::MultiStrategy;
pub use types::*;

use artemis_core::types::Strategy as ArtemisStrategy;
use async_trait::async_trait;
use ethers::prelude::{Middleware, Signer};

// Define the type alias for the Artemis strategy
pub type MultiStrategyArtemis<M, S> = dyn ArtemisStrategy<
    Event = Vec<u8>,
    Action = Vec<types::Action>,
> + Send
  + Sync;

#[async_trait]
impl<M: Middleware + 'static, S: Signer + 'static> ArtemisStrategy for MultiStrategy<M, S> {
    type Event = Vec<u8>;
    type Action = Vec<types::Action>;

    async fn process_event(&mut self, event: Self::Event) -> Vec<types::Action> {
        <strategy::MultiStrategy<M, S> as types::Strategy<M, S>>::process_event(self, event).await
    }

    async fn process_actions(&mut self, actions: &[types::Action]) {
        // This method is called after actions are processed by the engine
        // You can use it to update internal state or metrics
        
        // For example, track profits
        for action in actions {
            match action {
                types::Action::ExecuteArbitrage { expected_profit, .. } => {
                    let current_profit = self.state.historical_profits
                        .entry(types::StrategyType::Arbitrage)
                        .or_insert(0.0);
                    *current_profit += expected_profit;
                }
                types::Action::ExecuteJitLiquidity { expected_profit, .. } => {
                    let current_profit = self.state.historical_profits
                        .entry(types::StrategyType::JitLiquidity)
                        .or_insert(0.0);
                    *current_profit += expected_profit;
                }
                _ => {}
            }
        }
    }
}