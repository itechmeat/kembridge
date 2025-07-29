/**
 * Transaction Status Hook
 * Real-time status updates with polling
 * Only fetches when user is authenticated since endpoint requires auth
 */

import { useQuery, UseQueryResult } from "@tanstack/react-query";
import {
  bridgeService,
  SwapTransaction,
} from "../../services/api/bridgeService";
import { useAuthStatus } from "../api/useAuth";

export const useTransactionStatus = (
  transactionId: string,
  options?: {
    enabled?: boolean;
    refetchInterval?: number;
  }
): UseQueryResult<SwapTransaction, Error> => {
  const { isAuthenticated } = useAuthStatus();
  const queryKey = ["transaction-status", transactionId];

  const query = useQuery({
    queryKey,
    queryFn: () => bridgeService.getSwapStatus(transactionId),
    enabled: isAuthenticated && !!(transactionId && options?.enabled !== false),
    refetchInterval: options?.refetchInterval || 5000,
    retry: (failureCount, error) => {
      // Don't retry if transaction not found
      if (error.message?.includes("not found")) {
        return false;
      }
      return failureCount < 3;
    },
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });

  return query;
};
