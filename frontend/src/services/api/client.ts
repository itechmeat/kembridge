/**
 * API Client for KEMBridge backend communication
 * Handles authentication, requests, and error handling
 */

import axios, { AxiosInstance, AxiosError } from "axios";

export interface ApiError {
  message: string;
  code?: string;
  status?: number;
  details?: Record<string, unknown>;
}

export interface AuthTokens {
  accessToken: string;
  refreshToken?: string;
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
  wallet_type: "ethereum" | "near";
}

export interface VerifyWalletResponse {
  access_token: string;
  refresh_token?: string;
  user: {
    id: string;
    wallet_address: string;
    created_at: string;
    updated_at: string;
  };
}

export interface UserProfile {
  id: string;
  wallet_address: string;
  display_name?: string;
  email?: string;
  created_at: string;
  updated_at: string;
  wallets: Array<{
    id: string;
    address: string;
    wallet_type: string;
    is_primary: boolean;
    created_at: string;
  }>;
}

class ApiClient {
  private client: AxiosInstance;
  private accessToken: string | null = null;
  private refreshToken: string | null = null;

  constructor(baseURL?: string) {
    const apiBaseUrl =
      baseURL ||
      import.meta.env.VITE_API_BASE_URL ||
      "http://localhost:4000/api/v1";

    this.client = axios.create({
      baseURL: apiBaseUrl,
      timeout: 30000,
      headers: {
        "Content-Type": "application/json",
      },
    });

    // Load tokens from localStorage
    this.loadTokensFromStorage();

    // Setup request interceptor for auth
    this.client.interceptors.request.use(
      (config) => {
        if (this.accessToken) {
          config.headers.Authorization = `Bearer ${this.accessToken}`;
        }
        return config;
      },
      (error) => Promise.reject(error)
    );

    // Setup response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      async (error: AxiosError) => {
        if (error.response?.status === 401 && this.refreshToken) {
          try {
            await this.refreshAccessToken();
            // Retry the original request
            return this.client.request(error.config!);
          } catch (refreshError) {
            this.clearTokens();
            throw this.createApiError(refreshError as AxiosError);
          }
        }
        throw this.createApiError(error);
      }
    );
  }

  /**
   * Authentication methods
   */
  async getNonce(
    walletAddress: string,
    chainType: "ethereum" | "near" = "ethereum"
  ): Promise<NonceResponse> {
    try {
      const response = await this.client.get<NonceResponse>("/auth/nonce", {
        params: {
          wallet_address: walletAddress,
          chain_type: chainType,
        },
      });
      return response.data;
    } catch (error) {
      throw this.createApiError(error as AxiosError);
    }
  }

  async verifyWallet(
    request: VerifyWalletRequest
  ): Promise<VerifyWalletResponse> {
    try {
      const response = await this.client.post<VerifyWalletResponse>(
        "/auth/verify-wallet",
        request
      );

      // Store tokens
      this.setTokens({
        accessToken: response.data.access_token,
        refreshToken: response.data.refresh_token,
      });

      return response.data;
    } catch (error) {
      throw this.createApiError(error as AxiosError);
    }
  }

  async logout(): Promise<void> {
    try {
      await this.client.post("/auth/logout");
    } catch (error) {
      // Continue with logout even if API call fails
      console.warn("Logout API call failed:", error);
    } finally {
      this.clearTokens();
    }
  }

  async refreshAccessToken(): Promise<void> {
    if (!this.refreshToken) {
      throw new Error("No refresh token available");
    }

    try {
      const response = await this.client.post<{ access_token: string }>(
        "/auth/refresh",
        {
          refresh_token: this.refreshToken,
        }
      );

      this.setTokens({
        accessToken: response.data.access_token,
        refreshToken: this.refreshToken,
      });
    } catch (error) {
      this.clearTokens();
      throw this.createApiError(error as AxiosError);
    }
  }

  /**
   * User profile methods
   */
  async getUserProfile(): Promise<UserProfile> {
    try {
      const response = await this.client.get<UserProfile>("/user/profile");
      return response.data;
    } catch (error) {
      throw this.createApiError(error as AxiosError);
    }
  }

  async updateUserProfile(updates: Partial<UserProfile>): Promise<UserProfile> {
    try {
      const response = await this.client.put<UserProfile>(
        "/user/profile",
        updates
      );
      return response.data;
    } catch (error) {
      throw this.createApiError(error as AxiosError);
    }
  }

  /**
   * Health check
   */
  async getHealth(): Promise<{ status: string; timestamp: string }> {
    try {
      // Health endpoint is at root level, not under /api/v1
      const baseURL =
        this.client.defaults.baseURL?.replace("/api/v1", "") ||
        "http://localhost:4000";
      const response = await this.client.get("/health", {
        baseURL,
      });
      return response.data;
    } catch (error) {
      throw this.createApiError(error as AxiosError);
    }
  }

  /**
   * Token management
   */
  setTokens(tokens: AuthTokens): void {
    this.accessToken = tokens.accessToken;
    this.refreshToken = tokens.refreshToken || null;
    this.saveTokensToStorage();
  }

  clearTokens(): void {
    this.accessToken = null;
    this.refreshToken = null;
    this.clearTokensFromStorage();
  }

  getAccessToken(): string | null {
    return this.accessToken;
  }

  isAuthenticated(): boolean {
    return !!this.accessToken;
  }

  /**
   * Private methods
   */
  private createApiError(error: AxiosError): ApiError {
    if (error.response) {
      // Server responded with error status
      const responseData = error.response.data as Record<string, unknown>;
      return {
        message: (responseData?.message as string) || error.message,
        code: responseData?.code as string,
        status: error.response.status,
        details: responseData,
      };
    } else if (error.request) {
      // Request was made but no response received
      return {
        message: "Network error - please check your connection",
        code: "NETWORK_ERROR",
      };
    } else {
      // Something else happened
      return {
        message: error.message || "Unknown error occurred",
        code: "UNKNOWN_ERROR",
      };
    }
  }

  private saveTokensToStorage(): void {
    try {
      if (this.accessToken) {
        localStorage.setItem("kembridge_access_token", this.accessToken);
      }
      if (this.refreshToken) {
        localStorage.setItem("kembridge_refresh_token", this.refreshToken);
      }
    } catch (error) {
      console.warn("Failed to save tokens to storage:", error);
    }
  }

  private loadTokensFromStorage(): void {
    try {
      this.accessToken = localStorage.getItem("kembridge_access_token");
      this.refreshToken = localStorage.getItem("kembridge_refresh_token");
    } catch (error) {
      console.warn("Failed to load tokens from storage:", error);
    }
  }

  private clearTokensFromStorage(): void {
    try {
      localStorage.removeItem("kembridge_access_token");
      localStorage.removeItem("kembridge_refresh_token");
    } catch (error) {
      console.warn("Failed to clear tokens from storage:", error);
    }
  }
}

// Export singleton instance
export const apiClient = new ApiClient();
export default apiClient;
