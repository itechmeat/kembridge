# KEMBridge Smart Contracts

## Setup

1. Copy `.env.example` to `.env` and fill in your values:

   ```bash
   cp .env.example .env
   ```

2. Install dependencies:

   ```bash
   npm install
   ```

3. Compile contracts:
   ```bash
   npm run compile
   ```

## Deployment

### Deploy to Sepolia Testnet

1. Make sure you have ETH in your wallet on Sepolia testnet

2. Set your environment variables in `.env`:

   - `ETHEREUM_RPC_URL`: Your Infura/Alchemy Sepolia endpoint
   - `INFURA_API_KEY`: Your Infura project ID
   - `INFURA_API_SECRET`: Your Infura project secret (optional)
   - `PRIVATE_KEY`: Your deployment wallet private key (without 0x)
   - `ETHERSCAN_API_KEY`: For contract verification
   - `BRIDGE_CONTRACT_ADDRESS`: Deployed contract address (set after deployment)
   - `BRIDGE_CONTRACT_DEPLOYED_BLOCK`: Block number when contract was deployed

3. **Important**: In your Infura project settings, make sure "Require API Key Secret for all requests" is **DISABLED** for Hardhat deployment to work.

4. Deploy the contract:

   ```bash
   npm run deploy:sepolia
   ```

5. Copy the contract address from the output and update both `.env` and `backend/src/constants.rs`

### Local Testing

```bash
# Start local hardhat node
npx hardhat node

# Deploy to local network
npm run deploy:localhost
```

## Contract Features

- **Lock ETH**: Users can lock ETH for cross-chain transfer
- **Unlock ETH**: Owner can unlock ETH from other chains
- **Bridge Stats**: View total locked/unlocked amounts
- **Emergency Functions**: Owner can withdraw funds if needed

## Current Deployment

**Contract Address**: `0x52a1659A86287a10E228e1793a23604C0201d356`  
**Network**: Sepolia Testnet  
**Block Number**: 8789871  
**Explorer**: https://sepolia.etherscan.io/address/0x52a1659A86287a10E228e1793a23604C0201d356

**Current Status**: ✅ Deployed and funded (0.01 ETH)

## Security

- Minimum lock amount: 0.001 ETH
- Maximum lock amount: 10 ETH
- Only owner can unlock tokens
- Replay protection via processed hashes

## Integration

After deployment, update these files:

1. `contracts/.env`:

   ```env
   BRIDGE_CONTRACT_ADDRESS=YOUR_DEPLOYED_CONTRACT_ADDRESS
   BRIDGE_CONTRACT_DEPLOYED_BLOCK=YOUR_DEPLOYMENT_BLOCK_NUMBER
   ```

2. `backend/src/constants.rs`:

   ```rust
   // Helper functions to get values from environment (REQUIRED)
   pub fn get_bridge_contract_address() -> Result<String, std::env::VarError> {
       std::env::var("BRIDGE_CONTRACT_ADDRESS")
   }

   pub fn get_bridge_contract_deployed_block() -> Result<u64, Box<dyn std::error::Error>> {
       let block_str = std::env::var("BRIDGE_CONTRACT_DEPLOYED_BLOCK")?;
       let block_num = block_str.parse::<u64>()?;
       Ok(block_num)
   }
   ```

3. The contract ABI is already included in `backend/crates/kembridge-blockchain/src/ethereum/bridge_abi.rs`

## Testing

### Check Status

Check wallet balance and test deployed contract functions:

```bash
npm run check-status
```

This command will:
- Check your wallet balance and deployment readiness
- Test all deployed contract functions
- Display bridge statistics and contract status
- Verify contract functionality

Test the deployed contract with Rust:

```bash
cd ../backend
cargo test test_real_bridge_network -- --ignored
```

### Available Tests

- **JavaScript Integration**: `test-contract-integration.js`
- **Rust Integration**: `backend/tests/test_real_bridge_network.rs`
- **Bridge Integration**: `backend/tests/test_bridge_integration.rs`

## Project Structure

```
contracts/
├── contracts/
│   └── SimpleBridge.sol          # Main bridge contract
├── scripts/
│   ├── deploy.js                 # Hardhat deployment script
│   └── check-status.js           # Check wallet & test contract status
├── test-contract-integration.js  # JavaScript integration test
├── hardhat.config.js            # Hardhat configuration
├── .env                         # Environment variables
├── .env.example                 # Environment template
└── README.md                    # This file
```

## Environment Variables

The `.env` file contains all necessary configuration:

```env
ETHEREUM_RPC_URL=https://sepolia.infura.io/v3/YOUR_PROJECT_ID
INFURA_API_SECRET=YOUR_INFURA_API_SECRET
INFURA_API_KEY=YOUR_INFURA_API_KEY
ETHERSCAN_API_KEY=YOUR_ETHERSCAN_API_KEY
PRIVATE_KEY=YOUR_PRIVATE_KEY_WITHOUT_0x
BRIDGE_CONTRACT_ADDRESS=YOUR_DEPLOYED_CONTRACT_ADDRESS
BRIDGE_CONTRACT_DEPLOYED_BLOCK=YOUR_DEPLOYMENT_BLOCK_NUMBER
```

**Note**: These are testnet keys for development only. Never use mainnet keys in version control.
