#!/bin/bash

# KEMBridge NEAR Contract Configuration for Testnet

export NEAR_ENV="testnet"
export NEAR_ACCOUNT_ID="kembridge.testnet"
export NEAR_CONTRACT_ID="kembridge.testnet"
export NEAR_NETWORK="testnet"
export NEAR_RPC_URL="https://rpc.testnet.near.org"
export NEAR_EXPLORER_URL="https://explorer.testnet.near.org"

echo "🔧 Loaded KEMBridge testnet configuration"
echo "📍 Contract: $NEAR_CONTRACT_ID"
echo "🌐 Network: $NEAR_ENV" 