import { useQuery } from "@tanstack/react-query";
import { RiskService } from "../../services/security";

export const useUserRiskProfile = (userId?: string) => {
  const query = useQuery({
    queryKey: ["risk", "profile", userId || "current"],
    queryFn: () => RiskService.getUserRiskProfile(userId),
    refetchInterval: 300000, // 5 minutes
    staleTime: 60000, // 1 minute
    retry: 3,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });

  return {
    ...query,
    riskProfile: query.data,
    riskScore: query.data?.avg_risk_score || 0,
    transactionCount: query.data?.transaction_count || 0,
    riskLevel: query.data?.overall_risk_level || "unknown",
    highRiskTransactions: query.data?.high_risk_transactions || 0,
    lastAnalysisDate: query.data?.last_analysis_date,
    isLoading: query.isLoading,
    isError: query.isError,
    error: query.error,
  };
};

export default useUserRiskProfile;
