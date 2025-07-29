/**
 * Bridge Quote Hook
 * Quote management with TanStack Query integration
 * Only fetches when user is authenticated since endpoint requires auth
 */

import { useQuery, UseQueryResult } from "@tanstack/react-query";
import {
  bridgeService,
  SwapQuoteRequest,
  SwapQuote,
} from "../../services/api/bridgeService";
import { useAuthStatus } from "../api/useAuth";

export interface BridgeQuoteParams {
  fromToken?: string;
  toToken?: string;
  fromChain?: "ethereum" | "near";
  toChain?: "ethereum" | "near";
  amount?: string;
  slippage?: number;
  quantumProtection?: boolean;
}

export const useBridgeQuote = (
  params: BridgeQuoteParams,
  options?: {
    enabled?: boolean;
    refetchInterval?: number;
  }
): UseQueryResult<SwapQuote, Error> => {
  const { isAuthenticated } = useAuthStatus();
  const queryKey = ["bridge-quote", params];

  return useQuery({
    queryKey,
    queryFn: async () => {
      if (
        !params.fromToken ||
        !params.toToken ||
        !params.amount ||
        !params.fromChain ||
        !params.toChain
      ) {
        throw new Error("Missing required quote parameters");
      }

      const request: SwapQuoteRequest = {
        from_token: params.fromToken,
        to_token: params.toToken,
        from_chain: params.fromChain,
        to_chain: params.toChain,
        amount: params.amount,
        slippage: params.slippage,
        quantum_protection: params.quantumProtection,
      };

      return bridgeService.getSwapQuote(request);
    },
    enabled:
      isAuthenticated &&
      !!(
        params.fromToken &&
        params.toToken &&
        params.amount &&
        params.fromChain &&
        params.toChain &&
        options?.enabled !== false
      ),
    refetchInterval: options?.refetchInterval || 30000, // 30 seconds
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
};
