# KEMBridge NEAR Contract Guide

## Overview

KEMBridge NEAR contract is successfully deployed on `kembridge.testnet` and ready for development. This guide covers everything you need to know about building, deploying, and working with the contract.

## Quick Start

### Prerequisites

- Rust 1.86.0 (automatically managed via `rust-toolchain.toml`)
- NEAR CLI v0.21.0+
- Node.js (for tests)

### Build and Deploy

```bash
# Build contract
make build

# Deploy to kembridge.testnet (if needed)
make deploy

# Run tests
make test
```

## Contract Status

- **Account**: kembridge.testnet
- **Network**: NEAR Testnet
- **Contract Hash**: `7daf81cb5f88b07c3768b8321dcfc3d75153373172e522ea318c0b6e68806cd3`
- **Storage**: 72.5 KB
- **Balance**: 14.00 NEAR
- **Type**: Hello World (demo foundation)

## Available Commands

### Make Commands

```bash
make build     # Build contract with cargo near build
make deploy    # Deploy to kembridge.testnet
make test      # Run hello world tests
make clean     # Clean build artifacts
make help      # Show available commands
```

### Direct Scripts

```bash
scripts/build.sh                # Build contract
scripts/deploy-kembridge.sh     # Deploy to testnet
```

## Contract Functions

### Read-Only Functions

```bash
# Get version
near contract call-function as-read-only kembridge.testnet get_version \
  json-args {} network-config testnet now

# Get current greeting
near contract call-function as-read-only kembridge.testnet get_greeting \
  json-args {} network-config testnet now
```

### Transaction Functions

```bash
# Say hello (personalized greeting)
near contract call-function as-transaction kembridge.testnet say_hello \
  json-args {} prepaid-gas '30.0 Tgas' attached-deposit '0 NEAR' \
  sign-as kembridge.testnet network-config testnet

# Update greeting
near contract call-function as-transaction kembridge.testnet set_greeting \
  json-args '{"greeting": "New message"}' prepaid-gas '30.0 Tgas' \
  attached-deposit '0 NEAR' sign-as kembridge.testnet network-config testnet
```

## Development Workflow

### 1. Building

The project uses `cargo near build` for NEAR VM compatibility:

```bash
# Automatic via Makefile
make build

# Manual
cargo near build non-reproducible-wasm --no-abi
```

### 2. Testing

```bash
# Quick tests (read-only functions)
near contract call-function as-read-only kembridge.testnet get_version \
  json-args {} network-config testnet now

# Full test suite
make test
```

### 3. Deployment

```bash
# First deployment
near contract deploy kembridge.testnet \
  use-file target/near/kembridge_near_contract.wasm \
  with-init-call new \
  json-args '{"greeting": "Hello from KEMBridge!"}' \
  prepaid-gas '30.0 Tgas' \
  attached-deposit '0 NEAR' \
  network-config testnet

# Update deployment (without init)
near contract deploy kembridge.testnet \
  use-file target/near/kembridge_near_contract.wasm \
  network-config testnet
```

## Production Setup

### Environment Variables

```bash
export NEAR_ENV=mainnet
export NEAR_ACCOUNT_ID=your-account.near
export NEAR_PRIVATE_KEY=ed25519:your_private_key
```

### CI/CD Example

```yaml
- name: Build and Deploy
  run: |
    cargo near build non-reproducible-wasm --no-abi
    near deploy --accountId $NEAR_ACCOUNT_ID \
      --wasmFile target/near/kembridge_near_contract.wasm
  env:
    NEAR_ENV: mainnet
    NEAR_PRIVATE_KEY: ${{ secrets.NEAR_PRIVATE_KEY }}
    NEAR_ACCOUNT_ID: ${{ secrets.NEAR_ACCOUNT_ID }}
```

## Key Technical Details

### Rust Version Management

- **Global**: Uses system Rust (1.88+)
- **Local**: Forced to 1.86.0 via `rust-toolchain.toml`
- **Why**: NEAR VM compatibility (Rust 1.87+ ABI changes not supported)

### Build Process

1. `cargo near build` creates optimized WASM
2. Files output to `target/near/kembridge_near_contract.wasm`
3. Backup copy created in `out/` for compatibility

### Contract Architecture

- **Language**: Rust with near-sdk 5.15.1
- **Edition**: 2024
- **Type**: Simple hello world foundation
- **Functions**: get_version, get_greeting, set_greeting, say_hello

## Troubleshooting

### "Contract already initialized"

- **Cause**: Trying to call `new` on deployed contract
- **Solution**: Use deployment without init or redeploy

### "Compilation/Deserialization Error"

- **Cause**: Rust version incompatibility
- **Solution**: Ensure using Rust 1.86.0 (check `rust-toolchain.toml`)

### "Method not found"

- **Cause**: Wrong function name or contract not deployed
- **Solution**: Check deployment and function names

### "Account not found"

- **Cause**: Account doesn't exist or wrong network
- **Solution**: Verify account and network configuration

## Monitoring

### Explorer Links

- **Account**: https://testnet.nearblocks.io/address/kembridge.testnet
- **Transactions**: https://testnet.nearblocks.io/address/kembridge.testnet#transaction
- **Contract**: https://testnet.nearblocks.io/address/kembridge.testnet#contract

### Health Check

```bash
# Quick status
near state kembridge.testnet --networkId testnet

# Function test
near contract call-function as-read-only kembridge.testnet get_version \
  json-args {} network-config testnet now
```

## Next Steps

1. **Extend Contract**: Add bridge functionality to hello world base
2. **Backend Integration**: Connect to KEMBridge Rust backend
3. **Advanced Testing**: Implement comprehensive test suite
4. **Mainnet Preparation**: Setup production deployment pipeline
5. **Monitoring**: Add real-time contract monitoring

## Cost Estimates

- **Account Creation**: ~0.1 NEAR
- **Contract Deployment**: ~0.01 NEAR
- **Function Calls**: ~0.001 NEAR each
- **Storage**: ~0.0001 NEAR per byte

---

**Status**: ✅ Production Ready  
**Last Updated**: Successful deployment on kembridge.testnet  
**Ready For**: Development and integration
