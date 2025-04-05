#!/bin/bash

# Start the JIT liquidity strategy in simulation mode for testing
echo "Starting Polygon JIT strategy in SIMULATION mode..."
echo "This will detect opportunities but not execute any real transactions"

# Copy the simulation environment file
cp .env.simulation .env

# Run in simulation mode
cargo run --release -- --aggressive --enable-jit --enable-arb --min-profit-usd 0.1 --max-gas-price-gwei 100 --simulation
