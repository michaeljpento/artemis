use anyhow::Result;
use clap::Parser;
use ethers::{
    prelude::*,
    providers::{Middleware, Provider, Ws},
    signers::{LocalWallet, Signer},
    types::{Address, Filter, U256, BlockNumber, H256},
};
use std::{collections::HashSet, str::FromStr, sync::Arc, time::Duration};
use tokio::{sync::mpsc, time};
use tracing::{debug, error, info};

// Contract ABIs - we'll define these in a separate file
mod abis;

// DEX-related constants for Polygon
mod constants;
use constants::*;

// Monitor module for metrics collection
mod monitor;

// Strategy parameters and opportunity detection logic
mod strategy;
use strategy::{detect_opportunity, JitOpportunity, OpportunityType};

// Command line arguments for the JIT bot
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Enable aggressive mode for maximum profits
    #[clap(long, default_value = "true")]
    aggressive: bool,

    /// Enable JIT liquidity strategy
    #[clap(long, default_value = "true")]
    enable_jit: bool,

    /// Enable flash arbitrage strategy
    #[clap(long, default_value = "true")]
    enable_arb: bool,

    /// Minimum profit threshold in USD
    #[clap(long, default_value = "1.0")]
    min_profit_usd: f64,

    /// Maximum gas price in gwei
    #[clap(long, default_value = "100")]
    max_gas_price_gwei: f64,

    /// Run in simulation mode (no real transactions)
    #[clap(long)]
    simulation: bool,
    
    /// Enable metrics server and specify port
    #[clap(long)]
    metrics_port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Parse command line arguments
    let args = Args::parse();
    
    // Load WebSocket URL from environment
    let ws_url = std::env::var("POLYGON_WS_URL")
        .expect("POLYGON_WS_URL must be set in .env file");
    
    // Connect to Polygon via WebSocket
    info!("Connecting to Polygon via WebSocket: {}", ws_url);
    let ws = Ws::connect(ws_url).await?;
    let provider = Provider::new(ws);
    
    // Set up wallet with private key
    let private_key = std::env::var("PRIVATE_KEY")
        .expect("PRIVATE_KEY must be set in .env file");
    let wallet = private_key.parse::<LocalWallet>()?;
    let wallet_address = wallet.address();
    info!("Using wallet address: {}", wallet_address);
    
    // Create client with the wallet
    let client = SignerMiddleware::new(provider, wallet.clone());
    let client = Arc::new(client);
    
    // Contract addresses
    let jit_provider_address = std::env::var("JIT_LIQUIDITY_PROVIDER")
        .expect("JIT_LIQUIDITY_PROVIDER must be set in .env file");
    let jit_provider_address = Address::from_str(&jit_provider_address)?;
    
    let flash_arb_address = std::env::var("FLASH_ARB_EXECUTOR")
        .expect("FLASH_ARB_EXECUTOR must be set in .env file");
    let flash_arb_address = Address::from_str(&flash_arb_address)?;
    
    // Create contract instances
    let jit_contract = abis::JitLiquidityProvider::new(jit_provider_address, client.clone());
    let arb_contract = abis::FlashArbExecutor::new(flash_arb_address, client.clone());
    
    // Store our strategy configuration
    let _config = strategy::StrategyConfig {
        min_profit_threshold_usd: args.min_profit_usd,
        max_gas_price_gwei: args.max_gas_price_gwei,
        aggressive_mode: args.aggressive,
        simulation_mode: args.simulation,
    };
    
    // Log the current mode
    if args.simulation {
        info!("Running in SIMULATION mode - no real transactions will be executed");
    } else {
        info!("Running in PRODUCTION mode - real transactions will be executed");
    }
    
    // Create a channel for sending opportunities
    let (tx, mut rx) = mpsc::channel(100);
    
    // Start the transaction monitoring task
    let monitor_client = client.clone();
    let simulation_mode = args.simulation;
    tokio::spawn(async move {
        if let Err(e) = monitor_mempool(monitor_client, tx, simulation_mode).await {
            error!("Mempool monitoring error: {}", e);
        }
    });
    
    // Start metrics server if enabled
    if let Some(port) = args.metrics_port {
        info!("Starting metrics server on port {}", port);
        
        // Create metrics instance
        let (metrics, registry) = monitor::Metrics::new()?;
        let metrics: Arc<monitor::Metrics> = Arc::new(metrics);
        
        // Start monitoring wallet balance
        let balance_metrics = metrics.clone();
        let balance_client = client.clone();
        tokio::spawn(async move {
            monitor::monitor_wallet_balance(balance_client, wallet_address, balance_metrics).await;
        });
        
        // Start metrics server
        let metrics_server = metrics.clone();
        tokio::spawn(async move {
            monitor::start_metrics_server(metrics_server, registry, port).await;
        });
        
        info!("Metrics dashboard available at http://localhost:{}/dashboard", port);
    }
    
    // Main loop for processing detected opportunities
    info!("Starting main opportunity processing loop");
    while let Some(opportunity) = rx.recv().await {
        info!("Processing opportunity: {:?}", opportunity);
        
        // If running in simulation mode, just log and skip execution
        if args.simulation {
            info!("Simulation mode: Would execute opportunity with estimated profit ${:.2}", 
                 opportunity.estimated_profit_usd);
            continue;
        }
        
        // Check if the wallet has sufficient MATIC balance for gas
        let wallet_balance = match client.get_balance(wallet_address, None).await {
            Ok(balance) => balance,
            Err(e) => {
                error!("Failed to get wallet balance: {}", e);
                // Fall back to simulation mode
                info!("Simulation mode: Would execute opportunity with estimated profit ${:.2}", 
                     opportunity.estimated_profit_usd);
                continue;
            }
        };
        
        // Require at least 0.1 MATIC for gas
        if wallet_balance < U256::from(100000000000000000u64) {
            info!("Insufficient MATIC balance for gas. Running in simulation mode instead.");
            info!("Simulation mode: Would execute opportunity with estimated profit ${:.2}", 
                 opportunity.estimated_profit_usd);
            continue;
        }
        
        // Execute the appropriate strategy based on opportunity type
        match opportunity.opportunity_type {
            OpportunityType::JitLiquidity => {
                if args.enable_jit {
                    if args.aggressive {
                        // Use ultra-aggressive mode for maximum profit
                        if let Err(e) = execute_ultra_aggressive_jit(&jit_contract, &opportunity).await {
                            error!("Error executing ultra-aggressive JIT: {}", e);
                        }
                    } else {
                        // Use standard JIT with Balancer for zero-fee flash loans
                        if let Err(e) = execute_balancer_jit(&jit_contract, &opportunity).await {
                            error!("Error executing Balancer JIT: {}", e);
                        }
                    }
                }
            },
            OpportunityType::FlashArbitrage => {
                if args.enable_arb {
                    if let Err(e) = execute_flash_arbitrage(&arb_contract, &opportunity).await {
                        error!("Error executing flash arbitrage: {}", e);
                    }
                }
            },
            OpportunityType::BatchMicroJit => {
                // Batch micro opportunities are always simulated for now
                info!("Simulation mode: Would execute Batch Micro-JIT for {} opportunities with total profit ${:.2}", 
                     opportunity.batch_opportunities.len(),
                     opportunity.estimated_profit_usd);
            },
        }
    }
    
    Ok(())
}

// Monitor the mempool for potential opportunities
async fn monitor_mempool<M: Middleware + 'static>(
    client: Arc<M>,
    sender: mpsc::Sender<JitOpportunity>,
    simulation_mode: bool,
) -> Result<()> {
    // Initialize metrics if available
    let metrics = if let Ok((metrics, _)) = monitor::Metrics::new() {
        Some(Arc::<monitor::Metrics>::new(metrics))
    } else {
        None
    };
    info!("Starting mempool monitoring");
    
    // Create a filter for pending transactions
    let filter = Filter::new().from_block(BlockNumber::Pending);
    
    // Key DEXes to monitor
    let dex_addresses = vec![
        // Add QuickSwap router
        Address::from_str("0xa5E0829CaCEd8fFDD4De3c43696c57F7D7A678ff")?,
        // SushiSwap router
        Address::from_str("0x1b02dA8Cb0d097eB8D57A175b88c7D8b47997506")?,
        // Uniswap V3 router
        Address::from_str("0xE592427A0AEce92De3Edee1F18E0157C05861564")?,
        // Curve router
        Address::from_str("0x8474DdbE98F5aA3179B3B3F5942D724aFcdec9f6")?,
    ];
    
    // Track seen transactions to avoid duplicates
    let mut seen_txs = HashSet::new();
    
    // Also create a heartbeat to periodically check for opportunities
    let mut interval = time::interval(Duration::from_secs(5));
    
    info!("Starting manual block monitoring for opportunities...");
    
    // Since we can't use pubsub directly, we'll poll for new blocks and transactions
    loop {
        // Check for new blocks
        if let Ok(block_number) = client.get_block_number().await {
            // Get latest block
            if let Ok(Some(block)) = client.get_block_with_txs(block_number).await {
                // Process transactions in the block
                for transaction in block.transactions {
                    let tx_hash = transaction.hash;
                    
                    // Skip if we've seen this transaction before
                    if seen_txs.contains(&tx_hash) {
                        continue;
                    }
                    
                    // Add to seen transactions
                    seen_txs.insert(tx_hash);
                    
                    // Keep the set from growing too large
                    if seen_txs.len() > 10000 {
                        seen_txs.clear();
                    }
                    
                    // Check if this transaction involves our target DEXes
                    if let Some(to) = transaction.to {
                        if dex_addresses.contains(&to) {
                            // Analyze transaction for opportunities
                            if let Some(opportunity) = detect_opportunity(&transaction).await {
                                debug!("Detected opportunity: {:?}", opportunity);
                                
                                // Record metrics if available
                                if let Some(ref metrics) = metrics {
                                    metrics.record_opportunity(&opportunity);
                                }
                                
                                // Send opportunity to main thread
                                if let Err(e) = sender.send(opportunity).await {
                                    error!("Failed to send opportunity: {}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Check for batch opportunities periodically
        tokio::select! {
            _ = interval.tick() => {
                // This is where you would scan for micro-opportunities to batch
                debug!("Heartbeat: checking for batch opportunities");
                
                // For simulation mode only - create sample opportunities
                if simulation_mode {
                    if let Ok(block_number) = client.get_block_number().await {
                        if block_number.as_u64() % 20 == 0 {  // Every ~20 blocks
                            info!("Simulating a batch opportunity");
                            
                            // Create a sample batch opportunity
                            let opportunity = JitOpportunity {
                                opportunity_type: OpportunityType::BatchMicroJit,
                                token_pair: (
                                    Address::from_str(WMATIC_ADDRESS).unwrap(),
                                    Address::from_str(USDC_ADDRESS).unwrap()
                                ),
                                pool_address: Address::from_str("0x6e7a5FAFcec6BB1e78bAE2A1F0B612012BF14827").unwrap(),
                                pool_type: 0, // QuickSwap
                                amounts: (U256::from(1000000000000000000u64), U256::from(1000000000u64)),
                                estimated_profit_usd: 3.75,
                                gas_price: U256::from(50000000000u64),
                                competitor_tx: None,
                                v3_params: None,
                                batch_opportunities: vec![
                                    // Add some micro opportunities to the batch
                                    JitOpportunity {
                                        opportunity_type: OpportunityType::JitLiquidity,
                                        token_pair: (
                                            Address::from_str(WMATIC_ADDRESS).unwrap(),
                                            Address::from_str(USDC_ADDRESS).unwrap()
                                        ),
                                        pool_address: Address::from_str("0x6e7a5FAFcec6BB1e78bAE2A1F0B612012BF14827").unwrap(),
                                        pool_type: 0,
                                        amounts: (U256::from(500000000000000000u64), U256::from(500000000u64)),
                                        estimated_profit_usd: 1.25,
                                        gas_price: U256::from(50000000000u64),
                                        competitor_tx: None,
                                        v3_params: None,
                                        batch_opportunities: vec![],
                                    },
                                    JitOpportunity {
                                        opportunity_type: OpportunityType::JitLiquidity,
                                        token_pair: (
                                            Address::from_str(WMATIC_ADDRESS).unwrap(),
                                            Address::from_str(USDC_ADDRESS).unwrap()
                                        ),
                                        pool_address: Address::from_str("0x6e7a5FAFcec6BB1e78bAE2A1F0B612012BF14827").unwrap(),
                                        pool_type: 0,
                                        amounts: (U256::from(500000000000000000u64), U256::from(500000000u64)),
                                        estimated_profit_usd: 2.50,
                                        gas_price: U256::from(50000000000u64),
                                        competitor_tx: None,
                                        v3_params: None,
                                        batch_opportunities: vec![],
                                    },
                                ],
                            };
                            
                            // Record metrics if available
                            if let Some(ref metrics) = metrics {
                                metrics.record_opportunity(&opportunity);
                            }
                            
                            if let Err(e) = sender.send(opportunity).await {
                                error!("Failed to send batch opportunity: {}", e);
                            }
                        }
                    }
                }
            },
            _ = tokio::time::sleep(Duration::from_secs(1)) => {
                // Wait a bit to avoid hitting rate limits
            }
        }
    }
}

// Execute JIT liquidity provision using Balancer's zero-fee flash loans
async fn execute_balancer_jit<M: Middleware + 'static>(
    contract: &abis::JitLiquidityProvider<M>,
    opportunity: &JitOpportunity,
) -> Result<()> {
    info!("Executing Balancer JIT for opportunity with estimated profit ${:.2}", 
         opportunity.estimated_profit_usd);
    
    // Prepare JIT parameters from the opportunity
    let jit_params = strategy::prepare_jit_params(opportunity)?;
    let v3_params = strategy::prepare_v3_params(opportunity)?;
    
    // Get the Args to check if we're in simulation mode
    let args = Args::parse();
    if args.simulation {
        info!("Simulation mode: Would execute Balancer JIT opportunity with estimated profit ${:.2}", 
             opportunity.estimated_profit_usd);
        return Ok(());
    }
    
    // Execute the transaction with appropriate gas settings
    let call = contract.execute_balancer_jit_liquidity(jit_params, v3_params)
        .gas_price(opportunity.gas_price);
    
    // Try to send the transaction, handling errors gracefully
    let pending_tx = match call.send().await {
        Ok(tx) => tx,
        Err(e) => {
            // Check if error is due to insufficient token balance
            if e.to_string().contains("transfer amount exceeds balance") {
                info!("Insufficient token balance for Balancer JIT operation. Would have made ${:.2} profit.", 
                     opportunity.estimated_profit_usd);
                return Ok(());
            } else {
                return Err(e.into());
            }
        }
    };
    
    info!("Balancer JIT transaction sent: {:?}", pending_tx.tx_hash());
    
    // Wait for transaction to be mined
    let receipt = pending_tx.await?;
    info!("Balancer JIT transaction mined: {:?}", receipt);
    
    Ok(())
}

// Execute ultra-aggressive JIT for maximum profits
async fn execute_ultra_aggressive_jit<M: Middleware + 'static>(
    contract: &abis::JitLiquidityProvider<M>,
    opportunity: &JitOpportunity,
) -> Result<()> {
    info!("Executing Ultra-Aggressive JIT with estimated profit ${:.2}", 
         opportunity.estimated_profit_usd);
    
    // Get the Args to check if we're in simulation mode
    let args = Args::parse();
    if args.simulation {
        info!("Simulation mode: Would execute Ultra-Aggressive JIT opportunity with estimated profit ${:.2}", 
             opportunity.estimated_profit_usd);
        return Ok(());
    }
    
    // Prepare JIT parameters from the opportunity
    let jit_params = strategy::prepare_jit_params(opportunity)?;
    let v3_params = strategy::prepare_v3_params(opportunity)?;
    
    // Competitor transaction to frontrun (if any)
    let competitor_tx = opportunity.competitor_tx.unwrap_or(H256::zero()).into();
    
    // Use a high priority fee multiplier for aggressive execution
    let priority_fee_multiplier = 300; // 3x base priority fee
    
    // Execute the transaction with appropriate gas settings
    let call = contract
        .execute_ultra_aggressive_jit(
            jit_params,
            v3_params,
            competitor_tx,
            priority_fee_multiplier.into()
        )
        .gas_price(opportunity.gas_price);
        
    // Try to send the transaction, handling errors gracefully
    let pending_tx = match call.send().await {
        Ok(tx) => tx,
        Err(e) => {
            // Check if error is due to insufficient token balance
            if e.to_string().contains("transfer amount exceeds balance") {
                info!("Insufficient token balance for Ultra Aggressive JIT operation. Would have made ${:.2} profit.", 
                     opportunity.estimated_profit_usd);
                return Ok(());
            } else {
                return Err(e.into());
            }
        }
    };
    
    info!("Ultra-Aggressive JIT transaction sent: {:?}", pending_tx.tx_hash());
    
    // Wait for transaction to be mined
    let receipt = pending_tx.await?;
    info!("Ultra-Aggressive JIT transaction mined: {:?}", receipt);
    
    Ok(())
}

// Execute a batch of micro-profitable JIT opportunities
async fn execute_batch_micro_jit<M: Middleware + 'static>(
    _contract: &abis::JitLiquidityProvider<M>,
    opportunity: &JitOpportunity,
) -> Result<()> {
    info!("Executing Batch Micro-JIT for {} opportunities with total profit ${:.2}", 
         opportunity.batch_opportunities.len(),
         opportunity.estimated_profit_usd);
    
    // Always simulate batch opportunities for now until we fix the contract issues
    info!("Simulation mode: Would execute Batch Micro-JIT opportunities with total profit ${:.2}", 
         opportunity.estimated_profit_usd);
    
    // In a future version, we'll implement real batch execution
    Ok(())
}

// Execute flash loan arbitrage
async fn execute_flash_arbitrage<M: Middleware + 'static>(
    contract: &abis::FlashArbExecutor<M>,
    opportunity: &JitOpportunity,
) -> Result<()> {
    info!("Executing Flash Arbitrage with estimated profit ${:.2}", 
         opportunity.estimated_profit_usd);
    
    // Get the Args to check if we're in simulation mode
    let args = Args::parse();
    if args.simulation {
        info!("Simulation mode: Would execute Flash Arbitrage with estimated profit ${:.2}", 
             opportunity.estimated_profit_usd);
        return Ok(());
    }
    
    // Prepare arbitrage parameters
    let arb_params = strategy::prepare_arb_params(opportunity)?;
    
    // Choose the flash loan provider - Balancer for 0% fee
    let provider = 1; // 0 = Aave, 1 = Balancer, 2 = Uniswap V3
    
    // Execute the flash arbitrage
    let call = contract
        .execute_arbitrage(arb_params, provider.into())
        .gas_price(opportunity.gas_price);
        
    // Try to send the transaction, handling errors gracefully
    let pending_tx = match call.send().await {
        Ok(tx) => tx,
        Err(e) => {
            // Check if error is due to insufficient token balance
            if e.to_string().contains("transfer amount exceeds balance") {
                info!("Insufficient token balance for Flash Arbitrage operation. Would have made ${:.2} profit.", 
                     opportunity.estimated_profit_usd);
                return Ok(());
            } else {
                return Err(e.into());
            }
        }
    };
    
    info!("Flash Arbitrage transaction sent: {:?}", pending_tx.tx_hash());
    
    // Wait for transaction to be mined
    let receipt = pending_tx.await?;
    info!("Flash Arbitrage transaction mined: {:?}", receipt);
    
    Ok(())
}
