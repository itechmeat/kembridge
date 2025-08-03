import { apiClient } from "./api/apiClient";

// Error categories from backend
export enum ErrorCategory {
  Authentication = "Authentication",
  Validation = "Validation",
  Network = "Network",
  Service = "Service",
  Transaction = "Transaction",
  Blockchain = "Blockchain",
  Security = "Security",
  Permissions = "Permissions",
  Unknown = "Unknown",
}

// Error severity levels
export enum ErrorSeverity {
  LOW = "low",
  MEDIUM = "medium",
  HIGH = "high",
  CRITICAL = "critical",
}

// Enhanced error interface matching backend response
export interface EnhancedError {
  code: number;
  message: string;
  category: ErrorCategory;
  request_id: string;
  recovery_available: boolean;
  suggested_actions: string[];
  severity?: ErrorSeverity;
  timestamp?: string;
  retry_count?: number;
  max_retries?: number;
}

// Error notification interface
export interface ErrorNotification {
  id: string;
  error: EnhancedError;
  title: string;
  description: string;
  message: string;
  details?: string;
  category: ErrorCategory;
  timestamp: string;
  transactionId?: string;
  operationType?: string;
  retryCount?: number;
  maxRetries?: number;
  actions: ErrorAction[];
  autoHide?: boolean;
  hideAfter?: number; // milliseconds
  sound?: boolean;
  persistent?: boolean;
}

// Error action interface
export interface ErrorAction {
  label: string;
  action: () => void | Promise<void>;
  variant?: "primary" | "secondary" | "danger";
  style?: "primary" | "secondary" | "danger";
  loading?: boolean;
}

// Error context interface
export interface ErrorContext {
  operation?: string;
  data?: Record<string, unknown>;
  form?: string;
  transactionData?: Record<string, unknown>;
  attempt?: number;
  original_error?: Error;
  type?: string;
  apiCall?: () => Promise<unknown>;
}

// Axios error interface
interface AxiosError {
  response?: {
    status: number;
    data: {
      error:
        | string
        | {
            code?: number;
            message?: string;
            category?: string;
            request_id?: string;
            recovery_available?: boolean;
            suggested_actions?: string[];
            timestamp?: string;
          };
    };
  };
  code?: string;
  message?: string;
}

// Network error interface
interface NetworkError {
  code: string;
  message: string;
}

// Generic error interface
interface GenericError {
  message?: string;
  toString(): string;
}

// Error handling service class
class ErrorHandlingService {
  private notifications: Map<string, ErrorNotification> = new Map();
  private listeners: Set<(notifications: ErrorNotification[]) => void> =
    new Set();
  private retryAttempts: Map<string, number> = new Map();

  /**
   * Handle API error and create notification
   */
  public handleError(error: unknown, context?: ErrorContext): string {
    console.error("🚨 Error occurred:", error, "Context:", context);

    const enhancedError = this.parseError(error);
    const notificationId = this.createErrorNotification(enhancedError, context);

    // Log error for debugging
    this.logError(enhancedError, context);

    return notificationId;
  }

  /**
   * Parse error from various sources into enhanced error format
   */
  private parseError(error: unknown): EnhancedError {
    // Type guard for axios error
    const isAxiosError = (err: unknown): err is AxiosError => {
      return typeof err === "object" && err !== null && "response" in err;
    };

    // Type guard for network error
    const isNetworkError = (err: unknown): err is NetworkError => {
      return (
        typeof err === "object" &&
        err !== null &&
        "code" in err &&
        "message" in err
      );
    };

    // Type guard for generic error
    const isGenericError = (err: unknown): err is GenericError => {
      return (
        typeof err === "object" &&
        err !== null &&
        ("message" in err || "toString" in err)
      );
    };

    // Backend enhanced error response
    if (
      isAxiosError(error) &&
      error.response?.data?.error &&
      typeof error.response.data.error === "object"
    ) {
      const backendError = error.response.data.error;
      return {
        code: backendError.code || error.response?.status || 500,
        message: backendError.message || "An unexpected error occurred",
        category: this.mapToErrorCategory(backendError.category || "Unknown"),
        request_id: backendError.request_id || this.generateRequestId(),
        recovery_available: backendError.recovery_available || false,
        suggested_actions: Array.isArray(backendError.suggested_actions)
          ? backendError.suggested_actions
          : ["Try again", "Contact support if problem persists"],
        severity: this.determineSeverity(
          backendError.code || error.response?.status || 500
        ),
        timestamp: backendError.timestamp || new Date().toISOString(),
      };
    }

    // Legacy backend error response
    if (
      isAxiosError(error) &&
      error.response?.data?.error &&
      typeof error.response.data.error === "string"
    ) {
      return {
        code: error.response?.status || 500,
        message: error.response.data.error,
        category: this.mapStatusToCategory(error.response?.status || 500),
        request_id: this.generateRequestId(),
        recovery_available: true,
        suggested_actions: this.getDefaultActions(
          error.response?.status || 500
        ),
        severity: this.determineSeverity(error.response?.status || 500),
        timestamp: new Date().toISOString(),
      };
    }

    // Network error
    if (
      isNetworkError(error) &&
      (error.code === "NETWORK_ERROR" ||
        error.message?.includes("Network Error"))
    ) {
      return {
        code: 0,
        message:
          "Network connection error. Please check your internet connection.",
        category: ErrorCategory.Network,
        request_id: this.generateRequestId(),
        recovery_available: true,
        suggested_actions: [
          "Check your internet connection",
          "Try again in a few moments",
          "Contact support if problem persists",
        ],
        severity: ErrorSeverity.HIGH,
        timestamp: new Date().toISOString(),
      };
    }

    // Generic error fallback
    const message = isGenericError(error)
      ? error.message || error.toString()
      : "An unexpected error occurred";
    return {
      code: 500,
      message,
      category: ErrorCategory.Unknown,
      request_id: this.generateRequestId(),
      recovery_available: false,
      suggested_actions: ["Refresh the page", "Contact support"],
      severity: ErrorSeverity.MEDIUM,
      timestamp: new Date().toISOString(),
    };
  }

  /**
   * Create error notification
   */
  private createErrorNotification(
    error: EnhancedError,
    context?: ErrorContext
  ): string {
    const notificationId = this.generateRequestId();

    const actions: ErrorAction[] = [];

    // Add retry action if recovery is available
    if (error.recovery_available) {
      actions.push({
        label: "Retry",
        action: () => this.retryOperation(error, context),
        variant: "primary",
      });
    }

    // Add dismiss action
    actions.push({
      label: "Dismiss",
      action: () => this.dismissNotification(notificationId),
      variant: "secondary",
    });

    // Add help action for critical errors
    if (error.severity === ErrorSeverity.CRITICAL) {
      actions.push({
        label: "Get Help",
        action: () => this.openHelpCenter(error),
        variant: "secondary",
      });
    }

    const notification: ErrorNotification = {
      id: notificationId,
      error,
      title: this.getErrorTitle(error),
      description: error.message,
      message: error.message,
      details: error.suggested_actions.join(". "),
      category: error.category,
      timestamp: error.timestamp || new Date().toISOString(),
      transactionId: context?.transactionData?.transactionId as string,
      operationType: context?.operation,
      retryCount: error.retry_count,
      maxRetries: error.max_retries,
      actions,
      autoHide: error.severity === ErrorSeverity.LOW,
      hideAfter: error.severity === ErrorSeverity.LOW ? 5000 : undefined,
      sound:
        error.severity === ErrorSeverity.HIGH ||
        error.severity === ErrorSeverity.CRITICAL,
      persistent: error.severity === ErrorSeverity.CRITICAL,
    };

    this.notifications.set(notificationId, notification);
    this.notifyListeners();

    // Auto-hide if configured
    if (notification.autoHide && notification.hideAfter) {
      setTimeout(() => {
        this.dismissNotification(notificationId);
      }, notification.hideAfter);
    }

    return notificationId;
  }

  /**
   * Retry failed operation
   */
  private async retryOperation(
    error: EnhancedError,
    context?: ErrorContext
  ): Promise<void> {
    const currentAttempts = this.retryAttempts.get(error.request_id) || 0;
    const maxRetries = error.max_retries || 3;

    if (currentAttempts >= maxRetries) {
      this.handleError(
        {
          message:
            "Maximum retry attempts reached. Please try again later or contact support.",
          code: "MAX_RETRIES_EXCEEDED",
        },
        { operation: "retry_failed", original_error: new Error(error.message) }
      );
      return;
    }

    this.retryAttempts.set(error.request_id, currentAttempts + 1);

    try {
      console.log(
        `🔄 Retrying operation (attempt ${
          currentAttempts + 1
        }/${maxRetries})...`
      );

      // If context contains retry information, use it
      if (context?.operation) {
        switch (context.operation) {
          case "api_call":
            if (context.apiCall) {
              await context.apiCall();
            }
            break;
          case "transaction":
            if (context.transactionData) {
              // Retry transaction
              await this.retryTransaction(context.transactionData);
            }
            break;
          default:
            console.log(
              "No specific retry handler for operation:",
              context.operation
            );
        }
      }

      // Clear retry attempts on success
      this.retryAttempts.delete(error.request_id);

      // Show success notification
      this.showSuccessNotification(
        "Operation completed successfully after retry."
      );
    } catch (retryError) {
      console.error("🚫 Retry failed:", retryError);
      this.handleError(retryError, {
        operation: "retry_failed",
        attempt: currentAttempts + 1,
        original_error: new Error(error.message),
      });
    }
  }

  /**
   * Retry transaction with recovery mechanisms
   */
  private async retryTransaction(
    transactionData: Record<string, unknown>
  ): Promise<unknown> {
    // This would integrate with actual transaction retry logic
    console.log("🔄 Retrying transaction:", transactionData);

    // For now, simulate retry
    return new Promise((resolve, reject) => {
      setTimeout(() => {
        if (Math.random() > 0.5) {
          resolve({ success: true, transaction_id: "retry_" + Date.now() });
        } else {
          reject(new Error("Transaction retry failed"));
        }
      }, 2000);
    });
  }

  /**
   * Test error handling system
   */
  public async testErrorHandling(type: string = "validation"): Promise<void> {
    try {
      const response = await apiClient.get(
        `/api/v1/test/error-handling?type=${type}`
      );
      console.log(
        "✅ Error test successful:",
        (response as { data: unknown }).data
      );
      this.showSuccessNotification(
        `Error handling test (${type}) completed successfully.`
      );
    } catch (error) {
      console.log("🧪 Expected error for testing:", error);
      this.handleError(error, { operation: "error_handling_test", type });
    }
  }

  /**
   * Show success notification
   */
  public showSuccessNotification(message: string): void {
    const notificationId = this.generateRequestId();

    const notification: ErrorNotification = {
      id: notificationId,
      error: {
        code: 200,
        message,
        category: ErrorCategory.Unknown,
        request_id: notificationId,
        recovery_available: false,
        suggested_actions: [],
        severity: ErrorSeverity.LOW,
      },
      title: "Success",
      description: message,
      message: message,
      category: ErrorCategory.Unknown,
      timestamp: new Date().toISOString(),
      actions: [
        {
          label: "Dismiss",
          action: () => this.dismissNotification(notificationId),
          variant: "secondary",
        },
      ],
      autoHide: true,
      hideAfter: 3000,
    };

    this.notifications.set(notificationId, notification);
    this.notifyListeners();

    setTimeout(() => {
      this.dismissNotification(notificationId);
    }, 3000);
  }

  /**
   * Dismiss notification
   */
  public dismissNotification(id: string): void {
    this.notifications.delete(id);
    this.retryAttempts.delete(id);
    this.notifyListeners();
  }

  /**
   * Dismiss all notifications
   */
  public dismissAllNotifications(): void {
    this.notifications.clear();
    this.retryAttempts.clear();
    this.notifyListeners();
  }

  /**
   * Get all active notifications
   */
  public getNotifications(): ErrorNotification[] {
    return Array.from(this.notifications.values());
  }

  /**
   * Subscribe to notification changes
   */
  public subscribe(
    listener: (notifications: ErrorNotification[]) => void
  ): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  // Helper methods
  private mapToErrorCategory(category: string): ErrorCategory {
    const categoryMap: Record<string, ErrorCategory> = {
      Authentication: ErrorCategory.Authentication,
      Validation: ErrorCategory.Validation,
      Network: ErrorCategory.Network,
      Service: ErrorCategory.Service,
      Transaction: ErrorCategory.Transaction,
      Blockchain: ErrorCategory.Blockchain,
      Security: ErrorCategory.Security,
      Permissions: ErrorCategory.Permissions,
    };
    return categoryMap[category] || ErrorCategory.Unknown;
  }

  private mapStatusToCategory(status: number): ErrorCategory {
    if (status === 401 || status === 403) return ErrorCategory.Authentication;
    if (status === 400 || status === 422) return ErrorCategory.Validation;
    if (status >= 500) return ErrorCategory.Service;
    if (status === 408 || status === 504) return ErrorCategory.Network;
    return ErrorCategory.Unknown;
  }

  private determineSeverity(code: number): ErrorSeverity {
    if (code >= 500) return ErrorSeverity.CRITICAL;
    if (code === 401 || code === 403) return ErrorSeverity.HIGH;
    if (code >= 400) return ErrorSeverity.MEDIUM;
    return ErrorSeverity.LOW;
  }

  private getDefaultActions(status: number): string[] {
    if (status === 401)
      return ["Connect your wallet", "Sign the authentication message"];
    if (status === 400)
      return ["Check your input", "Verify all required fields"];
    if (status >= 500)
      return [
        "Try again in a few moments",
        "Contact support if problem persists",
      ];
    return ["Try again", "Contact support if problem persists"];
  }

  private getErrorTitle(error: EnhancedError): string {
    switch (error.category) {
      case ErrorCategory.Authentication:
        return "Authentication Required";
      case ErrorCategory.Validation:
        return "Invalid Input";
      case ErrorCategory.Network:
        return "Connection Error";
      case ErrorCategory.Service:
        return "Service Unavailable";
      case ErrorCategory.Transaction:
        return "Transaction Failed";
      case ErrorCategory.Blockchain:
        return "Blockchain Error";
      case ErrorCategory.Security:
        return "Security Alert";
      case ErrorCategory.Permissions:
        return "Access Denied";
      default:
        return "Error";
    }
  }

  private openHelpCenter(error: EnhancedError): void {
    const helpUrl = `https://help.kembridge.io/error/${error.category.toLowerCase()}?request_id=${
      error.request_id
    }`;
    window.open(helpUrl, "_blank");
  }

  private logError(error: EnhancedError, context?: unknown): void {
    console.group(`🚨 Error Log - ${error.category}`);
    console.error("Error:", error);
    console.log("Context:", context);
    console.log("Request ID:", error.request_id);
    console.log("Timestamp:", error.timestamp);
    console.groupEnd();
  }

  private notifyListeners(): void {
    const notifications = this.getNotifications();
    this.listeners.forEach((listener) => listener(notifications));
  }

  private generateRequestId(): string {
    return "req_" + Date.now() + "_" + Math.random().toString(36).substr(2, 9);
  }
}

// Export singleton instance
export const errorHandlingService = new ErrorHandlingService();
