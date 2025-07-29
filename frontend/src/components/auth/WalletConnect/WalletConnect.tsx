/**
 * Wallet Connect Component
 * Main wallet selection and connection interface
 */

import React from "react";
import { WalletButton } from "../WalletButton/WalletButton";
import { Modal } from "../../ui/Modal/Modal";
import { useWallet } from "../../../hooks/wallet/useWallet";
import "./WalletConnect.scss";

export interface WalletConnectProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess?: () => void;
  onError?: (error: Error) => void;
  allowReconnect?: boolean; // Allow showing modal even when already connected
}

export type WalletType = "metamask" | "walletconnect" | "coinbase" | "near";

export interface WalletConfig {
  id: WalletType;
  name: string;
  description: string;
  icon: string;
  isAvailable: boolean;
  isInstalled?: boolean;
}

export const WalletConnect: React.FC<WalletConnectProps> = ({
  isOpen,
  onClose,
  onSuccess,
  onError,
}) => {
  const { state, connect, clearError, isWalletAvailable, isConnected } =
    useWallet();

  const wallets: WalletConfig[] = [
    {
      id: "metamask",
      name: "MetaMask",
      description: "Connect using MetaMask wallet",
      icon: "/icons/metamask.svg",
      isAvailable: isWalletAvailable("metamask"),
      isInstalled:
        typeof window !== "undefined" &&
        !!window.ethereum &&
        window.ethereum.isMetaMask,
    },
    {
      id: "walletconnect",
      name: "WalletConnect",
      description: "Connect using WalletConnect protocol",
      icon: "/icons/walletconnect.svg",
      isAvailable: isWalletAvailable("walletconnect"),
    },
    {
      id: "coinbase",
      name: "Coinbase Wallet",
      description: "Connect using Coinbase Wallet",
      icon: "/icons/coinbase.svg",
      isAvailable: isWalletAvailable("coinbase"),
    },
    {
      id: "near",
      name: "NEAR Wallet",
      description: "Connect using NEAR Protocol wallet",
      icon: "/icons/near.svg",
      isAvailable: isWalletAvailable("near"),
    },
  ];

  const handleWalletSelect = async (walletType: WalletType) => {
    try {
      clearError();
      await connect(walletType);

      onSuccess?.();
      onClose();
    } catch (err) {
      const error = err as Error;
      onError?.(error);
    }
  };

  const handleClose = () => {
    if (!state.isConnecting) {
      clearError();
      onClose();
    }
  };

  if (isConnected) {
    return null;
  }

  return (
    <Modal
      isOpen={isOpen}
      onClose={handleClose}
      title="Connect Wallet"
      className="wallet-connect-modal"
    >
      <div className="wallet-connect">
        <div className="wallet-connect__header">
          <h3>Choose your wallet</h3>
          <p>Connect with one of our supported wallet providers</p>
        </div>

        {state.error && (
          <div className="wallet-connect__error">
            <span className="error-icon">⚠️</span>
            <span>{state.error}</span>
          </div>
        )}

        <div className="wallet-connect__options">
          {wallets.map((wallet) => (
            <WalletButton
              key={wallet.id}
              wallet={wallet}
              isSelected={false}
              isConnecting={state.isConnecting}
              onSelect={handleWalletSelect}
              disabled={state.isConnecting}
            />
          ))}
        </div>

        <div className="wallet-connect__footer">
          <p className="wallet-connect__disclaimer">
            By connecting a wallet, you agree to our Terms of Service and
            Privacy Policy.
          </p>
        </div>
      </div>
    </Modal>
  );
};
