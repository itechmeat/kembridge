/**
 * User Service
 * User profile and account management
 */

import apiClient from "./apiClient";
import { API_ENDPOINTS } from "./config";

// Type definitions for user API
export interface UserProfile {
  id: string;
  wallet_addresses: string[];
  tier: "free" | "premium" | "admin";
  risk_profile?: {
    score: number;
    level: "low" | "medium" | "high";
    last_updated: string;
  };
  preferences?: {
    default_slippage?: number;
    notifications_enabled?: boolean;
    quantum_protection_enabled?: boolean;
  };
  statistics?: {
    total_transactions: number;
    total_volume_usd: number;
    last_transaction_at?: string;
  };
  created_at: string;
  updated_at: string;
}

export interface UpdateProfileRequest {
  preferences?: {
    default_slippage?: number;
    notifications_enabled?: boolean;
    quantum_protection_enabled?: boolean;
  };
}

export interface UserStatistics {
  total_transactions: number;
  total_volume_usd: number;
  successful_transactions: number;
  failed_transactions: number;
  average_transaction_value: number;
  last_transaction_at?: string;
  monthly_volume: number;
  transaction_count_by_month: Array<{
    month: string;
    count: number;
    volume: number;
  }>;
}

class UserService {
  /**
   * Gets the profile of the current user
   */
  async getProfile(): Promise<UserProfile> {
    console.log("👤 User Service: Getting user profile");

    const response = await apiClient.get<UserProfile>(
      API_ENDPOINTS.USER.PROFILE
    );

    console.log("✅ User Service: Profile received:", {
      id: response.id,
      tier: response.tier,
      walletCount: response.wallet_addresses.length,
      riskScore: response.risk_profile?.score,
    });

    return response;
  }

  /**
   * Updates the user's profile
   */
  async updateProfile(updates: UpdateProfileRequest): Promise<UserProfile> {
    console.log("📝 User Service: Updating user profile:", updates);

    const response = await apiClient.put<UserProfile>(
      API_ENDPOINTS.USER.UPDATE_PROFILE,
      updates
    );

    console.log("✅ User Service: Profile updated successfully");
    return response;
  }

  /**
   * Gets the user's statistics
   */
  async getStatistics(): Promise<UserStatistics> {
    console.log("📊 User Service: Getting user statistics");

    const response = await apiClient.get<UserStatistics>(
      `${API_ENDPOINTS.USER.PROFILE}/statistics`
    );

    console.log("✅ User Service: Statistics received:", {
      totalTransactions: response.total_transactions,
      totalVolume: response.total_volume_usd,
      successRate:
        response.total_transactions > 0
          ? (
              (response.successful_transactions / response.total_transactions) *
              100
            ).toFixed(1) + "%"
          : "0%",
    });

    return response;
  }

  /**
   * Checks if the user is a premium user
   */
  isPremiumUser(profile: UserProfile): boolean {
    return profile.tier === "premium" || profile.tier === "admin";
  }

  /**
   * Checks if the user is an admin
   */
  isAdminUser(profile: UserProfile): boolean {
    return profile.tier === "admin";
  }

  /**
   * Gets the user's risk level
   */
  getRiskLevel(profile: UserProfile): "low" | "medium" | "high" | "unknown" {
    return profile.risk_profile?.level || "unknown";
  }

  /**
   * Gets the user's risk score
   */
  getRiskScore(profile: UserProfile): number {
    return profile.risk_profile?.score || 0;
  }

  /**
   * Formats the display of wallet addresses
   */
  formatWalletAddress(address: string): string {
    if (address.length <= 10) return address;
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
  }

  /**
   * Gets the user's primary wallet address
   */
  getPrimaryWalletAddress(profile: UserProfile): string | null {
    return profile.wallet_addresses.length > 0
      ? profile.wallet_addresses[0]
      : null;
  }

  /**
   * Checks if quantum protection is enabled
   */
  isQuantumProtectionEnabled(profile: UserProfile): boolean {
    return profile.preferences?.quantum_protection_enabled ?? true; // by default enabled
  }

  /**
   * Gets the default slippage setting
   */
  getDefaultSlippage(profile: UserProfile): number {
    return profile.preferences?.default_slippage ?? 0.5; // 0.5% by default
  }

  /**
   * Checks if notifications are enabled
   */
  areNotificationsEnabled(profile: UserProfile): boolean {
    return profile.preferences?.notifications_enabled ?? true; // by default enabled
  }
}

// Create singleton instance
export const userService = new UserService();

// Export for use in components
export default userService;
