import apiClient, {
  VerifyWalletRequest,
  VerifyWalletResponse as ApiVerifyWalletResponse,
} from "./apiClient";

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

// Re-export the correct interface from apiClient
export type VerifyWalletResponse = ApiVerifyWalletResponse;
export type { VerifyWalletRequest } from "./apiClient";

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

    const response = await apiClient.getNonce(walletAddress, chainType);

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
      messageLength: message?.length || 0,
    });

    const request: VerifyWalletRequest = {
      wallet_address: walletAddress,
      signature,
      nonce,
      chain_type: chainType,
      message,
    };

    console.log("📤 Auth Service: Sending verification request:", {
      wallet_address: request.wallet_address,
      signature_preview: request.signature?.substring(0, 50) + "...",
      signature_length: request.signature?.length,
      nonce: request.nonce,
      chain_type: request.chain_type,
      message_preview: request.message?.substring(0, 100) + "...",
      message_length: request.message?.length,
    });

    const response = await apiClient.verifyWallet(request);

    // Debug: Log the entire response to see what we're getting
    console.log("🔍 Auth Service: Full response from backend:", {
      status: "success",
      responseKeys: Object.keys(response),
      responseData: response,
      verified: response.verified,
      hasToken: !!response.session_token,
      tokenValue: response.session_token,
    });

    // Save token by wallet type and as main token (by apiClient.verifyWallet call above)
    if (response.verified && response.session_token) {
      // Save token specifically for this wallet type
      const walletType = chainType === "ethereum" ? "evm" : "near";
      this.saveTokenByType(response.session_token, walletType);

      console.log("✅ Auth Service: Authentication successful", {
        tokenLength: response.session_token.length,
        tokenPreview: response.session_token.substring(0, 20) + "...",
        walletAddress: response.wallet_address,
        chainType: response.chain_type,
        walletType,
      });

      // Verify token was actually saved by apiClient
      const savedToken = apiClient.getAuthToken();
      const isAuth = apiClient.isAuthenticated();
      console.log("🔍 Auth Service: Token verification after auth", {
        tokenSaved: !!savedToken,
        isAuthenticated: isAuth,
        tokensMatch: savedToken === response.session_token,
      });
    } else {
      console.error(
        "❌ Auth Service: Authentication failed or no session_token!",
        {
          verified: response.verified,
          hasSessionToken: !!response.session_token,
          responseType: typeof response,
          responseKeys: Object.keys(response || {}),
          fullResponse: response,
          walletAddress: response.wallet_address,
          chainType: response.chain_type,
        }
      );
    }

    return response;
  }

  /**
   * Updates JWT token
   */
  async refreshToken(): Promise<RefreshTokenResponse> {
    console.log("🔄 Auth Service: Refreshing token");

    await apiClient.refreshAccessToken();
    console.log("✅ Auth Service: Token refreshed successfully");

    // Return current state
    const token = apiClient.getAuthToken();
    return {
      token: token || "",
      expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    };
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
   * Gets all tokens by wallet type
   */
  getAllTokens(): { evm?: string; near?: string } {
    const evmToken = localStorage.getItem("kembridge_auth_token_evm");
    const nearToken = localStorage.getItem("kembridge_auth_token_near");

    return {
      evm: evmToken || undefined,
      near: nearToken || undefined,
    };
  }

  /**
   * Saves token for specific wallet type
   */
  saveTokenByType(token: string, walletType: "evm" | "near"): void {
    localStorage.setItem(`kembridge_auth_token_${walletType}`, token);

    // Always save as main token (both EVM and NEAR)
    // EVM has priority, but if no EVM token exists, NEAR token becomes main
    const currentEvmToken = localStorage.getItem("kembridge_auth_token_evm");
    const currentNearToken = localStorage.getItem("kembridge_auth_token_near");
    
    if (walletType === "evm" || !currentEvmToken) {
      apiClient.setAuthToken(token);
      console.log(`💾 Auth Service: Set ${token.substring(0, 10)}... as main token`);
    }

    console.log(`💾 Auth Service: Token saved for ${walletType}:`, {
      tokenLength: token.length,
      walletType,
      hasEvmToken: !!currentEvmToken,
      hasNearToken: !!currentNearToken,
    });
  }

  /**
   * Clears token for specific wallet type
   */
  clearTokenByType(walletType: "evm" | "near"): void {
    localStorage.removeItem(`kembridge_auth_token_${walletType}`);

    // Update main token based on remaining tokens
    const remainingEvmToken = localStorage.getItem("kembridge_auth_token_evm");
    const remainingNearToken = localStorage.getItem("kembridge_auth_token_near");
    
    if (walletType === "evm" && remainingNearToken) {
      // If we cleared EVM but NEAR remains, make NEAR the main token
      apiClient.setAuthToken(remainingNearToken);
      console.log("🔄 Auth Service: Switched main token to NEAR");
    } else if (!remainingEvmToken && !remainingNearToken) {
      // If no tokens remain, clear main token
      apiClient.logout();
      console.log("🗑️ Auth Service: Cleared main token (no tokens remain)");
    }

    console.log(`🗑️ Auth Service: Token cleared for ${walletType}`, {
      remainingEvmToken: !!remainingEvmToken,
      remainingNearToken: !!remainingNearToken,
    });
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
