# Multi-Strategy MEV Bot Deployment Guide

This guide outlines the steps to deploy the smart contracts for the multi-strategy MEV bot and connect them to the Artemis framework.

## Prerequisites

1. **Node.js and npm**: Required for contract compilation and deployment
2. **Foundry**: Smart contract development toolkit
3. **Rust**: Required for Artemis framework
4. **Ethereum RPC URL**: Access to a node provider (Infura, Alchemy, etc.)
5. **Private key**: For deploying contracts and sending transactions

## Step 1: Install Dependencies

First, install the required dependencies for the contracts:

```bash
cd crates/strategies/multi-strategy/contracts
forge install OpenZeppelin/openzeppelin-contracts
```

## Step 2: Compile Smart Contracts

Compile the smart contracts using Foundry:

```bash
cd crates/strategies/multi-strategy/contracts
forge build
```

## Step 3: Set Up Environment Variables

Create a `.env` file in the root directory with the following variables:

```
# Node provider
ETH_RPC_URL=YOUR_RPC_URL
WS_RPC_URL=YOUR_WEBSOCKET_URL

# Keys
PRIVATE_KEY=YOUR_PRIVATE_KEY
ETHERSCAN_API_KEY=YOUR_ETHERSCAN_API_KEY

# Contract addresses (to be filled after deployment)
FLASH_ARB_EXECUTOR_ADDRESS=
JIT_LIQUIDITY_PROVIDER_ADDRESS=

# MEV-Share
ENABLE_MEV_SHARE=true
MEV_SHARE_URL=https://mev-share.flashbots.net
```

## Step 4: Deploy the Contracts

Use the Foundry deployment script to deploy the contracts to the Ethereum network:

```bash
cd crates/strategies/multi-strategy/contracts
forge script script/Deploy.s.sol --rpc-url $ETH_RPC_URL --private-key $PRIVATE_KEY --broadcast
```

## Step 5: Generate Rust Bindings

After deployment, generate the Rust bindings for the contracts:

```bash
cd crates/strategies/multi-strategy
forge bind --bindings-path ./bindings/src --root ./contracts --crate-name multi-strategy-bindings --force
```

## Step 6: Update Configuration

Update your `config.json` file with the deployed contract addresses:

```json
{
  "flash_executor_address": "YOUR_DEPLOYED_FLASH_ARB_EXECUTOR_ADDRESS",
  "jit_provider_address": "YOUR_DEPLOYED_JIT_LIQUIDITY_PROVIDER_ADDRESS",
  "min_profit_threshold": 0.01,
  "max_gas_price": 100,
  "submission_timeout": 60,
  "enable_arbitrage": true,
  "enable_jit": true,
  "enable_backrunning": true,
  "monitored_tokens": [
    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", 
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    "0x6B175474E89094C44Da98b954EedeAC495271d0F",
    "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599"
  ],
  "monitored_pools": [
    {
      "address": "0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc",
      "pool_type": "UniswapV2",
      "tokens": [
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
      ],
      "fee_tier": null
    },
    {
      "address": "0x8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8",
      "pool_type": "UniswapV3",
      "tokens": [
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
      ],
      "fee_tier": 3000
    }
  ]
}
```

## Step 7: Add Additional Pools

To maximize arbitrage opportunities, add more pools to your configuration:

1. **Uniswap V2 pools**: Major token pairs on Uniswap V2
2. **Uniswap V3 pools**: Various fee tiers (0.05%, 0.3%, 1%)
3. **SushiSwap pools**: Main token pairs
4. **Curve pools**: Stablecoin pools for stablecoin arbitrage

## Step 8: Run the Bot

Now, start the multi-strategy MEV bot:

```bash
cd examples/multi-strategy-bot
cargo run -- --wss $WS_RPC_URL --private-key $PRIVATE_KEY --config-path path/to/config.json
```

## Step 9: Monitoring and Optimization

Monitor the bot's performance and optimize its parameters:

1. Adjust `min_profit_threshold` based on gas costs and market conditions
2. Fine-tune `max_gas_price` to balance competitiveness and profitability
3. Update pool configurations as new pools emerge or liquidity shifts
4. Optimize path finding parameters for better arbitrage detection

## Advanced Configuration

### Gas Optimization

For production use, consider deploying a gas-optimized version of the contracts:

1. Use Huff or assembly for critical functions
2. Optimize storage usage to minimize gas costs
3. Consider using gas tokens for additional savings

### Risk Management

Implement risk management controls in the bot:

1. Set maximum exposure limits per token
2. Implement circuit breakers for extreme market conditions
3. Gradually increase flash loan sizes as confidence in the system grows

### Infrastructure Optimization

For ultra-low latency:

1. Run the bot on dedicated servers close to major Ethereum nodes
2. Use private RPC connections to minimize latency
3. Consider running your own Ethereum node for direct access
4. Connect to miner/validator networks directly to minimize transaction inclusion time