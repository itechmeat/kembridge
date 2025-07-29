import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { RiskService, type TransactionRiskRequest } from '../../services/security';

export const useRiskAnalysis = (
  request: TransactionRiskRequest | null,
  options?: {
    enabled?: boolean;
    realTime?: boolean;
  }
) => {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ['risk', 'analysis', request],
    queryFn: () => RiskService.analyzeTransactionRisk(request!),
    enabled: options?.enabled !== false && !!request,
    refetchInterval: options?.realTime ? 5000 : false, // Real-time updates every 5 seconds
    staleTime: options?.realTime ? 1000 : 30000, // Stale time based on real-time mode
    retry: 2,
    retryDelay: 1000,
  });

  const analyzeRisk = useMutation({
    mutationFn: (newRequest: TransactionRiskRequest) => 
      RiskService.analyzeTransactionRisk(newRequest),
    onSuccess: (data, variables) => {
      // Update cache with new analysis
      queryClient.setQueryData(['risk', 'analysis', variables], data);
    },
  });

  return {
    ...query,
    riskAnalysis: query.data,
    analyzeRisk: analyzeRisk.mutate,
    isAnalyzing: analyzeRisk.isPending || query.isFetching,
    analysisError: query.error || analyzeRisk.error,
    riskScore: query.data?.riskScore.value ?? 0,
    riskLevel: query.data?.riskScore.level ?? 'low',
    isHighRisk: (query.data?.riskScore.value ?? 0) > 0.7,
    isMediumRisk: (query.data?.riskScore.value ?? 0) > 0.3 && (query.data?.riskScore.value ?? 0) <= 0.7,
    isLowRisk: (query.data?.riskScore.value ?? 0) <= 0.3,
    recommendations: query.data?.recommendations ?? [],
    isBlacklisted: query.data?.blacklistStatus.isBlacklisted ?? false,
  };
};

export const useUserRiskProfile = (userId?: string) => {
  const query = useQuery({
    queryKey: ['risk', 'profile', userId || 'current'],
    queryFn: () => RiskService.getUserRiskProfile(userId),
    staleTime: 60000, // 1 minute
    retry: 2,
  });

  return {
    ...query,
    riskProfile: query.data,
    currentRiskScore: query.data?.currentRiskScore.value ?? 0,
    riskHistory: query.data?.riskHistory ?? [],
    totalTransactions: query.data?.totalTransactions ?? 0,
    avgRiskScore: query.data?.avgRiskScore ?? 0,
  };
};

export const useRiskThresholds = () => {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ['risk', 'thresholds'],
    queryFn: RiskService.getRiskThresholds,
    staleTime: 300000, // 5 minutes
  });

  const updateThresholds = useMutation({
    mutationFn: RiskService.updateRiskThresholds,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['risk', 'thresholds'] });
    },
  });

  return {
    ...query,
    thresholds: query.data,
    updateThresholds: updateThresholds.mutate,
    isUpdating: updateThresholds.isPending,
    updateError: updateThresholds.error,
  };
};

export const useBlacklistCheck = () => {
  const checkBlacklist = useMutation({
    mutationFn: ({ address, chain }: { address: string; chain: string }) =>
      RiskService.checkAddressBlacklist(address, chain),
  });

  return {
    checkBlacklist: checkBlacklist.mutate,
    isChecking: checkBlacklist.isPending,
    blacklistResult: checkBlacklist.data,
    checkError: checkBlacklist.error,
    isBlacklisted: checkBlacklist.data?.isBlacklisted ?? false,
  };
};

export default useRiskAnalysis;