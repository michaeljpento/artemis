#!/bin/bash

# Deployment script for Polygon JIT strategy to Hetzner Cloud
# Server details
SERVER_IP="YOUR_HETZNER_SERVER_IP"  # Replace with actual IP before running
SERVER_USER="root"
DEPLOY_DIR="/opt/polygon-jit-strategy"

# Check if server IP has been set
if [[ "$SERVER_IP" == "YOUR_HETZNER_SERVER_IP" ]]; then
    echo "ERROR: Please edit this script to set your actual Hetzner server IP"
    echo "Edit the SERVER_IP variable in this file and try again"
    exit 1
fi

# Check if tarball exists
if [[ ! -f "polygon-jit-strategy.tar.gz" ]]; then
    echo "Creating deployment package..."
    tar -czvf polygon-jit-strategy.tar.gz --exclude="./target" --exclude="./polygon-jit-strategy.tar.gz" .
fi

echo "Deploying Polygon JIT Strategy to Hetzner Cloud server at $SERVER_IP"
echo "WARNING: This will install software and create services on the remote server."
echo "Make sure you have SSH access to the server at $SERVER_IP with user $SERVER_USER."
echo "Press Ctrl+C to cancel or ENTER to continue..."
read

# Transfer the package to the server
echo "Transferring package to server..."
scp polygon-jit-strategy.tar.gz $SERVER_USER@$SERVER_IP:/tmp/ || {
    echo "ERROR: Failed to transfer files to the server."
    echo "Please check your SSH connection and server details."
    exit 1
}

# Connect to the server and set up everything
echo "Setting up the deployment on the server..."
ssh $SERVER_USER@$SERVER_IP << 'EOF' || {
    echo "ERROR: Failed to connect to server. Check your SSH connection and try again."
    exit 1
}
    set -e  # Exit on first error
    echo "🔍 Creating deployment directory..."
    mkdir -p /opt/polygon-jit-strategy

    echo "📦 Extracting the package..."
    tar -xzvf /tmp/polygon-jit-strategy.tar.gz -C /opt/polygon-jit-strategy
    
    echo "🔧 Making scripts executable..."
    chmod +x /opt/polygon-jit-strategy/*.sh
    
    echo "🔍 Checking for Rust installation..."
    if ! command -v rustc &> /dev/null; then
        echo "🔄 Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source $HOME/.cargo/env
    else
        echo "✅ Rust is already installed."
    fi
    
    echo "🔄 Installing required packages..."
    apt-get update
    apt-get install -y build-essential pkg-config libssl-dev supervisor
    
    echo "🔨 Building the project (this might take a few minutes)..."
    cd /opt/polygon-jit-strategy
    source $HOME/.cargo/env
    cargo build --release
    
    echo "📝 Creating systemd service..."
    cat > /etc/systemd/system/polygon-jit.service << 'SYSTEMD'
[Unit]
Description=Polygon JIT Liquidity Strategy
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/polygon-jit-strategy
ExecStart=/opt/polygon-jit-strategy/start-simulation.sh
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=polygon-jit
Environment="RUST_BACKTRACE=1"

[Install]
WantedBy=multi-user.target
SYSTEMD

    echo "🔄 Reloading systemd..."
    systemctl daemon-reload
    
    echo "🚀 Enabling and starting the service..."
    systemctl enable polygon-jit
    systemctl start polygon-jit
    
    echo "🔍 Checking service status..."
    systemctl status polygon-jit
    
    echo ""
    echo "✅ DEPLOYMENT COMPLETE! The strategy is running in simulation mode."
    echo ""
    echo "📋 USEFUL COMMANDS:"
    echo "  • View logs: journalctl -u polygon-jit -f"
    echo "  • Check status: systemctl status polygon-jit"
    echo "  • Restart service: systemctl restart polygon-jit"
    echo ""
    echo "⚠️  IMPORTANT: The strategy is running in SIMULATION mode."
    echo "   To switch to production mode when ready:"
    echo "   1. Edit: nano /etc/systemd/system/polygon-jit.service"
    echo "   2. Change 'start-simulation.sh' to 'start-production.sh'"
    echo "   3. Run: systemctl daemon-reload && systemctl restart polygon-jit"
    echo ""
    echo "💰 Remember to fund your wallet with MATIC before switching to production mode!"
EOF

echo "Deployment script completed."
echo ""
echo "========== DEPLOYMENT SUMMARY =========="
echo "✅ The Polygon JIT strategy has been deployed to your Hetzner server"
echo "✅ The strategy is running in SIMULATION MODE"
echo "✅ No real transactions will be executed until you switch to production mode"
echo ""
echo "When ready to go live:"
echo "1. SSH to your server: ssh $SERVER_USER@$SERVER_IP"
echo "2. Check logs to verify simulation is working: journalctl -u polygon-jit -f"
echo "3. Fund your wallet with MATIC: $(grep PRIVATE_KEY /opt/polygon-jit-strategy/.env.production | cut -d= -f2)"
echo "4. Switch to production: edit /etc/systemd/system/polygon-jit.service"
echo "5. Change start-simulation.sh to start-production.sh"
echo "6. Reload and restart: systemctl daemon-reload && systemctl restart polygon-jit"
echo ""
echo "❗ IMPORTANT: Always verify simulation works correctly before switching to production mode"
echo "===========================================""