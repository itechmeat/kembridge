/**
 * Wagmi Configuration
 * Sets up Web3 wallet connections for Ethereum-based wallets
 */

import { createConfig, http } from "wagmi";
import { mainnet, sepolia } from "wagmi/chains";
import { injected, walletConnect, coinbaseWallet } from "wagmi/connectors";
import { appConfig } from "./env";

export const config = createConfig({
  chains: [mainnet, sepolia],
  connectors: [
    injected(),
    walletConnect({
      projectId: appConfig.wallet.walletConnectProjectId,
      metadata: {
        name: appConfig.app.name,
        description: appConfig.app.description,
        url: appConfig.app.url,
        icons: [appConfig.app.iconUrl],
      },
    }),
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
