/**
 * TypeScript type definitions for E2E tests
 */

export interface TestUser {
  walletAddress: string;
  privateKey: string;
  balance: {
    eth: string;
    near: string;
  };
}

export interface ApiResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: string;
}

export interface AuthResponse {
  token: string;
  expiresAt: string;
  user: {
    address: string;
    chainType: 'ethereum' | 'near';
  };
}

export interface NonceResponse {
  nonce: string;
  message: string;
  expiresAt: string;
}

export interface BridgeTransaction {
  id: string;
  fromChain: 'ethereum' | 'near';
  toChain: 'ethereum' | 'near';
  amount: string;
  status: 'pending' | 'processing' | 'completed' | 'failed';
  createdAt: string;
  updatedAt: string;
  txHashes: {
    source?: string;
    destination?: string;
  };
  quantumSignature?: string;
  riskScore?: number;
}

export interface WalletState {
  connected: boolean;
  address?: string;
  chainId?: number;
  balance?: string;
}

export interface SecurityIndicator {
  type: 'quantum' | 'risk' | 'audit';
  visible: boolean;
  text?: string;
  value?: string | number;
}

export interface TestConfiguration {
  apiBaseUrl: string;
  frontendUrl: string;
  testTimeout: number;
  retryCount: number;
  slowMo: number;
}

export interface MonitoringData {
  apiCalls: ApiCall[];
  networkErrors: NetworkError[];
  consoleMessages: ConsoleMessage[];
}

export interface ApiCall {
  url: string;
  method: string;
  status?: number;
  timestamp: number;
  duration?: number;
}

export interface NetworkError {
  url: string;
  error: string;
  timestamp: number;
}

export interface ConsoleMessage {
  level: 'log' | 'warn' | 'error' | 'info';
  text: string;
  timestamp: number;
}

export interface TestResults {
  testName: string;
  success: boolean;
  duration: number;
  authResult?: {
    success: boolean;
    hasNonceCalls: boolean;
    hasVerifyCalls: boolean;
    totalAuthCalls: number;
  };
  apiCallCounts?: Record<string, number>;
  errors?: string[];
  metrics?: Record<string, string | number>;
}

// Page Object interfaces
export interface BridgePageElements {
  amountInput: string;
  fromTokenSelector: string;
  toTokenSelector: string;
  submitButton: string;
  directionSwitch: string;
  connectWalletButton: string;
}

export interface TestSelectors {
  // Authentication
  ethWalletButton: string;
  nearWalletButton: string;
  connectWalletButton: string;
  disconnectButton: string;
  
  // Navigation
  homeLink: string;
  bridgeLink: string;
  dashboardLink: string;
  
  // Bridge Form
  amountInput: string;
  fromChainSelect: string;
  toChainSelect: string;
  reviewButton: string;
  confirmButton: string;
  cancelButton: string;
  
  // Status and Feedback
  successMessage: string;
  errorMessage: string;
  loadingSpinner: string;
  transactionStatus: string;
  
  // Security
  quantumBadge: string;
  riskScore: string;
  securityWarning: string;
}

// Utility types
export type ChainType = 'ethereum' | 'near';
export type TransactionStatus = 'pending' | 'processing' | 'completed' | 'failed';
export type WalletType = 'metamask' | 'near-wallet' | 'wallet-connect';

// Test environment configuration
export interface TestEnvironment {
  monitoring: MonitoringData;
  configuration: TestConfiguration;
  authenticatedUser?: TestUser;
  walletState?: WalletState;
}