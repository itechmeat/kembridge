import { useQuery, useQueryClient } from '@tanstack/react-query';
import { SecurityService } from '../../services/security';
import type { SecurityStatus } from '../../types/security';

export const useSecurityStatus = (options?: {
  refetchInterval?: number;
  enabled?: boolean;
}) => {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ['security', 'status'],
    queryFn: SecurityService.getSecurityStatus,
    refetchInterval: options?.refetchInterval ?? 10000, // 10 seconds
    enabled: options?.enabled ?? true,
    staleTime: 5000, // Data is fresh for 5 seconds
    retry: 3,
    retryDelay: attemptIndex => Math.min(1000 * 2 ** attemptIndex, 30000),
  });

  const refreshSecurityStatus = () => {
    queryClient.invalidateQueries({ queryKey: ['security', 'status'] });
  };

  const updateSecurityStatus = (updater: (old: SecurityStatus) => SecurityStatus) => {
    queryClient.setQueryData(['security', 'status'], updater);
  };

  return {
    ...query,
    securityStatus: query.data,
    refreshSecurityStatus,
    updateSecurityStatus,
    isOnline: query.data?.isOnline ?? false,
    quantumProtection: query.data?.quantumProtection.isActive ?? false,
    securityLevel: query.data?.overall,
  };
};

export default useSecurityStatus;