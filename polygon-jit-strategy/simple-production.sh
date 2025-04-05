#!/bin/bash

# Simple script to run a JIT liquidity bot for production
echo "===== IMPORTANT NOTICE ====="
echo "This script will run the JIT liquidity bot in PRODUCTION mode."
echo "It WILL execute real transactions using your funds."
echo "Make sure your wallet is properly funded with MATIC and the required tokens."
echo "=========================="
echo ""
echo "Starting in 5 seconds... Press Ctrl+C to cancel"
sleep 5

# Set your own Alchemy API key here
API_KEY="YOUR_ALCHEMY_API_KEY"

# Set your own private key here
PRIVATE_KEY="YOUR_PRIVATE_KEY"

# Create a temporary .env file with the required variables
cat > .env <<EOL
# Polygon RPC URLs
POLYGON_WS_URL=wss://polygon-mainnet.g.alchemy.com/v2/${API_KEY}
POLYGON_HTTP_URL=https://polygon-mainnet.g.alchemy.com/v2/${API_KEY}

# Wallet configuration - Replace with your actual private key
PRIVATE_KEY=${PRIVATE_KEY}

# Contract addresses
FLASH_ARB_EXECUTOR=0xb066B4d93Ae5205154e6F1B19Af9Beb7c66268d6
JIT_LIQUIDITY_PROVIDER=0x8D85e4359243469A929a8Db74D10ffE936bd7d0D

# Strategy configuration
MIN_PROFIT_THRESHOLD_USD=1.0
MAX_GAS_PRICE_GWEI=100
MAX_PRIORITY_FEE_GWEI=35

# Simulation mode flag
SIMULATION_MODE=false
EOL

# Run in production mode
cargo run --release -- --aggressive --enable-jit --enable-arb --min-profit-usd 1.0 --max-gas-price-gwei 100