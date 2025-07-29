/**
 * PriceQuote Component - Mobile-First
 * Dynamic price display optimized for mobile with simplified UI
 */

import React, { useState, useEffect } from "react";
import type { BridgeQuote } from "../../../types/bridge";
import "./PriceQuote.scss";

export interface PriceQuoteProps {
  quote?: BridgeQuote;
  loading?: boolean;
  error?: string;
  className?: string;
  showDetails?: boolean;
  compact?: boolean; // Mobile-first: option for compact display
}

export const PriceQuote: React.FC<PriceQuoteProps> = ({
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
      <div className={`price-quote price-quote--loading ${className}`}>
        <div className="price-quote__spinner">
          <div className="price-quote__spinner-icon">⏳</div>
          <span>Getting best quote...</span>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={`price-quote price-quote--error ${className}`}>
        <div className="price-quote__error">
          <span className="price-quote__error-icon">⚠️</span>
          <span className="price-quote__error-message">{error}</span>
        </div>
      </div>
    );
  }

  if (!quote) {
    return (
      <div className={`price-quote price-quote--empty ${className}`}>
        <div className="price-quote__empty">
          <span className="price-quote__empty-icon">💱</span>
          <p>Enter amount to get quote</p>
        </div>
      </div>
    );
  }

  const priceImpactSeverity = getPriceImpactSeverity(quote.priceImpact);
  const riskLevel = getRiskLevel(quote.riskScore);

  return (
    <div className={`price-quote ${className}`}>
      {/* Main Quote Display */}
      <div className="price-quote__main">
        <div className="price-quote__header">
          <div className="price-quote__rate">
            <span className="price-quote__rate-label">You'll receive</span>
            <span className="price-quote__rate-value">
              {formatAmount(quote.toAmount)} {quote.toToken}
            </span>
          </div>

          {timeRemaining > 0 && (
            <div className="price-quote__timer">
              <span className="price-quote__timer-icon">⏱️</span>
              <span className="price-quote__timer-value">
                {formatTimeRemaining(timeRemaining)}
              </span>
            </div>
          )}
        </div>

        <div className="price-quote__exchange-rate">
          1 {quote.fromToken} = {formatExchangeRate(quote.exchangeRate)}{" "}
          {quote.toToken}
        </div>
      </div>

      {/* Quick Stats */}
      <div className="price-quote__quick-stats">
        <div className="price-quote__stat-item">
          <span className="price-quote__stat-label">Fees</span>
          <span className="price-quote__stat-value">
            {formatFee(quote.totalFees)} ETH
          </span>
        </div>

        <div className="price-quote__stat-item">
          <span className="price-quote__stat-label">Time</span>
          <span className="price-quote__stat-value">
            {Math.ceil(quote.estimatedTime / 60)} min
          </span>
        </div>

        <div
          className={`price-quote__stat-item price-quote__stat-item--${priceImpactSeverity}`}
        >
          <span className="price-quote__stat-label">Impact</span>
          <span className="price-quote__stat-value">{quote.priceImpact}%</span>
        </div>
      </div>

      {/* Expandable Details - Mobile Optimized */}
      {showDetails && (
        <div className="price-quote__details-section">
          <button
            type="button"
            className="price-quote__details-toggle"
            onClick={() => setExpanded(!expanded)}
            aria-expanded={expanded}
            aria-label={expanded ? "Hide quote details" : "Show quote details"}
          >
            <span>{compact ? "Details" : "Quote Details"}</span>
            <span
              className={`price-quote__toggle-arrow ${
                expanded ? "price-quote__toggle-arrow--up" : ""
              }`}
            >
              ▼
            </span>
          </button>

          {expanded && (
            <div className="price-quote__details">
              {/* Fee Breakdown */}
              <div className="price-quote__section">
                <h4 className="price-quote__section-title">Fee Breakdown</h4>
                <div className="price-quote__fees">
                  <div className="price-quote__fee-item">
                    <span>Bridge Fee</span>
                    <span>{formatFee(quote.bridgeFee)} ETH</span>
                  </div>
                  <div className="price-quote__fee-item">
                    <span>Protocol Fee</span>
                    <span>{formatFee(quote.protocolFee)} ETH</span>
                  </div>
                  <div className="price-quote__fee-item">
                    <span>Gas Estimate</span>
                    <span>{formatFee(quote.estimatedGas)} ETH</span>
                  </div>
                  <div className="price-quote__fee-item price-quote__fee-item--total">
                    <span>Total Fees</span>
                    <span>{formatFee(quote.totalFees)} ETH</span>
                  </div>
                </div>
              </div>

              {/* Additional Stats */}
              <div className="price-quote__section">
                <h4 className="price-quote__section-title">Transaction Info</h4>
                <div className="price-quote__info-grid">
                  <div className="price-quote__info-item">
                    <span className="price-quote__info-label">
                      Max Slippage
                    </span>
                    <span className="price-quote__info-value">
                      {quote.slippage}%
                    </span>
                  </div>

                  <div
                    className={`price-quote__info-item price-quote__info-item--${priceImpactSeverity}`}
                  >
                    <span className="price-quote__info-label">
                      Price Impact
                    </span>
                    <span className="price-quote__info-value">
                      {quote.priceImpact}%
                      {priceImpactSeverity === "high" && " ⚠️"}
                    </span>
                  </div>

                  <div className="price-quote__info-item">
                    <span className="price-quote__info-label">Route</span>
                    <span className="price-quote__info-value">
                      {quote.fromChain} → {quote.toChain}
                    </span>
                  </div>

                  {quote.riskScore && (
                    <div
                      className={`price-quote__info-item price-quote__info-item--${riskLevel}`}
                    >
                      <span className="price-quote__info-label">
                        Risk Score
                      </span>
                      <span className="price-quote__info-value">
                        {quote.riskScore}/100
                      </span>
                    </div>
                  )}
                </div>
              </div>

              {/* Security Features - Mobile Simplified */}
              {!compact && (
                <div className="price-quote__section">
                  <h4 className="price-quote__section-title">Security</h4>
                  <div className="price-quote__security-features">
                    {quote.quantumProtected && (
                      <div className="price-quote__security-item">
                        <span className="price-quote__security-icon">🔒</span>
                        <div className="price-quote__security-info">
                          <div className="price-quote__security-name">
                            Quantum Protected
                          </div>
                          <div className="price-quote__security-desc">
                            ML-KEM-1024
                          </div>
                        </div>
                      </div>
                    )}

                    <div className="price-quote__security-item">
                      <span className="price-quote__security-icon">⛓️</span>
                      <div className="price-quote__security-info">
                        <div className="price-quote__security-name">
                          Atomic Swap
                        </div>
                        <div className="price-quote__security-desc">
                          Guaranteed
                        </div>
                      </div>
                    </div>

                    <div className="price-quote__security-item">
                      <span className="price-quote__security-icon">🧠</span>
                      <div className="price-quote__security-info">
                        <div className="price-quote__security-name">
                          AI Monitored
                        </div>
                        <div className="price-quote__security-desc">
                          Risk analysis
                        </div>
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
        <div className="price-quote__warning">
          <span className="price-quote__warning-icon">⚠️</span>
          <span className="price-quote__warning-text">
            High price impact. Consider smaller amount or check market
            conditions.
          </span>
        </div>
      )}

      {/* Expiry warning */}
      {timeRemaining > 0 &&
        timeRemaining < 60000 && ( // Less than 1 minute
          <div className="price-quote__expiry-warning">
            <span className="price-quote__warning-icon">⏰</span>
            <span className="price-quote__warning-text">
              Quote expires soon. Confirm swap to lock in this rate.
            </span>
          </div>
        )}
    </div>
  );
};
