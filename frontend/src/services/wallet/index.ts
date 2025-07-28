/**
 * Wallet service entry point
 * Exports all wallet-related functionality
 */

// Types
export * from "./types";

// Utils
export * from "./utils";

// Manager
export { WalletManager } from "./manager";

// Providers
export { MetaMaskProvider } from "./providers/metamask";
export { NearProvider } from "./providers/near";
export { WalletConnectProvider } from "./providers/walletconnect";

// Wallet service initialization
import { WalletManager } from "./manager";
import { MetaMaskProvider } from "./providers/metamask";
import { NearProvider } from "./providers/near";
import { WalletConnectProvider } from "./providers/walletconnect";

let globalWalletManager: WalletManager | null = null;

/**
 * Initializes the global wallet service
 */
export const initializeWalletService = (): WalletManager => {
  if (!globalWalletManager) {
    console.log("🚀 Wallet Service: Initializing wallet service...");
    globalWalletManager = new WalletManager({
      autoConnect: true,
      persistConnection: true,
    });

    console.log("📦 Wallet Service: Registering wallet providers...");
    // Register providers
    globalWalletManager.registerProvider(new MetaMaskProvider());
    console.log("✅ Wallet Service: MetaMask provider registered");

    console.log("📦 Wallet Service: Registering NEAR provider...");
    globalWalletManager.registerProvider(new NearProvider());
    console.log("✅ Wallet Service: NEAR provider registered");

    globalWalletManager.registerProvider(new WalletConnectProvider());

    // TODO (feat): Add Coinbase provider
    // globalWalletManager.registerProvider(new CoinbaseProvider());
  }

  return globalWalletManager;
};

/**
 * Gets the global wallet manager instance
 */
export const getWalletManager = (): WalletManager => {
  if (!globalWalletManager) {
    return initializeWalletService();
  }
  return globalWalletManager;
};

/**
 * Destroys the global wallet service
 */
export const destroyWalletService = (): void => {
  if (globalWalletManager) {
    globalWalletManager.destroy();
    globalWalletManager = null;
  }
};
