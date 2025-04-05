# Polygon Integration Guide for Multi-Strategy Flash

This document outlines how to integrate with the Polygon-optimized flash loan contracts for the multi-strategy MEV bot.

## Deployed Contract Addresses

### Final Optimized Contracts (Recommended)
- **PolygonFlashArbExecutor**: [`0x4BD6A863cB5EB1205b8f6eA9c0B3640C2Aa84d28`](https://polygonscan.com/address/0x4bd6a863cb5eb1205b8f6ea9c0b3640c2aa84d28)
- **PolygonJITLiquidityProvider**: [`0x46a309007878EACA588fd33e608C57722e88A404`](https://polygonscan.com/address/0x46a309007878eaca588fd33e608c57722e88a404)

### Earlier Versions
- Previous FlashArbExecutor: [`0x6f1A6680462c30d1203f6DBD233db5C55Bb788AF`](https://polygonscan.com/address/0x6f1a6680462c30d1203f6dbd233db5c55bb788af)
- Previous JITLiquidityProvider: [`0x26A4A3cE9387a39CeeA2524dBbD005Bcb2f99054`](https://polygonscan.com/address/0x26a4a3ce9387a39ceea2524dbbd005bcb2f99054)
- Original FlashArbExecutor: [`0xCBa31Ba5Ee017eC4A2b98104f41373d4D300938C`](https://polygonscan.com/address/0xcba31ba5ee017ec4a2b98104f41373d4d300938c)
- Original JITLiquidityProvider: [`0x35bf93b09a819503Ef7D02ca6b5FECC2EDd19556`](https://polygonscan.com/address/0x35bf93b09a819503ef7d02ca6b5fecc2edd19556)

## Polygon-Specific Optimization Features

### 1. Block Time Awareness
- Polygon blocks occur every ~2 seconds (vs Ethereum's ~12 seconds)
- Execution time tracking to ensure operations complete within tight timeframes
- Time constraints based on Polygon's faster block time
- Dynamic execution time limits based on operation type (stricter for competitive modes)

### 2. Gas Optimization
- Configurable gas price settings optimized for Polygon
- Priority fee management for competitive transaction inclusion
- Gas limit buffers specific to Polygon's network characteristics
- Assembly-level optimizations for gas efficiency
- Dynamic priority fee multipliers for competitive operations

### 3. DEX Integration
- Support for Polygon-native DEXes including QuickSwap and SushiSwap
- Optimized flash loan routing through Aave V3, Balancer, and Uniswap V3 on Polygon
- Common Polygon token pairs for efficient routing
- Specialized Balancer integration for zero-fee flash loans

### 4. Advanced Operation Modes
- Standard Operation (0x01): Balanced profitability and execution
- Ultra-Aggressive Mode (0x02): Maximum competitiveness, priority fees, and frontrunning
- Batch Operation Mode (0x03): Efficient micro-profit opportunity batching

### 5. Memory Efficiency
- Assembly-level parameter encoding and decoding
- Optimized calldata handling for gas savings
- Precise memory allocation for complex operations
- IR-based compilation for handling stack complexity

### 6. Competitive Strategy Features
- Competitor address monitoring and frontrunning
- Pre-emptive execution capabilities
- Micro-profit batching for cumulative gains
- Volume and liquidity-depth aware token selection algorithm

## Integration in Rust with Artemis

Add the following to your Rust strategy code:

```rust
use artemis_core::types::Strategy;
use ethers::prelude::*;

// Token addresses on Polygon
pub const WMATIC: &str = "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270";
pub const USDC: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
pub const USDT: &str = "0xc2132D05D31c914a87C6611C10748AEb04B58e8F";
pub const DAI: &str = "0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063";
pub const WBTC: &str = "0x1BFD67037B42Cf73acF2047067bd4F2C47D9BfD6";

// Contract addresses
pub const POLYGON_FLASH_ARB_EXECUTOR: &str = "0x4BD6A863cB5EB1205b8f6eA9c0B3640C2Aa84d28";
pub const POLYGON_JIT_LIQUIDITY_PROVIDER: &str = "0x46a309007878EACA588fd33e608C57722e88A404";

// Connect to the contracts
let wallet = LocalWallet::from_str("YOUR_PRIVATE_KEY")?;
let provider = Provider::<Http>::try_from("https://polygon-mainnet.g.alchemy.com/v2/YOUR_API_KEY")?;
let client = SignerMiddleware::new(provider, wallet);

// Create contract instances
let flash_arb_executor = FlashArbExecutor::new(
    Address::from_str(POLYGON_FLASH_ARB_EXECUTOR)?, 
    client.clone()
);

let jit_liquidity_provider = JITLiquidityProvider::new(
    Address::from_str(POLYGON_JIT_LIQUIDITY_PROVIDER)?, 
    client.clone()
);
```

## Gas and Priority Fee Configuration

The contracts include functions to adjust gas parameters based on network conditions:

```rust
// For Flash Arbitrage Executor
let max_gas_price = U256::from(50_000_000_000u64); // 50 gwei
let priority_fee = U256::from(30_000_000_000u64); // 30 gwei
flash_arb_executor.set_polygon_max_gas_price(max_gas_price).send().await?;
flash_arb_executor.set_polygon_priority_fee(priority_fee).send().await?;

// For JIT Liquidity Provider
jit_liquidity_provider.set_polygon_max_gas_price(max_gas_price).send().await?;
jit_liquidity_provider.set_polygon_priority_fee(priority_fee).send().await?;
```

## Executing Strategies

### Flash Loan Arbitrage
```rust
// Create arbitrage parameters
let start_token = Address::from_str(WMATIC)?;
let flash_loan_amount = ethers::utils::parse_ether("10.0")?; // 10 MATIC

// Create swap parameters
let swaps = vec![
    // Example: WMATIC -> USDC on QuickSwap
    SwapParams {
        pool: quickswap_pool_address,
        dex_type: DexType::QuickSwap,
        zero_for_one: true,
        amount_in: flash_loan_amount,
        min_amount_out: min_usdc_amount,
        // ... other parameters
    },
    // Example: USDC -> WMATIC on SushiSwap
    SwapParams {
        pool: sushiswap_pool_address,
        dex_type: DexType::SushiSwap,
        zero_for_one: false,
        amount_in: min_usdc_amount,
        min_amount_out: flash_loan_amount + profit,
        // ... other parameters
    },
];

// Create arb params
let arb_params = ArbParams {
    start_token,
    flash_loan_amount,
    swaps,
};

// Execute arbitrage
let provider = FlashLoanProvider::Aave; // or Balancer, UniswapV3
let tx = flash_arb_executor
    .execute_arbitrage(arb_params, provider)
    .gas_price(gas_price)
    .priority_fee(priority_fee)
    .send()
    .await?;
```

### JIT Liquidity Provision
```rust
// Create JIT parameters
let jit_params = JITParams {
    token0: Address::from_str(WMATIC)?,
    token1: Address::from_str(USDC)?,
    amount0: ethers::utils::parse_ether("5.0")?, // 5 MATIC
    amount1: ethers::utils::parse_units("10.0", 6)?, // 10 USDC (6 decimals)
    pool: quickswap_wmatic_usdc_pool,
    pool_type: PoolType::QuickSwap,
    min_fee_expected: min_fee,
};

// Execute JIT liquidity provision
let tx = jit_liquidity_provider
    .execute_jit_liquidity(jit_params, v3_params, provider)
    .gas_price(gas_price)
    .priority_fee(priority_fee)
    .send()
    .await?;
```

## Important Considerations for Polygon

1. **Fast Block Times**: Polygon blocks are produced every ~2 seconds, meaning:
   - Strategies must execute very quickly
   - Less time for complex calculations between blocks
   - MEV opportunities may last for fewer blocks

2. **Gas Dynamics**:
   - Polygon gas prices are generally much lower than Ethereum
   - Priority fees are crucial for competitive MEV extraction
   - Typically need higher priority fees during congested periods

3. **MEV Relay Integration**:
   - For maximum MEV extraction, consider using Polygon-specific MEV relays
   - Integration with services like Flashbots on Polygon may require special endpoints

4. **Transaction Monitoring**:
   - The contracts emit events with execution time metrics
   - Monitor these events to ensure optimal strategy execution
   - Adjust gas and priority fees based on network conditions

## Advanced Operation Modes Usage

### Ultra-Aggressive Mode

The ultra-aggressive mode is designed for maximum competitiveness in MEV extraction:

```rust
// Create JIT parameters
let jit_params = JITParams {
    // ... standard parameters
};

// Set up V3 parameters
let v3_params = V3PositionParams {
    // ... standard parameters
};

// Optional competitor transaction to frontrun
let competitor_tx = Some("0x1234..."); // Or None if not frontrunning specific tx

// Priority fee multiplier (100 = normal, 500 = extreme)
let fee_multiplier = 300; // 3x base priority fee

// Execute ultra-aggressive JIT
let tx = jit_liquidity_provider
    .execute_ultra_aggressive_jit(
        jit_params, 
        v3_params,
        competitor_tx.unwrap_or([0; 32]),
        fee_multiplier
    )
    .gas_price(gas_price)
    .send()
    .await?;
```

### Balancer Zero-Fee Flash Loans

For maximum profitability, use Balancer's 0% fee flash loans:

```rust
// Create JIT parameters
let jit_params = JITParams {
    // ... standard parameters
};

// Execute with Balancer's zero-fee flash loans
let tx = jit_liquidity_provider
    .execute_balancer_jit_liquidity(jit_params, v3_params)
    .gas_price(gas_price)
    .priority_fee(priority_fee)
    .send()
    .await?;
```

### Micro-Profit Batching

Batch multiple small-profit opportunities into a single transaction:

```rust
// Create array of JIT parameters for micro-opportunities
let jit_params_array = vec![
    JITParams { /* opportunity 1 */ },
    JITParams { /* opportunity 2 */ },
    JITParams { /* opportunity 3 */ },
    // ... more opportunities
];

// V3 parameters for each opportunity
let v3_params_array = vec![
    V3PositionParams { /* params 1 */ },
    V3PositionParams { /* params 2 */ },
    V3PositionParams { /* params 3 */ },
    // ... more params
];

// Execute batch of micro-profitable operations
let tx = jit_liquidity_provider
    .execute_batch_micro_jit(
        jit_params_array, 
        v3_params_array,
        jit_params_array.len() // Use all opportunities
    )
    .gas_price(gas_price)
    .priority_fee(priority_fee)
    .send()
    .await?;
```

### Competitor Frontrunning

Monitor and frontrun competitor JIT liquidity providers:

```rust
// Set up known competitors to monitor
let competitors = vec![
    "0xCompetitor1Address",
    "0xCompetitor2Address",
    // ... more competitors
];
let competitor_addresses = competitors.iter()
    .map(|addr| Address::from_str(addr).unwrap())
    .collect::<Vec<_>>();

// Register competitors with the contract
jit_liquidity_provider
    .set_known_competitors(competitor_addresses)
    .send()
    .await?;

// Enable frontrunning mode
jit_liquidity_provider
    .set_frontrun_competition(true)
    .send()
    .await?;

// Frontrun a specific competitor
let tx = jit_liquidity_provider
    .frontrun_competitor_jit(
        Address::from_str("0xCompetitor1Address")?,
        jit_params,
        v3_params,
        300 // Priority fee multiplier
    )
    .gas_price(gas_price)
    .priority_fee(priority_fee)
    .send()
    .await?;
```