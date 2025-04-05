#!/bin/bash

# Start the JIT liquidity strategy with monitoring
echo "Starting Polygon JIT strategy with monitoring..."

# Copy the production environment file
cp .env.production .env

# Run with monitoring on port 9090
echo "Metrics dashboard will be available at http://localhost:9090/dashboard"
echo "Press Ctrl+C to stop the strategy"
echo ""

cargo run --release -- --aggressive --enable-jit --enable-arb --metrics-port 9090