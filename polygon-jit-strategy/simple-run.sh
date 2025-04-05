#!/bin/bash

# Simple script to run a JIT liquidity bot simulation
echo "Starting Polygon JIT strategy simulation..."

# Set your own Alchemy API key here
API_KEY="YOUR_ALCHEMY_API_KEY"

# Create a temporary .env file with the required variables
cat > .env <<EOL
# Polygon RPC URLs
POLYGON_WS_URL=wss://polygon-mainnet.g.alchemy.com/v2/${API_KEY}
POLYGON_HTTP_URL=https://polygon-mainnet.g.alchemy.com/v2/${API_KEY}

# Wallet configuration - This is a dummy key for simulation
PRIVATE_KEY=0x0000000000000000000000000000000000000000000000000000000000000001

# Contract addresses
FLASH_ARB_EXECUTOR=0xb066B4d93Ae5205154e6F1B19Af9Beb7c66268d6
JIT_LIQUIDITY_PROVIDER=0x8D85e4359243469A929a8Db74D10ffE936bd7d0D

# Strategy configuration
MIN_PROFIT_THRESHOLD_USD=0.10
MAX_GAS_PRICE_GWEI=100
MAX_PRIORITY_FEE_GWEI=35

# Simulation mode flag
SIMULATION_MODE=true
EOL

# Run simple command to watch for arbitrage opportunities
cargo run -- --simulation --aggressive --enable-jit --enable-arb