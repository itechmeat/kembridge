// Base response interface for API calls
export interface BaseResponse {
  success: boolean;
  message?: string;
  data?: unknown; // Changed from any to unknown for better type safety
}

// Paginated response interface
export interface PaginatedResponse<T> extends BaseResponse {
  data?: T[];
  pagination?: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
  };
}

export interface LoadingState {
  isLoading: boolean;
  error: string | null;
}

export interface Network {
  id: string;
  name: string;
  chainId: number;
  rpcUrl: string;
  blockExplorer: string;
  nativeCurrency: {
    name: string;
    symbol: string;
    decimals: number;
  };
}

export interface Token {
  address: string;
  symbol: string;
  name: string;
  decimals: number;
  logoURI?: string;
  balance?: string;
}

export interface TokenBalance {
  token: Token;
  balance: string;
  formattedBalance: string;
  usdValue?: string;
}

export type WalletProvider =
  | "metamask"
  | "walletconnect"
  | "near-wallet"
  | "coinbase";

export interface AppConfig {
  apiBaseUrl: string;
  mockMode: boolean;
  supportedNetworks: Network[];
  defaultNetwork: Network;
}
