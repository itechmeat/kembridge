/**
 * Network management hook
 * Handles network switching and validation
 */

import { useCallback, useMemo } from "react";
import { useWallet } from "./useWallet";
import { NetworkInfo, NetworkType } from "../../services/wallet/types";
import {
  SUPPORTED_NETWORKS,
  isSupportedNetwork,
} from "../../services/wallet/utils";

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
  const { account, switchNetwork: walletSwitchNetwork } = useWallet();

  // Current network from account
  const currentNetwork = account?.network || null;

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
    async (network: NetworkInfo): Promise<void> => {
      await walletSwitchNetwork(network);
    },
    [walletSwitchNetwork]
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
