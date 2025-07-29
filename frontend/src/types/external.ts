/**
 * Types for external wallet APIs
 * Defines interfaces for MetaMask, NEAR and other external providers
 */

// Ethereum accounts response type
export type EthereumAccounts = string[];

// MetaMask/Ethereum Provider Types
export interface EthereumProvider {
  isMetaMask?: boolean;
  isCoinbaseWallet?: boolean;
  providers?: EthereumProvider[];
  request: <T = unknown>(args: {
    method: string;
    params?: unknown[];
  }) => Promise<T>;
  on: (event: string, handler: (...args: unknown[]) => void) => void;
  removeListener: (
    event: string,
    handler: (...args: unknown[]) => void
  ) => void;
  chainId?: string;
  selectedAddress?: string;
}

export interface WindowEthereum extends EthereumProvider {
  providers?: EthereumProvider[];
}

// Note: Window.ethereum is already declared by coinbase SDK

// Global Window Extensions
declare global {
  interface Window {
    Buffer: typeof Buffer;
  }
}

// NEAR Wallet Selector Types
export interface NEARWalletModal {
  show: () => void;
  hide: () => void;
}

// NEAR Action types (compatible with @near-wallet-selector)
export interface NEARTransferAction {
  type: "Transfer";
  params: {
    deposit: string;
  };
}

export interface NEARFunctionCallAction {
  type: "FunctionCall";
  params: {
    methodName: string;
    args: Record<string, unknown>;
    gas: string;
    deposit: string;
  };
}

export type NEARTransactionAction = NEARTransferAction | NEARFunctionCallAction;

export interface NEARTransactionParams {
  signerId?: string;
  receiverId: string;
  actions: NEARTransactionAction[];
}

export interface NEARTransactionResult {
  transaction: {
    hash: string;
  };
}

export interface NEARWallet {
  signOut: () => Promise<void>;
  getAccounts: () => Promise<{ accountId: string }[]>;
  signMessage?: (params: {
    message: string;
    recipient: string;
    nonce: Uint8Array;
  }) => Promise<{ signature: string }>;
  signAndSendTransaction: (
    params: NEARTransactionParams
  ) => Promise<NEARTransactionResult>;
}

// API Error Types
export interface APIError {
  message: string;
  code?: string | number;
  details?: Record<string, unknown>;
}

// Generic Provider Type
export interface WalletProviderGeneric {
  isMetaMask?: boolean;
  isCoinbaseWallet?: boolean;
  request?: (args: { method: string; params?: unknown[] }) => Promise<unknown>;
}
