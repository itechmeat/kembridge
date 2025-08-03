import { useState, useMemo, FC } from "react";
import cn from "classnames";
import type { BridgeToken, ChainType } from "../../../types/bridge";
import { useWallet } from "../../../hooks/wallet/useWallet";
import { Modal, CoinIcon } from "../../ui";
import { formatBalance, formatUsdValue } from "../../../utils/formatBalance";
import styles from "./TokenSelector.module.scss";

export interface TokenSelectorProps {
  selectedToken?: BridgeToken;
  chain: ChainType;
  tokens: BridgeToken[];
  onTokenSelect: (token: BridgeToken) => void;
  disabled?: boolean;
  showBalance?: boolean;
  className?: string;
}

export const TokenSelector: FC<TokenSelectorProps> = ({
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
  const { isConnected } = useWallet();

  // Filter tokens by chain and search query
  const filteredTokens = useMemo(() => {
    let filtered = tokens.filter((token) => token.chain === chain);

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
  }, [tokens, chain, searchQuery, showBalance, isConnected]);

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

  const handleModalClose = () => {
    setIsOpen(false);
    setSearchQuery("");
  };

  return (
    <div
      className={cn(styles.tokenSelector, className.trim(), {
        [styles.open]: isOpen,
      })}
    >
      <button
        type="button"
        className={cn(styles.trigger, {
          [styles.triggerSelected]: selectedToken,
        })}
        onClick={() => setIsOpen(!isOpen)}
        disabled={disabled}
        data-testid={`token-selector-${chain.toLowerCase()}`}
      >
        {selectedToken ? (
          <div className={styles.selected}>
            <div className={styles.tokenInfo}>
              <CoinIcon
                symbol={selectedToken.symbol}
                size="medium"
                className={styles.logo}
              />
              <div className={styles.tokenDetails}>
                <span className={styles.symbol}>{selectedToken.symbol}</span>
                <span className={styles.name}>{selectedToken.name}</span>
              </div>
            </div>
            {showBalance && selectedToken.balance && isConnected && (
              <div className={styles.balanceInfo}>
                <span className={styles.balance}>
                  {formatBalance(selectedToken.balance)}
                </span>
                {selectedToken.usdValue && (
                  <span className={styles.usdValue}>
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
          <div className={styles.placeholder}>
            <CoinIcon
              symbol="?"
              size="medium"
              className={styles.placeholderIcon}
            />
            <span>Select Token</span>
          </div>
        )}

        <div
          className={cn(styles.arrow, {
            [styles.arrowUp]: isOpen,
          })}
        >
          ▼
        </div>
      </button>

      <Modal
        isOpen={isOpen}
        onClose={handleModalClose}
        title="Select Token"
        className={styles.tokenSelectorModal}
      >
        <div className={styles.modalContent}>
          <div className={styles.searchWrapper}>
            <input
              type="text"
              placeholder="Search by name, symbol, or address..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className={styles.search}
              data-testid="token-search-input"
              autoFocus
            />
            <div className={styles.searchIcon}>🔍</div>
          </div>

          {!searchQuery && popularTokens.length > 0 && (
            <div className={styles.popular} data-testid="popular-tokens">
              <div className={styles.sectionTitle}>Popular Tokens</div>
              <div className={styles.popularGrid}>
                {popularTokens.slice(0, 4).map((token) => (
                  <button
                    key={`popular-${token.chain}-${token.address}`}
                    type="button"
                    className={styles.popularItem}
                    onClick={() => handleTokenSelect(token)}
                    data-testid={`popular-token-${token.symbol.toLowerCase()}`}
                  >
                    <CoinIcon symbol={token.symbol} size="medium" />
                    <span>{token.symbol}</span>
                  </button>
                ))}
              </div>
            </div>
          )}

          <div className={styles.list} data-testid="token-list">
            {filteredTokens.length === 0 ? (
              <div className={styles.noResults} data-testid="no-token-results">
                {searchQuery ? "No tokens found" : "No tokens available"}
              </div>
            ) : (
              filteredTokens.map((token) => (
                <button
                  key={`${token.chain}-${token.address}`}
                  type="button"
                  className={cn(styles.item, {
                    [styles.itemSelected]:
                      selectedToken?.address === token.address,
                  })}
                  onClick={() => handleTokenSelect(token)}
                  data-testid={`token-option-${token.symbol.toLowerCase()}`}
                >
                  <div className={styles.itemLeft}>
                    <CoinIcon
                      symbol={token.symbol}
                      size="medium"
                      className={styles.itemLogo}
                    />
                    <div className={styles.itemInfo}>
                      <span className={styles.itemSymbol}>{token.symbol}</span>
                      <span className={styles.itemName}>{token.name}</span>
                    </div>
                  </div>

                  {showBalance && isConnected && (
                    <div className={styles.itemBalance}>
                      {token.balance && parseFloat(token.balance) > 0 ? (
                        <>
                          <span className={styles.itemBalanceAmount}>
                            {formatBalance(token.balance)}
                          </span>
                          {token.usdValue && (
                            <span className={styles.itemBalanceUsd}>
                              {formatUsdValue(token.balance, token.usdValue)}
                            </span>
                          )}
                        </>
                      ) : (
                        <span className={styles.itemBalanceZero}>0</span>
                      )}
                    </div>
                  )}
                </button>
              ))
            )}
          </div>

          {!isConnected && showBalance && (
            <div
              className={styles.connectNotice}
              data-testid="auth-required-message"
            >
              Connect wallet to see balances
            </div>
          )}
        </div>
      </Modal>
    </div>
  );
};
