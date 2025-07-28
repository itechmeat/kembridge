/**
 * NEAR Wallet Types
 * Types for NEAR wallet integration
 *
 * @module Types
 * @description This module exports types for NEAR wallet integration.
 * @requires @near-wallet-selector/core
 * @requires @near-wallet-selector/core/modal
 * @requires @near-wallet-selector/core/accountId
 * @requires @near-wallet-selector/core/isConnected
 */

import type { WalletSelector } from "@near-wallet-selector/core";

export interface NearWalletContextType {
  selector: WalletSelector | null;
  modal: unknown; // NEAR modal type is complex, using unknown for now
  accountId: string | null;
  isConnected: boolean;
  signIn: () => void;
  signOut: () => void;
}
