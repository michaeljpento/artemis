# Deployment Instructions for Flash Loan Contracts on Polygon

These instructions will guide you through deploying the Flash Loan contracts to Polygon Mainnet.

## Prerequisites

1. Make sure you have Foundry installed. If not, install it:
   ```
   curl -L https://foundry.paradigm.xyz | bash
   foundryup
   ```

2. Update the `.env` file with your private key and API keys:
   ```
   PRIVATE_KEY=your_private_key_here
   POLYGON_RPC_URL=https://polygon-mainnet.g.alchemy.com/v2/your_api_key_here
   POLYGONSCAN_API_KEY=your_polygonscan_api_key_here
   ```

## Deployment Steps

1. Make sure you're in the contracts directory:
   ```
   cd /path/to/artemis/crates/strategies/multi-strategy-flash/contracts
   ```

2. Build the contracts:
   ```
   forge build
   ```

3. Deploy the contracts to Polygon Mainnet:
   ```
   forge script script/Deploy.s.sol --rpc-url $POLYGON_RPC_URL --private-key $PRIVATE_KEY --broadcast --verify
   ```

4. After deployment, you'll see the addresses of the deployed contracts in the console output. Note them down as you'll need them for your Rust strategy.

## Contract Addresses for Polygon

The following addresses are used in the contracts:

- **Aave V3 Address Provider**: `0xd05e3E715d945B59290df0ae8eF85c1BdB684744`
- **Balancer Vault**: `0xBA12222222228d8Ba445958a75a0704d566BF2C8`
- **Uniswap V3 Factory**: `0x1F98431c8aD98523631AE4a59f267346ea31F984`
- **QuickSwap Factory**: `0x5757371414417b8C6CAad45bAeF941aBc7d3Ab32`
- **Uniswap V3 NFT Position Manager**: `0xC36442b4a4522E871399CD717aBDD847Ab11FE88`

## Key Tokens on Polygon

- **WMATIC**: `0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270`
- **USDC**: `0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174`
- **USDT**: `0xc2132D05D31c914a87C6611C10748AEb04B58e8F`
- **DAI**: `0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063`
- **WBTC**: `0x1BFD67037B42Cf73acF2047067bd4F2C47D9BfD6`

## Testing on Testnet First

It's recommended to test on Mumbai (Polygon's testnet) first by updating the RPC URL to a Mumbai endpoint and using testnet tokens.

## Integrating with Artemis Strategy

After deployment, update your Rust strategy code with the deployed contract addresses:

```rust
// Example in your strategy.rs
let flash_arb_executor = "your_deployed_address".parse::<Address>()?;
let jit_liquidity_provider = "your_deployed_address".parse::<Address>()?;
```