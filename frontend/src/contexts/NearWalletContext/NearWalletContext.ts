import { createContext } from "react";
import type { NearWalletContextType } from "../../types/nearWallet";

export const NearWalletContext = createContext<NearWalletContextType | null>(
  null
);
