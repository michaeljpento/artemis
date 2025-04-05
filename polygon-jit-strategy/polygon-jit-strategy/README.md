# Polygon JIT Liquidity Strategy

A Polygon-optimized JIT (Just-In-Time) liquidity provision strategy designed to maximize profits by leveraging Balancer's 0% fee flash loans and Polygon's fast block times.

## Features

- **Multiple Strategy Modes**:
  - Standard JIT with Balancer's 0% fee flash loans
  - Ultra-aggressive mode with competitor frontrunning capabilities
  - Batch micro-profit mode for cumulative gains

- **Polygon-Optimized**:
  - Designed for Polygon's 2-second block times
  - Gas-optimized for Polygon's fee structure
  - Configured for Polygon's specific DEX ecosystem (QuickSwap, SushiSwap, Uniswap V3)

- **Advanced Features**:
  - Mempool monitoring
  - Competitor transaction detection and frontrunning
  - Priority fee optimization
  - Simulation mode for testing without real execution

## Deployed Contracts

- **PolygonFlashArbExecutor**: [0xb066B4d93Ae5205154e6F1B19Af9Beb7c66268d6](https://polygonscan.com/address/0xb066B4d93Ae5205154e6F1B19Af9Beb7c66268d6)
- **PolygonJITLiquidityProvider**: [0x8D85e4359243469A929a8Db74D10ffE936bd7d0D](https://polygonscan.com/address/0x8D85e4359243469A929a8Db74D10ffE936bd7d0D)

## Setup

1. Clone the repository
2. Create a `.env` file with the following variables:
    ```
    POLYGON_WS_URL=wss://polygon-mainnet.g.alchemy.com/v2/<YOUR_API_KEY>
    PRIVATE_KEY=0x<YOUR_PRIVATE_KEY>
    JIT_LIQUIDITY_PROVIDER=0x8D85e4359243469A929a8Db74D10ffE936bd7d0D
    FLASH_ARB_EXECUTOR=0xb066B4d93Ae5205154e6F1B19Af9Beb7c66268d6
    ```
3. Build the project:
    ```
    cargo build --release
    ```

## Running the Strategy

### Simulation Mode

Run the strategy in simulation mode (no real transactions):

```bash
./start-simulation.sh
```

This mode will detect opportunities without executing any transactions, allowing you to test the strategy without financial risk.

### Default Mode (Simulation)

For safety, the default script runs in simulation mode:

```bash
./start-jit-strategy.sh
```

This mode will detect opportunities without executing any transactions, but uses the production parameters.

### Production Mode (USE WITH CAUTION)

Run the strategy in production mode (will execute real transactions):

```bash
./start-production.sh
```

⚠️ **WARNING**: This mode will actively execute transactions using real funds. Make sure your wallet is funded with sufficient MATIC for gas and the required tokens (WMATIC, USDC, etc.) for the strategy operations.

Before running in production mode:
1. Fund your wallet with at least 1 MATIC for gas
2. Fund your wallet with the tokens needed for the strategy (WMATIC, USDC, etc.)
3. Double-check your private key in the .env file

## Command Line Options

- `--aggressive`: Enable ultra-aggressive mode for maximum profits (default: true)
- `--enable-jit`: Enable JIT liquidity strategy (default: true)
- `--enable-arb`: Enable flash arbitrage strategy (default: true)
- `--min-profit-usd`: Minimum profit threshold in USD (default: 1.0)
- `--max-gas-price-gwei`: Maximum gas price in gwei (default: 100)
- `--simulation`: Run in simulation mode (default: false)

Example:
```bash
cargo run --release -- --aggressive --enable-jit --min-profit-usd 2.0 --max-gas-price-gwei 50
```

## Strategy Optimization

The strategy is currently optimized with the following settings:

- Ultra-aggressive mode is enabled
- Preemptive execution is enabled
- Competitor frontrunning is enabled
- Maximum gas price is set to 100 gwei
- Priority fee is set to 35 gwei

These settings can be adjusted via direct contract interaction if needed.

## Architecture

1. **Mempool Monitoring**: The strategy monitors the Polygon mempool for potentially profitable transactions.
2. **Opportunity Detection**: When a large swap is detected, the strategy analyzes it for profitability.
3. **Execution**: If profitable, the strategy executes one of three approaches:
   - Standard JIT with Balancer's 0% fee flash loans
   - Ultra-aggressive JIT with competitor frontrunning
   - Batch micro-profitable JIT opportunities

## License

MIT