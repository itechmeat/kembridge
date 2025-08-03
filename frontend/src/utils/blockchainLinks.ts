// Blockchain explorer links utility

export type ChainType = "ethereum" | "near";

export interface ExplorerLink {
  url: string;
  name: string;
}

/**
 * Blockchain explorer configurations
 */
const EXPLORERS = {
  ethereum: {
    mainnet: {
      etherscan: "https://etherscan.io",
      blockscout: "https://eth.blockscout.com",
    },
    sepolia: {
      etherscan: "https://sepolia.etherscan.io", 
      blockscout: "https://eth-sepolia.blockscout.com",
    },
  },
  near: {
    mainnet: {
      nearblocks: "https://nearblocks.io",
      explorer: "https://explorer.near.org",
    },
    testnet: {
      nearblocks: "https://testnet.nearblocks.io",
      explorer: "https://explorer.testnet.near.org",
    },
  },
} as const;

/**
 * Get current network based on environment
 */
function getCurrentNetwork(): "mainnet" | "sepolia" {
  // For development, use testnets
  if (import.meta.env.DEV) {
    return "sepolia"; // For Ethereum testnet
  }
  
  // TODO: Get from environment or chain detection
  return "sepolia";
}

/**
 * Get current NEAR network
 */
function getCurrentNearNetwork(): "mainnet" | "testnet" {
  if (import.meta.env.DEV) {
    return "testnet";
  }
  
  // TODO: Get from environment or chain detection  
  return "testnet";
}

/**
 * Generate transaction link for Ethereum
 */
export function getEthereumTransactionLink(txHash: string): ExplorerLink[] {
  const network = getCurrentNetwork();
  const explorers = EXPLORERS.ethereum[network];
  
  return [
    {
      name: "Etherscan",
      url: `${explorers.etherscan}/tx/${txHash}`,
    },
    {
      name: "Blockscout", 
      url: `${explorers.blockscout}/tx/${txHash}`,
    },
  ];
}

/**
 * Generate transaction link for NEAR
 */
export function getNearTransactionLink(txHash: string): ExplorerLink[] {
  const network = getCurrentNearNetwork();
  const explorers = EXPLORERS.near[network];
  
  return [
    {
      name: "NEAR Blocks",
      url: `${explorers.nearblocks}/txns/${txHash}`,
    },
    {
      name: "NEAR Explorer",
      url: `${explorers.explorer}/transactions/${txHash}`,
    },
  ];
}

/**
 * Generate transaction links for any chain
 */
export function getTransactionLinks(
  chain: ChainType,
  txHash: string
): ExplorerLink[] {
  if (!txHash) {
    return [];
  }
  
  switch (chain) {
    case "ethereum":
      return getEthereumTransactionLink(txHash);
    case "near":
      return getNearTransactionLink(txHash);
    default:
      return [];
  }
}

/**
 * Generate address link for Ethereum
 */
export function getEthereumAddressLink(address: string): ExplorerLink[] {
  const network = getCurrentNetwork();
  const explorers = EXPLORERS.ethereum[network];
  
  return [
    {
      name: "Etherscan",
      url: `${explorers.etherscan}/address/${address}`,
    },
    {
      name: "Blockscout",
      url: `${explorers.blockscout}/address/${address}`,
    },
  ];
}

/**
 * Generate address link for NEAR
 */
export function getNearAddressLink(address: string): ExplorerLink[] {
  const network = getCurrentNearNetwork();
  const explorers = EXPLORERS.near[network];
  
  return [
    {
      name: "NEAR Blocks",
      url: `${explorers.nearblocks}/address/${address}`,
    },
    {
      name: "NEAR Explorer", 
      url: `${explorers.explorer}/accounts/${address}`,
    },
  ];
}

/**
 * Generate address links for any chain
 */
export function getAddressLinks(
  chain: ChainType,
  address: string
): ExplorerLink[] {
  if (!address) {
    return [];
  }
  
  switch (chain) {
    case "ethereum":
      return getEthereumAddressLink(address);
    case "near":
      return getNearAddressLink(address);
    default:
      return [];
  }
}

/**
 * Get primary transaction link (first explorer)
 */
export function getPrimaryTransactionLink(
  chain: ChainType,
  txHash: string
): ExplorerLink | null {
  const links = getTransactionLinks(chain, txHash);
  return links.length > 0 ? links[0] : null;
}

/**
 * Get primary address link (first explorer)
 */
export function getPrimaryAddressLink(
  chain: ChainType,
  address: string
): ExplorerLink | null {
  const links = getAddressLinks(chain, address);
  return links.length > 0 ? links[0] : null;
}