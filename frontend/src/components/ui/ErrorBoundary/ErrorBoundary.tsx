import { Component, type ReactNode, type ErrorInfo } from "react";
import cn from "classnames";
import { ERROR_MESSAGES } from "../../../constants";
import styles from "./ErrorBoundary.module.scss";

interface ErrorBoundaryState {
  hasError: boolean;
  error?: Error;
  errorInfo?: ErrorInfo;
}

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: ReactNode;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
}

export class ErrorBoundary extends Component<
  ErrorBoundaryProps,
  ErrorBoundaryState
> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return {
      hasError: true,
      error,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    this.setState({
      error,
      errorInfo,
    });

    // Log error details
    console.error("🔥 ErrorBoundary: Caught error:", {
      message: error.message,
      stack: error.stack,
      componentStack: errorInfo.componentStack,
      errorBoundary: true,
    });

    // In development, also log additional details
    if (import.meta.env.DEV) {
      console.group("🐛 Error Boundary - Development Details");
      console.error("Error:", error);
      console.error("Error Info:", errorInfo);
      console.groupEnd();
    }

    // Call optional error handler
    this.props.onError?.(error, errorInfo);

    // Send to error monitoring service in production
    // TODO: Integrate with error monitoring service
  }

  private handleRetry = () => {
    this.setState({ hasError: false, error: undefined, errorInfo: undefined });
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className={styles.root}>
          <div className={styles.container}>
            <div className={styles.icon}>⚠️</div>
            <h2 className={styles.title}>Something went wrong</h2>
            <p className={styles.message}>
              {this.state.error?.message || ERROR_MESSAGES.UNKNOWN_ERROR}
            </p>

            <div className={styles.actions}>
              <button
                onClick={this.handleRetry}
                className={cn(styles.button, styles.primary)}
                type="button"
              >
                Try Again
              </button>
              <button
                onClick={() => window.location.reload()}
                className={cn(styles.button, styles.ghost)}
                type="button"
              >
                Reload Page
              </button>
            </div>

            {import.meta.env.DEV && this.state.errorInfo && (
              <details className={styles.details}>
                <summary>Error Details (Development)</summary>
                <pre className={styles.stack}>{this.state.error?.stack}</pre>
                <pre className={styles.componentStack}>
                  {this.state.errorInfo.componentStack}
                </pre>
              </details>
            )}
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
