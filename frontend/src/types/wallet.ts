import { Network, TokenBalance, WalletProvider } from "./common";

export type TransactionStatus = "pending" | "confirmed" | "failed";

export interface WalletState {
  isConnected: boolean;
  address: string | null;
  network: Network | null;
  balance: TokenBalance[];
  isLoading: boolean;
  error: string | null;
  provider: WalletProvider | null;
}

export interface WalletConnection {
  provider: WalletProvider;
  address: string;
  chainId: number;
  isConnected: boolean;
}

export interface TransactionRequest {
  to: string;
  value?: string;
  data?: string;
  gasLimit?: string;
  gasPrice?: string;
}

export interface TransactionResponse {
  hash: string;
  blockNumber?: number;
  blockHash?: string;
  timestamp?: number;
  status: "pending" | "confirmed" | "failed";
  gasUsed?: string;
  effectiveGasPrice?: string;
}

export interface WalletError {
  code: number;
  message: string;
  data?: Record<string, unknown>;
}

export interface SignatureRequest {
  message: string;
  address: string;
}

export interface SignatureResponse {
  signature: string;
  address: string;
  message: string;
}

// Wallet transaction data
export interface WalletTransaction {
  hash: string;
  from: string;
  to: string;
  value: string;
  gasUsed: string;
  gasPrice: string;
  status: TransactionStatus;
  timestamp: number;
  blockNumber: number;
  data?: unknown; // Changed from any to unknown for better type safety
}
