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
  real_transaction_hash?: string; // Real blockchain transaction hash from MetaMask
}

export interface InitSwapResponse {
  swap_id: string;
  status: string;
  tx_preview: {
    bridge_fee_estimate: string;
    from: string;
    max_slippage: number;
    network_fee_estimate: string;
    protocol: string;
    quote_id: string;
    to: string;
  };
  expires_at: string;
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

    // Convert request to query parameters for GET request
    const params = new URLSearchParams({
      from_token: request.from_token,
      to_token: request.to_token,
      from_chain: request.from_chain,
      to_chain: request.to_chain,
      from_amount: request.amount,
    });

    const response = await apiClient.get<SwapQuote>(
      `${API_ENDPOINTS.BRIDGE.QUOTE}?${params.toString()}`
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
   * Maps API status to transaction status
   */
  private mapApiStatusToTransactionStatus(
    apiStatus: string
  ): SwapTransaction["status"] {
    const statusMap: Record<string, SwapTransaction["status"]> = {
      initiated: "pending",
      pending: "pending",
      confirmed: "confirmed",
      completed: "completed",
      failed: "failed",
      expired: "expired",
    };
    return statusMap[apiStatus] || "pending";
  }

  /**
   * Initiates swap operation
   */
  async initSwap(request: InitSwapRequest): Promise<SwapTransaction> {
    console.log("🚀 Bridge Service: Initiating REAL blockchain swap:", request);

    // 🔥 DIRECT TO 1INCH SERVICE for real blockchain transactions
    const oneinchUrl = "http://localhost:4001" + API_ENDPOINTS.BRIDGE.INIT_SWAP;
    console.log("📤 Sending to 1inch service:", oneinchUrl);

    // Transform frontend request to 1inch service format
    const oneinchRequest = {
      quote_id: request.quote_id,
      user_address: request.from_wallet_address,
      recipient_address: request.to_wallet_address,
      slippage: request.max_slippage || 0.5,
      real_transaction_hash: request.real_transaction_hash, // 🎯 REAL BLOCKCHAIN HASH!
    };

    console.log("🚀 Transformed request for 1inch service:", oneinchRequest);

    const response = await fetch(oneinchUrl, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(oneinchRequest),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`1inch service error: ${response.status} - ${errorText}`);
    }

    const rawResponse = await response.json();
    console.log("✅ Bridge Service: 1inch service response:", rawResponse);

    // 🔥 Handle 1inch service response format
    if (!rawResponse.success) {
      throw new Error(`1inch service error: ${rawResponse.error}`);
    }

    const data = rawResponse.data;

    // Convert 1inch service response to SwapTransaction format
    const swapTransaction: SwapTransaction = {
      id: data.order_hash || `swap_${Date.now()}`,
      quote_id: request.quote_id,
      status: data.status === "processing" ? "pending" : data.status,
      from_chain: "ethereum",
      to_chain: "near",
      from_token: "ETH",
      to_token: "USDT",
      from_amount: "0.001", // From MetaMask transaction
      to_amount: "0",
      from_wallet_address: request.from_wallet_address,
      to_wallet_address: request.to_wallet_address,
      from_transaction_hash: data.transaction_hash, // 🎉 REAL ETH BLOCKCHAIN HASH!
      to_transaction_hash: data.near_transaction_hash, // 🚀 REAL NEAR BLOCKCHAIN HASH!
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      estimated_completion_at: new Date(
        Date.now() + 5 * 60 * 1000
      ).toISOString(), // 5 minutes
      quantum_protection_used: true,
    };

    console.log("✅ Bridge Service: Swap initiated:", {
      transactionId: swapTransaction.id,
      status: swapTransaction.status,
      fromChain: swapTransaction.from_chain,
      toChain: swapTransaction.to_chain,
    });

    return swapTransaction;
  }

  /**
   * Executes a swap from SwapFormData (Phase 8.1.1)
   */
  async executeSwap(formData: {
    fromToken: { symbol: string; decimals?: number };
    toToken: { symbol: string; decimals?: number };
    fromChain: "ethereum" | "near";
    toChain: "ethereum" | "near";
    amount: string;
    recipient?: string;
    slippage?: number;
    fromWalletAddress?: string;
  }): Promise<{
    transaction_id: string;
    status: string;
    estimated_time_minutes: number;
    expires_at: string;
    next_steps: string[];
  }> {
    console.log("🔥 Bridge Service: Executing swap:", formData);

    // First get a quote to obtain quote_id
    const quote = await this.getSwapQuote({
      from_chain: formData.fromChain,
      to_chain: formData.toChain,
      from_token: formData.fromToken.symbol,
      to_token: formData.toToken.symbol,
      amount: formData.amount,
      slippage: formData.slippage,
      quantum_protection: true,
    });

    // Prepare swap request matching backend interface
    const swapRequest = {
      quote_id: quote.quote_id,
      from_wallet_address: formData.fromWalletAddress || "",
      to_wallet_address: formData.recipient || "",
      max_slippage: formData.slippage || 0.5,
    };

    console.log("📨 Bridge Service: Sending swap request:", swapRequest);

    const response = await apiClient.post<{
      transaction_id: string;
      status: string;
      estimated_time_minutes: number;
      expires_at: string;
      next_steps: string[];
    }>(API_ENDPOINTS.BRIDGE.INIT_SWAP, swapRequest);

    console.log("✅ Bridge Service: Swap executed successfully:", {
      transactionId: response.transaction_id,
      status: response.status,
      estimatedTime: response.estimated_time_minutes,
    });

    return response;
  }

  /**
   * Gets transaction status
   */
  async getSwapStatus(transactionId: string): Promise<SwapTransaction> {
    console.log("🔍 Bridge Service: Getting swap status for:", transactionId);

    const response = await apiClient.get<SwapTransaction>(
      `${API_ENDPOINTS.BRIDGE.TRANSACTION_STATUS}/${transactionId}`
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

    try {
      const response = await apiClient.get<SupportedToken[]>(
        API_ENDPOINTS.BRIDGE.SUPPORTED_TOKENS
      );

      console.log("✅ Bridge Service: Supported tokens received:", {
        count: response.length,
        ethereumTokens: response.filter((t) => t.chain === "ethereum").length,
        nearTokens: response.filter((t) => t.chain === "near").length,
      });

      return response;
    } catch (error) {
      console.error(
        "❌ Bridge Service: Failed to load supported tokens:",
        error
      );
      throw new Error(
        "Failed to load supported tokens. Please try again later."
      );
    }
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

    // Handle edge cases
    if (num === 0 || isNaN(num)) return "0.0";
    if (num < 0.000001) return "< 0.000001";
    if (num < 0.001) return num.toFixed(6);
    if (num < 1) return num.toFixed(4);
    if (num < 1000) return num.toFixed(3);

    return num.toLocaleString(undefined, {
      maximumFractionDigits: 2,
      minimumFractionDigits: 0,
    });
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
