require("@nomicfoundation/hardhat-toolbox");
require("@nomicfoundation/hardhat-verify");
require("dotenv").config();

// Try both basic auth and direct API key approaches
const INFURA_API_KEY = process.env.INFURA_API_KEY;
const INFURA_API_SECRET = process.env.INFURA_API_SECRET;

// First try the environment variable directly
let SEPOLIA_RPC_URL = process.env.ETHEREUM_RPC_URL;

// If that's not available, try to construct from API key
if (!SEPOLIA_RPC_URL && INFURA_API_KEY) {
  SEPOLIA_RPC_URL = `https://sepolia.infura.io/v3/${INFURA_API_KEY}`;
}

// Fallback to basic auth if needed
if (!SEPOLIA_RPC_URL && INFURA_API_KEY && INFURA_API_SECRET) {
  SEPOLIA_RPC_URL = `https://:${INFURA_API_SECRET}@sepolia.infura.io/v3/${INFURA_API_KEY}`;
}

const PRIVATE_KEY = process.env.PRIVATE_KEY || "0x" + "0".repeat(64); // Placeholder

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: {
    version: "0.8.28",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,
      },
    },
  },
  networks: {
    sepolia: {
      url: SEPOLIA_RPC_URL,
      accounts: PRIVATE_KEY !== "0x" + "0".repeat(64) ? [PRIVATE_KEY] : [],
      chainId: 11155111,
      gas: 6000000,
      gasPrice: 20000000000, // 20 gwei
    },
    localhost: {
      url: "http://127.0.0.1:8545",
      chainId: 31337,
    },
  },
  paths: {
    sources: "./contracts",
    tests: "./test",
    cache: "./cache",
    artifacts: "./artifacts",
  },
  etherscan: {
    apiKey: process.env.ETHERSCAN_API_KEY,
  },
  sourcify: {
    enabled: true,
  },
};
