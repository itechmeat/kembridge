/**
 * NEAR Wallet hook
 * Hook for accessing NEAR wallet context
 */

import { useContext } from "react";
import { NearWalletContext } from "../../contexts/NearWalletContext";

export const useNearWallet = () => {
  console.log("🔄 useNearWallet: Hook called");
  const context = useContext(NearWalletContext);
  if (!context) {
    console.error("❌ useNearWallet: Context not found");
    throw new Error("useNearWallet must be used within NearWalletProvider");
  }
  console.log("📊 useNearWallet: Context data:", {
    selector: !!context.selector,
    modal: !!context.modal,
    accountId: context.accountId,
    isConnected: context.isConnected,
  });
  return context;
};
