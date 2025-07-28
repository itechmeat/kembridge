import EthereumProvider from "@walletconnect/ethereum-provider";
import { appConfig } from "../../../config/env";
import {
  WalletProvider,
  WalletAccount,
  WalletType,
  WalletEvent,
  NetworkInfo,
  NetworkType,
  TransactionParams,
} from "../types";

/**
 * WalletConnect provider implementing real wallet integration
 * Follows SOLID principle: Single responsibility for WalletConnect protocol
 */
// Event handler type for wallet events
type WalletEventHandler = (data: unknown) => void;

export class WalletConnectProvider implements WalletProvider {
  private provider: EthereumProvider | null = null;

  private eventHandlers: Map<WalletEvent, Set<WalletEventHandler>> = new Map();

  readonly type = WalletType.WALLET_CONNECT;
  readonly name = "WalletConnect";
  readonly icon = "🔗";
  readonly isInstalled = true; // WalletConnect doesn't require installation
  readonly isAvailable = true;

  /**
   * Event handler methods for wallet event system
   */
  on(event: WalletEvent, handler: WalletEventHandler): void {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, new Set());
    }
    this.eventHandlers.get(event)!.add(handler);
  }

  off(event: WalletEvent, handler: WalletEventHandler): void {
    const handlers = this.eventHandlers.get(event);
    if (handlers) {
      handlers.delete(handler);
    }
  }

  private emit(event: WalletEvent, data: unknown): void {
    const handlers = this.eventHandlers.get(event);
    if (handlers) {
      handlers.forEach((handler) => handler(data));
    }
  }

  /**
   * Initialize WalletConnect provider with real configuration
   */
  private async initializeProvider(): Promise<EthereumProvider> {
    if (this.provider) {
      return this.provider;
    }

    try {
      this.provider = await EthereumProvider.init({
        projectId: appConfig.wallet.walletConnectProjectId,
        chains: [1, 11155111], // Ethereum mainnet and Sepolia testnet
        showQrModal: true,
        optionalChains: [137, 80001], // Polygon mainnet and Mumbai testnet
        methods: [
          "eth_sendTransaction",
          "eth_signTransaction",
          "eth_sign",
          "personal_sign",
          "eth_signTypedData",
          "eth_accounts",
          "eth_requestAccounts",
        ],
        events: ["chainChanged", "accountsChanged", "connect", "disconnect"],
        rpcMap: {
          1: "https://eth.llamarpc.com",
          11155111: "https://ethereum-sepolia.publicnode.com",
          137: "https://polygon-rpc.com",
          80001: "https://rpc-mumbai.maticvigil.com",
        },
        metadata: {
          name: appConfig.app.name,
          description: appConfig.app.description,
          url: appConfig.app.url,
          icons: [appConfig.app.iconUrl],
        },
      });

      return this.provider;
    } catch (error) {
      console.error("WalletConnect initialization failed:", error);
      throw new Error("Failed to initialize WalletConnect");
    }
  }

  /**
   * Connect to wallet using real WalletConnect protocol
   */
  async connect(): Promise<WalletAccount> {
    try {
      const provider = await this.initializeProvider();

      // Setup event listeners
      this.setupProviderEvents(provider);

      // Enable session (shows QR modal for mobile wallets)
      const accounts = await provider.enable();

      if (!accounts || accounts.length === 0) {
        throw new Error("No accounts returned from WalletConnect");
      }

      const chainId = parseInt(provider.chainId.toString());
      const address = accounts[0];
      const network = this.getNetworkInfo(chainId);
      const balance = await this.getBalance(address);

      const account: WalletAccount = {
        address,
        network,
        balances: [
          {
            symbol: network.nativeCurrency?.symbol || "ETH",
            balance,
            decimals: network.nativeCurrency?.decimals || 18,
          },
        ],
        isConnected: true,
      };

      return account;
    } catch (error) {
      console.error("WalletConnect connection failed:", error);
      this.emit(WalletEvent.ERROR, {
        code: "CONNECTION_FAILED",
        message: error instanceof Error ? error.message : "Unknown error",
      });
      throw new Error(
        `WalletConnect connection failed: ${
          error instanceof Error ? error.message : "Unknown error"
        }`
      );
    }
  }

  /**
   * Get current account information from connected session
   */
  async getAccount(): Promise<WalletAccount | null> {
    if (!this.provider || !this.provider.connected) {
      return null;
    }

    try {
      const accounts = this.provider.accounts;
      if (!accounts || accounts.length === 0) {
        return null;
      }

      const address = accounts[0];
      const chainId = parseInt(this.provider.chainId.toString());
      const network = this.getNetworkInfo(chainId);
      const balance = await this.getBalance(address);

      return {
        address,
        network,
        balances: [
          {
            symbol: network.nativeCurrency?.symbol || "ETH",
            balance,
            decimals: network.nativeCurrency?.decimals || 18,
          },
        ],
        isConnected: true,
      };
    } catch (error) {
      console.error("Failed to get WalletConnect account:", error);
      return null;
    }
  }

  /**
   * Get real balance from blockchain via RPC
   */
  async getBalance(address: string): Promise<string> {
    if (!this.provider) {
      return "0";
    }

    try {
      const balance = (await this.provider.request({
        method: "eth_getBalance",
        params: [address, "latest"],
      })) as string;

      // Convert from wei to ETH
      const balanceInEth = parseInt(balance, 16) / Math.pow(10, 18);
      return balanceInEth.toFixed(6);
    } catch (error) {
      console.error("Failed to get balance:", error);
      return "0";
    }
  }

  /**
   * Sign message using connected wallet
   */
  async signMessage(message: string): Promise<string> {
    if (!this.provider || !this.provider.connected) {
      throw new Error("WalletConnect not connected");
    }

    try {
      const accounts = this.provider.accounts;
      if (!accounts || accounts.length === 0) {
        throw new Error("No accounts available");
      }

      const signature = (await this.provider.request({
        method: "personal_sign",
        params: [message, accounts[0]],
      })) as string;

      return signature;
    } catch (error) {
      console.error("Failed to sign message:", error);
      throw new Error(
        `Failed to sign message: ${
          error instanceof Error ? error.message : "Unknown error"
        }`
      );
    }
  }

  /**
   * Disconnect from WalletConnect session
   */
  async disconnect(): Promise<void> {
    if (this.provider && this.provider.connected) {
      try {
        await this.provider.disconnect();
      } catch (error) {
        console.error("Failed to disconnect WalletConnect:", error);
      }
    }
    this.provider = null;
  }

  /**
   * Switch to a different network
   */
  async switchNetwork(network: NetworkInfo): Promise<void> {
    if (!this.provider) {
      throw new Error("WalletConnect not connected");
    }

    try {
      await this.provider.request({
        method: "wallet_switchEthereumChain",
        params: [{ chainId: `0x${network.chainId.toString(16)}` }],
      });
    } catch (error) {
      console.error("Failed to switch network:", error);
      throw new Error(
        `Failed to switch network: ${
          error instanceof Error ? error.message : "Unknown error"
        }`
      );
    }
  }

  /**
   * Send transaction
   */
  async sendTransaction(params: TransactionParams): Promise<string> {
    if (!this.provider || !this.provider.connected) {
      throw new Error("WalletConnect not connected");
    }

    try {
      const accounts = this.provider.accounts;
      if (!accounts || accounts.length === 0) {
        throw new Error("No accounts available");
      }

      const txHash = (await this.provider.request({
        method: "eth_sendTransaction",
        params: [
          {
            from: accounts[0],
            to: params.to,
            value: params.value,
            data: params.data,
            gas: params.gasLimit,
            gasPrice: params.gasPrice,
            maxFeePerGas: params.maxFeePerGas,
            maxPriorityFeePerGas: params.maxPriorityFeePerGas,
          },
        ],
      })) as string;

      return txHash;
    } catch (error) {
      console.error("Failed to send transaction:", error);
      throw new Error(
        `Failed to send transaction: ${
          error instanceof Error ? error.message : "Unknown error"
        }`
      );
    }
  }

  /**
   * Setup provider event listeners
   */
  private setupProviderEvents(provider: EthereumProvider): void {
    provider.on("accountsChanged", (accounts: string[]) => {
      if (accounts.length === 0) {
        this.emit(WalletEvent.DISCONNECTED, null);
      } else {
        this.getAccount().then((account) => {
          this.emit(WalletEvent.ACCOUNT_CHANGED, account);
        });
      }
    });

    provider.on("chainChanged", (chainId: string) => {
      const newChainId = parseInt(chainId, 16);
      const network = this.getNetworkInfo(newChainId);
      this.emit(WalletEvent.NETWORK_CHANGED, network);
    });

    provider.on("disconnect", () => {
      this.emit(WalletEvent.DISCONNECTED, null);
    });
  }

  /**
   * Get network information by chain ID
   * Follows DRY principle: centralized network configuration
   */
  private getNetworkInfo(chainId: number): NetworkInfo {
    const networks: Record<number, NetworkInfo> = {
      1: {
        chainId: 1,
        name: "Ethereum",
        type: NetworkType.ETHEREUM,
        rpcUrls: ["https://eth.llamarpc.com"],
        blockExplorerUrls: ["https://etherscan.io"],
        nativeCurrency: {
          name: "Ether",
          symbol: "ETH",
          decimals: 18,
        },
      },
      11155111: {
        chainId: 11155111,
        name: "Sepolia",
        type: NetworkType.ETHEREUM,
        rpcUrls: ["https://ethereum-sepolia.publicnode.com"],
        blockExplorerUrls: ["https://sepolia.etherscan.io"],
        nativeCurrency: {
          name: "Sepolia Ether",
          symbol: "ETH",
          decimals: 18,
        },
      },
      137: {
        chainId: 137,
        name: "Polygon",
        type: NetworkType.ETHEREUM,
        rpcUrls: ["https://polygon-rpc.com"],
        blockExplorerUrls: ["https://polygonscan.com"],
        nativeCurrency: {
          name: "MATIC",
          symbol: "MATIC",
          decimals: 18,
        },
      },
    };

    return (
      networks[chainId] || {
        chainId,
        name: `Chain ${chainId}`,
        type: NetworkType.ETHEREUM,
        rpcUrls: [],
        nativeCurrency: {
          name: "Unknown",
          symbol: "UNKNOWN",
          decimals: 18,
        },
      }
    );
  }
}
