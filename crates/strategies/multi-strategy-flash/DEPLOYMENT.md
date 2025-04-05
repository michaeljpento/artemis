# Deployment Guide

This guide walks through the process of deploying and running the Multi-Strategy Flash Loan bot.

## Prerequisites

- Rust and Cargo installed
- Foundry installed for smart contract deployment
- Ethereum RPC endpoint with good reliability and low latency
- Private key with ETH for gas costs
- (Optional) MEV-Share API key for backrunning strategies

## Step 1: Clone and Build the Repository

```bash
# Clone the repository
git clone <repository-url>
cd artemis

# Build the project
cargo build --release
```

## Step 2: Deploy Smart Contracts

```bash
# Navigate to the contracts directory
cd crates/strategies/multi-strategy-flash/contracts

# Install dependencies
forge install

# Create and configure .env file
cp ../.env.example .env
# Edit .env with your actual values

# Deploy contracts
forge script script/Deploy.s.sol --rpc-url $RPC_URL --broadcast --verify
```

Note the deployed contract addresses for the next step.

## Step 3: Configure the Bot

1. Create a configuration file based on the example:

```bash
cp examples/multi-strategy-flash/config.json config.json
```

2. Update the configuration file with your settings, including:
   - Update the deployed contract addresses
   - Configure the tokens you want to monitor
   - Set your profit thresholds and risk parameters
   - Enable/disable specific strategies

3. Create a `.env` file for runtime configuration:

```bash
cp examples/multi-strategy-flash/.env.example .env
```

4. Update the `.env` file with:
   - Your RPC URL
   - Your private key
   - The deployed contract addresses
   - Provider addresses (Aave, Balancer, Uniswap)
   - Gas configuration
   - MEV-Share settings

## Step 4: Run the Bot

```bash
# Run the bot in mempool mode
cargo run --release --example multi-strategy-flash -- run --mode mempool --config config.json

# Or run in MEV-Share mode
cargo run --release --example multi-strategy-flash -- run --mode mev-share --config config.json

# Or run in Flashbots mode
cargo run --release --example multi-strategy-flash -- run --mode flashbots --config config.json
```

## Advanced Configuration

### Gas Optimization

For competitive MEV capture, gas optimization is crucial:

1. Set a reasonable `gas_price_multiplier` (1.1-1.3 is typical)
2. Configure `MAX_GAS_PRICE` to prevent excessive gas costs
3. Use `GAS_BIDDING_INCREMENTS` to control how aggressively the bot bids

### Profit Tuning

To optimize profit capture:

1. Start with conservative profit thresholds
2. Monitor performance and adjust thresholds based on success rate
3. Consider adjusting `max_slippage` based on market volatility

### Risk Management

To manage risk effectively:

1. Set appropriate `max_flash_loan_amount` limits
2. Start with a limited set of tokens and pools
3. Monitor the bot's performance closely before scaling up

## Production Deployment

For a production deployment, consider:

1. Using a dedicated server with low latency to Ethereum nodes
2. Running multiple instances with different strategies or parameters
3. Setting up monitoring and alerting
4. Implementing circuit breakers for extreme market conditions
5. Using a secure key management solution

## Troubleshooting

Common issues:

1. **Transaction underpriced**: Increase your gas price multiplier
2. **Insufficient balance**: Ensure your wallet has enough ETH for gas
3. **Flash loan failures**: Check that your configured provider addresses are correct
4. **No opportunities found**: Verify your profit thresholds aren't too high
5. **RPC connection issues**: Switch to a more reliable RPC provider

## Maintenance

Regular maintenance tasks:

1. Monitor gas costs and adjust parameters as needed
2. Update provider addresses if protocols change
3. Add new tokens and pools as they become popular
4. Tune profit parameters based on market conditions
5. Update the bot when new Artemis framework versions are released