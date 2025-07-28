/**
 * Wallet connection component
 * Handles wallet selection and connection UI
 */

import { FC, useState } from "react";
import { useWallet } from "../../../hooks/wallet/useWallet";
import { WalletType } from "../../../services/wallet/types";
import { Button } from "../../ui/Button";
import { Modal } from "../../ui/Modal";
import { Spinner } from "../../ui/Spinner";
import {
  getWalletName,
  getWalletIcon,
  getWalletDescription,
} from "../../../utils/wallet";
import { APP_TEXT, UI_CONFIG } from "../../../constants";

import "./WalletConnect.scss";

interface WalletConnectProps {
  onConnect?: () => void;
  onCancel?: () => void;
  compact?: boolean;
}

export const WalletConnect: FC<WalletConnectProps> = ({
  onConnect,
  onCancel,
  compact = false,
}) => {
  console.log("🏗️ WalletConnect: Component rendering");
  const { isConnecting, isConnected, availableProviders, connect, error } =
    useWallet();

  console.log("📊 WalletConnect: Wallet state:", { isConnecting, isConnected });
  console.log(
    "📊 WalletConnect: Available providers count:",
    availableProviders.length
  );
  console.log(
    "📊 WalletConnect: Available providers:",
    availableProviders.map((p) => p.type)
  );

  // Debug: Log providers status (development only) - removed to reduce console spam

  const [isModalOpen, setIsModalOpen] = useState(false);
  const [selectedWallet, setSelectedWallet] = useState<WalletType | null>(null);

  // Note: NEAR modal will appear on top with higher z-index

  const handleWalletSelect = async (walletType: WalletType) => {
    try {
      console.log(`🔗 WalletConnect: Attempting to connect to ${walletType}`);
      setSelectedWallet(walletType);

      console.log(`📦 WalletConnect: Calling connect(${walletType})...`);
      const account = await connect(walletType);
      console.log("✅ WalletConnect: Connection successful:", account);

      setIsModalOpen(false);
      onConnect?.();
    } catch (err: unknown) {
      console.error("❌ WalletConnect: Wallet connection failed:", err);
      const error = err as { message?: string; code?: string; stack?: string };
      console.error("❌ WalletConnect: Error details:", {
        message: error.message,
        code: error.code,
        stack: error.stack,
      });
      setSelectedWallet(null);

      // Show error for a moment then clear
      setTimeout(() => {
        setSelectedWallet(null);
      }, 2000);
    }
  };

  const handleOpenModal = () => {
    console.log("🔄 WalletConnect: Opening modal");
    setIsModalOpen(true);
    setSelectedWallet(null);
  };

  const handleCloseModal = () => {
    console.log("🔄 WalletConnect: Closing modal");
    setIsModalOpen(false);
    setSelectedWallet(null);
    onCancel?.();
  };

  if (isConnected) {
    return null; // Don't show if already connected
  }

  return (
    <>
      <Button
        onClick={() => {
          console.log("🖱️ WalletConnect: Clicked on Connect button");
          handleOpenModal();
        }}
        variant={UI_CONFIG.BUTTON_VARIANTS.PRIMARY}
        disabled={isConnecting}
        className={`wallet-connect-trigger ${
          compact ? "wallet-connect-trigger--compact" : ""
        }`}
      >
        {isConnecting ? (
          <>
            <Spinner size={UI_CONFIG.SPINNER_SIZES.SM} />
            {compact ? "Connecting..." : APP_TEXT.BUTTONS.CONNECTING}
          </>
        ) : compact ? (
          "Connect"
        ) : (
          APP_TEXT.BUTTONS.CONNECT_WALLET
        )}
      </Button>

      <Modal
        isOpen={isModalOpen}
        onClose={() => {
          console.log("🖱️ WalletConnect: Closing modal");
          handleCloseModal();
        }}
        title={APP_TEXT.BUTTONS.CONNECT_WALLET}
        size={UI_CONFIG.MODAL_SIZES.MD}
      >
        <div className="wallet-connect">
          <div className="wallet-connect__header">
            <p className="wallet-connect__description">
              Choose a wallet to connect to KEMBridge
            </p>
          </div>

          {error && (
            <div className="wallet-connect__error">
              <span className="wallet-connect__error-text">
                {typeof error === "string"
                  ? error
                  : (error as { message?: string })?.message ||
                    "Connection failed"}
              </span>
            </div>
          )}

          <div className="wallet-connect__providers">
            {availableProviders.map((provider) => {
              console.log(
                "📋 WalletConnect: Rendering provider:",
                provider.type,
                "isAvailable:",
                provider.isAvailable
              );
              return (
                <button
                  key={provider.type}
                  className={`wallet-provider-mobile ${
                    selectedWallet === provider.type
                      ? "wallet-provider-mobile--loading"
                      : ""
                  } ${
                    !provider.isAvailable
                      ? "wallet-provider-mobile--unavailable"
                      : ""
                  }`}
                  onClick={() => {
                    console.log(
                      `🖱️ WalletConnect: Clicked on ${provider.type} wallet`
                    );
                    handleWalletSelect(provider.type);
                  }}
                  disabled={isConnecting || !provider.isAvailable}
                >
                  <div className="wallet-provider-mobile__icon">
                    {selectedWallet === provider.type && isConnecting ? (
                      <Spinner size={UI_CONFIG.SPINNER_SIZES.SM} />
                    ) : (
                      <span role="img" aria-label={`${provider.name} icon`}>
                        {getWalletIcon(provider.type)}
                      </span>
                    )}
                  </div>

                  <div className="wallet-provider-mobile__content">
                    <h3 className="wallet-provider-mobile__name">
                      {getWalletName(provider.type)}
                    </h3>
                    <p className="wallet-provider-mobile__description">
                      {getWalletDescription(provider.type)}
                    </p>
                  </div>

                  {!provider.isAvailable && (
                    <div className="wallet-provider-mobile__status">
                      Install
                    </div>
                  )}
                </button>
              );
            })}
          </div>

          {availableProviders.length === 0 && (
            <div className="wallet-connect__no-providers">
              <p>No wallet providers available.</p>
              <p>Please install a supported wallet extension.</p>
            </div>
          )}

          <div className="wallet-connect__footer">
            <p className="wallet-connect__footer-text">
              By connecting a wallet, you agree to KEMBridge's Terms of Service.
            </p>
          </div>
        </div>
      </Modal>
    </>
  );
};
