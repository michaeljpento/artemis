#!/bin/bash
# Run the JIT strategy with monitoring enabled

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed. Please install Rust and Cargo first."
    exit 1
fi

# Build the project if needed
echo "Building the project..."
cd "$(dirname "$0")/.." || exit 1
cargo build --release || exit 1

# Modify port if specified
PORT="${1:-9090}"

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "Creating sample .env file (please edit with your actual values)..."
    cat <<EOF > .env
POLYGON_WS_URL=wss://polygon-mainnet.g.alchemy.com/v2/YOUR_API_KEY
PRIVATE_KEY=YOUR_PRIVATE_KEY
JIT_LIQUIDITY_PROVIDER=0x0000000000000000000000000000000000000000
FLASH_ARB_EXECUTOR=0x0000000000000000000000000000000000000000
EOF
    echo "Please edit .env file with your actual values before running again."
    exit 1
fi

# Run the strategy with monitoring enabled
echo "Starting JIT strategy with monitoring on port $PORT..."
cargo run --release -- --metrics-port "$PORT" "$@"