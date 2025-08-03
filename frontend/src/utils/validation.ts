export const isValidEthereumAddress = (address: string): boolean => {
  return /^0x[a-fA-F0-9]{40}$/.test(address);
};

export const isValidNearAddress = (address: string): boolean => {
  // NEAR addresses can be account names or implicit accounts
  const accountNameRegex = /^[a-z0-9._-]+$/;
  const implicitAccountRegex = /^[a-f0-9]{64}$/;

  return accountNameRegex.test(address) || implicitAccountRegex.test(address);
};

export const isValidAddress = (address: string, network: string): boolean => {
  switch (network.toLowerCase()) {
    case "ethereum":
    case "ethereum-sepolia":
      return isValidEthereumAddress(address);
    case "near":
    case "near-testnet":
      return isValidNearAddress(address);
    default:
      return false;
  }
};

export const isValidAmount = (amount: string): boolean => {
  if (!amount || amount.trim() === "") return false;

  const num = parseFloat(amount);
  return !isNaN(num) && num > 0;
};

export const isValidSlippage = (slippage: number): boolean => {
  return slippage >= 0.1 && slippage <= 10;
};

export const isValidDeadline = (deadline: number): boolean => {
  return deadline >= 1 && deadline <= 60; // 1 to 60 minutes
};

export const isValidEmail = (email: string): boolean => {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
};

export const isValidUrl = (url: string): boolean => {
  try {
    new URL(url);
    return true;
  } catch {
    return false;
  }
};

export const isValidTransactionHash = (
  hash: string,
  network: string
): boolean => {
  switch (network.toLowerCase()) {
    case "ethereum":
    case "ethereum-sepolia":
      return /^0x[a-fA-F0-9]{64}$/.test(hash);
    case "near":
    case "near-testnet":
      return /^[A-Za-z0-9]{43,44}$/.test(hash);
    default:
      return false;
  }
};

export const validateSwapForm = (data: {
  fromToken: string;
  toToken: string;
  amount: string;
  slippage: number;
  deadline: number;
}): { isValid: boolean; errors: Record<string, string> } => {
  const errors: Record<string, string> = {};

  if (!data.fromToken) {
    errors.fromToken = "Please select a token to swap from";
  }

  if (!data.toToken) {
    errors.toToken = "Please select a token to swap to";
  }

  if (data.fromToken === data.toToken) {
    errors.toToken = "Cannot swap to the same token";
  }

  if (!isValidAmount(data.amount)) {
    errors.amount = "Please enter a valid amount";
  }

  if (!isValidSlippage(data.slippage)) {
    errors.slippage = "Slippage must be between 0.1% and 10%";
  }

  if (!isValidDeadline(data.deadline)) {
    errors.deadline = "Deadline must be between 1 and 60 minutes";
  }

  return {
    isValid: Object.keys(errors).length === 0,
    errors,
  };
};

export const sanitizeInput = (input: string): string => {
  return input.trim().replace(/[<>]/g, "");
};

export const isValidTokenAmount = (
  amount: string,
  balance: string
): boolean => {
  if (!isValidAmount(amount)) return false;

  const amountNum = parseFloat(amount);
  const balanceNum = parseFloat(balance);

  return amountNum <= balanceNum;
};

export const validateRequired = (
  value: string,
  fieldName: string
): string | null => {
  if (!value || value.trim() === "") {
    return `${fieldName} is required`;
  }
  return null;
};

export const validateMinLength = (
  value: string,
  minLength: number,
  fieldName: string
): string | null => {
  if (value.length < minLength) {
    return `${fieldName} must be at least ${minLength} characters`;
  }
  return null;
};

export const validateMaxLength = (
  value: string,
  maxLength: number,
  fieldName: string
): string | null => {
  if (value.length > maxLength) {
    return `${fieldName} must be no more than ${maxLength} characters`;
  }
  return null;
};
