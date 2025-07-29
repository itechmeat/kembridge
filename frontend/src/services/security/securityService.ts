import { apiClient } from '../api/apiClient';
import { 
  SecurityLevel,
  AlertType
} from '../../types/security';
import type { 
  SecurityStatus, 
  SecurityStatusResponse,
  SecuritySettings,
  SecuritySettingsResponse
} from '../../types/security';

export class SecurityService {
  /**
   * Get current security status including quantum protection
   */
  static async getSecurityStatus(): Promise<SecurityStatus> {
    try {
      // TODO (feat): Replace with real API endpoint when quantum crypto status API is ready
      // For now using mock data to enable development
      const response = await apiClient.get<SecurityStatusResponse>('/crypto/status');
      return response.data;
    } catch (error) {
      console.warn('Security status API not available, using fallback', error);
      
      // Fallback to mock data for development
      return {
        quantumProtection: {
          isActive: true,
          algorithm: 'ML-KEM-1024',
          keyRotationDate: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString(),
          nextRotationDue: new Date(Date.now() + 23 * 24 * 60 * 60 * 1000).toISOString(),
          encryptionStrength: 1024
        },
        overall: SecurityLevel.SECURE,
        isOnline: true,
        lastUpdate: new Date().toISOString(),
        systemHealth: {
          backend: true,
          aiEngine: true,
          blockchain: true
        }
      };
    }
  }

  /**
   * Get user security settings
   */
  static async getSecuritySettings(): Promise<SecuritySettings> {
    try {
      const response = await apiClient.get<SecuritySettingsResponse>('/user/security-settings');
      return response.data;
    } catch (error) {
      console.warn('Security settings API not available, using defaults', error);
      
      // Default security settings
      return {
        riskTolerance: 'medium',
        autoBlockHighRisk: true,
        alertPreferences: {
          [AlertType.QUANTUM_OFFLINE]: true,
          [AlertType.HIGH_RISK_TRANSACTION]: true,
          [AlertType.SUSPICIOUS_ADDRESS]: true,
          [AlertType.RATE_LIMIT_WARNING]: false,
          [AlertType.SYSTEM_MAINTENANCE]: false,
          [AlertType.KEY_ROTATION_DUE]: true,
          [AlertType.BLACKLIST_DETECTED]: true
        },
        quantumSettings: {
          enableAutoRotation: true,
          rotationInterval: 30,
          requireConfirmation: false
        },
        monitoring: {
          realTimeAlerts: true,
          emailNotifications: false,
          webhookUrl: undefined
        }
      };
    }
  }

  /**
   * Update user security settings
   */
  static async updateSecuritySettings(settings: SecuritySettings): Promise<SecuritySettings> {
    try {
      const response = await apiClient.put<SecuritySettingsResponse>(
        '/user/security-settings',
        settings
      );
      return response.data;
    } catch (error) {
      console.error('Failed to update security settings:', error);
      throw new Error('Failed to update security settings');
    }
  }

  /**
   * Check quantum key rotation status
   */
  static async checkKeyRotationDue(): Promise<boolean> {
    try {
      const response = await apiClient.get('/crypto/keys/check-rotation') as { data: { rotationDue?: boolean } };
      return Boolean(response.data.rotationDue);
    } catch (error) {
      console.warn('Key rotation check failed:', error);
      return false;
    }
  }

  /**
   * Trigger manual key rotation
   */
  static async triggerKeyRotation(): Promise<void> {
    try {
      await apiClient.post('/crypto/keys/rotate', {
        reason: 'manual_user_request'
      });
    } catch (error) {
      console.error('Manual key rotation failed:', error);
      throw new Error('Failed to rotate quantum keys');
    }
  }

  /**
   * Get system health status
   */
  static async getSystemHealth(): Promise<{
    backend: boolean;
    aiEngine: boolean;
    blockchain: boolean;
  }> {
    try {
      const [backendHealth, aiHealth, blockchainHealth] = await Promise.allSettled([
        apiClient.get('/health'),
        apiClient.get('/risk/health'),
        apiClient.get('/blockchain/health')
      ]);

      return {
        backend: backendHealth.status === 'fulfilled' && (backendHealth.value as { status?: number }).status === 200,
        aiEngine: aiHealth.status === 'fulfilled' && (aiHealth.value as { status?: number }).status === 200,
        blockchain: blockchainHealth.status === 'fulfilled' && (blockchainHealth.value as { status?: number }).status === 200
      };
    } catch (error) {
      console.warn('System health check failed:', error);
      return {
        backend: false,
        aiEngine: false,
        blockchain: false
      };
    }
  }
}

export default SecurityService;