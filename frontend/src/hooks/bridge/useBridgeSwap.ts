/**
 * Bridge Swap Hook
 * Swap execution with mutation handling
 */

import {
  useMutation,
  UseMutationResult,
  useQueryClient,
} from "@tanstack/react-query";
import {
  bridgeService,
  InitSwapRequest,
  SwapTransaction,
} from "../../services/api/bridgeService";

export const useBridgeSwap = (): UseMutationResult<
  SwapTransaction,
  Error,
  InitSwapRequest
> => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: InitSwapRequest) => bridgeService.initSwap(request),
    onSuccess: (data) => {
      // Invalidate and refetch relevant queries
      queryClient.invalidateQueries({ queryKey: ["bridge-history"] });
      queryClient.setQueryData(["transaction-status", data.id], data);

      console.log("Bridge swap initiated successfully:", data);
    },
    onError: (error) => {
      console.error("Bridge swap failed:", error);
    },
  });
};
