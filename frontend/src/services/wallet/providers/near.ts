/**
 * NEAR wallet provider implementation using React Context
 * Based on official NEAR wallet-selector examples
 */

import { NearWalletContextType } from "@/contexts/NearWalletContext";
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
  isValidNearAccountId,
  SUPPORTED_NETWORKS,
} from "../utils";

// We'll access the NEAR context through a global reference
let nearWalletContext: NearWalletContextType | null = null;

export const setNearWalletContext = (context: NearWalletContextType) => {
  console.log("🔗 NEAR Provider: Setting context:", {
    selector: !!context?.selector,
    modal: !!context?.modal,
    accountId: context?.accountId,
    isConnected: context?.isConnected,
  });
  nearWalletContext = context;
  console.log("✅ NEAR Provider: Context set successfully");
};

export const setCloseMainModalCallback = (callback: (() => void) | null) => {
  console.log(
    "🔗 NEAR Provider: Setting close main modal callback:",
    !!callback
  );
  // Note: Currently unused but kept for future use
};

export class NearProvider implements WalletProvider {
  type = WalletType.NEAR;
  name = "NEAR Wallet";
  icon = "🔷"; // Use emoji instead of SVG for consistency

  constructor() {
    console.log("🏗️ NEAR Provider: Initializing NearProvider");
  }

  private eventListeners: Map<WalletEvent, Set<(data: unknown) => void>> =
    new Map();

  get isInstalled(): boolean {
    console.log("🔍 NEAR Provider: Checking isInstalled");
    return true; // NEAR wallet is always available via web interface
  }

  get isAvailable(): boolean {
    console.log(
      "🔍 NEAR Provider: Checking isAvailable, context=",
      !!nearWalletContext,
      "selector=",
      !!nearWalletContext?.selector
    );
    // Always return true to show the NEAR option in the wallet list
    // The actual connection will be handled by the connect method
    return true;
  }

  async connect(): Promise<WalletAccount> {
    try {
      console.log("🚀 NEAR Provider: Starting connection process...");
      console.log("🔗 NEAR Provider: Context available:", !!nearWalletContext);

      if (!nearWalletContext) {
        console.error("❌ NEAR Provider: Context not initialized");
        throw createWalletError(
          WalletErrorCode.CONNECTION_FAILED,
          "NEAR wallet context not initialized. Please reload the page and try again."
        );
      }

      console.log("📊 NEAR Provider: Current context state:", {
        selector: !!nearWalletContext.selector,
        modal: !!nearWalletContext.modal,
        accountId: nearWalletContext.accountId,
        isConnected: nearWalletContext.isConnected,
      });

      // Note: NEAR modal should appear on top of main modal with higher z-index

      // Call signIn from context
      console.log("🔐 NEAR Provider: Calling signIn...");
      try {
        if (typeof nearWalletContext.signIn !== "function") {
          console.error(
            "❌ NEAR Provider: signIn is not a function:",
            typeof nearWalletContext.signIn
          );
          throw new Error("signIn method not available");
        }

        nearWalletContext.signIn();
        console.log("✅ NEAR Provider: signIn called successfully");
      } catch (error) {
        console.error("❌ NEAR Provider: Error calling signIn:", error);
        throw error;
      }

      // Wait for connection
      return new Promise((resolve, reject) => {
        console.log("⏳ NEAR Provider: Waiting for connection...");

        // Check if already connected
        if (nearWalletContext && nearWalletContext.isConnected) {
          console.log("✅ NEAR Provider: Already connected, getting account");
          this.getCurrentAccount()
            .then((account) => {
              if (account) resolve(account);
              else reject(new Error("Failed to get account"));
            })
            .catch(reject);
          return;
        }

        let attempts = 0;
        const maxAttempts = 60; // 60 seconds

        // Listen for connection changes
        const checkConnection = () => {
          attempts++;
          console.log(
            `🔍 NEAR Provider: Check attempt ${attempts}/${maxAttempts}`
          );

          if (nearWalletContext) {
            console.log("📊 NEAR Provider: Connection state:", {
              isConnected: nearWalletContext.isConnected,
              accountId: nearWalletContext.accountId,
            });

            if (nearWalletContext.isConnected && nearWalletContext.accountId) {
              console.log("🎉 NEAR Provider: Connection successful!");
              this.getCurrentAccount()
                .then((account) => {
                  if (account) resolve(account);
                  else reject(new Error("Failed to get account"));
                })
                .catch(reject);
              return;
            }
          }

          if (attempts < maxAttempts) {
            setTimeout(checkConnection, 1000);
          } else {
            console.error("⏰ NEAR Provider: Connection timeout");
            reject(
              createWalletError(
                WalletErrorCode.USER_REJECTED,
                "Connection timeout - user may have cancelled"
              )
            );
          }
        };

        setTimeout(checkConnection, 1000);
      });
    } catch (error: unknown) {
      const err = error as { message?: string };
      console.error("❌ NEAR Provider: Connection failed:", err.message);
      throw createWalletError(
        WalletErrorCode.CONNECTION_FAILED,
        err.message || "Failed to connect to NEAR wallet",
        error
      );
    }
  }

  async disconnect(): Promise<void> {
    try {
      if (nearWalletContext && nearWalletContext.signOut) {
        await nearWalletContext.signOut();
      }
      this.emit(WalletEvent.DISCONNECTED, null);
    } catch (error) {
      console.warn("Error during NEAR wallet disconnect:", error);
      // Force disconnect even if signOut fails
      this.emit(WalletEvent.DISCONNECTED, null);
    }
  }

  async getAccount(): Promise<WalletAccount | null> {
    return this.getCurrentAccount();
  }

  private async getCurrentAccount(): Promise<WalletAccount | null> {
    try {
      console.log("🔍 NEAR Provider: Getting current account");
      console.log("📊 NEAR Provider: Context state:", {
        accountId: nearWalletContext?.accountId,
        isConnected: nearWalletContext?.isConnected,
      });

      if (!nearWalletContext?.accountId) {
        console.log("ℹ️ NEAR Provider: No accountId in context");
        return null;
      }

      const accountId = nearWalletContext.accountId;
      console.log("👤 NEAR Provider: Account ID:", accountId);

      if (!isValidNearAccountId(accountId)) {
        console.error("❌ NEAR Provider: Invalid NEAR account ID:", accountId);
        return null;
      }

      // Determine network (testnet or mainnet based on account)
      const isTestnet =
        accountId.endsWith(".testnet") || accountId.includes("test");
      const network = isTestnet
        ? SUPPORTED_NETWORKS.near_testnet
        : SUPPORTED_NETWORKS.near_mainnet;

      // Get balances
      const balances = await this.getBalances(accountId);

      return {
        address: accountId,
        network,
        balances,
        isConnected: true,
      };
    } catch (error) {
      console.warn("Failed to get NEAR account:", error);
      return null;
    }
  }

  async switchNetwork(network: NetworkInfo): Promise<void> {
    if (network.type !== "near") {
      throw createWalletError(
        WalletErrorCode.UNSUPPORTED_CHAIN,
        "NEAR wallet only supports NEAR networks"
      );
    }

    // NEAR network switching requires wallet selector re-initialization
    // For now, just verify the network is supported
    const supportedNetworks = ["testnet", "mainnet"];
    if (!supportedNetworks.includes(network.chainId.toString())) {
      throw createWalletError(
        WalletErrorCode.UNSUPPORTED_CHAIN,
        `Unsupported NEAR network: ${network.chainId}`
      );
    }
  }

  async signMessage(message: string): Promise<string> {
    if (!nearWalletContext?.selector) {
      throw createWalletError(
        WalletErrorCode.CONNECTION_FAILED,
        "NEAR wallet is not connected"
      );
    }

    try {
      const wallet = await (
        nearWalletContext.selector as {
          wallet: () => Promise<{
            signMessage?: (params: {
              message: string;
              recipient: string;
              nonce: number[];
            }) => Promise<{ signature: string }>;
          }>;
        }
      ).wallet();

      // NEAR uses signMessage method
      const nonce = crypto.getRandomValues(new Uint8Array(32));
      const signedMessage = await wallet.signMessage?.({
        message,
        recipient: "kembridge.testnet",
        nonce: Array.from(nonce),
      });

      if (signedMessage && "signature" in signedMessage) {
        return signedMessage.signature;
      }

      throw new Error("Invalid signature response");
    } catch (error: unknown) {
      const err = error as { message?: string };
      if (err.message?.includes("User rejected")) {
        throw createWalletError(
          WalletErrorCode.USER_REJECTED,
          "User rejected message signing"
        );
      }

      throw createWalletError(
        WalletErrorCode.TRANSACTION_FAILED,
        `Failed to sign message: ${err.message || "Unknown error"}`,
        error
      );
    }
  }

  async sendTransaction(params: TransactionParams): Promise<string> {
    if (!nearWalletContext?.selector) {
      throw createWalletError(
        WalletErrorCode.CONNECTION_FAILED,
        "NEAR wallet is not connected"
      );
    }

    try {
      const wallet = await (
        nearWalletContext.selector as {
          wallet: () => Promise<{
            signAndSendTransaction: (params: {
              receiverId: string;
              actions: Array<{ type: string; params: Record<string, unknown> }>;
            }) => Promise<{ transaction: { hash: string } }>;
          }>;
        }
      ).wallet();

      // Convert Ethereum-style params to NEAR transaction
      const actions: Array<{ type: string; params: Record<string, unknown> }> =
        [];

      if (params.value && params.value !== "0x0") {
        actions.push({
          type: "Transfer",
          params: {
            deposit: params.value,
          },
        });
      }

      if (params.data && params.data !== "0x") {
        actions.push({
          type: "FunctionCall",
          params: {
            methodName: "bridge_transfer",
            args: {},
            gas: "30000000000000",
            deposit: params.value || "0",
          },
        });
      }

      const result = await wallet.signAndSendTransaction({
        receiverId: params.to,
        actions,
      });

      return result.transaction.hash;
    } catch (error: unknown) {
      const err = error as { message?: string };
      if (err.message?.includes("User rejected")) {
        throw createWalletError(
          WalletErrorCode.USER_REJECTED,
          "User rejected transaction"
        );
      }

      throw createWalletError(
        WalletErrorCode.TRANSACTION_FAILED,
        `Transaction failed: ${err.message || "Unknown error"}`,
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

  private emit(event: WalletEvent, data: unknown): void {
    const listeners = this.eventListeners.get(event);
    if (listeners) {
      listeners.forEach((handler) => handler(data));
    }
  }

  private async getBalances(accountId: string): Promise<TokenBalance[]> {
    try {
      console.log("💰 NEAR Provider: Getting balances for account:", accountId);

      // Use NEAR RPC to get account balance
      console.log("🔄 NEAR Provider: Fetching balance from RPC");
      const response = await fetch("https://rpc.testnet.near.org", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: "dontcare",
          method: "query",
          params: {
            request_type: "view_account",
            finality: "final",
            account_id: accountId,
          },
        }),
      });

      const data = await response.json();
      console.log("📊 NEAR Provider: RPC response:", data);

      if (data.result) {
        const balanceYocto = data.result.amount;
        console.log("💰 NEAR Provider: Balance in yoctoNEAR:", balanceYocto);

        return [
          {
            symbol: "NEAR",
            balance: balanceYocto,
            decimals: 24,
            usdValue: "0", // TODO: Get USD value from price oracle
          },
        ];
      }

      return [];
    } catch (error) {
      console.warn("Failed to get NEAR balances:", error);
      return [
        {
          symbol: "NEAR",
          balance: "0",
          decimals: 24,
          usdValue: "0",
        },
      ];
    }
  }
}
