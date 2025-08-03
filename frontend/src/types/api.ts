import { BaseResponse, Token } from "./common";

export interface SwapQuoteRequest {
  fromToken: string;
  toToken: string;
  amount: string;
  fromChain: string;
  toChain: string;
  slippageTolerance?: number;
  quantumProtection?: boolean;
}

export interface SwapQuoteResponse {
  fromToken: Token;
  toToken: Token;
  fromAmount: string;
  toAmount: string;
  estimatedOutput: string;
  minimumOutput: string;
  fees: SwapFees;
  riskScore: number;
  quantumProtection: boolean;
  route: SwapRoute[];
  validUntil: number;
  priceImpact: number;
}

export interface SwapFees {
  networkFee: string;
  protocolFee: string;
  quantumFee?: string;
  totalFee: string;
  feeToken: Token;
}

export interface SwapRoute {
  protocol: string;
  percentage: number;
  fromToken: Token;
  toToken: Token;
}

export interface SwapExecuteRequest {
  quoteId: string;
  userAddress: string;
  slippageTolerance: number;
  deadline: number;
}

export interface SwapExecuteResponse {
  transactionHash: string;
  status: "pending" | "confirmed" | "failed";
  fromTxHash?: string;
  toTxHash?: string;
  bridgeId: string;
}

export interface SwapStatus {
  id: string;
  status: "pending" | "processing" | "completed" | "failed";
  fromTxHash?: string;
  toTxHash?: string;
  createdAt: number;
  updatedAt: number;
  estimatedCompletionTime?: number;
}

export interface PriceData {
  token: Token;
  price: string;
  priceChange24h: number;
  volume24h: string;
  marketCap: string;
  lastUpdated: number;
}

export interface RiskAnalysis {
  score: number;
  level: "low" | "medium" | "high";
  factors: RiskFactor[];
  recommendations: string[];
}

export interface RiskFactor {
  type: string;
  severity: "low" | "medium" | "high";
  description: string;
  impact: number;
}

export interface UserProfile {
  id: string;
  address: string;
  createdAt: number;
  lastLoginAt: number;
  swapCount: number;
  totalVolume: string;
  riskProfile: RiskAnalysis;
}

// API Response wrappers
export interface SwapQuoteApiResponse extends BaseResponse {
  data?: SwapQuoteResponse;
}

export interface SwapExecuteApiResponse extends BaseResponse {
  data?: SwapExecuteResponse;
}

export interface SwapStatusApiResponse extends BaseResponse {
  data?: SwapStatus;
}

export interface PriceDataApiResponse extends BaseResponse {
  data?: PriceData[];
}

export interface UserProfileApiResponse extends BaseResponse {
  data?: UserProfile;
}
