/**
 * Network management hook
 * Handles network switching and validation
 */

import { useCallback, useMemo } from "react";
import { useWallet } from "./useWallet";

export enum NetworkType {
  ETHEREUM = "ethereum",
  NEAR = "near",
}

export interface NetworkInfo {
  name: string;
  type: NetworkType;
  chainId: number | string;
  rpcUrl?: string;
  blockExplorer?: string;
}

const SUPPORTED_NETWORKS: Record<string, NetworkInfo> = {
  "1": {
    name: "Ethereum Mainnet",
    type: NetworkType.ETHEREUM,
    chainId: 1,
  },
  "11155111": {
    name: "Sepolia Testnet", 
    type: NetworkType.ETHEREUM,
    chainId: 11155111,
  },
  "137": {
    name: "Polygon",
    type: NetworkType.ETHEREUM, 
    chainId: 137,
  },
  "near": {
    name: "NEAR Protocol",
    type: NetworkType.NEAR,
    chainId: "near",
  },
};

const isSupportedNetwork = (chainId: string | number): boolean => {
  return !!SUPPORTED_NETWORKS[chainId.toString()];
};

export interface UseNetworkReturn {
  currentNetwork: NetworkInfo | null;
  supportedNetworks: NetworkInfo[];
  isSupported: boolean;
  isEthereumNetwork: boolean;
  isNearNetwork: boolean;
  switchNetwork: (network: NetworkInfo) => Promise<void>;
  getSupportedNetworks: (type?: NetworkType) => NetworkInfo[];
  validateNetwork: (chainId: string | number) => boolean;
}

export const useNetwork = (): UseNetworkReturn => {
  const { account, state } = useWallet();

  // Current network from account or state
  const currentNetwork = useMemo(() => {
    if (account?.network) {
      return account.network as NetworkInfo;
    }
    
    if (state.walletType === "near") {
      return SUPPORTED_NETWORKS["near"];
    }
    
    if (state.chainId) {
      return SUPPORTED_NETWORKS[state.chainId.toString()] || null;
    }
    
    return null;
  }, [account, state]);

  // All supported networks
  const supportedNetworks = useMemo(() => {
    return Object.values(SUPPORTED_NETWORKS);
  }, []);

  // Check if current network is supported
  const isSupported = useMemo(() => {
    if (!currentNetwork) return false;
    return isSupportedNetwork(currentNetwork.chainId);
  }, [currentNetwork]);

  // Check network type
  const isEthereumNetwork = useMemo(() => {
    return currentNetwork?.type === NetworkType.ETHEREUM;
  }, [currentNetwork]);

  const isNearNetwork = useMemo(() => {
    return currentNetwork?.type === NetworkType.NEAR;
  }, [currentNetwork]);

  // Switch network
  const switchNetwork = useCallback(
    async (): Promise<void> => {
      // TODO: Implement network switching
      console.warn("Network switching not yet implemented in new useWallet");
      throw new Error("Network switching not implemented");
    },
    []
  );

  // Get networks filtered by type
  const getSupportedNetworks = useCallback(
    (type?: NetworkType): NetworkInfo[] => {
      if (!type) {
        return supportedNetworks;
      }
      return supportedNetworks.filter((network) => network.type === type);
    },
    [supportedNetworks]
  );

  // Validate if a chain ID is supported
  const validateNetwork = useCallback((chainId: string | number): boolean => {
    return isSupportedNetwork(chainId);
  }, []);

  return {
    currentNetwork,
    supportedNetworks,
    isSupported,
    isEthereumNetwork,
    isNearNetwork,
    switchNetwork,
    getSupportedNetworks,
    validateNetwork,
  };
};
