/**
 * User Hooks
 * React hooks for user profile and account management with TanStack Query
 */

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  userService,
  type UpdateProfileRequest,
} from "../../services/api/userService";
import { useAuthStatus } from "./useAuth";

// Query keys for caching
export const USER_QUERY_KEYS = {
  PROFILE: "user-profile",
  STATISTICS: "user-statistics",
} as const;

/**
 * Hook for getting user profile
 */
export const useUserProfile = () => {
  const { isAuthenticated } = useAuthStatus();

  return useQuery({
    queryKey: [USER_QUERY_KEYS.PROFILE],
    queryFn: async () => {
      console.log("👤 User Profile: Fetching user profile...");
      return userService.getProfile();
    },
    enabled: isAuthenticated, // Only fetch if user is authenticated
    staleTime: 60000, // Cache for 1 minute
    retry: (failureCount, error: unknown) => {
      // Don't retry on 401 errors
      if ((error as { response?: { status?: number } })?.response?.status === 401) {
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
 */
export const useUserStatistics = () => {
  const { isAuthenticated } = useAuthStatus();

  return useQuery({
    queryKey: [USER_QUERY_KEYS.STATISTICS],
    queryFn: async () => {
      return userService.getStatistics();
    },
    enabled: isAuthenticated,
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

  return {
    riskProfile: profile?.risk_profile,
    riskLevel: profile ? userService.getRiskLevel(profile) : "unknown",
    riskScore: profile ? userService.getRiskScore(profile) : 0,
    hasRiskData: !!profile?.risk_profile,
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
  const { riskLevel, riskScore } = useUserRisk();
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

    // Methods for updating
    refetch: () => {
      profileQuery.refetch();
      statisticsQuery.refetch();
    },
  };
};
