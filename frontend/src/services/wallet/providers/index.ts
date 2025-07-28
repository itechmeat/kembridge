/**
 * Wallet Providers Registry
 * Centralized export of all wallet providers
 * Follows DRY principle: single source of provider exports
 */

export { MetaMaskProvider } from "./metamask";
export { NearProvider } from "./near";
export { WalletConnectProvider } from "./walletconnect";
