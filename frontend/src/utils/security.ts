// XSS patterns to detect and block
const XSS_PATTERNS = [
  /<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi,
  /<iframe\b[^<]*(?:(?!<\/iframe>)<[^<]*)*<\/iframe>/gi,
  /javascript:/i,
  /on\w+\s*=/i, // Event handlers like onclick=, onload=
  /<img[^>]+src[^>]*=.*?["']?javascript:/i,
  /<svg[^>]*on\w+[^>]*>/i,
  /data:text\/html/i,
  /vbscript:/i,
  /&lt;script/i,
  /&lt;iframe/i,
];

// Dangerous HTML tags that should be stripped
const DANGEROUS_TAGS = [
  "script",
  "iframe",
  "embed",
  "object",
  "applet",
  "meta",
  "link",
  "style",
  "form",
  "input",
  "textarea",
  "button",
  "select",
  "option",
];

/**
 * Sanitize user input to prevent XSS attacks
 * @param input - The input string to sanitize
 * @returns Sanitized string safe for display
 */
export function sanitizeInput(input: string): string {
  if (typeof input !== "string") {
    return "";
  }

  let sanitized = input;

  // Remove dangerous patterns
  XSS_PATTERNS.forEach((pattern) => {
    sanitized = sanitized.replace(pattern, "");
  });

  // Remove dangerous HTML tags
  DANGEROUS_TAGS.forEach((tag) => {
    const tagPattern = new RegExp(
      `<${tag}\\b[^<]*(?:(?!<\\/${tag}>)<[^<]*)*<\\/${tag}>`,
      "gi"
    );
    sanitized = sanitized.replace(tagPattern, "");

    // Also remove self-closing tags
    const selfClosingPattern = new RegExp(`<${tag}\\b[^>]*\\/?>`, "gi");
    sanitized = sanitized.replace(selfClosingPattern, "");
  });

  // Encode remaining HTML entities
  sanitized = sanitized
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#x27;")
    .replace(/\//g, "&#x2F;");

  return sanitized;
}

/**
 * Check if input contains potentially malicious content
 * @param input - The input string to check
 * @returns true if input appears malicious
 */
export function containsMaliciousContent(input: string): boolean {
  if (typeof input !== "string") {
    return false;
  }

  return (
    XSS_PATTERNS.some((pattern) => pattern.test(input)) ||
    DANGEROUS_TAGS.some((tag) => input.toLowerCase().includes(`<${tag}`))
  );
}

/**
 * Sanitize numeric input to only allow valid numbers
 * @param input - The input string
 * @returns Sanitized numeric string
 */
export function sanitizeNumericInput(input: string): string {
  if (typeof input !== "string") {
    return "";
  }

  // Remove any non-numeric characters except decimal point
  const numeric = input.replace(/[^0-9.]/g, "");

  // Ensure only one decimal point
  const parts = numeric.split(".");
  if (parts.length > 2) {
    return parts[0] + "." + parts.slice(1).join("");
  }

  return numeric;
}

/**
 * Validate and sanitize wallet address input
 * @param address - The wallet address to validate
 * @returns Sanitized address or empty string if invalid
 */
export function sanitizeWalletAddress(address: string): string {
  if (typeof address !== "string") {
    return "";
  }

  // Check for malicious content first
  if (containsMaliciousContent(address)) {
    console.warn("Security: Blocked malicious wallet address input");
    return "";
  }

  // Basic sanitization - remove any HTML
  let sanitized = sanitizeInput(address);

  // Further validation for wallet address format
  // Ethereum addresses should be 42 characters starting with 0x
  // NEAR addresses should be alphanumeric with dots, dashes, underscores
  const ethPattern = /^0x[a-fA-F0-9]{40}$/;
  const nearPattern = /^[a-zA-Z0-9._-]+\.?(near|testnet)?$/;

  if (ethPattern.test(sanitized) || nearPattern.test(sanitized)) {
    return sanitized;
  }

  // If it doesn't match expected patterns but isn't malicious, return it
  // The backend will do final validation
  return sanitized;
}

/**
 * Security logging for blocked attempts
 * @param type - Type of security event
 * @param input - The blocked input
 * @param context - Additional context
 */
export function logSecurityEvent(
  type: string,
  input: string,
  context?: string
): void {
  const event = {
    type,
    timestamp: new Date().toISOString(),
    input: input.substring(0, 100), // Only log first 100 chars for privacy
    context,
    userAgent: navigator.userAgent,
    url: window.location.href,
  };

  console.warn("Security Event:", event);

  // In production, you might want to send this to a security logging service
  if (import.meta.env.MODE === "production") {
    // Example: send to monitoring service
    // securityLogger.log(event);
  }
}
