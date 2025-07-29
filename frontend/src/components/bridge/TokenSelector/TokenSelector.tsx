/**
 * TokenSelector Component
 * Token selection with search and balance display
 */

import React, { useState, useMemo, useEffect, useRef } from "react";
import type { BridgeToken, ChainType } from "../../../types/bridge";
import { useBalance, TokenBalance } from "../../../hooks/wallet/useBalance";
import { useWallet } from "../../../hooks/wallet/useWallet";

export interface TokenSelectorProps {
  selectedToken?: BridgeToken;
  chain: ChainType;
  tokens: BridgeToken[];
  onTokenSelect: (token: BridgeToken) => void;
  disabled?: boolean;
  showBalance?: boolean;
  className?: string;
}

export const TokenSelector: React.FC<TokenSelectorProps> = ({
  selectedToken,
  chain,
  tokens,
  onTokenSelect,
  disabled = false,
  showBalance = true,
  className = "",
}) => {
  const [searchQuery, setSearchQuery] = useState("");
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const { isConnected } = useWallet();

  // Get balance for wallet integration
  const { balances: balanceData } = useBalance();

  // Close dropdown when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
        setSearchQuery("");
      }
    };

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  // Enhance tokens with balance data
  const tokensWithBalance = useMemo(() => {
    if (!balanceData || !showBalance || !isConnected) {
      return tokens;
    }

    return tokens.map((token) => {
      // Real balance integration required - no mock data
      const realBalance = balanceData.find(
        (b: TokenBalance) => b.symbol.toLowerCase() === token.symbol.toLowerCase()
      );

      return {
        ...token,
        balance: realBalance?.balance || "0.00",
        usdValue: realBalance?.usdValue || "0",
      };
    });
  }, [tokens, balanceData, showBalance, isConnected]);

  // Filter tokens by chain and search query
  const filteredTokens = useMemo(() => {
    let filtered = tokensWithBalance.filter((token) => token.chain === chain);

    if (searchQuery) {
      filtered = filtered.filter(
        (token) =>
          token.symbol.toLowerCase().includes(searchQuery.toLowerCase()) ||
          token.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          token.address.toLowerCase().includes(searchQuery.toLowerCase())
      );
    }

    // Sort by balance (highest first) if balances available
    if (showBalance && isConnected) {
      filtered.sort((a, b) => {
        const balanceA = parseFloat(a.balance || "0");
        const balanceB = parseFloat(b.balance || "0");
        if (balanceA !== balanceB) return balanceB - balanceA;
        return a.symbol.localeCompare(b.symbol);
      });
    }

    return filtered;
  }, [tokensWithBalance, chain, searchQuery, showBalance, isConnected]);

  // Popular tokens (native tokens first)
  const popularTokens = useMemo(() => {
    return filteredTokens
      .filter(
        (token) =>
          token.symbol === "ETH" ||
          token.symbol === "NEAR" ||
          token.symbol === "USDC" ||
          token.symbol === "USDT"
      )
      .sort((a, b) => {
        const order = ["ETH", "NEAR", "USDC", "USDT"];
        return order.indexOf(a.symbol) - order.indexOf(b.symbol);
      });
  }, [filteredTokens]);

  const handleTokenSelect = (token: BridgeToken) => {
    onTokenSelect(token);
    setIsOpen(false);
    setSearchQuery("");
  };

  const formatBalance = (balance?: string) => {
    if (!balance) return "";
    const num = parseFloat(balance);
    if (num === 0) return "0";
    if (num < 0.0001) return "<0.0001";
    if (num < 1) return num.toFixed(6);
    if (num < 1000) return num.toFixed(4);
    return num.toLocaleString(undefined, { maximumFractionDigits: 2 });
  };

  const formatUsdValue = (balance?: string, usdPrice?: string) => {
    if (!balance || !usdPrice) return "";
    const balanceNum = parseFloat(balance);
    const priceNum = parseFloat(usdPrice);
    const usdValue = balanceNum * priceNum;

    if (usdValue < 0.01) return "";
    return `$${usdValue.toLocaleString(undefined, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    })}`;
  };

  return (
    <div
      className={`token-selector ${className} ${
        isOpen ? "token-selector--open" : ""
      }`}
      ref={dropdownRef}
    >
      <button
        type="button"
        className={`token-selector__trigger ${
          selectedToken ? "token-selector__trigger--selected" : ""
        }`}
        onClick={() => setIsOpen(!isOpen)}
        disabled={disabled}
      >
        {selectedToken ? (
          <div className="token-selector__selected">
            <div className="token-selector__token-info">
              {selectedToken.logoUrl ? (
                <img
                  src={selectedToken.logoUrl}
                  alt={selectedToken.symbol}
                  className="token-selector__logo"
                />
              ) : (
                <div className="token-selector__logo-placeholder">
                  {selectedToken.symbol.charAt(0)}
                </div>
              )}
              <div className="token-selector__token-details">
                <span className="token-selector__symbol">
                  {selectedToken.symbol}
                </span>
                <span className="token-selector__name">
                  {selectedToken.name}
                </span>
              </div>
            </div>
            {showBalance && selectedToken.balance && isConnected && (
              <div className="token-selector__balance-info">
                <span className="token-selector__balance">
                  {formatBalance(selectedToken.balance)}
                </span>
                {selectedToken.usdValue && (
                  <span className="token-selector__usd-value">
                    {formatUsdValue(
                      selectedToken.balance,
                      selectedToken.usdValue
                    )}
                  </span>
                )}
              </div>
            )}
          </div>
        ) : (
          <div className="token-selector__placeholder">
            <div className="token-selector__placeholder-icon">🪙</div>
            <span>Select Token</span>
          </div>
        )}

        <div
          className={`token-selector__arrow ${
            isOpen ? "token-selector__arrow--up" : ""
          }`}
        >
          ▼
        </div>
      </button>

      {isOpen && (
        <div className="token-selector__dropdown">
          <div className="token-selector__search-wrapper">
            <input
              type="text"
              placeholder="Search by name, symbol, or address..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="token-selector__search"
              autoFocus
            />
            <div className="token-selector__search-icon">🔍</div>
          </div>

          {!searchQuery && popularTokens.length > 0 && (
            <div className="token-selector__popular">
              <div className="token-selector__section-title">
                Popular Tokens
              </div>
              <div className="token-selector__popular-grid">
                {popularTokens.slice(0, 4).map((token) => (
                  <button
                    key={`popular-${token.chain}-${token.address}`}
                    type="button"
                    className="token-selector__popular-item"
                    onClick={() => handleTokenSelect(token)}
                  >
                    {token.logoUrl ? (
                      <img src={token.logoUrl} alt={token.symbol} />
                    ) : (
                      <div className="token-selector__logo-placeholder">
                        {token.symbol.charAt(0)}
                      </div>
                    )}
                    <span>{token.symbol}</span>
                  </button>
                ))}
              </div>
            </div>
          )}

          <div className="token-selector__list">
            {filteredTokens.length === 0 ? (
              <div className="token-selector__no-results">
                {searchQuery ? "No tokens found" : "No tokens available"}
              </div>
            ) : (
              filteredTokens.map((token) => (
                <button
                  key={`${token.chain}-${token.address}`}
                  type="button"
                  className={`token-selector__item ${
                    selectedToken?.address === token.address
                      ? "token-selector__item--selected"
                      : ""
                  }`}
                  onClick={() => handleTokenSelect(token)}
                >
                  <div className="token-selector__item-left">
                    {token.logoUrl ? (
                      <img
                        src={token.logoUrl}
                        alt={token.symbol}
                        className="token-selector__item-logo"
                      />
                    ) : (
                      <div className="token-selector__logo-placeholder">
                        {token.symbol.charAt(0)}
                      </div>
                    )}
                    <div className="token-selector__item-info">
                      <span className="token-selector__item-symbol">
                        {token.symbol}
                      </span>
                      <span className="token-selector__item-name">
                        {token.name}
                      </span>
                    </div>
                  </div>

                  {showBalance && isConnected && (
                    <div className="token-selector__item-balance">
                      {token.balance && parseFloat(token.balance) > 0 ? (
                        <>
                          <span className="token-selector__item-balance-amount">
                            {formatBalance(token.balance)}
                          </span>
                          {token.usdValue && (
                            <span className="token-selector__item-balance-usd">
                              {formatUsdValue(token.balance, token.usdValue)}
                            </span>
                          )}
                        </>
                      ) : (
                        <span className="token-selector__item-balance-zero">
                          0
                        </span>
                      )}
                    </div>
                  )}
                </button>
              ))
            )}
          </div>

          {!isConnected && showBalance && (
            <div className="token-selector__connect-notice">
              Connect wallet to see balances
            </div>
          )}
        </div>
      )}
    </div>
  );
};
