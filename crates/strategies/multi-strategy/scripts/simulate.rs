use anyhow::Result;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::*;
use ethers::providers::{Provider, Ws};
use ethers::signers::{LocalWallet, Signer};
use multi_strategy::{Config, Event, MultiStrategy};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::time::sleep;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Load environment variables
    dotenv::dotenv().ok();
    
    // Get RPC URL and private key from environment variables
    let rpc_url = std::env::var("WS_RPC_URL").expect("WS_RPC_URL must be set");
    let private_key = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");
    
    // Connect to Ethereum node
    info!("Connecting to Ethereum node at {}", rpc_url);
    let provider = Provider::<Ws>::connect(&rpc_url).await?;
    let chain_id = provider.get_chainid().await?.as_u64();
    let wallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
    let provider = Arc::new(SignerMiddleware::new(provider, wallet));
    
    // Load configuration
    let config_str = std::fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&config_str)?;
    
    // Create the strategy
    info!("Initializing multi-strategy with {} monitored pools", config.monitored_pools.len());
    let mut strategy = MultiStrategy::new(config, provider.clone());
    
    // Initialize the strategy state
    info!("Synchronizing strategy state");
    strategy.sync_state().await?;
    
    // Fetch the latest block
    info!("Fetching latest block");
    let latest_block = provider.get_block(BlockNumber::Latest).await?
        .expect("Failed to get latest block");
    
    // Process the block to find arbitrage opportunities
    info!("Processing block {} for arbitrage opportunities", latest_block.number.unwrap_or_default());
    let block_event = Event::NewBlock(latest_block);
    let actions = strategy.process_event(block_event).await;
    
    // Display results
    if actions.is_empty() {
        info!("No profitable opportunities found in the latest block");
    } else {
        info!("Found {} profitable opportunities:", actions.len());
        
        for (i, action) in actions.iter().enumerate() {
            match action {
                multi_strategy::Action::ExecuteArbitrage { path, expected_profit } => {
                    info!("Arbitrage #{}: Path with {} swaps, expected profit: {} ETH", 
                          i+1, path.swaps.len(), expected_profit);
                    
                    // Print detailed path information
                    info!("  Start token: {:?}", path.start_token);
                    info!("  Borrow amount: {}", path.borrow_amount);
                    
                    for (j, swap) in path.swaps.iter().enumerate() {
                        info!("  Swap {}: Pool {:?}, Direction: {}", 
                              j+1, swap.pool, if swap.zero_for_one { "0->1" } else { "1->0" });
                    }
                },
                multi_strategy::Action::ExecuteJitLiquidity { pool, amounts, expected_profit } => {
                    info!("JIT Liquidity: Pool {:?}, Expected profit: {} ETH", pool, expected_profit);
                },
                multi_strategy::Action::ExecuteBackrun { target_tx, expected_profit, .. } => {
                    info!("Backrun: Target tx {:?}, Expected profit: {} ETH", target_tx, expected_profit);
                }
            }
        }
    }
    
    // Print strategy metrics
    info!("Strategy metrics:");
    info!("  Total profit: {} ETH", strategy.metrics.total_profit);
    info!("  Arbitrage opportunities found: {}", strategy.metrics.arbitrage_opportunities);
    info!("  JIT opportunities found: {}", strategy.metrics.jit_opportunities);
    info!("  Backrunning opportunities found: {}", strategy.metrics.backrunning_opportunities);
    
    Ok(())
}