/**
 * SwapForm Component
 * Main swap interface with token selection and amount input
 */

import React, { useState, useCallback, useEffect } from "react";
import type { SwapFormData, BridgeToken } from "../../../types/bridge";
import { TokenSelector } from "../TokenSelector/TokenSelector";
import { AmountInput } from "../AmountInput/AmountInput";
import { PriceQuote } from "../PriceQuote/PriceQuote";
import { SwapConfirmation } from "../SwapConfirmation/SwapConfirmation";
import { AuthManager } from "../../auth/AuthManager/AuthManager";
import { useBridgeQuote } from "../../../hooks/bridge/useBridgeQuote";
import { useBridgeSwap } from "../../../hooks/bridge/useBridgeSwap";
import { useSupportedTokens } from "../../../hooks/bridge/useSupportedTokens";
import { useWallet } from "../../../hooks/wallet/useWallet";
import { useAuthStatus } from "../../../hooks/api/useAuth";
import type { BridgeSwapRequest } from "../../../types/bridge";

export interface SwapFormProps {
  onSwapExecute?: (data: SwapFormData) => void;
  className?: string;
  disabled?: boolean;
}

export const SwapForm: React.FC<SwapFormProps> = ({
  onSwapExecute,
  className = "",
  disabled = false,
}) => {
  const [formData, setFormData] = useState<Partial<SwapFormData>>({
    fromChain: "ethereum",
    toChain: "near",
    amount: "",
    slippage: 0.5,
  });

  const [showConfirmation, setShowConfirmation] = useState(false);
  const { account } = useWallet();
  const walletAddress = account?.address;
  const { isAuthenticated } = useAuthStatus();

  // Fetch supported tokens - only when authenticated
  const { data: supportedTokens = [], isLoading: tokensLoading } =
    useSupportedTokens({
      enabled: isAuthenticated,
    });

  // Get quote when form data changes - only when authenticated
  const {
    data: quote,
    isLoading: quoteLoading,
    error: quoteError,
  } = useBridgeQuote(
    {
      fromToken: formData.fromToken?.symbol,
      toToken: formData.toToken?.symbol,
      fromChain: formData.fromChain,
      toChain: formData.toChain,
      amount: formData.amount,
      slippage: formData.slippage,
      quantumProtection: true,
    },
    {
      enabled:
        isAuthenticated &&
        !!(
          formData.fromToken &&
          formData.toToken &&
          formData.amount &&
          parseFloat(formData.amount) > 0
        ),
    }
  );

  // Bridge swap mutation
  const bridgeSwap = useBridgeSwap();

  // Auto-select default tokens when chains change
  useEffect(() => {
    if (supportedTokens.length === 0) return;

    const fromTokens = supportedTokens.filter(
      (t) => t.chain === formData.fromChain
    );
    const toTokens = supportedTokens.filter(
      (t) => t.chain === formData.toChain
    );

    if (!formData.fromToken && fromTokens.length > 0) {
      const defaultFromToken =
        fromTokens.find((t) => t.is_native) || fromTokens[0];
      setFormData((prev) => ({
        ...prev,
        fromToken: defaultFromToken as BridgeToken,
      }));
    }

    if (!formData.toToken && toTokens.length > 0) {
      const defaultToToken = toTokens.find((t) => t.is_native) || toTokens[0];
      setFormData((prev) => ({
        ...prev,
        toToken: defaultToToken as BridgeToken,
      }));
    }
  }, [
    formData.fromChain,
    formData.toChain,
    supportedTokens,
    formData.fromToken,
    formData.toToken,
  ]);

  const handleFromTokenSelect = useCallback((token: BridgeToken) => {
    setFormData((prev) => ({ ...prev, fromToken: token }));
  }, []);

  const handleToTokenSelect = useCallback((token: BridgeToken) => {
    setFormData((prev) => ({ ...prev, toToken: token }));
  }, []);

  const handleAmountChange = useCallback((amount: string) => {
    setFormData((prev) => ({ ...prev, amount }));
  }, []);

  const handleMaxClick = useCallback(() => {
    if (formData.fromToken?.balance) {
      setFormData((prev) => ({
        ...prev,
        amount: formData.fromToken!.balance!,
      }));
    }
  }, [formData.fromToken]);

  const handleChainSwap = useCallback(() => {
    setFormData((prev) => ({
      ...prev,
      fromChain: prev.toChain,
      toChain: prev.fromChain,
      fromToken: prev.toToken,
      toToken: prev.fromToken,
    }));
  }, []);

  const handleGetQuote = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();

      if (quote && formData.fromToken && formData.toToken && formData.amount) {
        setShowConfirmation(true);
      }
    },
    [quote, formData]
  );

  const handleConfirmSwap = useCallback(
    async (request: BridgeSwapRequest) => {
      try {
        // Convert BridgeSwapRequest to InitSwapRequest
        const initRequest = {
          quote_id: request.quoteId,
          from_wallet_address: walletAddress || '',
          to_wallet_address: request.recipient,
          max_slippage: request.maxSlippage,
        };
        const result = await bridgeSwap.mutateAsync(initRequest);
        console.log("Swap initiated:", result);

        // Execute callback if provided
        if (onSwapExecute && formData.fromToken && formData.toToken) {
          onSwapExecute({
            fromToken: formData.fromToken,
            toToken: formData.toToken,
            fromChain: formData.fromChain!,
            toChain: formData.toChain!,
            amount: formData.amount!,
            slippage: formData.slippage || 0.5,
            recipient: request.recipient,
          });
        }

        setShowConfirmation(false);
        // TODO: Navigate to transaction tracking page
      } catch (error) {
        console.error("Swap failed:", error);
        // Error handling is done by the mutation
      }
    },
    [bridgeSwap, onSwapExecute, formData, walletAddress]
  );

  const isFormValid = !!(
    formData.fromToken &&
    formData.toToken &&
    formData.amount &&
    parseFloat(formData.amount) > 0 &&
    walletAddress &&
    isAuthenticated
  );

  // Show authentication prompt if not authenticated
  if (!isAuthenticated) {
    return (
      <div className={`swap-form ${className}`}>
        <div className="swap-form__auth-required">
          <div className="swap-form__auth-header">
            <h2 className="swap-form__title">Cross-Chain Bridge</h2>
            <div className="swap-form__quantum-badge">🔒 Quantum Protected</div>
          </div>
          <div className="swap-form__auth-prompt">
            <p className="swap-form__auth-message">
              Sign in with your wallet to access the bridge
            </p>
            <AuthManager
              onAuthSuccess={() => {
                console.log("✅ SwapForm: Authentication successful");
              }}
              onAuthError={(error) => {
                console.error("❌ SwapForm: Authentication failed:", error);
              }}
            />
          </div>
        </div>
      </div>
    );
  }

  return (
    <>
      <form className={`swap-form ${className}`} onSubmit={handleGetQuote}>
        <div className="swap-form__content">
          <div className="swap-form__header">
            <h2 className="swap-form__title">Cross-Chain Bridge</h2>
            <div className="swap-form__quantum-badge">🔒 Quantum Protected</div>
          </div>

          {/* From Section */}
          <div className="swap-form__section">
            <div className="swap-form__section-header">
              <label className="swap-form__label">From</label>
              <span className="swap-form__chain">{formData.fromChain}</span>
            </div>

            <div className="swap-form__input-group">
              <TokenSelector
                selectedToken={formData.fromToken}
                chain={formData.fromChain!}
                tokens={supportedTokens as BridgeToken[]}
                onTokenSelect={handleFromTokenSelect}
                disabled={disabled || tokensLoading}
                showBalance={true}
                className="swap-form__token-selector"
              />

              <AmountInput
                value={formData.amount!}
                onChange={handleAmountChange}
                token={formData.fromToken}
                balance={formData.fromToken?.balance}
                disabled={disabled}
                placeholder="0.0"
                showUsdValue={true}
                onMaxClick={handleMaxClick}
                className="swap-form__amount-input"
              />
            </div>
          </div>

          {/* Swap Direction Button */}
          <div className="swap-form__swap-direction">
            <button
              type="button"
              onClick={handleChainSwap}
              disabled={disabled}
              className="swap-form__swap-button"
            >
              ⇅
            </button>
          </div>

          {/* To Section */}
          <div className="swap-form__section">
            <div className="swap-form__section-header">
              <label className="swap-form__label">To</label>
              <span className="swap-form__chain">{formData.toChain}</span>
            </div>

            <div className="swap-form__input-group">
              <TokenSelector
                selectedToken={formData.toToken}
                chain={formData.toChain!}
                tokens={supportedTokens as BridgeToken[]}
                onTokenSelect={handleToTokenSelect}
                disabled={disabled || tokensLoading}
                showBalance={false}
                className="swap-form__token-selector"
              />

              <div className="swap-form__receive-amount">
                {quote ? quote.to_amount : "0.0"}
              </div>
            </div>
          </div>

          {/* Slippage Settings */}
          <div className="swap-form__settings">
            <label className="swap-form__slippage-label">
              Max Slippage: {formData.slippage}%
            </label>
            <input
              type="range"
              min="0.1"
              max="5"
              step="0.1"
              value={formData.slippage}
              onChange={(e) =>
                setFormData((prev) => ({
                  ...prev,
                  slippage: parseFloat(e.target.value),
                }))
              }
              className="swap-form__slippage-slider"
            />
          </div>

          {/* Price Quote Display */}
          <PriceQuote
            quote={
              quote
                ? {
                    id: quote.quote_id,
                    fromToken: formData.fromToken?.symbol || "",
                    toToken: formData.toToken?.symbol || "",
                    fromChain: formData.fromChain!,
                    toChain: formData.toChain!,
                    fromAmount: formData.amount!,
                    toAmount: quote.to_amount,
                    exchangeRate: quote.exchange_rate?.toString() || "0",
                    estimatedGas: quote.estimated_fees?.gas_fee || "0",
                    bridgeFee: quote.estimated_fees?.bridge_fee || "0",
                    protocolFee: quote.estimated_fees?.protocol_fee || "0",
                    totalFees: quote.estimated_fees?.total_fee || "0",
                    priceImpact: quote.price_impact?.toString() || "0",
                    slippage: formData.slippage!.toString(),
                    estimatedTime: quote.estimated_time_minutes * 60,
                    expiresAt: quote.expires_at,
                    quantumProtected: quote.quantum_protection_enabled,
                    riskScore: quote.route_info?.risk_score,
                  }
                : undefined
            }
            loading={quoteLoading}
            error={quoteError?.message}
            className="swap-form__price-quote"
            compact={true} // Mobile-first: use compact display
          />

          {/* Submit Button */}
          <button
            type="submit"
            className="swap-form__submit"
            disabled={disabled || !isFormValid || quoteLoading || !quote}
          >
            {!walletAddress
              ? "Connect Wallet"
              : quoteLoading
              ? "Getting Quote..."
              : !quote
              ? "Enter Amount"
              : "Review Swap"}
          </button>
        </div>
      </form>

      {/* Confirmation Modal */}
      <SwapConfirmation
        isOpen={showConfirmation}
        onClose={() => setShowConfirmation(false)}
        onConfirm={handleConfirmSwap}
        quote={
          quote
            ? {
                id: quote.quote_id,
                fromToken: formData.fromToken?.symbol || "",
                toToken: formData.toToken?.symbol || "",
                fromChain: formData.fromChain!,
                toChain: formData.toChain!,
                fromAmount: formData.amount!,
                toAmount: quote.to_amount,
                exchangeRate: quote.exchange_rate?.toString() || "0",
                estimatedGas: quote.estimated_fees?.gas_fee || "0",
                bridgeFee: quote.estimated_fees?.bridge_fee || "0",
                protocolFee: quote.estimated_fees?.protocol_fee || "0",
                totalFees: quote.estimated_fees?.total_fee || "0",
                priceImpact: quote.price_impact?.toString() || "0",
                slippage: formData.slippage!.toString(),
                estimatedTime: quote.estimated_time_minutes * 60,
                expiresAt: quote.expires_at,
                quantumProtected: quote.quantum_protection_enabled,
                riskScore: quote.route_info?.risk_score,
              }
            : undefined
        }
        loading={bridgeSwap.isPending}
      />
    </>
  );
};
