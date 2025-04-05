# Multi-Strategy Flash Loan Bot for Polygon

This strategy integrates multiple MEV strategies with flash loans to provide a comprehensive, capital-efficient approach to extracting value from the Polygon blockchain.

## Features

- **Flash Loan Integration**: Uses flash loans from multiple providers (Aave, Balancer, Uniswap V3) to source capital for all strategies
- **Multi-Strategy Approach**: Combines arbitrage, JIT liquidity, and MEV-Share backrunning into a single bot
- **Ultra-Low Latency**: Optimized for fast execution to remain competitive in the MEV landscape
- **Capital Efficiency**: No upfront capital required due to flash loan utilization
- **Production Grade**: Comprehensive error handling, monitoring, and robust architecture

## Strategies

### Arbitrage

Identifies profitable arbitrage opportunities across multiple DEXs using a graph-based pathfinding algorithm.

- Supports Uniswap V2/V3, SushiSwap, and Curve
- Automatically selects the most profitable path
- Uses flash loans to borrow the required capital
- Estimates gas costs and flash loan fees for accurate profit calculation

### JIT Liquidity

Provides Just-In-Time (JIT) liquidity to DEX pools to capture swap fees.

- Supports Uniswap V2/V3 pools
- Uses flash loans to provide liquidity capital
- Calculates optimal position size and duration
- Estimates expected fees based on pool activity

### MEV-Share Backrunning

Backruns transactions via the MEV-Share protocol.

- Analyzes MEV-Share bundles for backrunning opportunities
- Calculates expected profit for each opportunity
- Submits backrun transactions via MEV-Share

## Smart Contracts

- **FlashLoanCore.sol**: Base contract for flash loan functionality, supporting multiple providers on Polygon (Aave V2/V3, Balancer, Uniswap V3)
- **FlashArbExecutor.sol**: Executes arbitrage strategies using flash loans across QuickSwap, Uniswap V3, and Curve on Polygon
- **JITLiquidityProvider.sol**: Provides JIT liquidity using flash loans on Polygon DEXes

## Usage

1. Deploy the smart contracts using Foundry:

```bash
cd contracts
forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast --verify
```

2. Update your `.env` file with the deployed contract addresses

3. Run the bot:

```bash
cargo run --example multi-strategy-flash -- run --mode mev-share --config path/to/config.json
```

## Configuration

The bot is highly configurable through the `config.json` file. Key configuration options include:

- `enabled_strategies`: List of strategies to enable
- `min_profit_threshold`: Minimum profit threshold in ETH
- `gas_price_multiplier`: Multiplier for gas cost estimation
- `max_slippage`: Maximum allowed slippage
- `flash_loan_fee_multiplier`: Multiplier to account for flash loan fees
- Strategy-specific configurations for arbitrage, JIT liquidity, and MEV-Share

## Requirements

- Rust 1.54 or later
- Foundry for contract compilation and deployment
- Ethereum RPC endpoint
- Private key with ETH for gas costs

## License

[MIT](../../LICENSE-MIT) or [Apache-2.0](../../LICENSE-APACHE)