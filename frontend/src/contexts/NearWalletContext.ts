/**
 * NEAR Wallet Context Definition
 * Pure context without components
 */

import { createContext } from "react";
import type { NearWalletContextType } from "../types/nearWallet";

export const NearWalletContext = createContext<NearWalletContextType | null>(
  null
);
