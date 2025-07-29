/**
 * NEAR Wallet hook
 * Hook for accessing NEAR wallet context
 */

import { useContext } from "react";
import { NearWalletContext } from "../../contexts/NearWalletContext";
import type { NearWalletContextType } from "../../types/nearWallet";

export const useNearWallet = (): NearWalletContextType => {
  const context = useContext(NearWalletContext);
  if (!context) {
    console.error("❌ useNearWallet: Context not found");
    throw new Error("useNearWallet must be used within NearWalletProvider");
  }
  return context;
};
