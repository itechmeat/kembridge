/**
 * AI Risk Analysis Hook
 * React hook for real-time AI risk analysis integration
 */

import { useState, useCallback, useEffect } from 'react';
import { AIRiskService, type AIRiskAnalysisResponse } from '../services/ai/aiRiskService';

export interface UseAIRiskAnalysisParams {
  userId: string;
  autoAnalyze?: boolean;
  onRiskChange?: (risk: AIRiskAnalysisResponse) => void;
  onError?: (error: Error) => void;
}

export interface RiskAnalysisState {
  isAnalyzing: boolean;
  currentRisk: AIRiskAnalysisResponse | null;
  error: Error | null;
  lastAnalyzedAt: Date | null;
  isAIEngineHealthy: boolean;
}

export function useAIRiskAnalysis(params: UseAIRiskAnalysisParams) {
  const { userId, onRiskChange, onError } = params;

  const [state, setState] = useState<RiskAnalysisState>({
    isAnalyzing: false,
    currentRisk: null,
    error: null,
    lastAnalyzedAt: null,
    isAIEngineHealthy: false,
  });

  // Check AI Engine health on mount
  useEffect(() => {
    const checkHealth = async () => {
      try {
        await AIRiskService.checkHealthStatus();
        setState(prev => ({ ...prev, isAIEngineHealthy: true, error: null }));
      } catch (error) {
        const err = error instanceof Error ? error : new Error('AI Engine health check failed');
        setState(prev => ({ ...prev, isAIEngineHealthy: false, error: err }));
        onError?.(err);
      }
    };

    checkHealth();

    // Check health every 2 minutes to reduce API load
    const healthInterval = setInterval(checkHealth, 120000);
    return () => clearInterval(healthInterval);
  }, [onError]);

  /**
   * Analyze transaction risk
   */
  const analyzeTransactionRisk = useCallback(async (transaction: {
    transactionId?: string;
    amount: number;
    sourceChain: string;
    destinationChain: string;
    sourceToken: string;
    destinationToken: string;
    userAddress?: string;
  }) => {
    // Check health status at call time and set analyzing state
    let shouldProceed = false;
    setState(prev => {
      if (!prev.isAIEngineHealthy) {
        const error = new Error('AI Engine is not available for risk analysis');
        onError?.(error);
        return { ...prev, error };
      }
      shouldProceed = true;
      return { ...prev, isAnalyzing: true, error: null };
    });

    // Early return if engine not healthy
    if (!shouldProceed) {
      return null;
    }

    try {
      const riskResult = await AIRiskService.analyzeTransactionRisk({
        userId,
        ...transaction,
      });

      setState(prev => ({
        ...prev,
        isAnalyzing: false,
        currentRisk: riskResult,
        lastAnalyzedAt: new Date(),
        error: null,
      }));

      onRiskChange?.(riskResult);
      return riskResult;
    } catch (error) {
      const err = error instanceof Error ? error : new Error('Risk analysis failed');
      setState(prev => ({
        ...prev,
        isAnalyzing: false,
        error: err,
      }));
      onError?.(err);
      return null;
    }
  }, [userId, onRiskChange, onError]);

  /**
   * Check if address is blacklisted
   */
  const checkAddressBlacklist = useCallback(async (address: string, chain: string) => {
    if (!state.isAIEngineHealthy) {
      const error = new Error('AI Engine is not available for blacklist check');
      onError?.(error);
      return null;
    }

    try {
      return await AIRiskService.checkAddressBlacklist(address, chain);
    } catch (error) {
      const err = error instanceof Error ? error : new Error('Blacklist check failed');
      onError?.(err);
      return null;
    }
  }, [state.isAIEngineHealthy, onError]);

  /**
   * Get user risk profile
   */
  const getUserRiskProfile = useCallback(async () => {
    if (!state.isAIEngineHealthy) {
      const error = new Error('AI Engine is not available for user profile');
      onError?.(error);
      return null;
    }

    try {
      return await AIRiskService.getUserRiskProfile(userId);
    } catch (error) {
      const err = error instanceof Error ? error : new Error('User profile fetch failed');
      onError?.(err);
      return null;
    }
  }, [userId, state.isAIEngineHealthy, onError]);

  /**
   * Get real-time risk level for current transaction state
   */
  const getCurrentRiskLevel = useCallback(() => {
    if (!state.currentRisk) return 'unknown';
    return state.currentRisk.risk_level;
  }, [state.currentRisk]);

  /**
   * Check if current transaction should be blocked
   */
  const shouldBlockTransaction = useCallback(() => {
    if (!state.currentRisk) return false;
    return !state.currentRisk.approved || state.currentRisk.risk_score > 0.8;
  }, [state.currentRisk]);

  /**
   * Get risk score with confidence
   */
  const getRiskScoreWithConfidence = useCallback(() => {
    if (!state.currentRisk) return null;
    
    return {
      score: state.currentRisk.risk_score,
      confidence: state.currentRisk.ml_confidence || 0.8,
      level: state.currentRisk.risk_level,
      timestamp: state.currentRisk.analysis_timestamp,
    };
  }, [state.currentRisk]);

  /**
   * Get risk factors and recommendations
   */
  const getRiskFactors = useCallback(() => {
    if (!state.currentRisk) return { factors: [], recommendations: [] };
    
    return {
      factors: state.currentRisk.reasons,
      recommendations: state.currentRisk.recommended_action ? [state.currentRisk.recommended_action] : [],
      isAnomaly: state.currentRisk.is_anomaly || false,
    };
  }, [state.currentRisk]);

  /**
   * Clear current risk analysis
   */
  const clearRiskAnalysis = useCallback(() => {
    setState(prev => ({
      ...prev,
      currentRisk: null,
      error: null,
      lastAnalyzedAt: null,
    }));
  }, []);

  return {
    // State
    ...state,
    
    // Actions
    analyzeTransactionRisk,
    checkAddressBlacklist,
    getUserRiskProfile,
    clearRiskAnalysis,
    
    // Computed values
    getCurrentRiskLevel,
    shouldBlockTransaction,
    getRiskScoreWithConfidence,
    getRiskFactors,
    
    // Status checks
    isHealthy: state.isAIEngineHealthy,
    hasRisk: state.currentRisk !== null,
    isBlocked: state.currentRisk ? !state.currentRisk.approved : false,
  };
}

export default useAIRiskAnalysis;