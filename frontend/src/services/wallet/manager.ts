/**
 * Wallet Manager - Main orchestrator for wallet operations
 * Handles multiple wallet providers and state management
 */

import {
  WalletType,
  WalletState,
  WalletProvider,
  WalletAccount,
  WalletError,
  WalletErrorCode,
  WalletEvent,
  WalletManagerConfig,
  NetworkInfo,
} from "./types";
import {
  createWalletError,
  saveLastConnectedWallet,
  getLastConnectedWallet,
  clearWalletStorage,
  SUPPORTED_NETWORKS,
  DEFAULT_NETWORK,
} from "./utils";

export class WalletManager {
  private providers: Map<WalletType, WalletProvider> = new Map();
  private currentState: WalletState;
  private eventListeners: Map<string, Set<(state: WalletState) => void>> =
    new Map();
  private config: WalletManagerConfig;

  constructor(config?: Partial<WalletManagerConfig>) {
    this.config = {
      supportedNetworks: Object.values(SUPPORTED_NETWORKS),
      defaultNetwork: DEFAULT_NETWORK,
      autoConnect: true,
      persistConnection: true,
      ...config,
    };

    this.currentState = {
      type: null,
      isConnecting: false,
      isConnected: false,
      account: null,
      error: null,
      lastConnected: getLastConnectedWallet() || undefined,
    };
  }

  /**
   * Registers a wallet provider
   */
  registerProvider(provider: WalletProvider): void {
    this.providers.set(provider.type, provider);

    // Set up provider event listeners
    provider.on(WalletEvent.ACCOUNT_CHANGED, (data: unknown) =>
      this.handleAccountChanged(data as WalletAccount | null)
    );
    provider.on(WalletEvent.NETWORK_CHANGED, (data: unknown) =>
      this.handleNetworkChanged(data as NetworkInfo)
    );
    provider.on(WalletEvent.DISCONNECTED, () => this.handleDisconnected());
    provider.on(WalletEvent.ERROR, (data: unknown) =>
      this.handleError(data as WalletError)
    );
  }

  /**
   * Gets all available wallet providers
   */
  getAvailableProviders(): WalletProvider[] {
    console.log("📋 WalletManager: Getting available providers");
    const providers = Array.from(this.providers.values()).filter(
      (provider) => provider.isAvailable
    );
    console.log(
      `📊 WalletManager: Found ${providers.length} available providers`
    );
    return providers;
  }

  /**
   * Gets all wallet providers (including unavailable ones)
   */
  getAllProviders(): WalletProvider[] {
    console.log("📋 WalletManager: Getting all providers");
    const providers = Array.from(this.providers.values());
    // Debug providers in development only
    if (import.meta.env.DEV) {
      providers.forEach((p) => {
        console.log(
          `📊 WalletManager: Provider ${p.type}: isInstalled=${p.isInstalled}, isAvailable=${p.isAvailable}`
        );
      });
    }
    return providers;
  }

  /**
   * Gets current wallet state
   */
  getState(): WalletState {
    return { ...this.currentState };
  }

  /**
   * Connects to a specific wallet
   */
  async connect(walletType: WalletType): Promise<WalletAccount> {
    try {
      console.log(`🔗 WalletManager: Connecting to ${walletType}...`);
      this.updateState({
        isConnecting: true,
        error: null,
      });

      console.log(`🔍 WalletManager: Looking for provider ${walletType}`);
      const provider = this.providers.get(walletType);
      if (!provider) {
        console.error(`❌ WalletManager: Provider ${walletType} not found`);
        throw createWalletError(
          WalletErrorCode.WALLET_NOT_FOUND,
          `Wallet provider ${walletType} not found`
        );
      }

      console.log(
        `📊 WalletManager: Provider ${walletType} found, isAvailable=${provider.isAvailable}`
      );
      if (!provider.isAvailable) {
        console.error(
          `❌ WalletManager: Provider ${walletType} is not available`
        );
        throw createWalletError(
          WalletErrorCode.WALLET_NOT_FOUND,
          `Wallet ${walletType} is not available`
        );
      }

      console.log(
        `🔄 WalletManager: Calling provider.connect() for ${walletType}`
      );
      const account = await provider.connect();

      // Save successful connection
      if (this.config.persistConnection) {
        saveLastConnectedWallet(walletType);
      }

      this.updateState({
        type: walletType,
        isConnecting: false,
        isConnected: true,
        account,
        error: null,
        lastConnected: walletType,
      });

      return account;
    } catch (error) {
      const walletError =
        error instanceof Error
          ? createWalletError(
              WalletErrorCode.CONNECTION_FAILED,
              error.message,
              error
            )
          : (error as WalletError);

      this.updateState({
        isConnecting: false,
        isConnected: false,
        error: walletError,
      });

      throw walletError;
    }
  }

  /**
   * Disconnects current wallet
   */
  async disconnect(): Promise<void> {
    try {
      if (this.currentState.type) {
        const provider = this.providers.get(this.currentState.type);
        if (provider) {
          await provider.disconnect();
        }
      }

      if (this.config.persistConnection) {
        clearWalletStorage();
      }

      this.updateState({
        type: null,
        isConnecting: false,
        isConnected: false,
        account: null,
        error: null,
      });
    } catch (error) {
      console.warn("Error during disconnect:", error);
      // Force reset state even if disconnect fails
      this.updateState({
        type: null,
        isConnecting: false,
        isConnected: false,
        account: null,
        error: null,
      });
    }
  }

  /**
   * Attempts to auto-connect to last used wallet
   */
  async autoConnect(): Promise<boolean> {
    console.log("🔄 WalletManager: Auto-connect called");
    console.log("📊 WalletManager: Auto-connect config:", {
      autoConnect: this.config.autoConnect,
      lastConnected: this.currentState.lastConnected,
    });

    if (!this.config.autoConnect || !this.currentState.lastConnected) {
      console.log(
        "ℹ️ WalletManager: Auto-connect skipped (disabled or no last wallet)"
      );
      return false;
    }

    try {
      console.log(
        `🔗 WalletManager: Auto-connecting to ${this.currentState.lastConnected}...`
      );
      await this.connect(this.currentState.lastConnected);
      console.log("✅ WalletManager: Auto-connect successful");
      return true;
    } catch (error) {
      console.warn("❌ WalletManager: Auto-connect failed:", error);
      return false;
    }
  }

  /**
   * Switches to a different network
   */
  async switchNetwork(network: NetworkInfo): Promise<void> {
    if (!this.currentState.isConnected || !this.currentState.type) {
      throw createWalletError(
        WalletErrorCode.CONNECTION_FAILED,
        "No wallet connected"
      );
    }

    const provider = this.providers.get(this.currentState.type);
    if (!provider) {
      throw createWalletError(
        WalletErrorCode.WALLET_NOT_FOUND,
        "Current wallet provider not found"
      );
    }

    await provider.switchNetwork(network);
  }

  /**
   * Signs a message with current wallet
   */
  async signMessage(message: string): Promise<string> {
    if (!this.currentState.isConnected || !this.currentState.type) {
      throw createWalletError(
        WalletErrorCode.CONNECTION_FAILED,
        "No wallet connected"
      );
    }

    const provider = this.providers.get(this.currentState.type);
    if (!provider) {
      throw createWalletError(
        WalletErrorCode.WALLET_NOT_FOUND,
        "Current wallet provider not found"
      );
    }

    return provider.signMessage(message);
  }

  /**
   * Refreshes current account data
   */
  async refreshAccount(): Promise<void> {
    if (!this.currentState.isConnected || !this.currentState.type) {
      return;
    }

    const provider = this.providers.get(this.currentState.type);
    if (!provider) {
      return;
    }

    try {
      const account = await provider.getAccount();
      if (account) {
        this.updateState({ account });
      }
    } catch (error) {
      console.warn("Failed to refresh account:", error);
    }
  }

  /**
   * Subscribes to wallet state changes
   */
  subscribe(callback: (state: WalletState) => void): () => void {
    if (!this.eventListeners.has("stateChange")) {
      this.eventListeners.set("stateChange", new Set());
    }

    this.eventListeners.get("stateChange")!.add(callback);

    // Return unsubscribe function
    return () => {
      const listeners = this.eventListeners.get("stateChange");
      if (listeners) {
        listeners.delete(callback);
      }
    };
  }

  /**
   * Updates wallet state and notifies listeners
   */
  private updateState(newState: Partial<WalletState>): void {
    this.currentState = { ...this.currentState, ...newState };

    const listeners = this.eventListeners.get("stateChange");
    if (listeners) {
      listeners.forEach((callback) => callback(this.currentState));
    }
  }

  /**
   * Handles account change events from providers
   */
  private async handleAccountChanged(
    newAccount: WalletAccount | null
  ): Promise<void> {
    if (newAccount) {
      this.updateState({
        account: newAccount,
        isConnected: true,
      });
    } else {
      await this.disconnect();
    }
  }

  /**
   * Handles network change events from providers
   */
  private async handleNetworkChanged(network: NetworkInfo): Promise<void> {
    if (this.currentState.account) {
      const updatedAccount = {
        ...this.currentState.account,
        network,
      };
      this.updateState({ account: updatedAccount });
    }
  }

  /**
   * Handles disconnection events from providers
   */
  private async handleDisconnected(): Promise<void> {
    await this.disconnect();
  }

  /**
   * Handles error events from providers
   */
  private handleError(error: WalletError): void {
    this.updateState({ error });
  }

  /**
   * Cleanup - removes all event listeners
   */
  destroy(): void {
    this.providers.forEach((provider) => {
      // Note: We can't properly unsubscribe with the current setup since we use anonymous functions
      // This is a known limitation of the current event system design
      try {
        provider.disconnect();
      } catch (error) {
        console.warn("Error disconnecting provider during destroy:", error);
      }
    });

    this.eventListeners.clear();
    this.providers.clear();
  }
}
