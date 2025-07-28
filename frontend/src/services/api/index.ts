/**
 * API Services Index
 * Centralized export of all API services and hooks
 *
 * @module Services
 * @description This module exports all API services and hooks used in the application.
 * @requires api/apiClient
 * @requires api/authService
 * @requires api/userService
 * @requires api/bridgeService
 */

// API Client
export {
  default as apiClient,
  type ApiResponse,
  type ApiError,
} from "./apiClient";

// Configuration
export { API_CONFIG, API_ENDPOINTS, API_ERROR_CODES } from "./config";

// Services
export { default as authService } from "./authService";
export { default as userService } from "./userService";
export { default as bridgeService } from "./bridgeService";

// Service Types
export type {
  NonceRequest,
  NonceResponse,
  VerifyWalletRequest,
  VerifyWalletResponse,
  RefreshTokenResponse,
} from "./authService";

export type {
  UserProfile,
  UpdateProfileRequest,
  UserStatistics,
} from "./userService";

export type {
  SwapQuoteRequest,
  SwapQuote,
  InitSwapRequest,
  SwapTransaction,
  SwapHistory,
  SupportedToken,
} from "./bridgeService";

// React Hooks
export {
  useGetNonce,
  useVerifyWallet,
  useEthereumAuth,
  useNearAuth,
  useLogout,
  useAuthStatus,
  AUTH_QUERY_KEYS,
} from "../../hooks/api/useAuth";

export {
  useUserProfile,
  useUpdateProfile,
  useUserStatistics,
  useUserTier,
  useUserPreferences,
  useUserRisk,
  useUserWallets,
  useUserInfo,
  USER_QUERY_KEYS,
} from "../../hooks/api/useUser";

export {
  useSwapQuote,
  useInitSwap,
  useSwapStatus,
  useSwapHistory,
  useSupportedTokens,
  useMultipleSwapStatus,
  useBridgeUtils,
  useTrackTransaction,
  BRIDGE_QUERY_KEYS,
} from "../../hooks/api/useBridge";
