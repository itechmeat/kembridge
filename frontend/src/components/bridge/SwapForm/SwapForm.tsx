import {
  useState,
  useCallback,
  useEffect,
  FC,
  FormEvent,
  useRef,
  useMemo,
} from "react";
import cn from "classnames";
import type { SwapFormData, BridgeToken } from "../../../types/bridge";
import { TokenSelector } from "../TokenSelector/TokenSelector";
import { AmountInput } from "../AmountInput/AmountInput";
import { PriceQuote } from "../PriceQuote/PriceQuote";
import { SwapConfirmation } from "../SwapConfirmation/SwapConfirmation";
import { AuthManager } from "../../auth/AuthManager/AuthManager";
import { Button } from "../../ui";
import { Modal } from "../../ui/Modal/Modal";
import { useBridgeQuote } from "../../../hooks/bridge/useBridgeQuote";
import { useBridgeSwap } from "../../../hooks/bridge/useBridgeSwap";
import { useSupportedTokens } from "../../../hooks/bridge/useSupportedTokens";
import { useTransactionPolling } from "../../../hooks/bridge/useTransactionPolling";
import { useWallet } from "../../../hooks/wallet/useWallet";
import { useNearWallet } from "../../../hooks/wallet/useNearWallet";
import { useBalance } from "../../../hooks/wallet/useBalance";
import { useAuthStatus } from "../../../hooks/api/useAuth";
import { SecurityIndicator, RiskAnalysisDisplay } from "../../security";
import {
  useSecurityStatus,
  useRiskAnalysis,
  useRiskAnalysisLogger,
} from "../../../hooks/security";
import { SwapTransaction } from "../../../services/api/bridgeService";
import { ZERO_ADDRESS } from "../../../constants/services";
import type { BridgeSwapRequest } from "../../../types/bridge";
import type { NEARTransactionAction } from "../../../types/external";
import { useWebSocketContext } from "../../../contexts/WebSocketContext";
import { TransactionLinks } from "../TransactionLinks";
import styles from "./SwapForm.module.scss";

export interface SwapFormProps {
  onDataChange?: (data: Partial<SwapFormData>) => void;
  className?: string;
  disabled?: boolean;
}

export const SwapForm: FC<SwapFormProps> = ({
  onDataChange,
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
  const [currentTransaction, setCurrentTransaction] =
    useState<SwapTransaction | null>(null);
  const [successModal, setSuccessModal] = useState<{
    isOpen: boolean;
    txHash?: string;
    chain?: string;
  }>({ isOpen: false });

  // Polling for transaction updates
  const { transaction: polledTransaction, isPolling } = useTransactionPolling({
    transactionId: currentTransaction?.id || null,
    enabled: !!currentTransaction?.id,
  });
  const { account } = useWallet();
  const nearWallet = useNearWallet();
  const walletAddress = account?.address;
  const { isAuthenticated } = useAuthStatus();

  // Get wallet balances
  const { enrichTokensWithBalances, refresh: refreshBalances } = useBalance();

  // Refresh balances when transaction hashes are received
  useEffect(() => {
    const hasNewEthHash =
      polledTransaction?.from_transaction_hash ||
      currentTransaction?.from_transaction_hash;
    const hasNewNearHash =
      polledTransaction?.to_transaction_hash ||
      currentTransaction?.to_transaction_hash;

    if (hasNewEthHash || hasNewNearHash) {
      console.log("🔄 Transaction hash received, refreshing balances...");
      refreshBalances();
    }
  }, [
    polledTransaction?.from_transaction_hash,
    polledTransaction?.to_transaction_hash,
    currentTransaction?.from_transaction_hash,
    currentTransaction?.to_transaction_hash,
    refreshBalances,
  ]);

  // Debounce timeout для onDataChange
  const debounceTimeoutRef = useRef<number | null>(null);

  // Centralized risk analysis logger
  const { logRiskUpdate, cleanup: cleanupRiskLogger } =
    useRiskAnalysisLogger(1000);

  // WebSocket integration for real-time updates
  const {
    isConnected: wsConnected,
    latestBridgeUpdate,
    latestTransactionUpdate,
    subscribeToPriceUpdates,
    unsubscribeFromPriceUpdates,
  } = useWebSocketContext();

  // Fetch supported tokens - only when authenticated
  const { data: supportedTokens = [], isLoading: tokensLoading } =
    useSupportedTokens({
      enabled: isAuthenticated,
    });

  // Debounced amount for API calls
  const [debouncedAmount, setDebouncedAmount] = useState(formData.amount);

  useEffect(() => {
    const timer = setTimeout(() => {
      setDebouncedAmount(formData.amount);
    }, 500); // 500ms debounce

    return () => clearTimeout(timer);
  }, [formData.amount]);

  // Get quote when form data changes - only when authenticated (memoized params)
  const quoteParams = useMemo(
    () => ({
      fromToken: formData.fromToken?.symbol,
      toToken: formData.toToken?.symbol,
      fromChain: formData.fromChain,
      toChain: formData.toChain,
      amount: debouncedAmount, // Use debounced amount
      slippage: formData.slippage,
      quantumProtection: true,
    }),
    [
      formData.fromToken?.symbol,
      formData.toToken?.symbol,
      formData.fromChain,
      formData.toChain,
      debouncedAmount, // Use debounced amount
      formData.slippage,
    ]
  );

  const {
    data: quote,
    isLoading: quoteLoading,
    error: quoteError,
  } = useBridgeQuote(quoteParams, {
    enabled:
      isAuthenticated &&
      !!(
        formData.fromToken &&
        formData.toToken &&
        debouncedAmount &&
        parseFloat(debouncedAmount) > 0
      ),
  });

  // Bridge swap mutation
  const bridgeSwap = useBridgeSwap();

  // Security status
  const { quantumProtection, isOnline } = useSecurityStatus();

  // Risk analysis for current transaction (memoized)
  const riskAnalysisRequest = useMemo(() => {
    return isAuthenticated &&
      walletAddress &&
      formData.fromToken &&
      formData.toToken &&
      formData.amount &&
      parseFloat(formData.amount) > 0
      ? {
          fromAddress: walletAddress,
          toAddress: ZERO_ADDRESS, // Using constant instead of hardcoded
          amount: parseFloat(formData.amount) || 0,
          token: formData.fromToken?.symbol || "",
          chain: formData.fromChain || "",
        }
      : null;
  }, [
    isAuthenticated,
    walletAddress,
    formData.fromToken,
    formData.toToken,
    formData.amount,
    formData.fromChain,
  ]);

  const { riskAnalysis, riskScore } = useRiskAnalysis(riskAnalysisRequest, {
    enabled: !!riskAnalysisRequest,
    realTime: true,
  });

  // Notify parent component about form data changes (debounced)
  useEffect(() => {
    if (
      onDataChange &&
      formData.fromToken &&
      formData.toToken &&
      formData.amount &&
      parseFloat(formData.amount) > 0
    ) {
      // Очищаем предыдущий таймер
      if (debounceTimeoutRef.current) {
        clearTimeout(debounceTimeoutRef.current);
      }

      // Устанавливаем новый таймер с задержкой
      debounceTimeoutRef.current = window.setTimeout(() => {
        const completeData = {
          ...formData,
          recipient: walletAddress || ZERO_ADDRESS, // Using constant instead of hardcoded
        } as SwapFormData;
        onDataChange(completeData);
      }, 300); // 300ms дебаунс
    }

    // Очистка таймеров при размонтировании
    return () => {
      if (debounceTimeoutRef.current) {
        clearTimeout(debounceTimeoutRef.current);
      }
      cleanupRiskLogger();
    };
  }, [formData, walletAddress, onDataChange, cleanupRiskLogger]);

  // Auto-select default tokens when chains change (optimized)
  const fromTokens = useMemo(
    () =>
      enrichTokensWithBalances(
        supportedTokens.filter((t) => t.chain === formData.fromChain)
      ),
    [supportedTokens, formData.fromChain, enrichTokensWithBalances]
  );

  const toTokens = useMemo(
    () =>
      enrichTokensWithBalances(
        supportedTokens.filter((t) => t.chain === formData.toChain)
      ),
    [supportedTokens, formData.toChain, enrichTokensWithBalances]
  );

  useEffect(() => {
    if (supportedTokens.length === 0) return;

    if (!formData.fromToken && fromTokens.length > 0) {
      const defaultFromToken =
        fromTokens.find(
          (t) =>
            (t.chain === "ethereum" && t.symbol === "ETH") ||
            (t.chain === "near" && t.symbol === "NEAR")
        ) || fromTokens[0];
      setFormData((prev) => ({
        ...prev,
        fromToken: defaultFromToken as BridgeToken,
      }));
    }

    if (!formData.toToken && toTokens.length > 0) {
      const defaultToToken =
        toTokens.find(
          (t) =>
            (t.chain === "ethereum" && t.symbol === "ETH") ||
            (t.chain === "near" && t.symbol === "NEAR")
        ) || toTokens[0];
      setFormData((prev) => ({
        ...prev,
        toToken: defaultToToken as BridgeToken,
      }));
    }
  }, [
    fromTokens,
    toTokens,
    formData.fromToken,
    formData.toToken,
    supportedTokens.length,
  ]);

  // Update selected tokens with fresh balance data when balances change
  useEffect(() => {
    if (formData.fromToken && fromTokens.length > 0) {
      const updatedFromToken = fromTokens.find(
        (t) => t.symbol === formData.fromToken?.symbol
      );
      if (
        updatedFromToken &&
        updatedFromToken.balance !== formData.fromToken.balance
      ) {
        setFormData((prev) => ({ ...prev, fromToken: updatedFromToken }));
      }
    }

    if (formData.toToken && toTokens.length > 0) {
      const updatedToToken = toTokens.find(
        (t) => t.symbol === formData.toToken?.symbol
      );
      if (
        updatedToToken &&
        updatedToToken.balance !== formData.toToken.balance
      ) {
        setFormData((prev) => ({ ...prev, toToken: updatedToToken }));
      }
    }
  }, [fromTokens, toTokens, formData.fromToken, formData.toToken]);

  // Subscribe to price updates when tokens change (debounced)
  useEffect(() => {
    if (!formData.fromToken || !formData.toToken || !wsConnected) return;

    // Дебаунс для WebSocket подписок
    const timeoutId = setTimeout(() => {
      subscribeToPriceUpdates(
        formData.fromToken!.symbol,
        formData.toToken!.symbol
      );
    }, 500); // 500ms дебаунс для WebSocket подписок

    return () => {
      clearTimeout(timeoutId);
      unsubscribeFromPriceUpdates();
    };
  }, [
    formData.fromToken, // Полный объект токена
    formData.toToken,
    wsConnected,
    subscribeToPriceUpdates,
    unsubscribeFromPriceUpdates,
  ]);

  // Handle bridge operation updates
  useEffect(() => {
    if (latestBridgeUpdate) {
      // Update UI based on bridge operation status
      // This could trigger notifications, update progress bars, etc.
    }
  }, [latestBridgeUpdate]);

  // Handle transaction updates
  useEffect(() => {
    if (latestTransactionUpdate) {
      // Update UI based on transaction status
      // This could update progress indicators, show confirmations, etc.
    }
  }, [latestTransactionUpdate]);

  const handleFromTokenSelect = useCallback((token: BridgeToken) => {
    setFormData((prev) => ({ ...prev, fromToken: token }));
  }, []);

  const handleToTokenSelect = useCallback((token: BridgeToken) => {
    setFormData((prev) => ({ ...prev, toToken: token }));
  }, []);

  const handleAmountChange = useCallback((amount: string) => {
    // Update form data immediately for UI responsiveness
    setFormData((prev) => ({ ...prev, amount }));
  }, []);

  const handleMaxClick = useCallback(() => {
    if (formData.fromToken?.balance) {
      const maxAmount = formData.fromToken.balance;
      setFormData((prev) => ({
        ...prev,
        amount: maxAmount,
      }));
      // Update debounced amount immediately for MAX click
      setDebouncedAmount(maxAmount);
    }
  }, [formData.fromToken]);

  const handleChainSwap = useCallback(() => {
    setFormData((prev) => {
      // Calculate the correct receive amount to transfer
      let newAmount = "";
      if (quote && prev.amount) {
        // Get the corrected to_amount (same logic as in display)
        const isNearToEth =
          prev.fromChain === "near" && prev.toChain === "ethereum";
        let toAmount = parseFloat(quote.to_amount);

        if (isNearToEth) {
          toAmount = parseFloat(prev.amount) / toAmount;
        }

        newAmount = toAmount.toString();
      }

      // Update debounced amount immediately for chain swap
      setDebouncedAmount(newAmount);

      return {
        ...prev,
        fromChain: prev.toChain,
        toChain: prev.fromChain,
        fromToken: prev.toToken,
        toToken: prev.fromToken,
        amount: newAmount, // Transfer calculated amount
      };
    });
  }, [quote]);

  const handleGetQuote = useCallback(
    (e: FormEvent) => {
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
        const fromChain = formData.fromChain;
        let realTxHash: string;

        console.log(
          `🔥 REAL BLOCKCHAIN TRANSACTION: Starting swap from ${fromChain}:`,
          request
        );
        console.log("🚀 Wallet address:", walletAddress);

        if (!walletAddress) {
          throw new Error("Wallet not connected");
        }

        if (fromChain === "ethereum") {
          // Step 1: Execute REAL Ethereum transaction via MetaMask
          if (!window.ethereum) {
            throw new Error("MetaMask not connected");
          }

          console.log("📤 Sending REAL transaction via MetaMask...");

          const transactionParams = {
            to: walletAddress, // 🔥 REAL TEST: Send to self for testing
            value: "0x" + (parseFloat(request.amount) * 1e18).toString(16), // Convert ETH to wei
            gasLimit: "0x5208", // 21000 gas
          };

          realTxHash = await window.ethereum.request({
            method: "eth_sendTransaction",
            params: [
              {
                from: walletAddress,
                ...transactionParams,
              },
            ],
          });

          console.log("🎉 REAL ETHEREUM TRANSACTION SENT! Hash:", realTxHash);
        } else if (fromChain === "near") {
          // Step 1: Execute REAL NEAR transaction via NEAR Wallet
          if (!nearWallet.selector) {
            throw new Error("NEAR wallet not connected");
          }

          console.log("📤 Sending REAL transaction via NEAR Wallet...");

          const wallet = await nearWallet.selector.wallet();

          // Create NEAR transfer transaction
          const actions: NEARTransactionAction[] = [
            {
              type: "Transfer",
              params: {
                deposit: (parseFloat(request.amount) * 1e24).toString(), // Convert NEAR to yoctoNEAR
              },
            },
          ];

          const nearTransactionParams = {
            receiverId: walletAddress, // Send to self for testing
            actions,
          };

          const result = await wallet.signAndSendTransaction(
            nearTransactionParams
          );
          realTxHash = result.transaction.hash;

          console.log("🎉 REAL NEAR TRANSACTION SENT! Hash:", realTxHash);
        } else {
          throw new Error(`Unsupported chain: ${fromChain}`);
        }

        console.log("🎉 REAL TRANSACTION SENT! Hash:", realTxHash);

        // Log explorer links for different chains
        if (fromChain === "ethereum") {
          console.log(
            "🔍 Sepolia Etherscan:",
            `https://sepolia.etherscan.io/tx/${realTxHash}`
          );
        } else if (fromChain === "near") {
          console.log(
            "🔍 NEAR Explorer:",
            `https://testnet.nearblocks.io/txns/${realTxHash}`
          );
        }

        // Step 2: Send REAL transaction hash to backend
        const initRequest = {
          quote_id: request.quoteId,
          from_wallet_address: walletAddress,
          to_wallet_address: request.recipient,
          max_slippage: request.maxSlippage,
          real_transaction_hash: realTxHash, // Include real hash
        };

        console.log(
          "📨 Sending REAL transaction hash to backend:",
          initRequest
        );
        const result = await bridgeSwap.mutateAsync(initRequest);
        console.log("✅ Backend updated with REAL transaction:", result);

        // Save transaction for displaying links
        setCurrentTransaction({
          ...result,
          from_transaction_hash: realTxHash, // Ensure we show the real hash
        });
        setShowConfirmation(false);

        // Reset form to initial state after successful transaction
        setFormData({
          fromChain: "ethereum",
          toChain: "near",
          amount: "",
          slippage: 0.5,
          // Keep fromToken and toToken as they will be auto-selected
        });

        // Show success modal instead of alert
        setSuccessModal({
          isOpen: true,
          txHash: realTxHash,
          chain: fromChain,
        });
      } catch (error) {
        console.error("❌ REAL transaction failed:", error);
        if (
          error &&
          typeof error === "object" &&
          "code" in error &&
          error.code === 4001
        ) {
          console.log("User rejected transaction");
        }
        setShowConfirmation(false);
      }
    },
    [bridgeSwap, walletAddress, nearWallet, formData.fromChain]
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
      <div
        className={cn(styles.swapForm, className.trim())}
        data-testid="swap-form"
      >
        <div className={styles.authRequired} data-testid="auth-required">
          <div className={styles.authHeader}>
            <h2 className={styles.title}>Cross-Chain Bridge</h2>
            <div className={styles.quantumBadge}>🔒 Quantum Protected</div>
          </div>
          <div className={styles.authPrompt}>
            <p className={styles.authMessage} data-testid="sign-in-message">
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
      <form
        className={cn(styles.swapForm, className.trim())}
        onSubmit={handleGetQuote}
        data-testid="swap-form"
      >
        <div className={styles.content}>
          <div className={styles.header}>
            <h2 className={styles.title}>Cross-Chain Bridge</h2>
            <div className={styles.headerBadges}>
              <div className={styles.quantumBadge}>🔒 Quantum Protected</div>
            </div>

            <div className={styles.security}>
              <SecurityIndicator
                quantumProtection={quantumProtection}
                riskScore={riskScore || 0}
                isOnline={isOnline}
                compact={true}
              />
            </div>
          </div>

          {/* From Section */}
          <div className={styles.section}>
            <div className={styles.sectionHeader}>
              <label className={styles.label}>From</label>
              <span className={styles.chain}>{formData.fromChain}</span>
            </div>

            <div className={styles.inputGroup}>
              <TokenSelector
                selectedToken={formData.fromToken}
                chain={formData.fromChain!}
                tokens={supportedTokens as BridgeToken[]}
                onTokenSelect={handleFromTokenSelect}
                disabled={disabled || tokensLoading}
                showBalance={true}
                className={styles.tokenSelector}
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
                className={styles.amountInput}
              />
            </div>
          </div>

          {/* Swap Direction Button */}
          <div className={styles.swapDirection}>
            <button
              type="button"
              onClick={handleChainSwap}
              disabled={disabled}
              className={styles.swapButton}
              data-testid="swap-direction-button"
            >
              ⇅
            </button>
          </div>

          {/* To Section */}
          <div className={styles.section}>
            <div className={styles.sectionHeader}>
              <label className={styles.label}>To</label>
              <span className={styles.chain}>{formData.toChain}</span>
            </div>

            <div className={styles.inputGroup}>
              <TokenSelector
                selectedToken={formData.toToken}
                chain={formData.toChain!}
                tokens={supportedTokens as BridgeToken[]}
                onTokenSelect={handleToTokenSelect}
                disabled={disabled || tokensLoading}
                showBalance={true}
                className={styles.tokenSelector}
              />

              <div className={styles.receiveAmount}>
                {!formData.amount || parseFloat(formData.amount) === 0
                  ? "0.0"
                  : !quote
                  ? "..."
                  : (() => {
                      // API returns amounts in token units, but NEAR->ETH rates are inverted
                      let num = parseFloat(quote.to_amount);

                      // Fix inverted rates for NEAR -> ETH
                      if (
                        formData.fromChain === "near" &&
                        formData.toChain === "ethereum"
                      ) {
                        num = parseFloat(formData.amount) / num;
                      }

                      if (isNaN(num) || num === 0) return "0.0";
                      if (num < 0.000001) return "< 0.000001";
                      if (num < 0.001) return num.toFixed(6);
                      if (num < 1) return num.toFixed(4);
                      if (num >= 1000) return (num / 1000).toFixed(2) + "K";
                      return num.toLocaleString(undefined, {
                        maximumFractionDigits: 4,
                      });
                    })()}
              </div>
            </div>
          </div>

          {/* Slippage Settings */}
          <div className={styles.settings}>
            <label className={styles.slippageLabel}>
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
              className={styles.slippageSlider}
              data-testid="slippage-slider"
            />
          </div>

          {/* Risk Analysis */}
          {riskAnalysis && (
            <div className={styles.riskAnalysis}>
              <RiskAnalysisDisplay
                riskData={riskAnalysis}
                realTime={true}
                showDetails={false} // Compact for swap form
                onUpdate={logRiskUpdate}
              />
            </div>
          )}

          {/* Price Quote Display */}
          <PriceQuote
            quote={
              quote
                ? (() => {
                    // Fix inverted rates for NEAR -> ETH
                    const isNearToEth =
                      formData.fromChain === "near" &&
                      formData.toChain === "ethereum";
                    const toAmount = isNearToEth
                      ? (
                          parseFloat(formData.amount!) /
                          parseFloat(quote.to_amount)
                        ).toString()
                      : quote.to_amount;
                    const exchangeRate = isNearToEth
                      ? (1 / quote.exchange_rate).toString()
                      : quote.exchange_rate?.toString() || "0";

                    return {
                      id: quote.quote_id,
                      fromToken: formData.fromToken?.symbol || "",
                      toToken: formData.toToken?.symbol || "",
                      fromChain: formData.fromChain!,
                      toChain: formData.toChain!,
                      fromAmount: formData.amount!,
                      toAmount,
                      exchangeRate,
                      estimatedGas: quote.estimated_fees?.gas_fee || "0",
                      bridgeFee: quote.estimated_fees?.bridge_fee || "0",
                      protocolFee: quote.estimated_fees?.protocol_fee || "0",
                      totalFees: quote.estimated_fees?.total_fee || "0",
                      priceImpact:
                        typeof quote.price_impact === "object"
                          ? (
                              quote.price_impact as { percentage?: number }
                            )?.percentage?.toString() || "0"
                          : quote.price_impact?.toString() || "0",
                      slippage: formData.slippage!.toString(),
                      estimatedTime: quote.estimated_time_minutes * 60,
                      expiresAt: quote.expires_at,
                      quantumProtected: quote.quantum_protection_enabled,
                      riskScore: quote.route_info?.risk_score,
                    };
                  })()
                : undefined
            }
            loading={quoteLoading}
            error={quoteError?.message}
            className={styles.priceQuote}
            compact={true} // Mobile-first: use compact display
          />

          {/* Submit Button */}
          <button
            type="submit"
            className={styles.submit}
            disabled={disabled || !isFormValid || quoteLoading || !quote}
            data-testid="swap-button"
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
            ? (() => {
                // Fix inverted rates for NEAR -> ETH
                const isNearToEth =
                  formData.fromChain === "near" &&
                  formData.toChain === "ethereum";
                const toAmount = isNearToEth
                  ? (
                      parseFloat(formData.amount!) / parseFloat(quote.to_amount)
                    ).toString()
                  : quote.to_amount;
                const exchangeRate = isNearToEth
                  ? (1 / quote.exchange_rate).toString()
                  : quote.exchange_rate?.toString() || "0";

                return {
                  id: quote.quote_id,
                  fromToken: formData.fromToken?.symbol || "",
                  toToken: formData.toToken?.symbol || "",
                  fromChain: formData.fromChain!,
                  toChain: formData.toChain!,
                  fromAmount: formData.amount!,
                  toAmount,
                  exchangeRate,
                  estimatedGas: quote.estimated_fees?.gas_fee || "0",
                  bridgeFee: quote.estimated_fees?.bridge_fee || "0",
                  protocolFee: quote.estimated_fees?.protocol_fee || "0",
                  totalFees: quote.estimated_fees?.total_fee || "0",
                  priceImpact:
                    typeof quote.price_impact === "object"
                      ? (
                          quote.price_impact as { percentage?: number }
                        )?.percentage?.toString() || "0"
                      : quote.price_impact?.toString() || "0",
                  slippage: formData.slippage!.toString(),
                  estimatedTime: quote.estimated_time_minutes * 60,
                  expiresAt: quote.expires_at,
                  quantumProtected: quote.quantum_protection_enabled,
                  riskScore: quote.route_info?.risk_score,
                };
              })()
            : undefined
        }
        loading={bridgeSwap.isPending}
      />

      {/* Transaction Links - show after successful swap */}
      {currentTransaction && (
        <div>
          {(() => {
            const fromTxHash =
              polledTransaction?.from_transaction_hash ||
              currentTransaction.from_transaction_hash;
            const toTxHash =
              polledTransaction?.to_transaction_hash ||
              currentTransaction.to_transaction_hash;
            const bothTransactionsComplete = fromTxHash && toTxHash;

            return (
              <>
                {isPolling && !bothTransactionsComplete && (
                  <div
                    style={{
                      fontSize: "12px",
                      color: "#666",
                      marginBottom: "8px",
                      textAlign: "center",
                    }}
                  >
                    Checking transaction status...
                  </div>
                )}
                <TransactionLinks
                  fromChain={formData.fromChain || "ethereum"}
                  toChain={formData.toChain || "near"}
                  fromTxHash={fromTxHash}
                  toTxHash={toTxHash}
                  fromToken={formData.fromToken?.symbol}
                  toToken={formData.toToken?.symbol}
                  compact={true}
                />
              </>
            );
          })()}
        </div>
      )}

      {/* Success Modal */}
      <Modal
        isOpen={successModal.isOpen}
        onClose={() => setSuccessModal({ isOpen: false })}
        title="Transaction Sent Successfully"
        className={styles.successModal}
      >
        <div className={styles.successContent}>
          <div className={styles.successIcon}>✅</div>
          <p className={styles.successMessage}>
            Your transaction has been sent to the blockchain!
          </p>
          {successModal.txHash && (
            <div className={styles.txHashContainer}>
              <p className={styles.txHashLabel}>Transaction Hash:</p>
              <code className={styles.txHash}>{successModal.txHash}</code>
              <a
                href={
                  successModal.chain === "near"
                    ? `https://testnet.nearblocks.io/txns/${successModal.txHash}`
                    : `https://sepolia.etherscan.io/tx/${successModal.txHash}`
                }
                target="_blank"
                rel="noopener noreferrer"
                className={styles.etherscanLink}
              >
                View on{" "}
                {successModal.chain === "near"
                  ? "NEAR Explorer"
                  : "Sepolia Etherscan"}{" "}
                ↗
              </a>
            </div>
          )}
          <Button
            onClick={() => setSuccessModal({ isOpen: false })}
            className={styles.successButton}
          >
            Continue
          </Button>
        </div>
      </Modal>
    </>
  );
};
