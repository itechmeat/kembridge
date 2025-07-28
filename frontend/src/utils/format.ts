// Formatting utilities

export const formatNumber = (
  value: number | string,
  decimals: number = 2
): string => {
  const num = typeof value === "string" ? parseFloat(value) : value;

  if (isNaN(num)) return "0";

  return new Intl.NumberFormat("en-US", {
    minimumFractionDigits: 0,
    maximumFractionDigits: decimals,
  }).format(num);
};

export const formatCurrency = (
  value: number | string,
  currency: string = "USD"
): string => {
  const num = typeof value === "string" ? parseFloat(value) : value;

  if (isNaN(num)) return "$0.00";

  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency,
  }).format(num);
};

export const formatTokenAmount = (
  amount: string | number,
  decimals: number = 18,
  displayDecimals: number = 4
): string => {
  const num = typeof amount === "string" ? parseFloat(amount) : amount;

  if (isNaN(num)) return "0";

  // Convert from wei to token amount
  const tokenAmount = num / Math.pow(10, decimals);

  // Format with appropriate decimals
  if (tokenAmount < 0.0001) {
    return "< 0.0001";
  }

  return formatNumber(tokenAmount, displayDecimals);
};

export const formatAddress = (address: string, length: number = 4): string => {
  if (!address) return "";

  if (address.length <= length * 2 + 2) {
    return address;
  }

  return `${address.slice(0, length + 2)}...${address.slice(-length)}`;
};

export const formatHash = (hash: string, length: number = 6): string => {
  return formatAddress(hash, length);
};

export const formatTime = (timestamp: number): string => {
  const date = new Date(timestamp * 1000);

  return new Intl.DateTimeFormat("en-US", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  }).format(date);
};

export const formatDate = (timestamp: number): string => {
  const date = new Date(timestamp * 1000);

  return new Intl.DateTimeFormat("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  }).format(date);
};

export const formatDateTime = (timestamp: number): string => {
  const date = new Date(timestamp * 1000);

  return new Intl.DateTimeFormat("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
};

export const formatPercentage = (
  value: number,
  decimals: number = 2
): string => {
  return `${formatNumber(value, decimals)}%`;
};

export const formatDuration = (seconds: number): string => {
  if (seconds < 60) {
    return `${Math.round(seconds)}s`;
  }

  if (seconds < 3600) {
    const minutes = Math.floor(seconds / 60);
    return `${minutes}m`;
  }

  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  if (minutes === 0) {
    return `${hours}h`;
  }

  return `${hours}h ${minutes}m`;
};

export const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return "0 Bytes";

  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
};

export const formatSlippage = (slippage: number): string => {
  return `${formatNumber(slippage, 1)}%`;
};

export const formatGasPrice = (gasPrice: string | number): string => {
  const price = typeof gasPrice === "string" ? parseFloat(gasPrice) : gasPrice;

  if (isNaN(price)) return "0 gwei";

  const gwei = price / 1e9;

  return `${formatNumber(gwei, 2)} gwei`;
};
