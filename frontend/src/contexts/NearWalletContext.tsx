/**
 * NEAR Wallet Provider Component
 * Component implementation for NEAR wallet context
 */

import React, { useEffect, useState, useCallback, useMemo } from "react";
import { setupWalletSelector } from "@near-wallet-selector/core";
import { setupModal } from "@near-wallet-selector/modal-ui";
import { setupMyNearWallet } from "@near-wallet-selector/my-near-wallet";
import { setupNightly } from "@near-wallet-selector/nightly";
import type { WalletSelector } from "@near-wallet-selector/core";
import { NearWalletContext } from "./NearWalletContext";
import type { NearWalletContextType } from "../types/nearWallet";

interface NearWalletProviderProps {
  children: React.ReactNode;
}

export const NearWalletProvider: React.FC<NearWalletProviderProps> = ({
  children,
}) => {
  const [selector, setSelector] = useState<WalletSelector | null>(null);
  const [modal, setModal] = useState<unknown>(null);
  const [accountId, setAccountId] = useState<string | null>(null);

  useEffect(() => {
    const initWalletSelector = async () => {
      try {
        // Setup wallet modules
        const myNearWalletModule = setupMyNearWallet();
        const nightlyModule = setupNightly();

        // Setup wallet selector with multiple modules (Nightly first as default)
        const walletSelector = await setupWalletSelector({
          network: "testnet",
          modules: [nightlyModule, myNearWalletModule],
        });

        // Setup modal
        const walletModal = setupModal(walletSelector, {
          contractId: "kembridge.testnet",
        });

        // Set state
        setSelector(walletSelector);
        setModal(walletModal);

        // Subscribe to account changes
        const subscription = walletSelector.store.observable.subscribe(
          (state) => {
            const newAccountId = state.accounts[0]?.accountId || null;
            setAccountId(newAccountId);
          }
        );

        // Return unsubscribe function (store it for cleanup)
        return () => {
          subscription.unsubscribe();
        };
      } catch (error) {
        console.error("❌ NEAR: Failed to initialize wallet selector:", error);
        console.error("📝 NEAR: Error details:", (error as Error).stack);
      }
    };

    initWalletSelector();
  }, []);

  const signIn = useCallback(() => {
    if (modal) {
      try {
        (modal as { show: () => void }).show();
      } catch (error) {
        console.error("❌ NEAR: Error showing modal:", error);
      }
    } else {
      console.error("❌ NEAR: Modal not available for signIn");
    }
  }, [modal]);

  const signOut = useCallback(async () => {
    if (selector) {
      try {
        const wallet = await selector.wallet();
        await wallet.signOut();
        setAccountId(null);
      } catch (error) {
        console.error("❌ NEAR: Error signing out:", error);
      }
    } else {
      console.error("❌ NEAR: Selector not available for signOut");
    }
  }, [selector]);

  const value: NearWalletContextType = useMemo(() => ({
    selector,
    modal,
    accountId,
    isConnected: !!accountId,
    signIn,
    signOut,
  }), [selector, modal, accountId, signIn, signOut]);

  return (
    <NearWalletContext.Provider value={value}>
      {children}
    </NearWalletContext.Provider>
  );
};
