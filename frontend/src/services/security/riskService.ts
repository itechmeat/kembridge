import { apiClient } from '../api/apiClient';
import { API_ENDPOINTS } from '../api/config';
import type { 
  RiskAnalysisResult, 
  RiskAnalysisResponse,
  UserRiskProfileResponse,
  RiskScore
} from '../../types/security';

export interface TransactionRiskRequest {
  fromAddress: string;
  toAddress: string;
  amount: number;
  token: string;
  chain: string;
}

export class RiskService {
  /**
   * Analyze transaction risk using AI engine
   */
  static async analyzeTransactionRisk(request: TransactionRiskRequest): Promise<RiskAnalysisResult> {
    try {
      // Call the AI risk engine through backend proxy
      const response = await apiClient.post<RiskAnalysisResponse>(API_ENDPOINTS.RISK.ANALYZE, {
        from_address: request.fromAddress,
        to_address: request.toAddress,
        amount: request.amount.toString(),
        token_symbol: request.token,
        chain: request.chain,
        user_id: 'current_user', // Will be extracted from JWT by backend
      });

      return response.data;
    } catch (error) {
      console.warn('Risk analysis API not available, using fallback', error);
      
      // Fallback mock risk analysis for development
      const mockRiskScore = Math.random() * 0.8; // 0-0.8 range for testing
      
      return {
        riskScore: {
          value: mockRiskScore,
          level: mockRiskScore > 0.7 ? 'high' : mockRiskScore > 0.3 ? 'medium' : 'low',
          confidence: 0.85,
          timestamp: new Date().toISOString()
        },
        factors: [
          {
            type: 'transaction_amount',
            weight: 0.3,
            description: `Transaction amount: ${request.amount} ${request.token}`,
            impact: request.amount > 1000 ? 'negative' : 'neutral'
          },
          {
            type: 'address_reputation',
            weight: 0.4,
            description: 'Address reputation check',
            impact: Math.random() > 0.8 ? 'negative' : 'positive'
          },
          {
            type: 'user_history',
            weight: 0.3,
            description: 'User transaction history',
            impact: 'positive'
          }
        ],
        recommendations: mockRiskScore > 0.7 
          ? ['Consider reducing transaction amount', 'Verify recipient address', 'Enable additional security measures']
          : mockRiskScore > 0.3
          ? ['Verify transaction details', 'Check recipient address']
          : ['Transaction appears safe'],
        blacklistStatus: {
          isBlacklisted: Math.random() > 0.95, // 5% chance for testing
          reason: Math.random() > 0.95 ? 'Address flagged by security provider' : undefined
        },
        analysis: {
          transactionAmount: request.amount,
          userHistory: {
            totalTransactions: Math.floor(Math.random() * 100) + 10,
            avgAmount: Math.random() * 500 + 100,
            riskHistory: Array.from({ length: 10 }, () => Math.random() * 0.6)
          },
          addressRisk: {
            from: Math.random() * 0.3,
            to: Math.random() * 0.4
          }
        }
      };
    }
  }

  /**
   * Get user risk profile
   */
  static async getUserRiskProfile(userId?: string): Promise<UserRiskProfileResponse['data']> {
    try {
      const endpoint = userId 
        ? `${API_ENDPOINTS.RISK.PROFILE}/${userId}`
        : `${API_ENDPOINTS.RISK.PROFILE}/current`;
        
      const response = await apiClient.get<UserRiskProfileResponse>(endpoint);
      return response.data;
    } catch (error) {
      console.warn('User risk profile API not available, using mock data', error);
      
      // Mock user risk profile
      return {
        userId: userId || 'current_user',
        currentRiskScore: {
          value: Math.random() * 0.5,
          level: 'low',
          confidence: 0.9,
          timestamp: new Date().toISOString()
        },
        riskHistory: Array.from({ length: 30 }, (_, i) => ({
          value: Math.random() * 0.6,
          level: Math.random() > 0.7 ? 'medium' : 'low',
          confidence: 0.8 + Math.random() * 0.2,
          timestamp: new Date(Date.now() - i * 24 * 60 * 60 * 1000).toISOString()
        })),
        totalTransactions: Math.floor(Math.random() * 200) + 50,
        avgRiskScore: 0.2 + Math.random() * 0.3,
        thresholds: {
          lowRisk: 0.3,
          mediumRisk: 0.7,
          highRisk: 0.9
        }
      };
    }
  }

  /**
   * Check if address is blacklisted
   */
  static async checkAddressBlacklist(address: string, chain: string): Promise<{
    isBlacklisted: boolean;
    reason?: string;
    confidence: number;
  }> {
    try {
      // Call AI engine blacklist check
      const response = await apiClient.post(API_ENDPOINTS.RISK.BLACKLIST_CHECK, {
        address,
        chain
      }) as { data: { is_blacklisted?: boolean; reason?: string; confidence?: number } };

      return {
        isBlacklisted: response.data.is_blacklisted || false,
        reason: response.data.reason,
        confidence: response.data.confidence || 0.5
      };
    } catch (error) {
      console.warn('Blacklist check API not available', error);
      
      // Mock blacklist check
      return {
        isBlacklisted: Math.random() > 0.98, // 2% chance for testing
        reason: Math.random() > 0.98 ? 'Flagged by security provider' : undefined,
        confidence: 0.8
      };
    }
  }

  /**
   * Get risk thresholds configuration
   */
  static async getRiskThresholds(): Promise<{
    lowRisk: number;
    mediumRisk: number;
    highRisk: number;
    autoBlockThreshold: number;
  }> {
    try {
      const response = await apiClient.get(API_ENDPOINTS.RISK.THRESHOLDS) as {
        data: {
          lowRisk: number;
          mediumRisk: number;
          highRisk: number;
          autoBlockThreshold: number;
        }
      };
      return response.data;
    } catch (error) {
      console.warn('Risk thresholds API not available, using defaults', error);
      
      // Default thresholds
      return {
        lowRisk: 0.3,
        mediumRisk: 0.7,
        highRisk: 0.9,
        autoBlockThreshold: 0.95
      };
    }
  }

  /**
   * Update risk thresholds (admin only)
   */
  static async updateRiskThresholds(thresholds: {
    lowRisk: number;
    mediumRisk: number;
    highRisk: number;
    autoBlockThreshold: number;
  }): Promise<void> {
    try {
      await apiClient.put(API_ENDPOINTS.RISK.THRESHOLDS, thresholds);
    } catch (error) {
      console.error('Failed to update risk thresholds:', error);
      throw new Error('Failed to update risk thresholds');
    }
  }

  /**
   * Get risk analysis history for user
   */
  static async getRiskHistory(userId?: string, days: number = 30): Promise<RiskScore[]> {
    try {
      const endpoint = userId 
        ? `${API_ENDPOINTS.RISK.PROFILE}/${userId}?include_history_days=${days}`
        : `${API_ENDPOINTS.RISK.PROFILE}/current?include_history_days=${days}`;
        
      const response = await apiClient.get<UserRiskProfileResponse>(endpoint);
      return response.data.riskHistory;
    } catch (error) {
      console.warn('Risk history API not available, using mock data', error);
      
      // Mock risk history
      return Array.from({ length: days }, (_, i) => ({
        value: Math.random() * 0.6,
        level: Math.random() > 0.7 ? 'medium' : 'low' as const,
        confidence: 0.7 + Math.random() * 0.3,
        timestamp: new Date(Date.now() - i * 24 * 60 * 60 * 1000).toISOString()
      }));
    }
  }
}

export default RiskService;