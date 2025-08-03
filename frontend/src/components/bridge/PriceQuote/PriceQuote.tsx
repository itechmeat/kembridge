import { useState, useEffect, FC } from "react";
import cn from "classnames";
import type { BridgeQuote } from "../../../types/bridge";
import { CoinIcon } from "../../ui";
import styles from "./PriceQuote.module.scss";

export interface PriceQuoteProps {
  quote?: BridgeQuote;
  loading?: boolean;
  error?: string;
  className?: string;
  showDetails?: boolean;
  compact?: boolean; // Mobile-first: option for compact display
}

export const PriceQuote: FC<PriceQuoteProps> = ({
  quote,
  loading = false,
  error,
  className = "",
  showDetails = true,
  compact = false,
}) => {
  const [timeRemaining, setTimeRemaining] = useState<number>(0);
  const [expanded, setExpanded] = useState(false);

  // Update countdown timer
  useEffect(() => {
    if (!quote?.expiresAt) return;

    const updateTimer = () => {
      const remaining = new Date(quote.expiresAt).getTime() - Date.now();
      setTimeRemaining(Math.max(0, remaining));
    };

    updateTimer();
    const interval = setInterval(updateTimer, 1000);

    return () => clearInterval(interval);
  }, [quote?.expiresAt]);

  const formatTimeRemaining = (ms: number): string => {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;

    if (minutes > 0) {
      return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
    }
    return `${remainingSeconds}s`;
  };

  const formatFee = (fee: string): string => {
    const num = parseFloat(fee);
    if (isNaN(num)) return fee;

    if (num < 0.0001) return "< 0.0001";
    if (num < 1) return num.toFixed(6);
    return num.toFixed(4);
  };

  const formatAmount = (amount: string): string => {
    const num = parseFloat(amount);
    if (isNaN(num)) return amount;

    // Format very large numbers with commas and limit decimals
    if (num >= 1000000) {
      return (num / 1000000).toFixed(2) + "M";
    }
    if (num >= 1000) {
      return (num / 1000).toFixed(2) + "K";
    }
    if (num >= 1) {
      return num.toLocaleString(undefined, { maximumFractionDigits: 6 });
    }
    // For small numbers, show up to 6 decimals
    return num.toFixed(6);
  };

  const formatExchangeRate = (rate: string): string => {
    const num = parseFloat(rate);
    if (isNaN(num)) return rate;

    // Show at most 4 decimal places for exchange rate
    if (num >= 1000) {
      return num.toLocaleString(undefined, { maximumFractionDigits: 2 });
    }
    return num.toFixed(4);
  };

  const getPriceImpactSeverity = (
    impact: string
  ): "low" | "medium" | "high" => {
    const num = parseFloat(impact);
    if (num < 1) return "low";
    if (num < 5) return "medium";
    return "high";
  };

  const getRiskLevel = (score?: number): "low" | "medium" | "high" => {
    if (!score) return "low";
    if (score < 30) return "low";
    if (score < 70) return "medium";
    return "high";
  };

  if (loading) {
    return (
      <div className={cn(styles.priceQuote, styles.loading, className.trim())}>
        <div className={styles.spinner}>
          <div className={styles.spinnerIcon}>⏳</div>
          <span>Getting best quote...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={cn(styles.priceQuote, styles.error, className.trim())}>
        <div className={styles.errorMsg}>
          <span className={styles.errorIcon}>⚠️</span>
          <span>{error}</span>
        </div>
      </div>
    );
  }

  if (!quote) {
    return (
      <div className={cn(styles.priceQuote, styles.empty, className.trim())}>
        <div className={styles.empty}>
          <span className={styles.emptyIcon}>💱</span>
          <p>Enter amount to get quote</p>
        </div>
      </div>
    );
  }

  const priceImpactSeverity = getPriceImpactSeverity(quote.priceImpact);
  const riskLevel = getRiskLevel(quote.riskScore);

  return (
    <div
      className={cn(styles.priceQuote, className.trim())}
      data-testid="price-quote"
    >
      {/* Main Quote Display */}
      <div className={styles.main}>
        <div className={styles.header}>
          <div className={styles.rate}>
            <span className={styles.rateLabel}>You'll receive</span>
            <div className={styles.rateValue}>
              <span>{formatAmount(quote.toAmount)}</span>
              <div className={styles.tokenDisplay}>
                <CoinIcon symbol={quote.toToken} size="small" />
                <span>{quote.toToken}</span>
              </div>
            </div>
          </div>

          {timeRemaining > 0 && (
            <div className={styles.timer}>
              <span className={styles.timerIcon}>⏱️</span>
              <span>{formatTimeRemaining(timeRemaining)}</span>
            </div>
          )}
        </div>

        <div className={styles.exchangeRate}>
          <div className={styles.exchangeTokens}>
            <div className={styles.tokenDisplay}>
              <CoinIcon symbol={quote.fromToken} size="small" />
              <span>1 {quote.fromToken}</span>
            </div>
            <span>=</span>
            <div className={styles.tokenDisplay}>
              <span>{formatExchangeRate(quote.exchangeRate)}</span>
              <CoinIcon symbol={quote.toToken} size="small" />
              <span>{quote.toToken}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Quick Stats */}
      <div className={styles.quickStats}>
        <div className={styles.statItem}>
          <span className={styles.statLabel}>Fees</span>
          <span className={styles.statValue}>
            {formatFee(quote.totalFees)} ETH
          </span>
        </div>

        <div className={styles.statItem}>
          <span className={styles.statLabel}>Time</span>
          <span className={styles.statValue}>
            {Math.ceil(quote.estimatedTime / 60)} min
          </span>
        </div>

        <div className={cn(styles.statItem, styles[priceImpactSeverity])}>
          <span className={styles.statLabel}>Impact</span>
          <span className={styles.statValue}>{quote.priceImpact}%</span>
        </div>
      </div>

      {/* Expandable Details - Mobile Optimized */}
      {showDetails && (
        <div className={styles.detailsSection}>
          <button
            type="button"
            className={styles.detailsToggle}
            onClick={() => setExpanded(!expanded)}
            aria-expanded={expanded}
            aria-label={expanded ? "Hide quote details" : "Show quote details"}
          >
            <span>{compact ? "Details" : "Quote Details"}</span>
            <span
              className={cn(styles.toggleArrow, {
                [styles.up]: expanded,
              })}
            >
              ▼
            </span>
          </button>

          {expanded && (
            <div className={styles.details}>
              {/* Fee Breakdown */}
              <div className={styles.section}>
                <h4 className={styles.sectionTitle}>Fee Breakdown</h4>
                <div className={styles.fees}>
                  <div className={styles.feeItem}>
                    <span>Bridge Fee</span>
                    <span>{formatFee(quote.bridgeFee)} ETH</span>
                  </div>
                  <div className={styles.feeItem}>
                    <span>Protocol Fee</span>
                    <span>{formatFee(quote.protocolFee)} ETH</span>
                  </div>
                  <div className={styles.feeItem}>
                    <span>Gas Estimate</span>
                    <span>{formatFee(quote.estimatedGas)} ETH</span>
                  </div>
                  <div className={cn(styles.feeItem, styles.total)}>
                    <span>Total Fees</span>
                    <span>{formatFee(quote.totalFees)} ETH</span>
                  </div>
                </div>
              </div>

              {/* Additional Stats */}
              <div className={styles.section}>
                <h4 className={styles.sectionTitle}>Transaction Info</h4>
                <div className={styles.infoGrid}>
                  <div className={styles.infoItem}>
                    <span className={styles.infoLabel}>Max Slippage</span>
                    <span className={styles.infoValue}>{quote.slippage}%</span>
                  </div>

                  <div
                    className={cn(styles.infoItem, styles[priceImpactSeverity])}
                  >
                    <span className={styles.infoLabel}>Price Impact</span>
                    <span className={styles.infoValue}>
                      {quote.priceImpact}%
                      {priceImpactSeverity === "high" && " ⚠️"}
                    </span>
                  </div>

                  <div className={styles.infoItem}>
                    <span className={styles.infoLabel}>Route</span>
                    <span className={styles.infoValue}>
                      {quote.fromChain} → {quote.toChain}
                    </span>
                  </div>

                  {quote.riskScore && (
                    <div className={cn(styles.infoItem, styles[riskLevel])}>
                      <span className={styles.infoLabel}>Risk Score</span>
                      <span className={styles.infoValue}>
                        {quote.riskScore}/100
                      </span>
                    </div>
                  )}
                </div>
              </div>

              {/* Security Features - Mobile Simplified */}
              {!compact && (
                <div className={styles.section}>
                  <h4 className={styles.sectionTitle}>Security</h4>
                  <div className={styles.securityFeatures}>
                    {quote.quantumProtected && (
                      <div className={styles.securityItem}>
                        <span className={styles.securityIcon}>🔒</span>
                        <div className={styles.securityInfo}>
                          <div className={styles.securityName}>
                            Quantum Protected
                          </div>
                          <div className={styles.securityDesc}>ML-KEM-1024</div>
                        </div>
                      </div>
                    )}

                    <div className={styles.securityItem}>
                      <span className={styles.securityIcon}>⛓️</span>
                      <div className={styles.securityInfo}>
                        <div className={styles.securityName}>Atomic Swap</div>
                        <div className={styles.securityDesc}>Guaranteed</div>
                      </div>
                    </div>

                    <div className={styles.securityItem}>
                      <span className={styles.securityIcon}>🧠</span>
                      <div className={styles.securityInfo}>
                        <div className={styles.securityName}>AI Monitored</div>
                        <div className={styles.securityDesc}>Risk analysis</div>
                      </div>
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      )}

      {/* Warning for high impact */}
      {priceImpactSeverity === "high" && (
        <div className={styles.warning}>
          <span className={styles.warningIcon}>⚠️</span>
          <span className={styles.warningText}>
            High price impact. Consider smaller amount or check market
            conditions.
          </span>
        </div>
      )}

      {/* Expiry warning */}
      {timeRemaining > 0 &&
        timeRemaining < 60000 && ( // Less than 1 minute
          <div className={styles.expiryWarning}>
            <span className={styles.warningIcon}>⏰</span>
            <span className={styles.warningText}>
              Quote expires soon. Confirm swap to lock in this rate.
            </span>
          </div>
        )}
    </div>
  );
};
