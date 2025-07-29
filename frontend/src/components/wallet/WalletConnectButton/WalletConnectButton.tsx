/**
 * Simple Wallet Connect Button
 * Shows a button that opens wallet selection modal
 */

import React, { useState } from "react";
import { Button } from "../../ui/Button";
import { WalletConnect } from "../../auth/WalletConnect/WalletConnect";
import { useWallet } from "../../../hooks/wallet/useWallet";
import "./WalletConnectButton.scss";

interface WalletConnectButtonProps {
  compact?: boolean;
  className?: string;
}

export const WalletConnectButton: React.FC<WalletConnectButtonProps> = ({
  compact = false,
  className = "",
}) => {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const { isConnecting } = useWallet();

  const handleConnect = () => {
    setIsModalOpen(true);
  };

  const handleClose = () => {
    setIsModalOpen(false);
  };

  const handleSuccess = () => {
    setIsModalOpen(false);
  };

  const handleError = (error: Error) => {
    console.error("Wallet connection failed:", error);
    // Keep modal open to allow retry
  };

  return (
    <>
      <Button
        variant="primary"
        size={compact ? "sm" : "md"}
        onClick={handleConnect}
        disabled={isConnecting}
        className={`wallet-connect-button ${className}`}
      >
        {isConnecting ? "Connecting..." : "Connect"}
      </Button>

      <WalletConnect
        isOpen={isModalOpen}
        onClose={handleClose}
        onSuccess={handleSuccess}
        onError={handleError}
      />
    </>
  );
};