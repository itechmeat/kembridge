import { useState, FC, memo, useMemo } from "react";
import cn from "classnames";
import { Modal } from "../../ui/Modal/Modal";
import type { BridgeQuote, BridgeSwapRequest } from "../../../types/bridge";
import styles from "./SwapConfirmation.module.scss";

export interface SwapConfirmationProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: (request: BridgeSwapRequest) => void;
  quote?: BridgeQuote;
  loading?: boolean;
  className?: string;
}

export const SwapConfirmation: FC<SwapConfirmationProps> = memo(
  ({ isOpen, onClose, onConfirm, quote, loading = false, className = "" }) => {
    const [termsAccepted, setTermsAccepted] = useState(false);
    const [recipient, setRecipient] = useState("");

    // Мемоизация request объекта для избежания пересоздания
    const swapRequest = useMemo((): BridgeSwapRequest | null => {
      if (!quote) return null;

      return {
        quoteId: quote.id,
        fromChain: quote.fromChain,
        toChain: quote.toChain,
        fromToken: quote.fromToken,
        toToken: quote.toToken,
        amount: quote.fromAmount,
        recipient: recipient || "", // TODO: Get from wallet
        maxSlippage: parseFloat(quote.slippage),
      };
    }, [quote, recipient]);

    if (!isOpen || !quote) return null;

    const handleConfirm = () => {
      if (!termsAccepted || !swapRequest) return;
      onConfirm(swapRequest);
    };

    return (
      <Modal
        isOpen={isOpen}
        onClose={onClose}
        title="Confirm Swap"
        className={cn(styles.swapConfirmation, className)}
      >
        <div className={styles.content}>
          <div className={styles.transaction}>
            <div className={styles.amount}>
              <div className={styles.from}>
                <span>
                  {quote.fromAmount} {quote.fromToken}
                </span>
                <span>on {quote.fromChain}</span>
              </div>
              <div className={styles.arrow}>↓</div>
              <div className={styles.to}>
                <span>
                  {quote.toAmount} {quote.toToken}
                </span>
                <span>on {quote.toChain}</span>
              </div>
            </div>

            <div className={styles.details}>
              <div className={styles.detail}>
                <span>Exchange Rate</span>
                <span>{quote.exchangeRate}</span>
              </div>
              <div className={styles.detail}>
                <span>Total Fees</span>
                <span>{quote.totalFees}</span>
              </div>
              <div className={styles.detail}>
                <span>Price Impact</span>
                <span>{quote.priceImpact}%</span>
              </div>
              <div className={styles.detail}>
                <span>Estimated Time</span>
                <span>{Math.ceil(quote.estimatedTime / 60)} minutes</span>
              </div>
            </div>

            {quote.quantumProtected && (
              <div className={styles.security}>
                🔒 This transaction will be protected with quantum cryptography
              </div>
            )}

            <div className={styles.recipient}>
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

          <div className={styles.terms}>
            <label className={styles.checkbox}>
              <input
                type="checkbox"
                checked={termsAccepted}
                onChange={(e) => setTermsAccepted(e.target.checked)}
              />
              I understand the risks and accept the terms of service
            </label>
          </div>

          <div className={styles.actions}>
            <button
              onClick={onClose}
              className={styles.cancel}
              disabled={loading}
            >
              Cancel
            </button>
            <button
              onClick={handleConfirm}
              disabled={!termsAccepted || loading}
              className={styles.confirm}
              data-testid="confirm-swap-button"
            >
              {loading ? "Processing..." : "Confirm Swap"}
            </button>
          </div>
        </div>
      </Modal>
    );
  }
);
