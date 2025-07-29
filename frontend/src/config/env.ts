/**
 * Centralized environment configuration
 * Single source of truth for all environment variables
 */

/// <reference types="vite/client" />

export interface AppConfig {
  // App metadata
  app: {
    name: string;
    description: string;
    url: string;
    iconUrl: string;
  };

  // Wallet configuration
  wallet: {
    walletConnectProjectId: string;
  };

  // API configuration
  api: {
    baseUrl: string;
  };

  // Blockchain configuration
  blockchain: {
    defaultChainId: number;
    networkEnv: string;
    ethereum: {
      rpcUrl?: string;
    };
    sepolia: {
      rpcUrl?: string;
    };
  };
}

// Real WalletConnect project ID is required for production
// Set VITE_WALLETCONNECT_PROJECT_ID environment variable

/**
 * Global application configuration
 * All environment variables should be accessed through this config
 */
export const appConfig: AppConfig = {
  app: {
    name: import.meta.env.VITE_APP_NAME || "KEMBridge",
    description:
      import.meta.env.VITE_APP_DESCRIPTION ||
      "Quantum-Secured Cross-Chain Bridge",
    url: import.meta.env.VITE_APP_URL || "http://localhost:4001",
    iconUrl:
      import.meta.env.VITE_APP_ICON_URL || "http://localhost:4001/icon.png",
  },

  wallet: {
    walletConnectProjectId: import.meta.env.VITE_WALLETCONNECT_PROJECT_ID || "",
  },

  api: {
    baseUrl: import.meta.env.VITE_API_BASE_URL || "http://localhost:4001",
  },

  blockchain: {
    defaultChainId: Number(import.meta.env.VITE_DEFAULT_CHAIN_ID) || 11155111,
    networkEnv: import.meta.env.VITE_NETWORK_ENV || "development",
    ethereum: {
      rpcUrl: import.meta.env.VITE_ETHEREUM_RPC_URL,
    },
    sepolia: {
      rpcUrl: import.meta.env.VITE_SEPOLIA_RPC_URL,
    },
  },
};

/**
 * Validates that all required environment variables are set
 */
export const validateConfig = (): void => {
  const issues: string[] = [];

  // Check critical configurations
  if (!appConfig.wallet.walletConnectProjectId) {
    issues.push(
      "⚠️  WalletConnect Project ID is not configured! Set VITE_WALLETCONNECT_PROJECT_ID environment variable"
    );
  }

  if (
    appConfig.app.url.includes("kembridge.io") &&
    window.location.hostname === "localhost"
  ) {
    issues.push("⚠️  App URL should be localhost for local development");
  }

  // Log warnings for development
  if (issues.length > 0 && appConfig.blockchain.networkEnv === "development") {
    console.warn("🔧 Configuration Issues:");
    issues.forEach((issue) => console.warn(issue));
    console.warn("📖 See WALLET_SETUP.md for setup instructions");
  }
};

/**
 * Check if running in development mode
 */
export const isDevelopment = (): boolean => {
  return (
    appConfig.blockchain.networkEnv === "development" || import.meta.env.DEV
  );
};

/**
 * Check if running in production mode
 */
export const isProduction = (): boolean => {
  return (
    appConfig.blockchain.networkEnv === "production" || import.meta.env.PROD
  );
};
