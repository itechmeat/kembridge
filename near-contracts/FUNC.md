# KEMBridge NEAR Contract

## Overview

This is the NEAR Protocol smart contract for KEMBridge cross-chain bridge. It enables secure token transfers between NEAR and Ethereum networks with quantum-protected operations.

## Requirements

- **Rust Version**: 1.86.0 (specified in `rust-toolchain.toml`)
  - The project uses `rust-toolchain.toml` to ensure compatibility with NEAR VM
  - Rust 1.87+ has WebAssembly C ABI changes that are not yet supported by NEAR
  - Global Rust version can be different, but this project will use 1.86.0 locally

## Features

- **Token Locking**: Lock NEAR tokens for bridge transfer to Ethereum
- **Token Unlocking**: Unlock NEAR tokens from Ethereum bridge proof
- **Token Minting**: Mint wrapped tokens from Ethereum deposits
- **Token Burning**: Burn wrapped tokens for bridge back to Ethereum
- **Bridge Statistics**: View total locked/unlocked amounts
- **Quantum Integration**: Support for quantum-protected operations
- **Event Logging**: Comprehensive bridge event emissions
- **Replay Protection**: Prevent double-spending attacks
- **Emergency Functions**: Owner can pause/unpause and emergency withdraw

## Contract Methods

### Bridge Operations

- `lock_tokens(eth_recipient, quantum_hash)` - Lock NEAR tokens for Ethereum transfer
- `unlock_tokens(amount, near_recipient, eth_tx_hash, quantum_hash)` - Unlock tokens from Ethereum
- `mint_tokens(recipient, amount, eth_tx_hash, quantum_hash)` - Mint wrapped tokens
- `burn_tokens(amount, eth_recipient, quantum_hash)` - Burn wrapped tokens

### View Methods

- `get_bridge_stats()` - Get total locked/unlocked amounts
- `get_locked_balance(account)` - Get user's locked balance
- `get_transaction(tx_id)` - Get transaction details
- `get_config()` - Get contract configuration
- `is_eth_tx_processed(eth_tx_hash)` - Check if Ethereum transaction was processed

### Owner Methods

- `update_config(min_amount, max_amount, fee)` - Update bridge configuration
- `set_paused(paused)` - Pause/unpause contract
- `emergency_withdraw(amount)` - Emergency withdrawal
- `transfer_ownership(new_owner)` - Transfer contract ownership

## Configuration

- **Minimum Bridge Amount**: 0.1 NEAR
- **Maximum Bridge Amount**: 10 NEAR
- **Bridge Fee**: 0.5% (50 basis points)
- **Network**: NEAR Testnet (for demo)

## Build & Deploy

### Prerequisites

1. Install Rust and Cargo
2. Install NEAR CLI: `npm install -g near-cli`
3. Add WASM target: `rustup target add wasm32-unknown-unknown`

### Build Contract

```bash
./build.sh
```

This will compile the contract to `out/kembridge_near_contract.wasm`.

### Deploy Contract

1. Login to NEAR CLI:

   ```bash
   near login
   ```

2. Deploy and initialize:
   ```bash
   ./deploy.sh
   ```

### Manual Deployment

```bash
# Build contract
cargo build --target wasm32-unknown-unknown --release

# Deploy to testnet
near deploy --accountId kembridge-demo.testnet --wasmFile out/kembridge_near_contract.wasm

# Initialize contract
near call kembridge-demo.testnet new '{"owner": "kembridge-demo.testnet"}' --accountId kembridge-demo.testnet
```

## Usage Examples

### Lock NEAR Tokens

```bash
near call kembridge-demo.testnet lock_tokens \
  '{"eth_recipient": "0x742d35Cc6634C0532925a3b8D295759d7816d1aB", "quantum_hash": "qhash123"}' \
  --accountId user.testnet \
  --amount 1 \
  --networkId testnet
```

### Check Bridge Stats

```bash
near call kembridge-demo.testnet get_bridge_stats \
  --accountId user.testnet \
  --networkId testnet
```

### Get User Balance

```bash
near call kembridge-demo.testnet get_locked_balance \
  '{"account": "user.testnet"}' \
  --accountId user.testnet \
  --networkId testnet
```

### Owner Operations

```bash
# Update configuration
near call kembridge-demo.testnet update_config \
  '{"min_bridge_amount": "100000000000000000000000", "bridge_fee_bp": 30}' \
  --accountId kembridge-demo.testnet \
  --networkId testnet

# Pause contract
near call kembridge-demo.testnet set_paused \
  '{"paused": true}' \
  --accountId kembridge-demo.testnet \
  --networkId testnet
```

## Integration with KEMBridge Backend

The contract integrates with the KEMBridge Rust backend through:

1. **Event Listeners**: Monitor bridge events for cross-chain operations
2. **Transaction Calls**: Execute bridge operations via NEAR RPC
3. **State Queries**: Check contract state and balances
4. **Quantum Protection**: Use quantum hashes for additional security

## Security Features

- **Replay Protection**: Ethereum transaction hashes are tracked to prevent double-spending
- **Amount Limits**: Minimum and maximum bridge amounts to prevent abuse
- **Owner Controls**: Emergency pause and withdrawal capabilities
- **Fee System**: Configurable bridge fees for sustainability
- **Quantum Integration**: Support for quantum-protected operations

## Development

### Project Structure

```
near-contracts/
├── src/
│   └── lib.rs              # Main contract implementation
├── Cargo.toml              # Project dependencies
├── build.sh                # Build script
├── deploy.sh               # Deploy script
├── README.md               # This file
└── out/                    # Compiled WASM output
```

### Testing

```bash
# Run unit tests
cargo test

# Test deployed contract
near call kembridge-demo.testnet get_config --accountId test.testnet --networkId testnet
```

## Current Status

**Status**: ✅ Demo Contract Ready  
**Network**: NEAR Testnet  
**Contract Account**: kembridge-demo.testnet (to be created)  
**Integration**: Ready for KEMBridge backend integration

## Future Enhancements

- Full fungible token standard (NEP-141) integration
- Advanced proof verification for Ethereum transactions
- Multi-signature support for admin operations
- Cross-contract calls for complex bridge operations
- Integration with NEAR Chain Signatures for Ethereum operations
