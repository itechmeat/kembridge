/**
 * Wagmi Configuration
 * Sets up Web3 wallet connections for Ethereum-based wallets
 */

import { createConfig, http } from "wagmi";
import { mainnet, sepolia } from "wagmi/chains";
import { injected, walletConnect, coinbaseWallet } from "wagmi/connectors";
import { createStorage } from "wagmi";
import { appConfig } from "./env";

export const config = createConfig({
  chains: [mainnet, sepolia],
  storage: createStorage({
    storage: typeof window !== "undefined" ? window.localStorage : undefined,
    key: "kembridge-wagmi", // Namespace wagmi storage to avoid conflicts
  }),
  connectors: [
    injected(),
    // Only enable WalletConnect if project ID is configured
    ...(appConfig.wallet.walletConnectProjectId
      ? [
          walletConnect({
            projectId: appConfig.wallet.walletConnectProjectId,
            metadata: {
              name: appConfig.app.name,
              description: appConfig.app.description,
              url: appConfig.app.url,
              icons: [appConfig.app.iconUrl],
            },
          }),
        ]
      : []),
    coinbaseWallet({
      appName: appConfig.app.name,
      appLogoUrl: appConfig.app.iconUrl,
    }),
  ],
  transports: {
    [mainnet.id]: http(
      appConfig.blockchain.ethereum.rpcUrl || "https://eth.llamarpc.com"
    ),
    [sepolia.id]: http(
      appConfig.blockchain.sepolia.rpcUrl ||
        "https://ethereum-sepolia.publicnode.com"
    ),
  },
});
