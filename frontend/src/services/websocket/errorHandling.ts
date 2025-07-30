/**
 * WebSocket Error Handling and Recovery
 * Advanced error handling and reconnection strategies for WebSocket connections
 */

import { wsClient } from './wsClient';

export interface WebSocketError {
  code: number;
  message: string;
  timestamp: number;
  severity: 'low' | 'medium' | 'high' | 'critical';
  recoverable: boolean;
  retryAfter?: number;
}

export interface RecoveryStrategy {
  name: string;
  condition: (error: WebSocketError) => boolean;
  action: (error: WebSocketError) => Promise<boolean>;
  priority: number;
}

export interface ErrorHandlerOptions {
  maxRetries?: number;
  retryDelayMs?: number;
  enableAutoRecovery?: boolean;
  onError?: (error: WebSocketError) => void;
  onRecovery?: () => void;
}

class WebSocketErrorHandler {
  private errors: WebSocketError[] = [];
  private recoveryStrategies: RecoveryStrategy[] = [];
  private autoRecoveryEnabled = true;
  private isRecovering = false;
  private recoveryAttempts = 0;
  private maxRecoveryAttempts = 3;
  private listeners = new Set<(error: WebSocketError) => void>();

  constructor() {
    console.log('🛠️ WebSocket Error Handler: Initializing');
    this.setupDefaultRecoveryStrategies();
    this.setupErrorMonitoring();
  }

  /**
   * Setup default recovery strategies
   */
  private setupDefaultRecoveryStrategies(): void {
    // Strategy 1: Simple reconnection for network issues
    this.addRecoveryStrategy({
      name: 'simple_reconnect',
      condition: (error) => error.code === 1006 && error.recoverable,
      action: async () => {
        console.log('🔄 Attempting simple reconnection...');
        try {
          await wsClient.connect();
          return true;
        } catch (e) {
          console.error('❌ Simple reconnection failed:', e);
          return false;
        }
      },
      priority: 1,
    });

    // Strategy 2: Reset and reconnect for authentication issues
    this.addRecoveryStrategy({
      name: 'auth_reset_reconnect',
      condition: (error) => error.code === 1008 || error.code === 4001,
      action: async () => {
        console.log('🔐 Attempting authentication reset and reconnection...');
        try {
          // Clear any stored auth tokens that might be invalid
          // This would require integration with auth service
          await new Promise(resolve => setTimeout(resolve, 2000));
          await wsClient.connect();
          return true;
        } catch (e) {
          console.error('❌ Auth reset reconnection failed:', e);
          return false;
        }
      },
      priority: 2,
    });

    // Strategy 3: Progressive backoff for persistent issues
    this.addRecoveryStrategy({
      name: 'progressive_backoff',
      condition: () => this.recoveryAttempts >= 2,
      action: async () => {
        const delay = Math.min(1000 * Math.pow(2, this.recoveryAttempts), 30000);
        console.log(`⏳ Progressive backoff: waiting ${delay}ms before retry...`);
        
        await new Promise(resolve => setTimeout(resolve, delay));
        
        try {
          await wsClient.connect();
          return true;
        } catch (e) {
          console.error('❌ Progressive backoff reconnection failed:', e);
          return false;
        }
      },
      priority: 3,
    });

    // Strategy 4: Server health check and retry
    this.addRecoveryStrategy({
      name: 'health_check_retry',
      condition: (error) => error.severity === 'high',
      action: async () => {
        console.log('🏥 Checking server health before retry...');
        
        try {
          // Simulate health check - in real implementation would check backend health
          const healthCheck = await fetch('/health', { method: 'GET' });
          if (healthCheck.ok) {
            console.log('✅ Server health check passed, attempting reconnection...');
            await wsClient.connect();
            return true;
          } else {
            console.log('⚠️ Server health check failed, delaying retry...');
            return false;
          }
        } catch (e) {
          console.error('❌ Health check failed:', e);
          return false;
        }
      },
      priority: 4,
    });

    console.log(`🛠️ Initialized ${this.recoveryStrategies.length} recovery strategies`);
  }

  /**
   * Setup error monitoring for WebSocket events
   */
  private setupErrorMonitoring(): void {
    // Monitor connection errors
    const originalConnect = wsClient.connect.bind(wsClient);
    wsClient.connect = async (authToken?: string) => {
      try {
        return await originalConnect(authToken);
      } catch (error) {
        this.handleError({
          code: 1006,
          message: `Connection failed: ${error}`,
          timestamp: Date.now(),
          severity: 'high',
          recoverable: true,
        });
        throw error;
      }
    };

    console.log('🛠️ Error monitoring setup complete');
  }

  /**
   * Handle WebSocket error with automatic recovery
   */
  async handleError(error: WebSocketError): Promise<void> {
    console.error('🚨 WebSocket error:', error);
    
    // Add to error log
    this.errors.push(error);
    
    // Keep only last 50 errors
    if (this.errors.length > 50) {
      this.errors.shift();
    }

    // Notify listeners
    this.listeners.forEach(listener => {
      try {
        listener(error);
      } catch (e) {
        console.error('Error in error listener:', e);
      }
    });

    // Attempt recovery if enabled and not already recovering
    if (this.autoRecoveryEnabled && !this.isRecovering && error.recoverable) {
      await this.attemptRecovery(error);
    }
  }

  /**
   * Attempt error recovery using available strategies
   */
  private async attemptRecovery(error: WebSocketError): Promise<boolean> {
    if (this.recoveryAttempts >= this.maxRecoveryAttempts) {
      console.error('❌ Max recovery attempts reached, giving up');
      return false;
    }

    this.isRecovering = true;
    this.recoveryAttempts++;

    console.log(`🛠️ Attempting recovery (attempt ${this.recoveryAttempts}/${this.maxRecoveryAttempts})`);

    // Sort strategies by priority
    const applicableStrategies = this.recoveryStrategies
      .filter(strategy => strategy.condition(error))
      .sort((a, b) => a.priority - b.priority);

    if (applicableStrategies.length === 0) {
      console.warn('⚠️ No applicable recovery strategies found');
      this.isRecovering = false;
      return false;
    }

    // Try each applicable strategy
    for (const strategy of applicableStrategies) {
      console.log(`🛠️ Trying recovery strategy: ${strategy.name}`);
      
      try {
        const success = await strategy.action(error);
        if (success) {
          console.log(`✅ Recovery successful using strategy: ${strategy.name}`);
          this.isRecovering = false;
          this.recoveryAttempts = 0;
          
          // Notify listeners of recovery
          this.listeners.forEach(listener => {
            try {
              // Use a recovery event type
              const recoveryEvent = {
                ...error,
                message: `Recovered using ${strategy.name}`,
                severity: 'low' as const,
              };
              listener(recoveryEvent);
            } catch (e) {
              console.error('Error in recovery listener:', e);
            }
          });
          
          return true;
        }
      } catch (strategyError) {
        console.error(`❌ Recovery strategy ${strategy.name} failed:`, strategyError);
      }
    }

    console.error('❌ All recovery strategies failed');
    this.isRecovering = false;
    return false;
  }

  /**
   * Add custom recovery strategy
   */
  addRecoveryStrategy(strategy: RecoveryStrategy): void {
    this.recoveryStrategies.push(strategy);
    this.recoveryStrategies.sort((a, b) => a.priority - b.priority);
    console.log(`🛠️ Added recovery strategy: ${strategy.name}`);
  }

  /**
   * Remove recovery strategy
   */
  removeRecoveryStrategy(name: string): boolean {
    const index = this.recoveryStrategies.findIndex(s => s.name === name);
    if (index !== -1) {
      this.recoveryStrategies.splice(index, 1);
      console.log(`🛠️ Removed recovery strategy: ${name}`);
      return true;
    }
    return false;
  }

  /**
   * Add error listener
   */
  onError(listener: (error: WebSocketError) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  /**
   * Enable/disable auto recovery
   */
  setAutoRecovery(enabled: boolean): void {
    this.autoRecoveryEnabled = enabled;
    console.log(`🛠️ Auto recovery ${enabled ? 'enabled' : 'disabled'}`);
  }

  /**
   * Get error history
   */
  getErrorHistory(): WebSocketError[] {
    return [...this.errors];
  }

  /**
   * Get error statistics
   */
  getErrorStats() {
    const now = Date.now();
    const last24h = this.errors.filter(e => now - e.timestamp < 24 * 60 * 60 * 1000);
    const lastHour = this.errors.filter(e => now - e.timestamp < 60 * 60 * 1000);

    const severityCounts = this.errors.reduce((acc, error) => {
      acc[error.severity] = (acc[error.severity] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    return {
      totalErrors: this.errors.length,
      errorsLast24h: last24h.length,
      errorsLastHour: lastHour.length,
      severityCounts,
      isRecovering: this.isRecovering,
      recoveryAttempts: this.recoveryAttempts,
      maxRecoveryAttempts: this.maxRecoveryAttempts,
      autoRecoveryEnabled: this.autoRecoveryEnabled,
      availableStrategies: this.recoveryStrategies.length,
    };
  }

  /**
   * Clear error history
   */
  clearErrorHistory(): void {
    this.errors.length = 0;
    console.log('🛠️ Error history cleared');
  }

  /**
   * Force recovery attempt
   */
  async forceRecovery(): Promise<boolean> {
    if (this.isRecovering) {
      console.warn('⚠️ Recovery already in progress');
      return false;
    }

    const mockError: WebSocketError = {
      code: 1006,
      message: 'Manual recovery triggered',
      timestamp: Date.now(),
      severity: 'medium',
      recoverable: true,
    };

    return await this.attemptRecovery(mockError);
  }

  /**
   * Reset recovery state
   */
  reset(): void {
    this.isRecovering = false;
    this.recoveryAttempts = 0;
    this.errors.length = 0;
    console.log('🛠️ Error handler reset');
  }
}

// Create singleton instance
export const webSocketErrorHandler = new WebSocketErrorHandler();

// Utility functions for common error scenarios
export const createWebSocketError = (
  code: number,
  message: string,
  severity: WebSocketError['severity'] = 'medium',
  recoverable = true
): WebSocketError => ({
  code,
  message,
  timestamp: Date.now(),
  severity,
  recoverable,
});

export const isNetworkError = (error: WebSocketError): boolean => {
  return [1006, 1015].includes(error.code);
};

export const isAuthenticationError = (error: WebSocketError): boolean => {
  return [1008, 4001, 4003].includes(error.code);
};

export const isServerError = (error: WebSocketError): boolean => {
  return error.code >= 1011 && error.code <= 1015;
};

// Export for use in components
export default webSocketErrorHandler;