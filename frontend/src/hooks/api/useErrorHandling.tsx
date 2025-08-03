import { useState, useEffect, useCallback } from "react";
import {
  errorHandlingService,
  ErrorNotification,
  ErrorContext,
  ErrorSeverity,
} from "../../services/errorHandlingService";

export interface UseErrorHandlingReturn {
  notifications: ErrorNotification[];
  handleError: (error: unknown, context?: ErrorContext) => string;
  dismissNotification: (id: string) => void;
  dismissAllNotifications: () => void;
  testErrorHandling: (type?: string) => Promise<void>;
  showSuccess: (message: string) => void;
  hasErrors: boolean;
  criticalErrors: ErrorNotification[];
  errorCount: number;
}

/**
 * Hook for managing error handling in React components
 */
export const useErrorHandling = (): UseErrorHandlingReturn => {
  const [notifications, setNotifications] = useState<ErrorNotification[]>([]);

  useEffect(() => {
    // Subscribe to error handling service
    const unsubscribe = errorHandlingService.subscribe((newNotifications) => {
      setNotifications(newNotifications);
    });

    // Initialize with current notifications
    setNotifications(errorHandlingService.getNotifications());

    return unsubscribe;
  }, []);

  const handleError = useCallback(
    (error: unknown, context?: ErrorContext): string => {
      return errorHandlingService.handleError(error, context);
    },
    []
  );

  const dismissNotification = useCallback((id: string) => {
    errorHandlingService.dismissNotification(id);
  }, []);

  const dismissAllNotifications = useCallback(() => {
    errorHandlingService.dismissAllNotifications();
  }, []);

  const testErrorHandling = useCallback(async (type: string = "validation") => {
    await errorHandlingService.testErrorHandling(type);
  }, []);

  const showSuccess = useCallback((message: string) => {
    errorHandlingService.showSuccessNotification(message);
  }, []);

  // Computed values
  const hasErrors = notifications.length > 0;
  const criticalErrors = notifications.filter(
    (n) => n.error.severity === ErrorSeverity.CRITICAL
  );
  const errorCount = notifications.length;

  return {
    notifications,
    handleError,
    dismissNotification,
    dismissAllNotifications,
    testErrorHandling,
    showSuccess,
    hasErrors,
    criticalErrors,
    errorCount,
  };
};

/**
 * Hook for handling API errors with automatic error notification
 */
export const useApiErrorHandler = () => {
  const { handleError } = useErrorHandling();

  const handleApiError = useCallback(
    (error: unknown, operation?: string) => {
      return handleError(error, { operation: operation || "api_call" });
    },
    [handleError]
  );

  return { handleApiError };
};

/**
 * Hook for handling form validation errors
 */
export const useFormErrorHandler = () => {
  const { handleError } = useErrorHandling();

  const handleValidationError = useCallback(
    (errors: Record<string, string>, formName?: string) => {
      Object.entries(errors).forEach(([field, message]) => {
        handleError(
          {
            message,
            code: "VALIDATION_ERROR",
            field,
          },
          {
            operation: "form_validation",
            form: formName || "unknown_form",
            data: { field },
          } as ErrorContext
        );
      });
    },
    [handleError]
  );

  return { handleValidationError };
};

/**
 * Hook for handling transaction errors
 */
export const useTransactionErrorHandler = () => {
  const { handleError } = useErrorHandling();

  const handleTransactionError = useCallback(
    (error: unknown, transactionData?: Record<string, unknown>) => {
      return handleError(error, {
        operation: "transaction",
        transactionData,
      } as ErrorContext);
    },
    [handleError]
  );

  return { handleTransactionError };
};

export default useErrorHandling;
