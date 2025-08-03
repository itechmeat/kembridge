import { useState, useEffect, useCallback, FC, ChangeEvent } from "react";
import cn from "classnames";
import type { BridgeToken } from "../../../types/bridge";
import {
  sanitizeNumericInput,
  containsMaliciousContent,
  logSecurityEvent,
} from "../../../utils/security";
import { formatBalance } from "../../../utils/formatBalance";
import { CoinIcon } from "../../../components/ui";
import styles from "./AmountInput.module.scss";

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

export const AmountInput: FC<AmountInputProps> = ({
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

  const handleInputChange = (e: ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;

    // Security: Check for malicious content
    if (containsMaliciousContent(newValue)) {
      logSecurityEvent("XSS_ATTEMPT", newValue, "AmountInput");
      return;
    }

    // Allow empty string
    if (newValue === "") {
      onChange("");
      return;
    }

    // Sanitize numeric input
    const sanitizedValue = sanitizeNumericInput(newValue);

    // If sanitization changed the value, it contained invalid characters
    if (sanitizedValue !== newValue) {
      onChange(sanitizedValue);
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
      className={cn(styles.amountInput, className.trim(), {
        [styles.error]: !!error,
      })}
    >
      <div className={styles.wrapper}>
        <input
          type="text"
          inputMode="decimal"
          placeholder={placeholder}
          value={value}
          onChange={handleInputChange}
          disabled={disabled}
          className={styles.field}
          data-testid="amount-input"
        />

        {balance && (
          <button
            type="button"
            onClick={handleMaxClick}
            disabled={disabled}
            className={styles.maxButton}
            data-testid="max-button"
          >
            MAX
          </button>
        )}
      </div>

      <div className={styles.info}>
        {balance && (
          <div className={styles.balance}>
            <span>Balance: {formatBalance(balance)}</span>
            <div className={styles.tokenInfo}>
              <CoinIcon symbol={token?.symbol || ""} size="small" />
              <span>{token?.symbol}</span>
            </div>
          </div>
        )}

        {showUsdValue && usdValue && (
          <span className={styles.usdValue}>{usdValue}</span>
        )}
      </div>

      {error && (
        <div className={styles.error} data-testid="insufficient-balance-error">
          {error}
        </div>
      )}
    </div>
  );
};
