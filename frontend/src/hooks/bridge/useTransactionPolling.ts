import { useState, useEffect, useRef, useCallback } from 'react';
import { bridgeService, SwapTransaction } from '../../services/api/bridgeService';

export interface UseTransactionPollingProps {
  transactionId: string | null;
  enabled?: boolean;
  interval?: number;
  maxAttempts?: number;
}

export interface UseTransactionPollingResult {
  transaction: SwapTransaction | null;
  isLoading: boolean;
  error: string | null;
  isPolling: boolean;
  stopPolling: () => void;
  startPolling: () => void;
}

export function useTransactionPolling({
  transactionId,
  enabled = true,
  interval = 3000, // 3 seconds
  maxAttempts = 20, // 1 minute total
}: UseTransactionPollingProps): UseTransactionPollingResult {
  const [transaction, setTransaction] = useState<SwapTransaction | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [isPolling, setIsPolling] = useState(false);
  const [attempts, setAttempts] = useState(0);

  const intervalRef = useRef<number | null>(null);
  const isMountedRef = useRef(true);

  const stopPolling = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
    setIsPolling(false);
    console.log('🛑 Transaction polling stopped');
  }, []);

  const startPolling = useCallback(() => {
    if (!transactionId || !enabled || isPolling) return;

    console.log(`🔄 Starting transaction polling for ${transactionId}`);
    setIsPolling(true);
    setAttempts(0);
  }, [transactionId, enabled, isPolling]);

  // Fetch function
  useEffect(() => {
    if (!isPolling || !transactionId) return;

    const fetchTransaction = async () => {
      try {
        setIsLoading(true);
        setError(null);

        console.log(`📡 Fetching transaction status for ${transactionId}...`);
        const result = await bridgeService.getSwapStatus(transactionId);
        
        if (isMountedRef.current) {
          setTransaction(result);
          setAttempts(prev => prev + 1);
          
          // Stop polling if transaction is completed or failed
          if (result.status === 'completed' || result.status === 'failed' || result.status === 'expired') {
            console.log(`✅ Transaction ${transactionId} finished with status: ${result.status}`);
            stopPolling();
          }
        }
      } catch (err) {
        console.error('❌ Failed to fetch transaction status:', err);
        if (isMountedRef.current) {
          setError(err instanceof Error ? err.message : 'Failed to fetch transaction');
          setAttempts(prev => prev + 1);
        }
      } finally {
        if (isMountedRef.current) {
          setIsLoading(false);
        }
      }
    };

    // Fetch immediately
    fetchTransaction();

    // Start interval polling
    intervalRef.current = window.setInterval(fetchTransaction, interval);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, [isPolling, transactionId, interval, stopPolling]);

  // Auto-start polling when transaction ID is set
  useEffect(() => {
    if (transactionId && enabled) {
      startPolling();
    } else {
      stopPolling();
    }
  }, [transactionId, enabled, startPolling, stopPolling]);

  // Stop polling after max attempts
  useEffect(() => {
    if (attempts >= maxAttempts && isPolling) {
      console.warn(`⚠️ Stopping transaction polling after ${maxAttempts} attempts`);
      stopPolling();
    }
  }, [attempts, maxAttempts, isPolling, stopPolling]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      isMountedRef.current = false;
      stopPolling();
    };
  }, [stopPolling]);

  return {
    transaction,
    isLoading,
    error,
    isPolling,
    stopPolling,
    startPolling,
  };
}