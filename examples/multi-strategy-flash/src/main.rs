use artemis_core::engine::Engine;
use artemis_core::executors::{FlashbotsExecutor, MemPoolExecutor, MevShareExecutor};
use artemis_core::types::{Collector, CollectorStream, ExecutionSummary, Executor, Strategy};
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use ethers::middleware::{Middleware, SignerMiddleware};
use ethers::providers::{Http, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, H256, U256};
use futures::stream::StreamExt;
use multi_strategy_flash::{Action, Config, MultiStrategy};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, Level};
use tracing_subscriber;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to the configuration file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the multi-strategy bot
    Run {
        /// The execution mode (mempool, flashbots, mev-share)
        #[arg(short, long, default_value = "mempool")]
        mode: String,
    },

    /// Simulate a strategy execution
    Simulate {
        /// Path to a transaction to simulate
        #[arg(short, long, value_name = "FILE")]
        tx_path: PathBuf,
    },
}

#[derive(Clone)]
struct BlockCollector {
    provider: Arc<Provider<Http>>,
}

impl BlockCollector {
    fn new(provider: Arc<Provider<Http>>) -> Self {
        Self { provider }
    }
}

#[async_trait::async_trait]
impl Collector for BlockCollector {
    type Event = Vec<u8>;

    async fn get_event_stream(&self) -> CollectorStream<Self::Event> {
        let provider = self.provider.clone();
        let (tx, rx) = mpsc::channel(100);

        tokio::spawn(async move {
            let mut block_stream = provider.watch_blocks().await.unwrap();

            while let Some(block) = block_stream.next().await {
                let block_data = serde_json::to_vec(&block).unwrap_or_default();
                if let Err(e) = tx.send(block_data).await {
                    error!("Error sending block to channel: {}", e);
                    break;
                }
            }
        });

        Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx))
    }
}

// Map our strategy's Action to the executor's Action
fn map_action_to_executor_action(actions: Vec<Action>) -> Vec<H256> {
    let mut tx_hashes = Vec::new();

    for action in actions {
        match action {
            Action::ExecuteArbitrage { path, expected_profit } => {
                // Create transaction to FlashArbExecutor
                let tx_data = create_arb_transaction(path);
                
                // In a real implementation, you would submit this transaction
                // For now, we'll just log the details
                info!("Creating arbitrage transaction with expected profit: {} ETH", expected_profit);
                
                // For testing purposes, we're returning a random hash
                // In production, this would be the actual transaction hash
                tx_hashes.push(H256::random());
            }
            Action::ExecuteJitLiquidity { params, expected_profit } => {
                // Create transaction to JITLiquidityProvider
                let tx_data = create_jit_transaction(params);
                
                // Log details
                info!("Creating JIT liquidity transaction with expected profit: {} ETH", expected_profit);
                
                // Return transaction hash
                tx_hashes.push(H256::random());
            }
            Action::ExecuteBackrun { params } => {
                // For MEV-Share backruns
                info!("Creating backrun for tx {} with expected profit: {} ETH", 
                    params.target_tx, params.expected_profit);
                
                // Return transaction hash
                tx_hashes.push(H256::random());
            }
            Action::None => {}
        }
    }

    tx_hashes
}

// Create transaction data for arbitrage
fn create_arb_transaction(path: multi_strategy_flash::ArbitragePath) -> Vec<u8> {
    // Convert the arbitrage path to calldata for FlashArbExecutor
    
    // This would be the actual ABI encoding in production
    // For now, we'll create a simplified version
    
    // Function selector for executeArbitrage 
    // In production, this would be the keccak256 hash of the function signature
    let function_selector = [0x12, 0x34, 0x56, 0x78]; 
    
    // Create simplified calldata
    let mut calldata = Vec::new();
    calldata.extend_from_slice(&function_selector);
    
    // In production, this would be properly ABI encoded
    // For now, we're just creating a placeholder
    
    calldata
}

// Create transaction data for JIT liquidity
fn create_jit_transaction(params: multi_strategy_flash::JITLiquidityParams) -> Vec<u8> {
    // Convert the JIT parameters to calldata for JITLiquidityProvider
    
    // Function selector for executeJITLiquidity
    let function_selector = [0x87, 0x65, 0x43, 0x21]; 
    
    // Create simplified calldata
    let mut calldata = Vec::new();
    calldata.extend_from_slice(&function_selector);
    
    // In production, this would be properly ABI encoded
    
    calldata
}

async fn run_engine<M: Middleware + 'static, S: Signer + 'static>(
    strategy: MultiStrategy<M, S>,
    collector: BlockCollector,
    execution_mode: &str,
    client: Arc<SignerMiddleware<Arc<Provider<Http>>, LocalWallet>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create the appropriate executor based on the mode
    let executor: Box<dyn Executor<Action = H256, Event = ExecutionSummary>> = match execution_mode {
        "flashbots" => Box::new(FlashbotsExecutor::new(
            client,
            // Add any Flashbots-specific parameters here
        )),
        "mev-share" => Box::new(MevShareExecutor::new(
            client,
            // Add any MEV-Share-specific parameters here
        )),
        _ => Box::new(MemPoolExecutor::new(client)),
    };

    // Create the engine
    let mut engine = Engine::new(
        strategy,
        collector,
        executor,
        map_action_to_executor_action,
    );

    // Run the engine
    engine.run().await;

    Ok(())
}

async fn load_config(path: PathBuf) -> Result<Config, Box<dyn Error + Send + Sync>> {
    let config_str = tokio::fs::read_to_string(path).await?;
    let config: Config = serde_json::from_str(&config_str)?;
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Get the RPC URL from environment variables
    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL must be set");
    let provider = Provider::<Http>::try_from(rpc_url.clone())?;
    let provider = Arc::new(provider);

    // Get the private key from environment variables
    let private_key = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");
    let wallet = private_key.parse::<LocalWallet>()?;
    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

    match &cli.command {
        Some(Commands::Run { mode }) => {
            // Load configuration
            let config_path = cli
                .config
                .unwrap_or_else(|| PathBuf::from("config.json"));
            let config = load_config(config_path).await?;

            // Create strategy and collector
            let strategy = MultiStrategy::new(client.clone(), config);
            let collector = BlockCollector::new(provider.clone());

            // Run the engine
            info!("Starting multi-strategy bot in {} mode", mode);
            run_engine(strategy, collector, mode, client).await?
        }
        Some(Commands::Simulate { tx_path }) => {
            // Load transaction data
            let tx_data = tokio::fs::read_to_string(tx_path).await?;
            // In a real implementation, you would parse and simulate the transaction
            info!("Simulating transaction: {}", tx_data);
        }
        None => {
            // No command provided, just print help
            Cli::parse_from(["multi-strategy-flash-example", "--help"]);
        }
    }

    Ok(())
}
