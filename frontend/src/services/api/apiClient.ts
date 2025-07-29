/**
 * Centralized API Client
 * Axios-based client for KEMBridge backend communication
 */

import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from "axios";
import { API_CONFIG } from "./config";

// Type definitions for API responses
export interface ApiResponse<T = unknown> {
  data: T;
  success: boolean;
  message?: string;
}

export interface ApiError {
  message: string;
  code?: string;
  details?: unknown;
}

class ApiClient {
  private client: AxiosInstance;
  private authToken: string | null = null;

  constructor() {
    // Create Axios instance with base configuration
    this.client = axios.create({
      baseURL: `${API_CONFIG.BASE_URL}${API_CONFIG.VERSION}`,
      timeout: API_CONFIG.TIMEOUT,
      headers: {
        "Content-Type": "application/json",
        Accept: "application/json",
      },
    });

    // Load stored token
    this.loadStoredToken();

    // Setup interceptors
    this.setupRequestInterceptor();
    this.setupResponseInterceptor();
  }

  /**
   * Loads token from localStorage
   */
  private loadStoredToken(): void {
    try {
      const token = localStorage.getItem(API_CONFIG.TOKEN_STORAGE_KEY);
      if (token) {
        this.authToken = token;
        console.log("🔑 API Client: Auth token loaded from storage", {
          tokenLength: token.length,
          tokenPreview: token.substring(0, 20) + "...",
          storageKey: API_CONFIG.TOKEN_STORAGE_KEY,
        });
      } else {
        console.log("ℹ️ API Client: No stored token found");
      }
    } catch (error) {
      console.warn("⚠️ API Client: Failed to load token from storage:", error);
    }
  }

  /**
   * Saves token to localStorage
   */
  private saveToken(token: string): void {
    try {
      localStorage.setItem(API_CONFIG.TOKEN_STORAGE_KEY, token);
      this.authToken = token;
      console.log("✅ API Client: Auth token saved to storage", {
        tokenLength: token.length,
        storageKey: API_CONFIG.TOKEN_STORAGE_KEY,
        tokenPreview: token.substring(0, 20) + "...",
      });

      // Verify it was actually saved
      const storedToken = localStorage.getItem(API_CONFIG.TOKEN_STORAGE_KEY);
      console.log("🔍 API Client: Token verification", {
        storedSuccessfully: !!storedToken,
        tokensMatch: storedToken === token,
        memoryToken: !!this.authToken,
      });
    } catch (error) {
      console.error("❌ API Client: Failed to save token:", error);
    }
  }

  /**
   * Removes token from storage
   */
  private clearToken(): void {
    try {
      localStorage.removeItem(API_CONFIG.TOKEN_STORAGE_KEY);
      this.authToken = null;
      console.log("🗑️ API Client: Auth token cleared");
    } catch (error) {
      console.warn("⚠️ API Client: Failed to clear token:", error);
    }
  }

  /**
   * Setup request interceptor for adding auth token
   */
  private setupRequestInterceptor(): void {
    this.client.interceptors.request.use(
      (config) => {
        // Add auth token if available
        if (this.authToken) {
          config.headers.Authorization = `Bearer ${this.authToken}`;
        }

        // Log request in dev mode
        if (import.meta.env.DEV) {
          console.log(
            `🔄 API Request: ${config.method?.toUpperCase()} ${config.url}`,
            {
              data: config.data,
              params: config.params,
            }
          );
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
          console.warn("🔒 API Client: Unauthorized, clearing token");
          this.clearToken();

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
   * Sets auth token
   */
  public setAuthToken(token: string): void {
    this.saveToken(token);
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
}

// Create singleton instance
export const apiClient = new ApiClient();

// Export for use in components
export default apiClient;
