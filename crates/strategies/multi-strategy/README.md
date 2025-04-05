# Multi-Strategy MEV Bot

A comprehensive MEV capture system built on the Artemis framework that combines multiple strategies:

## Features

1. **Flash Loan Arbitrage**: Executes profitable trading paths across multiple DEXes using flash loans for capital efficiency.
   - Path finding algorithm for identifying profitable routes
   - Optimal loan size calculation
   - Profit estimation with gas costs factored in

2. **Just-In-Time (JIT) Liquidity**: Adds liquidity to DEX pools just before large swaps and removes it immediately after.
   - Transaction analysis for swap detection
   - Fee capture calculation
   - Multi-pool monitoring

3. **MEV-Share Backrunning**: Identifies and executes profitable backrun opportunities from the MEV-Share network.
   - Integration with the MEV-Share API
   - Targeted backrun submission
   - Bundle creation for reliable execution

## Architecture

### Smart Contracts

The system includes two main smart contracts:

1. **FlashArbExecutor**: Executes cross-DEX arbitrage using flash loans
   - AAVE flash loan integration
   - Multi-DEX routing
   - V2 and V3 DEX support

2. **JITLiquidityProvider**: Manages just-in-time liquidity provision
   - V2 and V3 liquidity management
   - Efficient fee collection
   - Gas-optimized implementation

### Rust Implementation

The Rust implementation integrates with the Artemis framework:

- **Event Processing**: Handles blockchain events (blocks, transactions, MEV-Share)
- **Path Finding**: Graph-based algorithm for discovering profitable arbitrage paths
- **Pricing Models**: Calculates expected profits with gas costs factored in
- **Transaction Creation**: Builds optimized transactions for execution

## Getting Started

1. Clone the repository:
   ```
   git clone https://github.com/your-username/artemis.git
   cd artemis
   ```

2. Copy `.env.example` to `.env` and fill in your configuration:
   ```
   cp crates/strategies/multi-strategy/.env.example .env
   ```

3. Deploy the smart contracts:
   ```
   cd crates/strategies/multi-strategy/contracts
   forge script script/Deploy.s.sol --rpc-url $ETH_RPC_URL --private-key $PRIVATE_KEY --broadcast
   ```

4. Update the configuration with your deployed contract addresses:
   ```
   cd ../..
   # Edit config.json with your deployed contract addresses
   ```

5. Run the simulation to test your setup:
   ```
   cargo run --bin simulate
   ```

6. Start the bot:
   ```
   cargo run --bin multi-strategy-bot -- --wss $WS_RPC_URL --private-key $PRIVATE_KEY
   ```

## Advanced Configuration

See the [DEPLOYMENT.md](./DEPLOYMENT.md) file for advanced configuration options and optimization techniques.

## Warning

MEV extraction is highly competitive and can involve significant financial risk. This software is provided for educational purposes only. Use at your own risk.

## License

This project is licensed under the MIT License - see the LICENSE file for details.