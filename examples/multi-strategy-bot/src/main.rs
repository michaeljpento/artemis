use anyhow::Result;
use artemis_core::engine::Engine;
use artemis_core::executors::mev_share_executor::{MevShareExecutor, SubmitToMevShare};
use artemis_core::collectors::block_collector::BlockCollector;
use artemis_core::collectors::mempool_collector::MempoolCollector;
use artemis_core::collectors::mevshare_collector::MevShareCollector;
use artemis_core::types::{Collector, Executor, Strategy};
use clap::Parser;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::*;
use ethers::providers::{Provider, Ws};
use ethers::signers::{LocalWallet, Signer};
use futures::stream::StreamExt;
use multi_strategy::{Event as MultiStrategyEvent, Config, Action as MultiStrategyAction, MultiStrategy};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};
use tracing_subscriber::FmtSubscriber;

/// Collector adapter that converts from one event type to another
struct CollectorAdapter<C, E1, E2> {
    inner: C,
    _phantom: std::marker::PhantomData<(E1, E2)>,
}

impl<C, E1, E2> CollectorAdapter<C, E1, E2>
where
    C: Collector<E1>,
    E1: 'static,
    E2: From<E1> + 'static,
{
    fn new(collector: C) -> Self {
        Self {
            inner: collector,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<C, E1, E2> Collector<E2> for CollectorAdapter<C, E1, E2>
where
    C: Collector<E1>,
    E1: 'static,
    E2: From<E1> + 'static,
{
    async fn get_event_stream(&self) -> anyhow::Result<artemis_core::types::CollectorStream<'_, E2>> {
        let stream = self.inner.get_event_stream().await?;
        let mapped_stream = stream.map(E2::from);
        Ok(Box::pin(mapped_stream))
    }
}

/// Strategy adapter that converts from one event/action type to another
struct StrategyAdapter<S, E1, E2, A1, A2> {
    inner: S,
    _phantom: std::marker::PhantomData<(E1, E2, A1, A2)>,
}

impl<S, E1, E2, A1, A2> StrategyAdapter<S, E1, E2, A1, A2>
where
    S: Strategy<E1, A1>,
    E1: 'static,
    E2: Into<E1> + 'static,
    A1: 'static,
    A2: From<A1> + 'static,
{
    fn new(strategy: S) -> Self {
        Self {
            inner: strategy,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<S, E1, E2, A1, A2> Strategy<E2, A2> for StrategyAdapter<S, E1, E2, A1, A2>
where
    S: Strategy<E1, A1>,
    E1: 'static,
    E2: Into<E1> + 'static,
    A1: 'static,
    A2: From<A1> + 'static,
{
    async fn process_event(&mut self, event: E2) -> Vec<A2> {
        let actions = self.inner.process_event(event.into()).await;
        actions.into_iter().map(A2::from).collect()
    }

    async fn sync_state(&mut self) -> anyhow::Result<()> {
        self.inner.sync_state().await
    }
}

/// Multi-strategy MEV bot for arbitrage, JIT liquidity, and backrunning
#[derive(Parser, Debug)]
struct Args {
    /// RPC WebSocket URL
    #[clap(long, env = "WS_RPC_URL")]
    wss: String,
    
    /// Private key for transactions
    #[clap(long, env = "PRIVATE_KEY")]
    private_key: String,
    
    /// Path to configuration file
    #[clap(long, env = "CONFIG_PATH", default_value = "config.json")]
    config_path: PathBuf,
    
    /// Whether to enable MEV-Share integration
    #[clap(long, env = "ENABLE_MEV_SHARE", default_value = "true")]
    enable_mev_share: bool,
    
    /// HTTP URL for MEV-Share
    #[clap(long, env = "MEV_SHARE_URL", default_value = "https://mev-share-goerli.flashbots.net")]
    mev_share_url: String,
}

/// Event types that the engine uses
enum EngineEvent {
    Block(Block<H256>),
    Transaction(Transaction),
    MevShare(mev_share::sse::Event),
}

impl From<Block<H256>> for EngineEvent {
    fn from(block: Block<H256>) -> Self {
        EngineEvent::Block(block)
    }
}

impl From<Transaction> for EngineEvent {
    fn from(tx: Transaction) -> Self {
        EngineEvent::Transaction(tx)
    }
}

impl From<mev_share::sse::Event> for EngineEvent {
    fn from(event: mev_share::sse::Event) -> Self {
        EngineEvent::MevShare(event)
    }
}

impl From<EngineEvent> for MultiStrategyEvent {
    fn from(event: EngineEvent) -> Self {
        match event {
            EngineEvent::Block(block) => MultiStrategyEvent::NewBlock(block),
            EngineEvent::Transaction(tx) => MultiStrategyEvent::Transaction(tx),
            EngineEvent::MevShare(event) => MultiStrategyEvent::MevShareEvent(event),
        }
    }
}

/// Action types that the engine uses
enum EngineAction {
    SubmitToMevShare(SubmitToMevShare),
}

impl From<MultiStrategyAction> for EngineAction {
    fn from(action: MultiStrategyAction) -> Self {
        match action {
            MultiStrategyAction::ExecuteArbitrage { path, expected_profit } => {
                info!("Creating arbitrage transaction with expected profit: {} ETH", expected_profit);
                
                // Build the transaction for the arbitrage
                let mut tx = TransactionRequest::new();
                tx = tx.to(path.start_token); // In reality, this would be the executor contract
                
                // Create data for the arbitrage transaction
                let mut data = Vec::new();
                data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]); // Example function selector
                
                // For a real implementation, we would:
                // 1. Encode the path information
                // 2. Calculate optimal gas settings
                // 3. Estimate gas costs
                
                tx = tx.data(data);
                
                // Create a MEV-Share submission
                let submission = SubmitToMevShare {
                    tx: tx.into(),
                    target: Some(path.start_token), // Target the first pool in the path
                    hints: None, // No additional hints
                };
                
                EngineAction::SubmitToMevShare(submission)
            },
            MultiStrategyAction::ExecuteJitLiquidity { pool, amounts, expected_profit } => {
                info!("Creating JIT liquidity transaction with expected profit: {} ETH", expected_profit);
                
                // Build the transaction for JIT liquidity provision
                let mut tx = TransactionRequest::new();
                tx = tx.to(pool); // In reality, this would be the JIT provider contract
                
                // Create data for the JIT transaction
                let mut data = Vec::new();
                data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef]); // Example function selector
                
                // For a real implementation, we would:
                // 1. Encode the amount information
                // 2. Calculate optimal gas settings
                // 3. Estimate gas costs
                
                tx = tx.data(data);
                
                // Create a MEV-Share submission
                let submission = SubmitToMevShare {
                    tx: tx.into(),
                    target: Some(pool),
                    hints: None, // No additional hints
                };
                
                EngineAction::SubmitToMevShare(submission)
            },
            MultiStrategyAction::ExecuteBackrun { target_tx, backrun_data, expected_profit } => {
                info!("Creating backrun transaction with expected profit: {} ETH", expected_profit);
                
                // Build the transaction for the backrun
                let mut tx = TransactionRequest::new();
                // In reality, this would be the executor contract
                tx = tx.to(Address::zero());
                
                // In a real implementation, we would:
                // 1. Use the backrun_data to create the full transaction
                // 2. Calculate optimal gas settings
                // 3. Estimate gas costs
                
                tx = tx.data(backrun_data);
                
                // Create a MEV-Share submission
                let submission = SubmitToMevShare {
                    tx: tx.into(),
                    target: None,
                    hints: Some(json!({
                        "txs": [format!("0x{}", hex::encode(target_tx.as_bytes()))]
                    })),
                };
                
                EngineAction::SubmitToMevShare(submission)
            },
        }
    }
}

/// Executor adapter that converts from one action type to another
struct ExecutorAdapter<E, A1, A2> {
    inner: E,
    _phantom: std::marker::PhantomData<(A1, A2)>,
}

impl<E, A1, A2> ExecutorAdapter<E, A1, A2>
where
    E: Executor<A1>,
    A1: 'static,
    A2: Into<A1> + 'static,
{
    fn new(executor: E) -> Self {
        Self {
            inner: executor,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<E, A1, A2> Executor<A2> for ExecutorAdapter<E, A1, A2>
where
    E: Executor<A1>,
    A1: 'static,
    A2: Into<A1> + 'static,
{
    async fn execute(&self, action: A2) -> anyhow::Result<()> {
        self.inner.execute(action.into()).await
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    let args = Args::parse();
    
    // Load config
    let config_file = std::fs::File::open(&args.config_path)?;
    let config: Config = serde_json::from_reader(config_file)?;
    
    // Set up provider with signer
    let provider = Provider::<Ws>::connect(&args.wss).await?;
    let chain_id = provider.get_chainid().await?.as_u64();
    let wallet = args.private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
    let provider = Arc::new(SignerMiddleware::new(provider, wallet.clone()));
    
    // Create collectors with adapters
    let block_collector = CollectorAdapter::new(BlockCollector::new(provider.clone()));
    let mempool_collector = CollectorAdapter::new(MempoolCollector::new(provider.clone()));
    
    // Create strategy with adapter
    let strategy = MultiStrategy::new(config, provider.clone());
    let strategy_adapter = StrategyAdapter::new(strategy);
    
    // Create executor with adapter
    let mev_share_executor = MevShareExecutor::new(
        provider.clone(),
        args.mev_share_url.clone(),
        None,
    );
    let executor_adapter = ExecutorAdapter::new(mev_share_executor);
    
    // Set up engine to connect components
    let mut engine = Engine::new();
    
    // Add collectors to the engine
    engine.add_collector(Box::new(block_collector));
    engine.add_collector(Box::new(mempool_collector));
    
    if args.enable_mev_share {
        let mev_share_collector = CollectorAdapter::new(MevShareCollector::new(args.mev_share_url.clone()));
        engine.add_collector(Box::new(mev_share_collector));
    }
    
    // Add strategy to the engine
    engine.add_strategy(Box::new(strategy_adapter));
    
    // Add executor to the engine
    engine.add_executor(Box::new(executor_adapter));
    
    // Start the engine
    info!("Starting multi-strategy MEV bot...");
    engine.run().await;
    
    Ok(())
}