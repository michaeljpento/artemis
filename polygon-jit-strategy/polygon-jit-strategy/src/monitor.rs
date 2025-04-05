use anyhow::Result;
use ethers::types::{U256, Address};
use prometheus::{
    Registry, register_counter, register_gauge, register_histogram,
    Counter, Gauge, Histogram, HistogramOpts,
};
use serde_json::json;
use std::{
    collections::HashMap, 
    sync::{Arc, Mutex}, 
    time::{Duration, Instant}
};
use warp::Filter as WarpFilter;
use tracing::{debug, error, info};

use crate::strategy::{JitOpportunity, OpportunityType};

// Metrics storage
#[derive(Debug, Clone)]
pub struct Metrics {
    // Counters
    opportunities_detected: Counter,
    transactions_executed: Counter,
    transactions_failed: Counter,
    
    // Gauges
    current_gas_price: Gauge,
    estimated_profit_usd: Gauge,
    wallet_balance: Gauge,
    
    // Histograms
    opportunity_profit: Histogram,
    transaction_execution_time: Histogram,
    
    // Opportunity metrics by type
    opportunities_by_type: HashMap<OpportunityType, Counter>,
    profit_by_type: HashMap<OpportunityType, Counter>,
    
    // Recent opportunities
    recent_opportunities: Arc<Mutex<Vec<(Instant, JitOpportunity)>>>,
    
    // Statistics
    total_profit_usd: Counter,
    total_gas_spent: Counter,
}

impl Metrics {
    pub fn new() -> Result<(Self, Registry)> {
        let registry = Registry::new();
        
        // Create counter metrics
        let opportunities_detected = register_counter!(
            "jit_opportunities_detected_total", 
            "Total number of detected opportunities", 
            registry
        )?;
        
        let transactions_executed = register_counter!(
            "jit_transactions_executed_total", 
            "Total number of executed transactions", 
            registry
        )?;
        
        let transactions_failed = register_counter!(
            "jit_transactions_failed_total", 
            "Total number of failed transactions", 
            registry
        )?;
        
        // Create gauge metrics
        let current_gas_price = register_gauge!(
            "jit_current_gas_price_gwei", 
            "Current gas price in gwei", 
            registry
        )?;
        
        let estimated_profit_usd = register_gauge!(
            "jit_estimated_profit_usd", 
            "Current estimated profit in USD", 
            registry
        )?;
        
        let wallet_balance = register_gauge!(
            "jit_wallet_balance_eth", 
            "Wallet balance in ETH", 
            registry
        )?;
        
        // Create histogram metrics
        let opportunity_profit = register_histogram!(
            "jit_opportunity_profit_usd", 
            "Distribution of opportunity profits in USD",
            vec![0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0],
            registry
        )?;
        
        let transaction_execution_time = register_histogram!(
            HistogramOpts::new(
                "jit_transaction_execution_time_seconds", 
                "Time to execute transactions in seconds"
            ).buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0]),
            registry
        )?;
        
        // Create total profit counter
        let total_profit_usd = register_counter!(
            "jit_total_profit_usd", 
            "Total profit in USD", 
            registry
        )?;
        
        // Create total gas spent counter
        let total_gas_spent = register_counter!(
            "jit_total_gas_spent_wei", 
            "Total gas spent in wei", 
            registry
        )?;
        
        // Create opportunity type specific counters
        let mut opportunities_by_type = HashMap::new();
        let mut profit_by_type = HashMap::new();
        
        for op_type in [OpportunityType::JitLiquidity, OpportunityType::FlashArbitrage, OpportunityType::BatchMicroJit] {
            let type_name = match op_type {
                OpportunityType::JitLiquidity => "jit_liquidity",
                OpportunityType::FlashArbitrage => "flash_arb",
                OpportunityType::BatchMicroJit => "batch_micro_jit",
            };
            
            opportunities_by_type.insert(
                op_type.clone(),
                register_counter!(
                    format!("jit_opportunities_{}_total", type_name),
                    format!("Total number of {} opportunities", type_name),
                    registry
                )?
            );
            
            profit_by_type.insert(
                op_type.clone(),
                register_counter!(
                    format!("jit_profit_{}_usd_total", type_name),
                    format!("Total profit from {} in USD", type_name),
                    registry
                )?
            );
        }
        
        Ok((
            Self {
                opportunities_detected,
                transactions_executed,
                transactions_failed,
                current_gas_price,
                estimated_profit_usd,
                wallet_balance,
                opportunity_profit,
                transaction_execution_time,
                opportunities_by_type,
                profit_by_type,
                recent_opportunities: Arc::new(Mutex::new(Vec::new())),
                total_profit_usd,
                total_gas_spent,
            },
            registry
        ))
    }
    
    // Record a detected opportunity
    pub fn record_opportunity(&self, opportunity: &JitOpportunity) {
        self.opportunities_detected.inc();
        
        if let Some(counter) = self.opportunities_by_type.get(&opportunity.opportunity_type) {
            counter.inc();
        }
        
        self.opportunity_profit.observe(opportunity.estimated_profit_usd);
        self.estimated_profit_usd.set(opportunity.estimated_profit_usd);
        
        // Update recent opportunities list (keep only last 100)
        let mut recent = self.recent_opportunities.lock().unwrap();
        recent.push((Instant::now(), opportunity.clone()));
        
        // Keep only the last 100 opportunities
        if recent.len() > 100 {
            recent.remove(0);
        }
    }
    
    // Record a successful transaction
    pub fn record_transaction_success(&self, opportunity: &JitOpportunity, duration: Duration, gas_used: Option<U256>) {
        self.transactions_executed.inc();
        self.transaction_execution_time.observe(duration.as_secs_f64());
        
        // Record profit
        let profit_usd = opportunity.estimated_profit_usd;
        self.total_profit_usd.inc_by(profit_usd);
        
        if let Some(counter) = self.profit_by_type.get(&opportunity.opportunity_type) {
            counter.inc_by(profit_usd);
        }
        
        // Record gas used if available
        if let Some(gas) = gas_used {
            let gas_cost = gas * opportunity.gas_price;
            self.total_gas_spent.inc_by(gas_cost.as_u64() as f64);
        }
    }
    
    // Record a failed transaction
    pub fn record_transaction_failure(&self) {
        self.transactions_failed.inc();
    }
    
    // Update current gas price
    pub fn update_gas_price(&self, gas_price_gwei: f64) {
        self.current_gas_price.set(gas_price_gwei);
    }
    
    // Update wallet balance
    pub fn update_wallet_balance(&self, balance_eth: f64) {
        self.wallet_balance.set(balance_eth);
    }
    
    // Get recent opportunities
    pub fn get_recent_opportunities(&self) -> Vec<(Instant, JitOpportunity)> {
        let recent = self.recent_opportunities.lock().unwrap();
        recent.clone()
    }
    
    // Get statistics summary
    pub fn get_statistics(&self) -> serde_json::Value {
        json!({
            "opportunities": {
                "detected": self.opportunities_detected.get(),
                "executed": self.transactions_executed.get(),
                "failed": self.transactions_failed.get(),
                "by_type": {
                    "jit_liquidity": self.opportunities_by_type.get(&OpportunityType::JitLiquidity).map(|c| c.get()).unwrap_or(0.0),
                    "flash_arb": self.opportunities_by_type.get(&OpportunityType::FlashArbitrage).map(|c| c.get()).unwrap_or(0.0),
                    "batch_micro_jit": self.opportunities_by_type.get(&OpportunityType::BatchMicroJit).map(|c| c.get()).unwrap_or(0.0),
                }
            },
            "profit": {
                "total_usd": self.total_profit_usd.get(),
                "by_type": {
                    "jit_liquidity": self.profit_by_type.get(&OpportunityType::JitLiquidity).map(|c| c.get()).unwrap_or(0.0),
                    "flash_arb": self.profit_by_type.get(&OpportunityType::FlashArbitrage).map(|c| c.get()).unwrap_or(0.0),
                    "batch_micro_jit": self.profit_by_type.get(&OpportunityType::BatchMicroJit).map(|c| c.get()).unwrap_or(0.0),
                }
            },
            "current": {
                "wallet_balance_eth": self.wallet_balance.get(),
                "gas_price_gwei": self.current_gas_price.get(),
            },
            "gas": {
                "total_spent_wei": self.total_gas_spent.get()
            }
        })
    }
}

// Start the metrics server with dashboard
pub async fn start_metrics_server(metrics: Arc<Metrics>, registry: Registry, port: u16) {
    info!("Starting metrics server on port {}", port);
    
    // Endpoint for Prometheus metrics
    let metrics_route = warp::path("metrics").map(move || {
        let encoder = prometheus::TextEncoder::new();
        let metric_families = registry.gather();
        
        match encoder.encode_to_string(&metric_families) {
            Ok(metrics_text) => warp::reply::with_status(
                metrics_text,
                warp::http::StatusCode::OK,
            ),
            Err(e) => {
                error!("Failed to encode metrics: {}", e);
                warp::reply::with_status(
                    "Failed to encode metrics".to_string(),
                    warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
        }
    });
    
    // Endpoint for opportunities JSON data
    let metrics_clone = metrics.clone();
    let opportunities_route = warp::path("opportunities").map(move || {
        let opportunities = metrics_clone.get_recent_opportunities();
        
        // Convert to simplified format
        let json_data: Vec<serde_json::Value> = opportunities.iter()
            .map(|(time, opp)| {
                json!({
                    "timestamp": time.elapsed().as_secs(),
                    "type": format!("{:?}", opp.opportunity_type),
                    "profit_usd": opp.estimated_profit_usd,
                    "pool": format!("{:?}", opp.pool_address),
                    "pool_type": opp.pool_type,
                })
            })
            .collect();
            
        warp::reply::json(&json_data)
    });
    
    // Endpoint for statistics summary
    let metrics_clone = metrics.clone();
    let stats_route = warp::path("stats").map(move || {
        let stats = metrics_clone.get_statistics();
        warp::reply::json(&stats)
    });
    
    // Serve a simple dashboard HTML page
    let dashboard_route = warp::path("dashboard").map(|| {
        let html = include_str!("../../dashboard.html").to_string();
        warp::reply::html(html)
    }).or(warp::path::end().map(|| {
        let html = include_str!("../../dashboard.html").to_string();
        warp::reply::html(html)
    }));
    
    let routes = metrics_route
        .or(opportunities_route)
        .or(stats_route)
        .or(dashboard_route);
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
}

// Monitor wallet balance
pub async fn monitor_wallet_balance<M: ethers::prelude::Middleware + 'static>(
    client: Arc<M>,
    address: Address,
    metrics: Arc<Metrics>,
) {
    use tokio::time;
    use ethers::utils::format_units;
    
    let mut interval = time::interval(Duration::from_secs(60));
    
    loop {
        interval.tick().await;
        
        match client.get_balance(address, None).await {
            Ok(balance) => {
                let balance_eth = format_units(balance, "ether")
                    .unwrap_or_else(|_| "0".to_string())
                    .parse::<f64>()
                    .unwrap_or(0.0);
                
                metrics.update_wallet_balance(balance_eth);
                debug!("Current wallet balance: {} ETH", balance_eth);
            },
            Err(e) => {
                error!("Failed to get wallet balance: {}", e);
            }
        }
    }
}