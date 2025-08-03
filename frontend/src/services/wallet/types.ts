export enum WalletType {
  METAMASK = "metamask",
  NEAR = "near",
  WALLET_CONNECT = "walletconnect",
  COINBASE = "coinbase",
}

export enum NetworkType {
  ETHEREUM = "ethereum",
  NEAR = "near",
}

export enum WalletErrorCode {
  WALLET_NOT_FOUND = "WALLET_NOT_FOUND",
  USER_REJECTED = "USER_REJECTED",
  NETWORK_MISMATCH = "NETWORK_MISMATCH",
  INSUFFICIENT_FUNDS = "INSUFFICIENT_FUNDS",
  CONNECTION_FAILED = "CONNECTION_FAILED",
  INVALID_ADDRESS = "INVALID_ADDRESS",
  TRANSACTION_FAILED = "TRANSACTION_FAILED",
  UNSUPPORTED_CHAIN = "UNSUPPORTED_CHAIN",
}

export interface WalletError {
  code: WalletErrorCode;
  message: string;
  details?: unknown;
}

export interface NetworkInfo {
  chainId: string | number;
  name: string;
  type: NetworkType;
  rpcUrls: string[];
  blockExplorerUrls?: string[];
  nativeCurrency?: {
    name: string;
    symbol: string;
    decimals: number;
  };
}

export interface TokenBalance {
  symbol: string;
  balance: string;
  decimals: number;
  contractAddress?: string;
  usdValue?: string;
}

export interface WalletAccount {
  address: string;
  network: NetworkInfo;
  balances: TokenBalance[];
  isConnected: boolean;
}

export interface WalletState {
  type: WalletType | null;
  isConnecting: boolean;
  isConnected: boolean;
  account: WalletAccount | null;
  error: WalletError | null;
  lastConnected?: WalletType;
}

export interface WalletProvider {
  type: WalletType;
  name: string;
  icon: string;
  isInstalled: boolean;
  isAvailable: boolean;
  connect: () => Promise<WalletAccount>;
  disconnect: () => Promise<void>;
  getAccount: () => Promise<WalletAccount | null>;
  switchNetwork: (network: NetworkInfo) => Promise<void>;
  signMessage: (message: string) => Promise<string>;
  sendTransaction: (params: TransactionParams) => Promise<string>;
  on: (event: WalletEvent, handler: (data: unknown) => void) => void;
  off: (event: WalletEvent, handler: (data: unknown) => void) => void;
}

export interface TransactionParams {
  to: string;
  value?: string;
  data?: string;
  gasLimit?: string;
  gasPrice?: string;
  maxFeePerGas?: string;
  maxPriorityFeePerGas?: string;
}

export enum WalletEvent {
  ACCOUNT_CHANGED = "accountChanged",
  NETWORK_CHANGED = "networkChanged",
  DISCONNECTED = "disconnected",
  ERROR = "error",
}

export interface WalletConnectionOptions {
  autoConnect?: boolean;
  persistConnection?: boolean;
  preferredWallet?: WalletType;
}

export interface WalletManagerConfig {
  supportedNetworks: NetworkInfo[];
  defaultNetwork: NetworkInfo;
  autoConnect: boolean;
  persistConnection: boolean;
}
