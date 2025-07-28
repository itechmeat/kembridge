/**
 * Bridge Service
 * Cross-chain bridge operations and swap management
 */

import apiClient from "./apiClient";
import { API_ENDPOINTS } from "./config";

// Type definitions for bridge API
export interface SwapQuoteRequest {
  from_chain: "ethereum" | "near";
  to_chain: "ethereum" | "near";
  from_token: string;
  to_token: string;
  amount: string;
  slippage?: number;
  quantum_protection?: boolean;
}

export interface SwapQuote {
  quote_id: string;
  from_amount: string;
  to_amount: string;
  exchange_rate: number;
  price_impact: number;
  estimated_fees: {
    bridge_fee: string;
    gas_fee: string;
    protocol_fee: string;
    total_fee: string;
  };
  estimated_time_minutes: number;
  expires_at: string;
  quantum_protection_enabled: boolean;
  route_info: {
    strategy: string;
    confidence: number;
    risk_score: number;
  };
}

export interface InitSwapRequest {
  quote_id: string;
  from_wallet_address: string;
  to_wallet_address: string;
  return_address?: string;
  max_slippage?: number;
}

export interface SwapTransaction {
  id: string;
  quote_id: string;
  status: "pending" | "confirmed" | "completed" | "failed" | "expired";
  from_chain: string;
  to_chain: string;
  from_token: string;
  to_token: string;
  from_amount: string;
  to_amount: string;
  from_wallet_address: string;
  to_wallet_address: string;
  from_transaction_hash?: string;
  to_transaction_hash?: string;
  created_at: string;
  updated_at: string;
  estimated_completion_at?: string;
  actual_completion_at?: string;
  risk_analysis?: {
    score: number;
    level: "low" | "medium" | "high";
    flags: string[];
  };
  quantum_protection_used: boolean;
}

export interface SwapHistory {
  transactions: SwapTransaction[];
  total_count: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export interface SupportedToken {
  symbol: string;
  name: string;
  address: string;
  decimals: number;
  chain: "ethereum" | "near";
  logo_url?: string;
  is_native: boolean;
}

class BridgeService {
  /**
   * Gets swap quote
   */
  async getSwapQuote(request: SwapQuoteRequest): Promise<SwapQuote> {
    console.log("💱 Bridge Service: Getting swap quote:", request);

    const response = await apiClient.post<SwapQuote>(
      API_ENDPOINTS.BRIDGE.QUOTE,
      request
    );

    console.log("✅ Bridge Service: Quote received:", {
      quoteId: response.quote_id,
      fromAmount: request.amount,
      toAmount: response.to_amount,
      exchangeRate: response.exchange_rate,
      priceImpact: response.price_impact,
      estimatedTime: response.estimated_time_minutes,
    });

    return response;
  }

  /**
   * Initiates swap operation
   */
  async initSwap(request: InitSwapRequest): Promise<SwapTransaction> {
    console.log("🚀 Bridge Service: Initiating swap:", request);

    const response = await apiClient.post<SwapTransaction>(
      API_ENDPOINTS.BRIDGE.INIT_SWAP,
      request
    );

    console.log("✅ Bridge Service: Swap initiated:", {
      transactionId: response.id,
      status: response.status,
      fromChain: response.from_chain,
      toChain: response.to_chain,
    });

    return response;
  }

  /**
   * Gets transaction status
   */
  async getSwapStatus(transactionId: string): Promise<SwapTransaction> {
    console.log("🔍 Bridge Service: Getting swap status for:", transactionId);

    const response = await apiClient.get<SwapTransaction>(
      `${API_ENDPOINTS.BRIDGE.STATUS}/${transactionId}`
    );

    console.log("📊 Bridge Service: Status received:", {
      id: response.id,
      status: response.status,
      fromTxHash: response.from_transaction_hash,
      toTxHash: response.to_transaction_hash,
    });

    return response;
  }

  /**
   * Gets user swap history
   */
  async getSwapHistory(
    page: number = 1,
    pageSize: number = 20
  ): Promise<SwapHistory> {
    console.log("📜 Bridge Service: Getting swap history:", { page, pageSize });

    const response = await apiClient.get<SwapHistory>(
      `${API_ENDPOINTS.BRIDGE.HISTORY}?page=${page}&page_size=${pageSize}`
    );

    console.log("✅ Bridge Service: History received:", {
      totalCount: response.total_count,
      transactionsCount: response.transactions.length,
      currentPage: response.page,
      totalPages: response.total_pages,
    });

    return response;
  }

  /**
   * Gets list of supported tokens
   */
  async getSupportedTokens(): Promise<SupportedToken[]> {
    console.log("🪙 Bridge Service: Getting supported tokens");

    const response = await apiClient.get<SupportedToken[]>(
      "/bridge/supported-tokens"
    );

    console.log("✅ Bridge Service: Supported tokens received:", {
      count: response.length,
      ethereumTokens: response.filter((t) => t.chain === "ethereum").length,
      nearTokens: response.filter((t) => t.chain === "near").length,
    });

    return response;
  }

  /**
   * Formats transaction status for display
   */
  formatTransactionStatus(status: SwapTransaction["status"]): string {
    const statusMap = {
      pending: "Pending",
      confirmed: "Confirmed",
      completed: "Completed",
      failed: "Failed",
      expired: "Expired",
    };
    return statusMap[status] || status;
  }

  /**
   * Gets status color for UI
   */
  getStatusColor(status: SwapTransaction["status"]): string {
    const colorMap = {
      pending: "#ffa500", // orange
      confirmed: "#4169e1", // blue
      completed: "#32cd32", // green
      failed: "#ff4444", // red
      expired: "#888888", // gray
    };
    return colorMap[status] || "#888888";
  }

  /**
   * Checks if transaction is completed
   */
  isTransactionCompleted(status: SwapTransaction["status"]): boolean {
    return status === "completed";
  }

  /**
   * Checks if transaction failed
   */
  isTransactionFailed(status: SwapTransaction["status"]): boolean {
    return status === "failed" || status === "expired";
  }

  /**
   * Gets transaction completion percentage
   */
  getTransactionProgress(transaction: SwapTransaction): number {
    switch (transaction.status) {
      case "pending":
        return 25;
      case "confirmed":
        return 75;
      case "completed":
        return 100;
      case "failed":
      case "expired":
        return 0;
      default:
        return 0;
    }
  }

  /**
   * Formats token amount for display
   */
  formatTokenAmount(amount: string, decimals: number = 18): string {
    const num = parseFloat(amount) / Math.pow(10, decimals);
    if (num < 0.001) return "<0.001";
    if (num < 1) return num.toFixed(6);
    if (num < 1000) return num.toFixed(4);
    return num.toLocaleString(undefined, { maximumFractionDigits: 2 });
  }

  /**
   * Calculates total fees cost in USD
   */
  calculateTotalFeesUsd(
    fees: SwapQuote["estimated_fees"],
    ethPriceUsd: number = 3000
  ): Promise<number> {
    // TODO: Implement via price oracle service
    // For now using approximate value
    const totalEth = parseFloat(fees.total_fee) / Math.pow(10, 18);
    return Promise.resolve(totalEth * ethPriceUsd);
  }
}

// Create singleton instance
export const bridgeService = new BridgeService();

// Export for use in components
export default bridgeService;
