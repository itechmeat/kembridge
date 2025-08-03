import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from "axios";
import { API_CONFIG, API_ENDPOINTS } from "./config";

// Type definitions for API responses
export interface ApiResponse<T = unknown> {
  data: T;
  success: boolean;
  message?: string;
}

export interface ApiError {
  message: string;
  code?: string;
  status?: number;
  details?: unknown;
}

// Authentication interfaces from old client
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
  verified: boolean;
  wallet_address: string;
  chain_type: string;
  session_token: string | null;
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
  private authToken: string | null = null;
  private refreshToken: string | null = null;

  constructor() {
    // Create Axios instance with base configuration
    this.client = axios.create({
      baseURL: `${API_CONFIG.BASE_URL}${API_CONFIG.VERSION}`,
      timeout: API_CONFIG.TIMEOUT,
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
      },
      // Important: allow sending/receiving cookies (for CORS + CSRF if server uses cookies)
      withCredentials: true,
    });

    // Load stored token
    this.loadStoredToken();

    // Setup interceptors
    this.setupRequestInterceptor();
    this.setupResponseInterceptor();
  }

  /**
   * Loads tokens from localStorage
   */
  private loadStoredToken(): void {
    try {
      // Load access token (new format)
      const authToken = localStorage.getItem(API_CONFIG.TOKEN_STORAGE_KEY);
      if (authToken) {
        this.authToken = authToken;
        console.log("🔑 API Client: Auth token loaded from storage", {
          tokenLength: authToken.length,
          tokenPreview: authToken.substring(0, 20) + "...",
          storageKey: API_CONFIG.TOKEN_STORAGE_KEY,
        });
      }

      // Load refresh token (old format for compatibility)
      const refreshToken = localStorage.getItem("kembridge_refresh_token");
      if (refreshToken) {
        this.refreshToken = refreshToken;
        console.log("🔄 API Client: Refresh token loaded from storage");
      }

      // Migrate old access token if present
      const oldToken = localStorage.getItem("kembridge_access_token");
      if (oldToken && !this.authToken) {
        this.authToken = oldToken;
        localStorage.setItem(API_CONFIG.TOKEN_STORAGE_KEY, oldToken);
        localStorage.removeItem("kembridge_access_token");
        console.log("📦 API Client: Migrated old access token to new format");
      }

      if (!this.authToken && !this.refreshToken) {
        console.log("ℹ️ API Client: No stored tokens found");
      }
    } catch (error) {
      console.warn("⚠️ API Client: Failed to load tokens from storage:", error);
    }
  }

  /**
   * Saves tokens to localStorage
   */
  private saveToken(token: string, refresh?: string): void {
    try {
      localStorage.setItem(API_CONFIG.TOKEN_STORAGE_KEY, token);
      this.authToken = token;

      if (refresh) {
        localStorage.setItem("kembridge_refresh_token", refresh);
        this.refreshToken = refresh;
      }

      console.log("✅ API Client: Tokens saved to storage", {
        tokenLength: token.length,
        storageKey: API_CONFIG.TOKEN_STORAGE_KEY,
        tokenPreview: token.substring(0, 20) + "...",
        hasRefreshToken: !!refresh,
      });

      // Verify it was actually saved
      const storedToken = localStorage.getItem(API_CONFIG.TOKEN_STORAGE_KEY);
      console.log("🔍 API Client: Token verification", {
        storedSuccessfully: !!storedToken,
        tokensMatch: storedToken === token,
        memoryToken: !!this.authToken,
      });

      // Use setTimeout to prevent setState during render issues
      setTimeout(() => {
        // Dispatch custom event to notify components immediately
        window.dispatchEvent(
          new CustomEvent("auth-token-changed", {
            detail: { token, authenticated: true },
          })
        );
      }, 0);
    } catch (error) {
      console.error("❌ API Client: Failed to save tokens:", error);
    }
  }

  /**
   * Removes tokens from storage
   */
  private clearToken(): void {
    try {
      localStorage.removeItem(API_CONFIG.TOKEN_STORAGE_KEY);
      localStorage.removeItem("kembridge_refresh_token");
      // Clean up old format tokens too
      localStorage.removeItem("kembridge_access_token");
      this.authToken = null;
      this.refreshToken = null;
      console.log("🗑️ API Client: All tokens cleared");

      // Use setTimeout to prevent setState during render issues
      setTimeout(() => {
        // Dispatch custom event to notify components immediately
        window.dispatchEvent(
          new CustomEvent("auth-token-changed", {
            detail: { token: null, authenticated: false },
          })
        );
      }, 0);
    } catch (error) {
      console.warn("⚠️ API Client: Failed to clear tokens:", error);
    }
  }

  /**
   * Setup request interceptor for adding auth token + CSRF
   */
  private setupRequestInterceptor(): void {
    this.client.interceptors.request.use(
      async (config) => {
        // Add auth token if available
        if (this.authToken) {
          config.headers = config.headers || {};
          config.headers.Authorization = `Bearer ${this.authToken}`;
        }

        // Ensure credentials for CSRF/cookies
        // AxiosRequestConfig already supports withCredentials
        (config as AxiosRequestConfig).withCredentials = true;

        // CSRF: require one-time token for state-changing methods
        const method = (config.method || "get").toUpperCase();
        const isStateChanging = ["POST", "PUT", "DELETE", "PATCH"].includes(
          method
        );

        // Normalize URL: avoid duplicating /api/v1
        const rawUrl = config.url || "";
        const base = `${API_CONFIG.BASE_URL}${API_CONFIG.VERSION}`;
        
        let normalizedUrl: string;
        if (rawUrl.startsWith("http")) {
          // Absolute URL - use as is
          normalizedUrl = rawUrl;
        } else {
          // Relative URL - ensure it starts with single /api/v1
          let relativeUrl = rawUrl.startsWith("/") ? rawUrl : `/${rawUrl}`;
          
          // Remove multiple /api/v1 prefixes if they exist
          relativeUrl = relativeUrl.replace(/^(\/api\/v1)+/i, "/api/v1");
          
          // If doesn't start with /api/v1, add it
          if (!relativeUrl.startsWith("/api/v1")) {
            relativeUrl = `/api/v1${relativeUrl}`;
          }
          
          // Combine with base URL
          const baseNoTrailing = base.endsWith("/") ? base.slice(0, -1) : base;
          const cleanBase = baseNoTrailing.replace(/\/api\/v1$/, ""); // Remove /api/v1 from base
          normalizedUrl = `${cleanBase}${relativeUrl}`;
        }

        const absoluteUrl = new URL(normalizedUrl);

        // Determine auth-route reliably relative to api-prefix
        const pathname = absoluteUrl.pathname.replace(
          /^(\/api\/v1)+/i,
          "/api/v1"
        );
        const isAuthRoute = pathname.startsWith("/api/v1/auth/");

        // Request specifically to our gateway
        const isGateway =
          absoluteUrl.hostname === "localhost" && absoluteUrl.port === "4000";

        if (isGateway && isStateChanging && !isAuthRoute) {
          try {
            // Get one-time token only from our gateway
            interface CsrfData {
              data?: {
                csrf_token: string;
                header_name?: string;
                expires_in?: number;
                usage?: string;
              };
              // Fallback for direct structure
              csrf_token?: string;
              header_name?: string;
            }
            const csrfResp = await this.client.get<CsrfData>(
              "/auth/csrf-token",
              {
                baseURL: `${API_CONFIG.BASE_URL}${API_CONFIG.VERSION}`,
                withCredentials: true,
              }
            );
            const token = csrfResp.data?.data?.csrf_token || csrfResp.data?.csrf_token;
            if (!token) {
              console.warn("⚠️ CSRF token not found in response:", csrfResp.data);
              return config;
            }
            const headerName: string =
              csrfResp.data?.data?.header_name || csrfResp.data?.header_name || "x-csrf-token";

            config.headers = config.headers || {};
            // Header expected by backend
            const headers = config.headers as Record<string, unknown> & {
              set?: (k: string, v: string) => void;
            };
            if (typeof headers.set === "function") {
              headers.set(headerName, token);
            } else {
              (config.headers as Record<string, string>)[headerName] = token;
            }

            if (import.meta.env.DEV && pathname.includes("/risk/analyze")) {
              console.debug("🧪 CSRF attached for risk/analyze", {
                headerName,
                hasToken: !!token,
                method,
                url: normalizedUrl,
              });
            }
          } catch (e) {
            console.warn(
              "⚠️ Failed to fetch CSRF token, request may be rejected:",
              e
            );
          }
        }

        // Update config URL with normalized URL
        config.url = normalizedUrl;

        // Log request in dev mode
        if (import.meta.env.DEV) {
          console.log(`🔄 API Request: ${method} ${normalizedUrl}`, {
            data: config.data,
            params: config.params,
            isGateway,
            pathname,
          });
        }

        return config;
      },
      (error) => {
        console.error("❌ API Request Error:", error);
        return Promise.reject(error);
      }
    );
  }

  /**
   * Setup response interceptor for error handling and tokens
   */
  private setupResponseInterceptor(): void {
    this.client.interceptors.response.use(
      (response: AxiosResponse) => {
        // Log successful response in dev mode
        if (import.meta.env.DEV) {
          console.log(
            `✅ API Response: ${response.config.method?.toUpperCase()} ${
              response.config.url
            }`,
            {
              status: response.status,
              data: response.data,
            }
          );
        }

        return response;
      },
      async (error) => {
        const { response, config } = error;

        // Log error
        console.error(
          `❌ API Error: ${config?.method?.toUpperCase()} ${config?.url}`,
          {
            status: response?.status,
            message: response?.data?.message || error.message,
            data: response?.data,
          }
        );

        // Handle 401 errors (unauthorized)
        if (response?.status === 401) {
          console.warn("🔒 API Client: Unauthorized - token may be expired");

          // Try to refresh token if we have one
          if (
            this.refreshToken &&
            config &&
            !config.url?.includes("/auth/refresh")
          ) {
            try {
              console.log("🔄 API Client: Attempting token refresh");
              await this.refreshAccessToken();
              // Retry the original request with new token
              if (this.authToken) {
                config.headers = config.headers || {};
                config.headers.Authorization = `Bearer ${this.authToken}`;
                return this.client.request(config);
              }
            } catch (refreshError) {
              console.error(
                "❌ API Client: Token refresh failed:",
                refreshError
              );
              // Only clear tokens if refresh explicitly failed, not on initial 401
              this.clearToken();
            }
          } else {
            // Don't automatically clear tokens on 401 - let user re-authenticate manually
            console.log(
              "ℹ️ API Client: 401 error, but keeping tokens for manual re-auth"
            );
          }

          // TODO: Add redirect to login page or show authorization modal
          // window.location.href = '/login';
        }

        // Handle network errors
        if (!response) {
          console.error("🌐 API Client: Network error, backend may be down");
          throw new Error(
            "Unable to connect to the server. Please check your internet connection."
          );
        }

        // Convert to standard error format
        const apiError: ApiError = {
          message:
            response?.data?.message ||
            error.message ||
            "An unknown error occurred",
          code: response?.data?.code,
          details: response?.data,
        };

        throw apiError;
      }
    );
  }

  /**
   * Sets auth token and optional refresh token
   */
  public setAuthToken(token: string, refresh?: string): void {
    this.saveToken(token, refresh);
  }

  /**
   * Gets the current auth token
   */
  public getAuthToken(): string | null {
    return this.authToken;
  }

  /**
   * Checks if the user is authenticated
   */
  public isAuthenticated(): boolean {
    return !!this.authToken;
  }

  /**
   * Performs logout (clears token)
   */
  public logout(): void {
    this.clearToken();
  }

  /**
   * Generic method for making requests
   */
  public async request<T = unknown>(config: AxiosRequestConfig): Promise<T> {
    const response = await this.client.request<ApiResponse<T> | T>(config);

    // Check if response is wrapped in ApiResponse
    if (
      response.data &&
      typeof response.data === "object" &&
      "data" in response.data
    ) {
      return (response.data as ApiResponse<T>).data;
    }

    // Otherwise return data directly
    return response.data as T;
  }

  /**
   * GET request
   */
  public async get<T = unknown>(
    url: string,
    config?: AxiosRequestConfig
  ): Promise<T> {
    return this.request<T>({ ...config, method: "GET", url });
  }

  /**
   * POST request
   */
  public async post<T = unknown>(
    url: string,
    data?: unknown,
    config?: AxiosRequestConfig
  ): Promise<T> {
    return this.request<T>({ ...config, method: "POST", url, data });
  }

  /**
   * PUT request
   */
  public async put<T = unknown>(
    url: string,
    data?: unknown,
    config?: AxiosRequestConfig
  ): Promise<T> {
    return this.request<T>({ ...config, method: "PUT", url, data });
  }

  /**
   * DELETE request
   */
  public async delete<T = unknown>(
    url: string,
    config?: AxiosRequestConfig
  ): Promise<T> {
    return this.request<T>({ ...config, method: "DELETE", url });
  }

  /**
   * PATCH request
   */
  public async patch<T = unknown>(
    url: string,
    data?: unknown,
    config?: AxiosRequestConfig
  ): Promise<T> {
    return this.request<T>({ ...config, method: "PATCH", url, data });
  }

  /**
   * Authentication methods from old client
   */

  /**
   * Get authentication nonce
   */
  public async getNonce(
    walletAddress: string,
    chainType: "ethereum" | "near" = "ethereum"
  ): Promise<NonceResponse> {
    return this.get<NonceResponse>(API_ENDPOINTS.AUTH.NONCE, {
      params: {
        wallet_address: walletAddress,
        chain_type: chainType,
      },
    });
  }

  /**
   * Verify wallet signature and authenticate
   */
  public async verifyWallet(
    request: VerifyWalletRequest
  ): Promise<VerifyWalletResponse> {
    const response = await this.post<VerifyWalletResponse>(
      API_ENDPOINTS.AUTH.VERIFY_WALLET,
      request
    );

    // Store token if authentication was successful
    if (response.verified && response.session_token) {
      this.setAuthToken(response.session_token);
    }

    return response;
  }

  /**
   * Refresh access token using refresh token
   */
  public async refreshAccessToken(): Promise<void> {
    if (!this.refreshToken) {
      throw new Error("No refresh token available");
    }

    const response = await this.post<{ access_token: string }>(
      API_ENDPOINTS.AUTH.REFRESH,
      {
        refresh_token: this.refreshToken,
      }
    );

    // Update access token, keep same refresh token
    this.setAuthToken(response.access_token, this.refreshToken);
  }

  /**
   * Get user profile
   */
  public async getUserProfile(): Promise<UserProfile> {
    return this.get<UserProfile>(API_ENDPOINTS.USER.PROFILE);
  }

  /**
   * Update user profile
   */
  public async updateUserProfile(
    updates: Partial<UserProfile>
  ): Promise<UserProfile> {
    return this.put<UserProfile>(API_ENDPOINTS.USER.UPDATE_PROFILE, updates);
  }

  /**
   * Health check
   */
  public async getHealth(): Promise<{ status: string; timestamp: string }> {
    return this.get<{ status: string; timestamp: string }>(
      API_ENDPOINTS.HEALTH
    );
  }
}

// Create singleton instance
export const apiClient = new ApiClient();

// Export for use in components
export default apiClient;
