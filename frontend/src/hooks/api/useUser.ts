import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  userService,
  type UpdateProfileRequest,
} from "../../services/api/userService";
import { useWalletAuthStatus } from "./useAuth";

// Query keys for caching
export const USER_QUERY_KEYS = {
  PROFILE: "user-profile",
  STATISTICS: "user-statistics",
} as const;

/**
 * Hook for getting user profile
 */
export const useUserProfile = () => {
  const { hasAnyAuth } = useWalletAuthStatus();
  const [has401Error, setHas401Error] = useState(false);

  return useQuery({
    queryKey: [USER_QUERY_KEYS.PROFILE],
    queryFn: async () => {
      console.log("👤 User Profile: Fetching user profile...");
      return userService.getProfile();
    },
    enabled: hasAnyAuth && !has401Error, // Disabled on 401 error
    staleTime: 60000, // Cache for 1 minute
    refetchInterval: false, // Disable automatic refetching
    refetchOnMount: false, // Don't refetch on component mount
    refetchOnWindowFocus: false, // Don't refetch on window focus
    retry: (failureCount, error: unknown) => {
      const errorStatus = (error as { response?: { status?: number } })?.response?.status;
      if (errorStatus === 401) {
        console.log("🚫 User Profile: 401 error, stopping all requests");
        setHas401Error(true); // This will re-render and disable enabled
        return false;
      }
      return failureCount < 3;
    },
  });
};

/**
 * Hook for updating user profile
 */
export const useUpdateProfile = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (updates: UpdateProfileRequest) => {
      return userService.updateProfile(updates);
    },
    onSuccess: (data) => {
      console.log("✅ useUpdateProfile: Profile updated successfully");
      // Update profile cache
      queryClient.setQueryData([USER_QUERY_KEYS.PROFILE], data);
    },
    onError: (error) => {
      console.error("❌ useUpdateProfile: Failed to update profile:", error);
    },
  });
};

/**
 * Hook for getting user statistics
 * TODO: Implement proper statistics endpoint on backend
 */
export const useUserStatistics = () => {
  return useQuery({
    queryKey: [USER_QUERY_KEYS.STATISTICS],
    queryFn: async () => {
      return userService.getStatistics();
    },
    enabled: true, // Re-enabled since service now returns mock data
    staleTime: 300000, // Cache for 5 minutes
  });
};

/**
 * Hook for checking user tier
 */
export const useUserTier = () => {
  const { data: profile } = useUserProfile();

  return {
    tier: profile?.tier || "free",
    isPremium: profile ? userService.isPremiumUser(profile) : false,
    isAdmin: profile ? userService.isAdminUser(profile) : false,
    isFree: profile?.tier === "free",
  };
};

/**
 * Hook for working with user preferences
 */
export const useUserPreferences = () => {
  const { data: profile } = useUserProfile();
  const updateProfile = useUpdateProfile();

  const updatePreferences = async (
    preferences: UpdateProfileRequest["preferences"]
  ) => {
    return updateProfile.mutateAsync({ preferences });
  };

  return {
    preferences: profile?.preferences,
    isQuantumProtectionEnabled: profile
      ? userService.isQuantumProtectionEnabled(profile)
      : true,
    defaultSlippage: profile ? userService.getDefaultSlippage(profile) : 0.5,
    areNotificationsEnabled: profile
      ? userService.areNotificationsEnabled(profile)
      : true,
    updatePreferences,
    isUpdating: updateProfile.isPending,
  };
};

/**
 * Hook for working with user risk profile
 */
export const useUserRisk = () => {
  const { data: profile } = useUserProfile();

  // Try to get real risk data from new API first
  const newRiskProfileQuery = useQuery({
    queryKey: ["risk", "profile", "current"],
    queryFn: async () => {
      const { RiskService } = await import("../../services/security");
      return RiskService.getUserRiskProfile();
    },
    staleTime: 60000, // 1 minute
    retry: 1, // Only try once, fallback to old data if failed
  });

  // Use new API data if available, fallback to profile data
  if (newRiskProfileQuery.data && !newRiskProfileQuery.isError) {
    return {
      riskProfile: newRiskProfileQuery.data,
      riskLevel: newRiskProfileQuery.data.overall_risk_level,
      riskScore: newRiskProfileQuery.data.avg_risk_score,
      hasRiskData: true,
      transactionCount: newRiskProfileQuery.data.transaction_count,
      isLoading: newRiskProfileQuery.isLoading,
    };
  }

  // Fallback to old profile data
  return {
    riskProfile: profile?.risk_profile,
    riskLevel: profile ? userService.getRiskLevel(profile) : "unknown",
    riskScore: profile ? userService.getRiskScore(profile) : 0,
    hasRiskData: !!profile?.risk_profile,
    transactionCount: 0,
    isLoading: newRiskProfileQuery.isLoading,
  };
};

/**
 * Hook for working with user wallet addresses
 */
export const useUserWallets = () => {
  const { data: profile } = useUserProfile();

  return {
    walletAddresses: profile?.wallet_addresses || [],
    primaryWallet: profile
      ? userService.getPrimaryWalletAddress(profile)
      : null,
    walletCount: profile?.wallet_addresses.length || 0,
    formatWalletAddress: userService.formatWalletAddress,
  };
};

/**
 * Hook for getting complete user information
 */
export const useUserInfo = () => {
  const profileQuery = useUserProfile();
  const statisticsQuery = useUserStatistics();
  const { tier, isPremium, isAdmin } = useUserTier();
  const { riskLevel, riskScore, transactionCount } = useUserRisk();
  const { walletAddresses, primaryWallet } = useUserWallets();

  return {
    // Data
    profile: profileQuery.data,
    statistics: statisticsQuery.data,

    // Loading states
    isLoading: profileQuery.isLoading || statisticsQuery.isLoading,
    isError: profileQuery.isError || statisticsQuery.isError,
    error: profileQuery.error || statisticsQuery.error,

    // User information
    tier,
    isPremium,
    isAdmin,
    riskLevel,
    riskScore,
    walletAddresses,
    primaryWallet,
    transactionCount,

    // Methods for updating
    refetch: () => {
      profileQuery.refetch();
      statisticsQuery.refetch();
    },
  };
};
