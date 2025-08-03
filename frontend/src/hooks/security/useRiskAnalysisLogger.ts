import { useRef, useCallback } from "react";

export interface RiskScoreData {
  value: number;
  level: string;
  confidence: number;
}

export interface RiskData {
  riskScore: RiskScoreData;
}

export const useRiskAnalysisLogger = (debounceMs: number = 1000) => {
  const timeoutRef = useRef<number | null>(null);
  const lastLoggedScore = useRef<number>(0);

  const logRiskUpdate = useCallback((data: RiskData) => {
    // Clear previous timeout
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }

    timeoutRef.current = window.setTimeout(() => {
      const { riskScore } = data;
      
      // Only log if score changed significantly or is high risk
      const scoreChanged = Math.abs(riskScore.value - lastLoggedScore.current) > 0.1;
      const isHighRisk = riskScore.value > 0.5;
      
      if (scoreChanged && isHighRisk) {
        console.log("⚠️ Risk analysis:", {
          score: riskScore.value,
          level: riskScore.level,
          confidence: riskScore.confidence
        });
        lastLoggedScore.current = riskScore.value;
      }

      // Handle high-risk transaction blocking
      if (riskScore.value > 0.8) {
        console.warn("🚨 High risk transaction detected!", riskScore);
      }
    }, debounceMs);
  }, [debounceMs]);

  // Cleanup on unmount
  const cleanup = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
  }, []);

  return { logRiskUpdate, cleanup };
};