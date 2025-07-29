/**
 * Authentication Service
 * Web3 wallet authentication with backend
 */

import apiClient from "./apiClient";
import { API_ENDPOINTS } from "./config";

// Type definitions for auth API
export interface NonceRequest {
  wallet_address: string;
  chain_type: "ethereum" | "near";
}

export interface NonceResponse {
  nonce: string;
  message: string;
  expires_at: string;
}

export interface VerifyWalletRequest {
  wallet_address: string;
  signature: string;
  nonce: string;
  chain_type: "ethereum" | "near";
  message: string;
}

export interface VerifyWalletResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
  user_id: string;
  wallet_address: string;
  chain_type: "ethereum" | "near";
}

export interface RefreshTokenResponse {
  token: string;
  expires_at: string;
}

class AuthService {
  /**
   * Gets nonce for message signing
   */
  async getNonce(
    walletAddress: string,
    chainType: "ethereum" | "near"
  ): Promise<NonceResponse> {
    console.log(
      "🔑 Auth Service: Getting nonce for wallet:",
      walletAddress,
      "chain:",
      chainType
    );

    const response = await apiClient.get<NonceResponse>(
      `${API_ENDPOINTS.AUTH.NONCE}?wallet_address=${walletAddress}&chain_type=${chainType}`
    );

    console.log("✅ Auth Service: Nonce received:", response);
    return response;
  }

  /**
   * Verifies wallet signature and gets JWT token
   */
  async verifyWallet(
    walletAddress: string,
    signature: string,
    nonce: string,
    chainType: "ethereum" | "near",
    message: string
  ): Promise<VerifyWalletResponse> {
    // Validate input parameters
    if (!signature) {
      throw new Error("Signature is required");
    }
    if (!walletAddress) {
      throw new Error("Wallet address is required");
    }
    if (!nonce) {
      throw new Error("Nonce is required");
    }

    console.log("🔐 Auth Service: Verifying wallet signature:", {
      walletAddress,
      chainType,
      nonce,
      signatureLength: signature?.length || 0,
    });

    const request: VerifyWalletRequest = {
      wallet_address: walletAddress,
      signature,
      nonce,
      chain_type: chainType,
      message,
    };

    const response = await apiClient.post<VerifyWalletResponse>(
      API_ENDPOINTS.AUTH.VERIFY_WALLET,
      request
    );

    // Debug: Log the entire response to see what we're getting
    console.log("🔍 Auth Service: Full response from backend:", {
      status: "success",
      responseKeys: Object.keys(response),
      responseData: response,
      hasToken: "token" in response,
      tokenValue: response.token,
    });

    // Save token in API client
    if (response.access_token) {
      apiClient.setAuthToken(response.access_token);
      console.log("✅ Auth Service: Authentication successful, token saved", {
        tokenLength: response.access_token.length,
        tokenPreview: response.access_token.substring(0, 20) + "...",
        tokenType: response.token_type,
        expiresIn: response.expires_in,
        userId: response.user_id,
      });

      // Verify token was actually saved
      const savedToken = apiClient.getAuthToken();
      const isAuth = apiClient.isAuthenticated();
      console.log("🔍 Auth Service: Token verification after save", {
        tokenSaved: !!savedToken,
        isAuthenticated: isAuth,
        tokensMatch: savedToken === response.access_token,
      });
    } else {
      console.error("❌ Auth Service: No access_token in response!", {
        responseType: typeof response,
        responseKeys: Object.keys(response || {}),
        fullResponse: response,
      });
    }

    return response;
  }

  /**
   * Updates JWT token
   */
  async refreshToken(): Promise<RefreshTokenResponse> {
    console.log("🔄 Auth Service: Refreshing token");

    const response = await apiClient.post<RefreshTokenResponse>(
      API_ENDPOINTS.AUTH.REFRESH
    );

    // Update token in API client
    if (response.token) {
      apiClient.setAuthToken(response.token);
      console.log("✅ Auth Service: Token refreshed successfully");
    }

    return response;
  }

  /**
   * Performs logout
   */
  async logout(): Promise<void> {
    console.log("🚪 Auth Service: Logging out");

    // Clear token in API client
    apiClient.logout();

    console.log("✅ Auth Service: Logged out successfully");
  }

  /**
   * Checks if the user is authenticated
   */
  isAuthenticated(): boolean {
    return apiClient.isAuthenticated();
  }

  /**
   * Gets the current token
   */
  getToken(): string | null {
    return apiClient.getAuthToken();
  }

  /**
   * Generates a message for signing
   * Uses standard format for Web3 authentication
   */
  generateSignMessage(nonce: string): string {
    const message = `Welcome to KEMBridge!

This request will not trigger a blockchain transaction or cost any gas fees.

Your authentication status will reset after 24 hours.

Wallet address:
{wallet_address}

Nonce:
${nonce}`;

    return message;
  }

  /**
   * Prepares a message for signing a specific wallet
   */
  prepareSignMessage(walletAddress: string, nonce: string): string {
    const message = this.generateSignMessage(nonce);
    return message.replace("{wallet_address}", walletAddress);
  }
}

// Create singleton instance
export const authService = new AuthService();

// Export for use in components
export default authService;
