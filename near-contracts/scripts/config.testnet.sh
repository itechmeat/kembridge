#!/bin/bash

# KEMBridge NEAR Testnet Configuration
# Source this file: source config.testnet.sh

export NEAR_ENV=testnet
export NEAR_ACCOUNT_ID=kembridge.testnet
export NEAR_CONTRACT_ID=kembridge.testnet
export NEAR_NETWORK=testnet
export NEAR_RPC_URL=https://rpc.testnet.near.org
export NEAR_EXPLORER_URL=https://testnet.nearblocks.io

echo "✅ NEAR Testnet configuration loaded:"
echo "   Account: $NEAR_ACCOUNT_ID"
echo "   Network: $NEAR_ENV"
echo "   Contract: $NEAR_CONTRACT_ID" 