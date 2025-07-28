/**
 * NEAR Wallet Provider Component
 * Component implementation for NEAR wallet context
 */

import React, { useEffect, useState } from "react";
import { setupWalletSelector } from "@near-wallet-selector/core";
import { setupModal } from "@near-wallet-selector/modal-ui";
import { setupMyNearWallet } from "@near-wallet-selector/my-near-wallet";
import { setupNightly } from "@near-wallet-selector/nightly";
import type { WalletSelector } from "@near-wallet-selector/core";
import {
  NearWalletContext,
  type NearWalletContextType,
} from "./NearWalletContext";

interface NearWalletProviderProps {
  children: React.ReactNode;
}

export const NearWalletProvider: React.FC<NearWalletProviderProps> = ({
  children,
}) => {
  console.log("🏗️ NEAR: NearWalletProvider component rendering");
  const [selector, setSelector] = useState<WalletSelector | null>(null);
  const [modal, setModal] = useState<unknown>(null);
  const [accountId, setAccountId] = useState<string | null>(null);

  useEffect(() => {
    const initWalletSelector = async () => {
      try {
        console.log("🚀 NEAR: Starting wallet selector initialization...");
        console.log("📦 NEAR: Setting up wallet selector with testnet");

        // Setup wallet modules
        console.log("🔧 NEAR: Setting up MyNearWallet module");
        const myNearWalletModule = setupMyNearWallet();
        console.log("✅ NEAR: MyNearWallet module created");

        console.log("🔧 NEAR: Setting up Nightly module");
        const nightlyModule = setupNightly();
        console.log("✅ NEAR: Nightly module created");

        // Setup wallet selector with multiple modules (Nightly first as default)
        console.log("🔧 NEAR: Setting up wallet selector with modules");
        const walletSelector = await setupWalletSelector({
          network: "testnet",
          modules: [nightlyModule, myNearWalletModule],
        });

        console.log("✅ NEAR: Wallet selector created");

        // Setup modal
        console.log("🔧 NEAR: Setting up modal");
        const walletModal = setupModal(walletSelector, {
          contractId: "kembridge.testnet",
        });

        console.log("✅ NEAR: Modal created");

        // Set state
        console.log("🔄 NEAR: Setting selector and modal in state");
        setSelector(walletSelector);
        setModal(walletModal);
        console.log("✅ NEAR: Selector and modal set in state");

        // Subscribe to account changes
        console.log("🔄 NEAR: Setting up account subscription");
        const subscription = walletSelector.store.observable.subscribe(
          (state) => {
            console.log("🔄 NEAR: State changed:", {
              accounts: state.accounts,
              selectedWalletId: state.selectedWalletId,
              accountsLength: state.accounts.length,
            });

            const newAccountId = state.accounts[0]?.accountId || null;
            console.log("👤 NEAR: Setting accountId to:", newAccountId);
            setAccountId(newAccountId);
          }
        );

        console.log("🎉 NEAR: Wallet selector initialized successfully!");

        // Return unsubscribe function (store it for cleanup)
        console.log("📊 NEAR: Initial state:", walletSelector.store.getState());

        return () => {
          subscription.unsubscribe();
          console.log("🧹 NEAR: Cleaned up subscription");
        };
      } catch (error) {
        console.error("❌ NEAR: Failed to initialize wallet selector:", error);
        console.error("📝 NEAR: Error details:", (error as Error).stack);
      }
    };

    initWalletSelector();
  }, []);

  const signIn = () => {
    console.log("🔐 NEAR: signIn called");
    console.log("🪟 NEAR: Modal available:", !!modal);
    console.log("⚙️ NEAR: Selector available:", !!selector);

    if (modal) {
      console.log("✅ NEAR: Opening wallet selection modal...");
      try {
        console.log("🔍 NEAR: Modal object details:", {
          type: typeof modal,
          methods: Object.keys(modal),
          show: typeof (modal as { show?: () => void }).show,
        });
        (modal as { show: () => void }).show();
        console.log("👁️ NEAR: Modal.show() called");
      } catch (error) {
        console.error("❌ NEAR: Error showing modal:", error);
      }
    } else {
      console.error("❌ NEAR: Modal not available for signIn");
    }
  };

  const signOut = async () => {
    console.log("🚪 NEAR: signOut called");
    console.log("⚙️ NEAR: Selector available:", !!selector);

    if (selector) {
      try {
        console.log("🔄 NEAR: Getting wallet instance...");
        const wallet = await selector.wallet();
        console.log("✅ NEAR: Wallet instance:", wallet);

        console.log("🔄 NEAR: Calling wallet.signOut()...");
        await wallet.signOut();

        console.log("✅ NEAR: Sign out successful, clearing accountId");
        setAccountId(null);
      } catch (error) {
        console.error("❌ NEAR: Error signing out:", error);
      }
    } else {
      console.error("❌ NEAR: Selector not available for signOut");
    }
  };

  const value: NearWalletContextType = {
    selector,
    modal,
    accountId,
    isConnected: !!accountId,
    signIn,
    signOut,
  };

  return (
    <NearWalletContext.Provider value={value}>
      {children}
    </NearWalletContext.Provider>
  );
};
