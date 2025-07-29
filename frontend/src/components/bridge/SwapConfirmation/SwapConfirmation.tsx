/**
 * SwapConfirmation Component - Mobile-First
 * Confirmation modal optimized for mobile with full-screen approach
 */

import React, { useState, useEffect } from "react";
import type { BridgeQuote, BridgeSwapRequest } from "../../../types/bridge";
import "./SwapConfirmation.scss";

export interface SwapConfirmationProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: (request: BridgeSwapRequest) => void;
  quote?: BridgeQuote;
  loading?: boolean;
  className?: string;
}

export const SwapConfirmation: React.FC<SwapConfirmationProps> = ({
  isOpen,
  onClose,
  onConfirm,
  quote,
  loading = false,
  className = "",
}) => {
  const [termsAccepted, setTermsAccepted] = useState(false);
  const [recipient, setRecipient] = useState("");

  // Mobile: Prevent body scroll when modal is open
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = "";
    }

    return () => {
      document.body.style.overflow = "";
    };
  }, [isOpen]);

  // Handle escape key and overlay click
  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape" && isOpen) {
        onClose();
      }
    };

    if (isOpen) {
      document.addEventListener("keydown", handleEscape);
    }

    return () => {
      document.removeEventListener("keydown", handleEscape);
    };
  }, [isOpen, onClose]);

  if (!isOpen || !quote) return null;

  const handleConfirm = () => {
    if (!termsAccepted) return;

    const request: BridgeSwapRequest = {
      quoteId: quote.id,
      fromChain: quote.fromChain,
      toChain: quote.toChain,
      fromToken: quote.fromToken,
      toToken: quote.toToken,
      amount: quote.fromAmount,
      recipient: recipient || "", // TODO: Get from wallet
      maxSlippage: parseFloat(quote.slippage),
    };

    onConfirm(request);
  };

  // Handle overlay click to close modal
  const handleOverlayClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  };

  return (
    <div className="swap-confirmation-overlay" onClick={handleOverlayClick}>
      <div className={`swap-confirmation ${className}`}>
        <div className="swap-confirmation__header">
          <h3>Confirm Swap</h3>
          <button onClick={onClose} className="swap-confirmation__close">
            ×
          </button>
        </div>

        <div className="swap-confirmation__content">
          <div className="swap-confirmation__transaction">
            <div className="swap-confirmation__amount">
              <div className="swap-confirmation__from">
                <span>
                  {quote.fromAmount} {quote.fromToken}
                </span>
                <span>on {quote.fromChain}</span>
              </div>
              <div className="swap-confirmation__arrow">↓</div>
              <div className="swap-confirmation__to">
                <span>
                  {quote.toAmount} {quote.toToken}
                </span>
                <span>on {quote.toChain}</span>
              </div>
            </div>

            <div className="swap-confirmation__details">
              <div className="swap-confirmation__detail">
                <span>Exchange Rate</span>
                <span>{quote.exchangeRate}</span>
              </div>
              <div className="swap-confirmation__detail">
                <span>Total Fees</span>
                <span>{quote.totalFees}</span>
              </div>
              <div className="swap-confirmation__detail">
                <span>Price Impact</span>
                <span>{quote.priceImpact}%</span>
              </div>
              <div className="swap-confirmation__detail">
                <span>Estimated Time</span>
                <span>{Math.ceil(quote.estimatedTime / 60)} minutes</span>
              </div>
            </div>

            {quote.quantumProtected && (
              <div className="swap-confirmation__security">
                🔒 This transaction will be protected with quantum cryptography
              </div>
            )}

            <div className="swap-confirmation__recipient">
              <label>
                Recipient Address (optional)
                <input
                  type="text"
                  value={recipient}
                  onChange={(e) => setRecipient(e.target.value)}
                  placeholder="Leave empty to use connected wallet"
                />
              </label>
            </div>
          </div>

          <div className="swap-confirmation__terms">
            <label className="swap-confirmation__checkbox">
              <input
                type="checkbox"
                checked={termsAccepted}
                onChange={(e) => setTermsAccepted(e.target.checked)}
              />
              I understand the risks and accept the terms of service
            </label>
          </div>

          <div className="swap-confirmation__actions">
            <button
              onClick={onClose}
              className="swap-confirmation__cancel"
              disabled={loading}
            >
              Cancel
            </button>
            <button
              onClick={handleConfirm}
              disabled={!termsAccepted || loading}
              className="swap-confirmation__confirm"
            >
              {loading ? "Processing..." : "Confirm Swap"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
