import { FC, memo } from "react";
import {
  getTransactionLinks,
  ChainType,
  ExplorerLink,
} from "../../../utils/blockchainLinks";
import styles from "./TransactionLinks.module.scss";
import { CoinIcon } from "../../ui";

export interface TransactionLinksProps {
  fromChain: ChainType;
  toChain: ChainType;
  fromTxHash?: string;
  toTxHash?: string;
  fromToken?: string; // Token symbol for icons
  toToken?: string; // Token symbol for icons
  className?: string;
  compact?: boolean;
}

interface TransactionLinkItemProps {
  chain: ChainType;
  txHash: string;
  label: string;
  links: ExplorerLink[];
  tokenSymbol?: string;
  compact?: boolean;
}

const TransactionLinkItem: FC<TransactionLinkItemProps> = memo(
  ({ chain, txHash, label, links, tokenSymbol, compact = false }) => {
    if (!txHash || links.length === 0) {
      return (
        <div className={styles.linkItem}>
          <div className={styles.label}>{label}:</div>
          <div className={styles.pending}>Pending...</div>
        </div>
      );
    }

    const primaryLink = links[0];
    const hasMultipleLinks = links.length > 1;

    return (
      <div className={styles.linkItem}>
        <div className={styles.label}>{label}:</div>
        <div className={styles.links}>
          {/* Primary link */}
          <a
            href={primaryLink.url}
            target="_blank"
            rel="noopener noreferrer"
            className={styles.primaryLink}
            title={`View transaction on ${primaryLink.name}`}
          >
            <span className={styles.chainIcon}>
              <CoinIcon
                symbol={tokenSymbol || (chain === "ethereum" ? "ETH" : "NEAR")}
                size={compact ? "small" : "medium"}
              />
            </span>
            <span className={styles.hash}>
              {compact
                ? `${txHash.slice(0, 8)}...${txHash.slice(-6)}`
                : `${txHash.slice(0, 12)}...${txHash.slice(-8)}`}
            </span>
            <span className={styles.external}>↗</span>
          </a>

          {/* Additional explorers dropdown */}
          {hasMultipleLinks && !compact && (
            <div className={styles.dropdown}>
              <button className={styles.dropdownToggle} title="More explorers">
                <span className={styles.moreIcon}>⋯</span>
              </button>
              <div className={styles.dropdownMenu}>
                {links.slice(1).map((link) => (
                  <a
                    key={link.name}
                    href={link.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className={styles.dropdownLink}
                    title={`View on ${link.name}`}
                  >
                    {link.name} ↗
                  </a>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    );
  }
);

export const TransactionLinks: FC<TransactionLinksProps> = memo(
  ({
    fromChain,
    toChain,
    fromTxHash,
    toTxHash,
    fromToken,
    toToken,
    className = "",
    compact = false,
  }) => {
    const fromLinks = fromTxHash
      ? getTransactionLinks(fromChain, fromTxHash)
      : [];
    const toLinks = toTxHash ? getTransactionLinks(toChain, toTxHash) : [];

    const hasAnyTransaction = fromTxHash || toTxHash;

    // Определяем статус транзакций
    const getStatusInfo = () => {
      if (fromTxHash && toTxHash) {
        return {
          text: "Both transactions confirmed",
          className: "success",
        };
      }
      if (fromTxHash && !toTxHash) {
        return {
          text: "Source confirmed, awaiting destination",
          className: "pending",
        };
      }
      if (!fromTxHash && toTxHash) {
        return {
          text: "Destination confirmed, awaiting source",
          className: "pending",
        };
      }
      return {
        text: "Waiting for confirmation...",
        className: "processing",
      };
    };

    const statusInfo = getStatusInfo();

    if (!hasAnyTransaction) {
      return (
        <div
          className={`${styles.transactionLinks} ${
            compact ? styles.compact : ""
          } ${className}`}
        >
          <div className={styles.header}>
            <span className={styles.title}>Transaction Links</span>
            <span className={`${styles.status} ${styles.processing}`}>
              {statusInfo.text}
            </span>
          </div>
        </div>
      );
    }

    return (
      <div
        className={`${styles.transactionLinks} ${
          compact ? styles.compact : ""
        } ${className}`}
      >
        <div className={styles.header}>
          <span className={styles.title}>Transaction Links</span>
          <span className={`${styles.status} ${styles[statusInfo.className]}`}>
            {statusInfo.text}
          </span>
        </div>

        <div className={styles.links}>
          <TransactionLinkItem
            chain={fromChain}
            txHash={fromTxHash || ""}
            label={`${
              fromChain.charAt(0).toUpperCase() + fromChain.slice(1)
            } Transaction`}
            links={fromLinks}
            tokenSymbol={fromToken}
            compact={compact}
          />

          <TransactionLinkItem
            chain={toChain}
            txHash={toTxHash || ""}
            label={`${
              toChain.charAt(0).toUpperCase() + toChain.slice(1)
            } Transaction`}
            links={toLinks}
            tokenSymbol={toToken}
            compact={compact}
          />
        </div>
      </div>
    );
  }
);
