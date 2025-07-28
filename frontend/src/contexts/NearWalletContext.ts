/**
 * NEAR Wallet Context Definition
 * Pure context without components
 */

import { createContext } from "react";
import type { WalletSelector } from "@near-wallet-selector/core";

export interface NearWalletContextType {
  selector: WalletSelector | null;
  modal: unknown; // NEAR modal type is complex, using unknown for now
  accountId: string | null;
  isConnected: boolean;
  signIn: () => void;
  signOut: () => void;
}

export const NearWalletContext = createContext<NearWalletContextType | null>(
  null
);
