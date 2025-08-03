import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  bridgeService,
  type SwapQuoteRequest,
  type InitSwapRequest,
  type SwapTransaction,
} from "../../services/api/bridgeService";

// Query keys for caching
export const BRIDGE_QUERY_KEYS = {
  QUOTE: "bridge-quote",
  TRANSACTION: "bridge-transaction",
  HISTORY: "bridge-history",
  SUPPORTED_TOKENS: "bridge-supported-tokens",
} as const;

/**
 * Hook for getting swap quote
 */
export const useSwapQuote = () => {
  return useMutation({
    mutationFn: async (request: SwapQuoteRequest) => {
      return bridgeService.getSwapQuote(request);
    },
    onSuccess: (data) => {
      console.log(
        "✅ useSwapQuote: Quote received successfully:",
        data.quote_id
      );
    },
    onError: (error) => {
      console.error("❌ useSwapQuote: Failed to get quote:", error);
    },
  });
};

/**
 * Hook for initiating swap
 */
export const useInitSwap = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (request: InitSwapRequest) => {
      return bridgeService.initSwap(request);
    },
    onSuccess: (data) => {
      console.log("✅ useInitSwap: Swap initiated successfully:", data.id);
      // Invalidate history for updates
      queryClient.invalidateQueries({ queryKey: [BRIDGE_QUERY_KEYS.HISTORY] });
    },
    onError: (error) => {
      console.error("❌ useInitSwap: Failed to initiate swap:", error);
    },
  });
};

/**
 * Hook for getting transaction status
 */
export const useSwapStatus = (
  transactionId: string | null,
  enabled: boolean = true
) => {
  return useQuery({
    queryKey: [BRIDGE_QUERY_KEYS.TRANSACTION, transactionId],
    queryFn: async () => {
      if (!transactionId) throw new Error("Transaction ID is required");
      return bridgeService.getSwapStatus(transactionId);
    },
    enabled: enabled && !!transactionId,
    refetchInterval: (query) => {
      // Auto-refresh every 5 seconds for active transactions
      if (
        query.state.data?.status === "pending" ||
        query.state.data?.status === "confirmed"
      ) {
        return 5000; // 5 seconds
      }
      return false; // Don't refresh for completed transactions
    },
    staleTime: 1000, // Data becomes stale after 1 second
  });
};

/**
 * Hook for getting swap history
 */
export const useSwapHistory = (page: number = 1, pageSize: number = 20) => {
  return useQuery({
    queryKey: [BRIDGE_QUERY_KEYS.HISTORY, page, pageSize],
    queryFn: async () => {
      return bridgeService.getSwapHistory(page, pageSize);
    },
    staleTime: 30000, // Cache for 30 seconds
  });
};

/**
 * Hook for getting supported tokens
 */
export const useSupportedTokens = () => {
  return useQuery({
    queryKey: [BRIDGE_QUERY_KEYS.SUPPORTED_TOKENS],
    queryFn: async () => {
      return bridgeService.getSupportedTokens();
    },
    staleTime: 300000, // Cache for 5 minutes
    gcTime: 600000, // Keep in cache for 10 minutes
  });
};

/**
 * Hook for monitoring multiple transactions
 */
export const useMultipleSwapStatus = (transactionIds: string[]) => {
  return useQuery({
    queryKey: [BRIDGE_QUERY_KEYS.TRANSACTION, "multiple", transactionIds],
    queryFn: async () => {
      const promises = transactionIds.map((id) =>
        bridgeService.getSwapStatus(id)
      );
      return Promise.all(promises);
    },
    enabled: transactionIds.length > 0,
    refetchInterval: (query) => {
      // Check if there are active transactions
      const hasActiveTransactions = query.state.data?.some(
        (tx: SwapTransaction) =>
          tx.status === "pending" || tx.status === "confirmed"
      );
      return hasActiveTransactions ? 5000 : false;
    },
    staleTime: 1000,
  });
};

/**
 * Utility hook for working with data formatting
 */
export const useBridgeUtils = () => {
  return {
    formatTransactionStatus: bridgeService.formatTransactionStatus,
    getStatusColor: bridgeService.getStatusColor,
    isTransactionCompleted: bridgeService.isTransactionCompleted,
    isTransactionFailed: bridgeService.isTransactionFailed,
    getTransactionProgress: bridgeService.getTransactionProgress,
    formatTokenAmount: bridgeService.formatTokenAmount,
  };
};

/**
 * Hook for tracking specific transaction until completion
 */
export const useTrackTransaction = (transactionId: string | null) => {
  const { data: transaction, isLoading, error } = useSwapStatus(transactionId);

  const isCompleted = transaction
    ? bridgeService.isTransactionCompleted(transaction.status)
    : false;
  const isFailed = transaction
    ? bridgeService.isTransactionFailed(transaction.status)
    : false;
  const progress = transaction
    ? bridgeService.getTransactionProgress(transaction)
    : 0;

  return {
    transaction,
    isLoading,
    error,
    isCompleted,
    isFailed,
    progress,
    status: transaction?.status,
  };
};
