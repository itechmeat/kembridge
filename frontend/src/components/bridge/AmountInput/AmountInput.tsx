/**
 * AmountInput Component
 * Amount input with real-time validation and balance checking
 */

import React, { useState, useEffect, useCallback } from "react";
import type { BridgeToken } from "../../../types/bridge";

export interface AmountInputProps {
  value: string;
  onChange: (value: string) => void;
  token?: BridgeToken;
  balance?: string;
  disabled?: boolean;
  placeholder?: string;
  className?: string;
  showUsdValue?: boolean;
  onMaxClick?: () => void;
}

export const AmountInput: React.FC<AmountInputProps> = ({
  value,
  onChange,
  token,
  balance,
  disabled = false,
  placeholder = "0.0",
  className = "",
  showUsdValue = true,
  onMaxClick,
}) => {
  const [error, setError] = useState<string>("");
  const [usdValue, setUsdValue] = useState<string>("");

  // Validate amount with comprehensive checks
  useEffect(() => {
    if (!value) {
      setError("");
      return;
    }

    const numValue = parseFloat(value);

    // Basic validation
    if (isNaN(numValue) || numValue <= 0) {
      setError("Invalid amount");
      return;
    }

    // Check minimum amount (different for each token)
    const minAmount =
      token?.symbol === "ETH"
        ? 0.001
        : token?.symbol === "NEAR"
        ? 0.1
        : token?.symbol?.includes("USD")
        ? 1
        : 0.001;

    if (numValue < minAmount) {
      setError(`Minimum amount: ${minAmount} ${token?.symbol || ""}`);
      return;
    }

    // Check maximum amount (for safety)
    const maxAmount =
      token?.symbol === "ETH"
        ? 100
        : token?.symbol === "NEAR"
        ? 10000
        : token?.symbol?.includes("USD")
        ? 100000
        : 1000;

    if (numValue > maxAmount) {
      setError(
        `Maximum amount: ${maxAmount.toLocaleString()} ${token?.symbol || ""}`
      );
      return;
    }

    // Balance validation
    if (balance) {
      const numBalance = parseFloat(balance);

      if (numValue > numBalance) {
        setError("Insufficient balance");
        return;
      }

      // Warn if using more than 95% of balance (for gas fees)
      if (numValue > numBalance * 0.95 && token?.symbol === "ETH") {
        setError("Leave some ETH for gas fees");
        return;
      }
    }

    setError("");
  }, [value, balance, token]);

  // Calculate USD value with debouncing
  useEffect(() => {
    const timeoutId = setTimeout(() => {
      if (!value || !token?.usdValue) {
        setUsdValue("");
        return;
      }

      const numValue = parseFloat(value);
      const tokenUsdPrice = parseFloat(token.usdValue);

      if (!isNaN(numValue) && !isNaN(tokenUsdPrice) && numValue > 0) {
        const totalUsd = numValue * tokenUsdPrice;
        if (totalUsd < 0.01) {
          setUsdValue("< $0.01");
        } else {
          setUsdValue(
            `≈ $${totalUsd.toLocaleString(undefined, {
              minimumFractionDigits: 2,
              maximumFractionDigits: 2,
            })}`
          );
        }
      } else {
        setUsdValue("");
      }
    }, 300); // 300ms debounce

    return () => clearTimeout(timeoutId);
  }, [value, token?.usdValue]);

  const handleMaxClick = useCallback(() => {
    if (balance && onMaxClick) {
      onMaxClick();
    } else if (balance) {
      onChange(balance);
    }
  }, [balance, onChange, onMaxClick]);

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;

    // Allow empty string
    if (newValue === "") {
      onChange("");
      return;
    }

    // Allow only numbers and one decimal point
    const regex = /^\d*\.?\d*$/;
    if (!regex.test(newValue)) {
      return; // Don't update if invalid characters
    }

    // Prevent multiple decimal points
    if ((newValue.match(/\./g) || []).length > 1) {
      return;
    }

    // Limit decimal places based on token
    const maxDecimals = token?.decimals || 18;
    const decimalPlaces = newValue.split(".")[1]?.length || 0;

    if (decimalPlaces > Math.min(maxDecimals, 8)) {
      // Cap at 8 for UI
      return;
    }

    // Prevent leading zeros (except for 0.xxx)
    if (
      newValue.length > 1 &&
      newValue.startsWith("0") &&
      !newValue.startsWith("0.")
    ) {
      return;
    }

    onChange(newValue);
  };

  return (
    <div
      className={`amount-input ${className} ${
        error ? "amount-input--error" : ""
      }`}
    >
      <div className="amount-input__wrapper">
        <input
          type="text"
          inputMode="decimal"
          placeholder={placeholder}
          value={value}
          onChange={handleInputChange}
          disabled={disabled}
          className="amount-input__field"
        />

        {balance && (
          <button
            type="button"
            onClick={handleMaxClick}
            disabled={disabled}
            className="amount-input__max-button"
          >
            MAX
          </button>
        )}
      </div>

      <div className="amount-input__info">
        {balance && (
          <span className="amount-input__balance">
            Balance: {balance} {token?.symbol}
          </span>
        )}

        {showUsdValue && usdValue && (
          <span className="amount-input__usd-value">{usdValue}</span>
        )}
      </div>

      {error && <div className="amount-input__error">{error}</div>}
    </div>
  );
};
