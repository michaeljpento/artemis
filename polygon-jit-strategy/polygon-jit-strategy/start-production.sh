#!/bin/bash

# Start the JIT liquidity strategy in production mode - USE ONLY WHEN WALLET IS FUNDED
echo "Starting Polygon JIT strategy in PRODUCTION mode (will execute real transactions)..."
echo "WARNING: This will use real funds. Make sure you have funded your wallet with MATIC and tokens."
echo "Press Ctrl+C now to cancel, or wait 5 seconds to continue..."
sleep 5

# Copy the production environment file
cp .env.production .env

# Run in production mode (no simulation flag)
cargo run --release -- --aggressive --enable-jit --enable-arb --min-profit-usd 1.0 --max-gas-price-gwei 100