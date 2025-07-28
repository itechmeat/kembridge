#!/bin/bash

# KEMBridge NEAR Contract Deploy Script for kembridge.testnet
# This script deploys the compiled contract to kembridge.testnet account

set -e

# Load configuration
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
source "$SCRIPT_DIR/config.testnet.sh"

# Configuration
NETWORK="testnet"
ACCOUNT_ID="kembridge.testnet"
CONTRACT_WASM="target/near/kembridge_near_contract.wasm"

echo "🚀 Deploying KEMBridge NEAR contract to kembridge.testnet..."

# Check if near-cli is installed
if ! command -v near &> /dev/null; then
    echo "❌ near-cli is not installed. Please install it first:"
    echo "npm install -g near-cli"
    exit 1
fi

# Check if contract WASM exists
if [ ! -f "$CONTRACT_WASM" ]; then
    echo "❌ Contract WASM not found. Please build the contract first:"
    echo "./build.sh"
    exit 1
fi

# Check account state
echo "🔑 Checking account status..."
near state "$ACCOUNT_ID" --networkId "$NETWORK"

# Deploy contract with initialization
echo "📡 Deploying and initializing contract..."
near contract deploy "$ACCOUNT_ID" \
  use-file "$CONTRACT_WASM" \
  with-init-call new \
  json-args '{"owner": "kembridge.testnet"}' \
  prepaid-gas '30.0 Tgas' \
  attached-deposit '0 NEAR' \
  network-config testnet

# Verify deployment
echo "✅ Verifying deployment..."
near contract call-function as-read-only "$ACCOUNT_ID" get_owner \
  json-args {} network-config testnet now

echo ""
echo "📊 Contract deployed successfully!"
echo "🌐 Network: $NETWORK"
echo "🏠 Contract Account: $ACCOUNT_ID"
echo "📄 Contract WASM: $CONTRACT_WASM"
echo "🔗 Explorer: https://testnet.nearblocks.io/address/$ACCOUNT_ID"

# Test basic functionality
echo ""
echo "🧪 Testing basic functionality..."

echo "📋 Getting configuration..."
near contract call-function as-read-only "$ACCOUNT_ID" get_config \
  json-args {} network-config testnet now

echo "📊 Getting bridge stats..."
near contract call-function as-read-only "$ACCOUNT_ID" get_bridge_stats \
  json-args {} network-config testnet now

echo ""
echo "✅ Deployment and testing complete!"
echo ""
echo "🎯 Next steps:"
echo "1. Run full tests: node test_after_deploy.js"
echo "2. Try demo functions: node demo_test.js"
echo "3. Test lock tokens (requires NEAR balance)"
echo ""
echo "💡 Useful commands:"
echo "# Check contract owner"
echo "near contract call-function as-read-only $ACCOUNT_ID get_owner json-args {} network-config testnet now"
echo ""
echo "# Lock tokens (requires balance)"
echo "near contract call-function as-transaction $ACCOUNT_ID lock_tokens json-args '{\"eth_recipient\": \"0x742d35Cc6634C0532925a3b8D295759d7816d1aB\", \"quantum_hash\": \"demo_hash\"}' prepaid-gas '30.0 Tgas' attached-deposit '1 NEAR' sign-as $ACCOUNT_ID network-config testnet send" 