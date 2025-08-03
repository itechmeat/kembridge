import { useContext } from "react";
import { NearWalletContext } from "../../contexts/NearWalletContext/NearWalletContext";
import type { NearWalletContextType } from "../../types/nearWallet";

export const useNearWallet = (): NearWalletContextType => {
  const context = useContext(NearWalletContext);
  if (!context) {
    console.error("❌ useNearWallet: Context not found");
    throw new Error("useNearWallet must be used within NearWalletProvider");
  }
  return context;
};
