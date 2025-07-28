/**
 * Global error handler utilities
 * Handles uncaught errors and WebSocket connection issues
 */

// Handle WebSocket connection errors gracefully
const handleWebSocketError = (error: Error) => {
  // Don't show WebSocket errors to users in development
  if (import.meta.env.DEV) {
    console.warn(
      "WebSocket connection error (expected in development):",
      error.message
    );
    return;
  }

  // In production, log but don't crash
  console.error("WebSocket connection error:", error.message);
};

// Handle general uncaught errors
const handleUncaughtError = (error: Error) => {
  // WebSocket-related errors
  if (
    error.message.includes("Connection interrupted") ||
    error.message.includes("WebSocket")
  ) {
    handleWebSocketError(error);
    return;
  }

  // Other errors should be logged properly
  console.error("Uncaught error:", error);
};

// Setup global error handlers
export const setupGlobalErrorHandlers = () => {
  // Global error handler
  window.addEventListener("error", (event) => {
    console.error("🔥 Global Error:", event.error);

    // Log in development
    if (import.meta.env.DEV) {
      console.error("Error details:", {
        message: event.message,
        filename: event.filename,
        lineno: event.lineno,
        colno: event.colno,
        error: event.error,
      });
    }
  });

  // Handle unhandled promise rejections
  window.addEventListener("unhandledrejection", (event) => {
    const error = event.reason;

    if (error instanceof Error) {
      handleUncaughtError(error);
    } else {
      console.error("Unhandled promise rejection:", error);
    }

    // Prevent the error from appearing in console for WebSocket issues
    if (
      error instanceof Error &&
      (error.message.includes("Connection interrupted") ||
        error.message.includes("WebSocket"))
    ) {
      event.preventDefault();
    }
  });
};

// Cleanup function
export const cleanupGlobalErrorHandlers = () => {
  // Remove listeners if needed
  // This is mainly for testing purposes
};
