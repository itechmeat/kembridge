/**
 * Bridge History Hook
 * User transaction history with pagination
 * Only fetches when user is authenticated since endpoint requires auth
 */

import { useQuery, UseQueryResult } from "@tanstack/react-query";
import { bridgeService, SwapHistory } from "../../services/api/bridgeService";
import { useAuthStatus } from "../api/useAuth";

export const useBridgeHistory = (
  page: number = 1,
  pageSize: number = 20,
  options?: {
    enabled?: boolean;
  }
): UseQueryResult<SwapHistory, Error> => {
  const { isAuthenticated } = useAuthStatus();
  const queryKey = ["bridge-history", page, pageSize];

  return useQuery({
    queryKey,
    queryFn: () => bridgeService.getSwapHistory(page, pageSize),
    enabled: isAuthenticated && options?.enabled !== false,
    staleTime: 5 * 60 * 1000, // 5 minutes
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
};
