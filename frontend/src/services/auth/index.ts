/**
 * Authentication service
 * Integrates wallet signing with backend authentication
 */

import {
  apiClient,
  type VerifyWalletRequest,
  type UserProfile,
} from "../api/client";
import { WalletType } from "../wallet/types";

export interface AuthenticateWalletParams {
  walletAddress: string;
  walletType: WalletType;
  signMessage: (message: string) => Promise<string>;
}

export interface AuthenticationResult {
  success: boolean;
  user?: UserProfile;
  error?: string;
}

export class AuthService {
  /**
   * Authenticate wallet with backend
   */
  async authenticateWallet(
    params: AuthenticateWalletParams
  ): Promise<AuthenticationResult> {
    try {
      const { walletAddress, walletType, signMessage } = params;

      // Step 1: Get nonce from backend
      console.log("Getting nonce for wallet:", walletAddress);
      const chainType = walletType === WalletType.NEAR ? "near" : "ethereum";
      const nonceResponse = await apiClient.getNonce(walletAddress, chainType);

      // Step 2: Use message from backend
      const message = nonceResponse.message;

      // Step 3: Sign message with wallet
      console.log("Signing authentication message...");
      const signature = await signMessage(message);

      // Step 4: Verify signature with backend
      const verifyRequest: VerifyWalletRequest = {
        wallet_address: walletAddress,
        signature,
        nonce: nonceResponse.nonce,
        wallet_type: this.mapWalletType(walletType),
      };

      console.log("Verifying wallet signature with backend...");
      await apiClient.verifyWallet(verifyRequest);

      // Step 5: Get user profile
      const userProfile = await apiClient.getUserProfile();

      return {
        success: true,
        user: userProfile,
      };
    } catch (error) {
      console.error("Wallet authentication failed:", error);

      let errorMessage = "Authentication failed";
      if (error && typeof error === "object" && "message" in error) {
        errorMessage = error.message as string;
      }

      return {
        success: false,
        error: errorMessage,
      };
    }
  }

  /**
   * Check if user is currently authenticated
   */
  isAuthenticated(): boolean {
    return apiClient.isAuthenticated();
  }

  /**
   * Get current user profile
   */
  async getCurrentUser(): Promise<UserProfile | null> {
    if (!this.isAuthenticated()) {
      return null;
    }

    try {
      return await apiClient.getUserProfile();
    } catch (error) {
      console.error("Failed to get current user:", error);
      return null;
    }
  }

  /**
   * Logout user
   */
  async logout(): Promise<void> {
    await apiClient.logout();
  }

  /**
   * Update user profile
   */
  async updateProfile(
    updates: Partial<UserProfile>
  ): Promise<UserProfile | null> {
    try {
      return await apiClient.updateUserProfile(updates);
    } catch (error) {
      console.error("Failed to update profile:", error);
      return null;
    }
  }

  /**
   * Check backend health
   */
  async checkBackendHealth(): Promise<boolean> {
    try {
      await apiClient.getHealth();
      return true;
    } catch (error) {
      console.warn("Backend health check failed:", error);
      return false;
    }
  }

  /**
   * Map wallet type to backend format
   */
  private mapWalletType(walletType: WalletType): "ethereum" | "near" {
    switch (walletType) {
      case WalletType.METAMASK:
      case WalletType.COINBASE:
      case WalletType.WALLET_CONNECT:
        return "ethereum";
      case WalletType.NEAR:
        return "near";
      default:
        throw new Error(`Unsupported wallet type: ${walletType}`);
    }
  }
}

// Export singleton instance
export const authService = new AuthService();
export default authService;
