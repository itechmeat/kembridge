/**
 * MetaMask wallet provider implementation
 * Handles MetaMask-specific wallet operations
 */

import {
  WalletProvider,
  WalletType,
  WalletAccount,
  WalletEvent,
  WalletErrorCode,
  NetworkInfo,
  TransactionParams,
  TokenBalance,
} from "../types";
import {
  createWalletError,
  isValidEthereumAddress,
  getNetworkByChainId,
} from "../utils";
import { EthereumProvider, EthereumAccounts } from "../../../types/external";

export class MetaMaskProvider implements WalletProvider {
  type = WalletType.METAMASK;
  name = "MetaMask";
  icon = "/icons/metamask.svg"; // TODO (feat): Add wallet icons

  private ethereum: EthereumProvider | null;
  private eventListeners: Map<WalletEvent, Set<(data: unknown) => void>> =
    new Map();

  constructor() {
    console.log("🏗️ MetaMask: Initializing MetaMaskProvider");
    this.ethereum = this.getEthereumProvider();

    if (import.meta.env.DEV) {
      console.log(
        "🔍 MetaMask: Constructor - ethereum available:",
        !!this.ethereum
      );
    }
  }

  private getEthereumProvider(): EthereumProvider | null {
    console.log("🔍 MetaMask: Getting ethereum provider");

    if (import.meta.env.DEV) {
      console.log("🔍 MetaMask: Window ethereum available:", !!window.ethereum);
    }

    if (typeof window !== "undefined" && window.ethereum) {
      console.log("✅ MetaMask: Found window.ethereum");

      if (import.meta.env.DEV) {
        console.log("🔍 MetaMask: Checking ethereum provider details");
        const ethereum = window.ethereum as Record<string, unknown>;
        console.log("🔍 MetaMask: isMetaMask:", ethereum.isMetaMask);
        console.log("🔍 MetaMask: request method:", typeof ethereum.request);
        console.log("🔍 MetaMask: on method:", typeof ethereum.on);
        console.log(
          "🔍 MetaMask: removeListener method:",
          typeof ethereum.removeListener
        );
        console.log(
          "🔍 MetaMask: providers array:",
          Array.isArray(ethereum.providers)
        );
        console.log(
          "🔍 MetaMask: _metamask object:",
          typeof ethereum._metamask
        );
      }

      // Handle multiple providers (e.g., MetaMask + Coinbase)
      const providers = (window.ethereum as { providers?: EthereumProvider[] })
        .providers;
      if (Array.isArray(providers)) {
        if (import.meta.env.DEV) {
          console.log(
            "🔍 MetaMask: Multiple providers found:",
            providers.length
          );
        }

        const metamaskProvider = providers.find(
          (provider: unknown) =>
            (provider as { isMetaMask?: boolean }).isMetaMask
        );

        if (metamaskProvider) {
          if (import.meta.env.DEV) {
            console.log("✅ MetaMask: Found MetaMask in providers array");
          }
          return metamaskProvider;
        }
      }

      // Single provider case
      const singleProvider = window.ethereum as EthereumProvider & {
        isMetaMask?: boolean;
      };
      if (singleProvider.isMetaMask) {
        if (import.meta.env.DEV) {
          console.log("✅ MetaMask: Found single MetaMask provider");
        }
        return singleProvider;
      }

      if (import.meta.env.DEV) {
        console.log("⚠️ MetaMask: Provider found but not MetaMask");
      }
    }

    if (import.meta.env.DEV) {
      console.log("❌ MetaMask: No ethereum provider found");
    }
    return null;
  }

  get isInstalled(): boolean {
    if (typeof window === "undefined") {
      return false;
    }

    // Check if MetaMask provider can be found
    const provider = this.getEthereumProvider();
    return !!provider;
  }

  get isAvailable(): boolean {
    return this.isInstalled;
  }

  async connect(): Promise<WalletAccount> {
    if (!this.isAvailable || !this.ethereum) {
      throw createWalletError(
        WalletErrorCode.WALLET_NOT_FOUND,
        "MetaMask is not installed or available"
      );
    }

    try {
      // Request account access
      const accounts = await this.ethereum.request<EthereumAccounts>({
        method: "eth_requestAccounts",
      });

      if (!accounts || accounts.length === 0) {
        throw createWalletError(
          WalletErrorCode.USER_REJECTED,
          "User rejected connection request"
        );
      }

      const address = accounts[0];
      if (!isValidEthereumAddress(address)) {
        throw createWalletError(
          WalletErrorCode.INVALID_ADDRESS,
          "Invalid Ethereum address received"
        );
      }

      // Get current network
      const chainId = await this.ethereum.request<string>({
        method: "eth_chainId",
      });

      const currentChainId = parseInt(chainId, 16);
      getNetworkByChainId(currentChainId);

      // Auto-switch to Sepolia if not already on it
      if (currentChainId !== 11155111) {
        console.log(
          `Connected to network ${currentChainId}, switching to Sepolia testnet (11155111)...`
        );

        try {
          // Try to switch to Sepolia using direct ethereum request
          const chainIdHex = "0xaa36a7"; // Sepolia chainId in hex

          try {
            await this.ethereum.request({
              method: "wallet_switchEthereumChain",
              params: [{ chainId: chainIdHex }],
            });
          } catch (switchError: unknown) {
            // If network doesn't exist, try to add it
            const err = switchError as { code?: number };
            if (err.code === 4902) {
              await this.ethereum.request({
                method: "wallet_addEthereumChain",
                params: [
                  {
                    chainId: chainIdHex,
                    chainName: "Sepolia Testnet",
                    rpcUrls: ["https://ethereum-sepolia.publicnode.com"],
                    blockExplorerUrls: ["https://sepolia.etherscan.io"],
                    nativeCurrency: {
                      name: "Ethereum",
                      symbol: "ETH",
                      decimals: 18,
                    },
                  },
                ],
              });
            } else {
              throw switchError;
            }
          }
          console.log("Successfully switched to Sepolia");
        } catch (switchError) {
          console.warn("Failed to auto-switch to Sepolia:", switchError);
          // Continue with current network if switch fails
        }
      }

      // Get final network info after potential switch
      const finalChainId = await this.ethereum.request<string>({
        method: "eth_chainId",
      });
      const finalNetwork = getNetworkByChainId(parseInt(finalChainId, 16));

      if (!finalNetwork) {
        throw createWalletError(
          WalletErrorCode.UNSUPPORTED_CHAIN,
          `Unsupported network: ${finalChainId}. Please switch to Sepolia testnet manually.`
        );
      }

      // Get balances
      const balances = await this.getBalances(address);

      return {
        address,
        network: finalNetwork,
        balances,
        isConnected: true,
      };
    } catch (error: unknown) {
      const err = error as { code?: number; message?: string };
      if (err.code === 4001) {
        throw createWalletError(
          WalletErrorCode.USER_REJECTED,
          "User rejected the connection request"
        );
      }

      if (error instanceof Error) {
        throw createWalletError(
          WalletErrorCode.CONNECTION_FAILED,
          error.message,
          error
        );
      }

      throw error;
    }
  }

  async disconnect(): Promise<void> {
    // MetaMask doesn't have a disconnect method
    // We'll just clear our internal state without emitting events
    // The wallet manager will handle state updates
  }

  async getAccount(): Promise<WalletAccount | null> {
    if (!this.isAvailable || !this.ethereum) {
      return null;
    }

    try {
      const accounts = await this.ethereum.request<EthereumAccounts>({
        method: "eth_accounts",
      });

      if (!accounts || accounts.length === 0) {
        return null;
      }

      const address = accounts[0];
      const chainId = await this.ethereum.request<string>({
        method: "eth_chainId",
      });

      const network = getNetworkByChainId(chainId);
      if (!network) {
        return null;
      }

      const balances = await this.getBalances(address);

      return {
        address,
        network,
        balances,
        isConnected: true,
      };
    } catch (error) {
      console.warn("Failed to get MetaMask account:", error);
      return null;
    }
  }

  async switchNetwork(network: NetworkInfo): Promise<void> {
    if (!this.isAvailable) {
      throw createWalletError(
        WalletErrorCode.WALLET_NOT_FOUND,
        "MetaMask is not available"
      );
    }

    const chainIdHex = `0x${network.chainId.toString(16)}`;

    try {
      // Try to switch to the network
      await this.ethereum!.request({
        method: "wallet_switchEthereumChain",
        params: [{ chainId: chainIdHex }],
      });
    } catch (switchError: unknown) {
      // If network doesn't exist, try to add it
      const err = switchError as { code?: number; message?: string };
      if (err.code === 4902) {
        try {
          await this.ethereum!.request({
            method: "wallet_addEthereumChain",
            params: [
              {
                chainId: chainIdHex,
                chainName: network.name,
                rpcUrls: network.rpcUrls,
                blockExplorerUrls: network.blockExplorerUrls,
                nativeCurrency: network.nativeCurrency,
              },
            ],
          });
        } catch (addError: unknown) {
          const addErr = addError as { message?: string };
          throw createWalletError(
            WalletErrorCode.NETWORK_MISMATCH,
            `Failed to add network: ${addErr.message || "Unknown error"}`,
            addError
          );
        }
      } else {
        const switchErr = switchError as { message?: string };
        throw createWalletError(
          WalletErrorCode.NETWORK_MISMATCH,
          `Failed to switch network: ${switchErr.message || "Unknown error"}`,
          switchError
        );
      }
    }
  }

  async signMessage(message: string): Promise<string> {
    if (!this.isAvailable) {
      throw createWalletError(
        WalletErrorCode.WALLET_NOT_FOUND,
        "MetaMask is not available"
      );
    }

    try {
      const accounts = await this.ethereum!.request<EthereumAccounts>({
        method: "eth_accounts",
      });

      if (!accounts || accounts.length === 0) {
        throw createWalletError(
          WalletErrorCode.CONNECTION_FAILED,
          "No connected accounts found"
        );
      }

      const signature = await this.ethereum!.request<string>({
        method: "personal_sign",
        params: [message, accounts[0]],
      });

      return signature;
    } catch (error: unknown) {
      const err = error as { code?: number; message?: string };
      if (err.code === 4001) {
        throw createWalletError(
          WalletErrorCode.USER_REJECTED,
          "User rejected message signing"
        );
      }

      const errMsg = error as { message?: string };
      throw createWalletError(
        WalletErrorCode.TRANSACTION_FAILED,
        `Failed to sign message: ${errMsg.message || "Unknown error"}`,
        error
      );
    }
  }

  async sendTransaction(params: TransactionParams): Promise<string> {
    if (!this.isAvailable) {
      throw createWalletError(
        WalletErrorCode.WALLET_NOT_FOUND,
        "MetaMask is not available"
      );
    }

    try {
      const accounts = await this.ethereum!.request<EthereumAccounts>({
        method: "eth_accounts",
      });

      if (!accounts || accounts.length === 0) {
        throw createWalletError(
          WalletErrorCode.CONNECTION_FAILED,
          "No connected accounts found"
        );
      }

      const transactionParams = {
        from: accounts[0],
        to: params.to,
        value: params.value || "0x0",
        data: params.data || "0x",
        gasLimit: params.gasLimit,
        gasPrice: params.gasPrice,
        maxFeePerGas: params.maxFeePerGas,
        maxPriorityFeePerGas: params.maxPriorityFeePerGas,
      };

      const txHash = await this.ethereum!.request<string>({
        method: "eth_sendTransaction",
        params: [transactionParams],
      });

      return txHash;
    } catch (error: unknown) {
      const err = error as { code?: number; message?: string };
      if (err.code === 4001) {
        throw createWalletError(
          WalletErrorCode.USER_REJECTED,
          "User rejected transaction"
        );
      }

      const errMsg = error as { message?: string };
      throw createWalletError(
        WalletErrorCode.TRANSACTION_FAILED,
        `Transaction failed: ${errMsg.message || "Unknown error"}`,
        error
      );
    }
  }

  on(event: WalletEvent, handler: (data: unknown) => void): void {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Set());
    }
    this.eventListeners.get(event)!.add(handler);
  }

  off(event: WalletEvent, handler: (data: unknown) => void): void {
    const listeners = this.eventListeners.get(event);
    if (listeners) {
      listeners.delete(handler);
    }
  }

  // Note: emit method available for future event handling

  private async getBalances(address: string): Promise<TokenBalance[]> {
    if (!this.ethereum) {
      return [];
    }

    try {
      // Get ETH balance using a more compatible approach
      const ethBalance = await this.ethereum.request<string>({
        method: "eth_getBalance",
        params: [address, "latest"],
      });

      console.log("Raw balance response:", ethBalance);

      if (!ethBalance || ethBalance === "0x") {
        console.warn("No balance returned from eth_getBalance");
        return [
          {
            symbol: "ETH",
            balance: "0",
            decimals: 18,
            usdValue: "0",
          },
        ];
      }

      // Convert hex to decimal
      let balanceWei: string;
      try {
        balanceWei = BigInt(ethBalance).toString();
      } catch (conversionError) {
        console.warn("Failed to convert balance:", conversionError);
        balanceWei = "0";
      }

      // TODO (feat): Add support for ERC-20 token balances
      const balances: TokenBalance[] = [
        {
          symbol: "ETH",
          balance: balanceWei,
          decimals: 18,
          usdValue: "0", // TODO (feat): Get USD value from price oracle
        },
      ];

      console.log("Processed balances:", balances);
      return balances;
    } catch (error) {
      console.warn("Failed to get balances:", error);
      // Return empty balance instead of failing
      return [
        {
          symbol: "ETH",
          balance: "0",
          decimals: 18,
          usdValue: "0",
        },
      ];
    }
  }
}
