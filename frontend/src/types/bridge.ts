export type ChainType = "ethereum" | "near";

export interface BridgeToken {
  symbol: string;
  name: string;
  address: string;
  decimals: number;
  logoUrl?: string;
  chain: ChainType;
  balance?: string;
  usdValue?: string;
}

export interface BridgeQuoteParams {
  fromToken: string;
  toToken: string;
  fromChain: ChainType;
  toChain: ChainType;
  fromAmount: string;
  maxSlippage?: number;
  userId?: string;
}

export interface BridgeQuote {
  id: string;
  fromToken: string;
  toToken: string;
  fromChain: ChainType;
  toChain: ChainType;
  fromAmount: string;
  toAmount: string;
  exchangeRate: string;
  estimatedGas: string;
  bridgeFee: string;
  protocolFee: string;
  totalFees: string;
  priceImpact: string;
  slippage: string;
  estimatedTime: number; // seconds
  expiresAt: string;
  quantumProtected: boolean;
  riskScore?: number;
}

export interface SwapFormData {
  fromToken: BridgeToken;
  toToken: BridgeToken;
  fromChain: ChainType;
  toChain: ChainType;
  amount: string;
  slippage: number;
  recipient?: string;
}

export interface BridgeSwapRequest {
  quoteId: string;
  fromChain: ChainType;
  toChain: ChainType;
  fromToken: string;
  toToken: string;
  amount: string;
  recipient: string;
  maxSlippage: number;
}

export interface BridgeSwapResponse {
  transactionId: string;
  status: TransactionStatus;
  fromTxHash?: string;
  toTxHash?: string;
  estimatedCompletion: string;
  quantumKeyId?: string;
}

export type TransactionStatus =
  | "pending"
  | "in_progress"
  | "validating"
  | "locked"
  | "processing"
  | "confirming"
  | "confirmed"
  | "completed"
  | "failed"
  | "cancelled"
  | "expired"
  | "refunded";

export interface TransactionProgress {
  transactionId: string;
  status: TransactionStatus;
  progress: number; // 0-100
  currentStep: string;
  steps: TransactionStep[];
  fromTxHash?: string;
  toTxHash?: string;
  errorMessage?: string;
  estimatedTimeRemaining?: number;
}

export interface TransactionStep {
  name: string;
  status: "pending" | "in_progress" | "completed" | "failed";
  description: string;
  txHash?: string;
  timestamp?: string;
}

export interface TransactionHistoryItem {
  id: string;
  fromChain: ChainType;
  toChain: ChainType;
  fromToken: string;
  toToken: string;
  fromAmount: string;
  toAmount: string;
  status: TransactionStatus;
  createdAt: string;
  completedAt?: string;
  fromTxHash?: string;
  toTxHash?: string;
  usdValue?: string;
}

// API Response types
export interface BridgeQuoteResponse {
  success: boolean;
  data?: BridgeQuote;
  error?: string;
}

export interface BridgeSwapResponseData {
  success: boolean;
  data?: BridgeSwapResponse;
  error?: string;
}

export interface BridgeHistoryResponse {
  success: boolean;
  data?: {
    transactions: TransactionHistoryItem[];
    total: number;
    page: number;
    pageSize: number;
  };
  error?: string;
}
