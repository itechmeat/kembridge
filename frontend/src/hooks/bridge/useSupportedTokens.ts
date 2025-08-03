import { useQuery, UseQueryResult } from "@tanstack/react-query";
import {
  bridgeService,
  SupportedToken,
} from "../../services/api/bridgeService";
import { useAuthStatus } from "../api/useAuth";

export const useSupportedTokens = (options?: {
  enabled?: boolean;
}): UseQueryResult<SupportedToken[], Error> => {
  const { isAuthenticated } = useAuthStatus();
  const queryKey = ["supported-tokens"];

  return useQuery({
    queryKey,
    queryFn: () => bridgeService.getSupportedTokens(),
    enabled: isAuthenticated && options?.enabled !== false,
    staleTime: 10 * 60 * 1000, // 10 minutes - tokens don't change often
    gcTime: 30 * 60 * 1000, // 30 minutes
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
};
