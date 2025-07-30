// Security and Risk related types

// Security Status Types
export enum SecurityLevel {
  SECURE = 'secure',
  WARNING = 'warning',
  DANGER = 'danger',
  OFFLINE = 'offline'
}

export interface QuantumProtectionStatus {
  isActive: boolean;
  algorithm: string; // ML-KEM-1024
  keyRotationDate: string;
  nextRotationDue: string;
  encryptionStrength: number;
}

export interface SecurityStatus {
  quantumProtection: QuantumProtectionStatus;
  overall: SecurityLevel;
  isOnline: boolean;
  lastUpdate: string;
  systemHealth: {
    backend: boolean;
    aiEngine: boolean;
    blockchain: boolean;
  };
}

// Risk Analysis Types
export interface RiskScore {
  value: number; // 0.0 - 1.0
  level: 'low' | 'medium' | 'high';
  confidence: number;
  timestamp: string;
}

export interface RiskFactor {
  type: string;
  weight: number;
  description: string;
  impact: 'positive' | 'negative' | 'neutral';
}

export interface RiskAnalysisResult {
  riskScore: RiskScore;
  factors: RiskFactor[];
  recommendations: string[];
  blacklistStatus: {
    isBlacklisted: boolean;
    reason?: string;
  };
  analysis: {
    transactionAmount: number;
    userHistory: {
      totalTransactions: number;
      avgAmount: number;
      riskHistory: number[];
    };
    addressRisk: {
      from: number;
      to: number;
    };
  };
}

// Security Alert Types
export enum AlertType {
  QUANTUM_OFFLINE = 'quantum_offline',
  HIGH_RISK_TRANSACTION = 'high_risk_transaction',
  SUSPICIOUS_ADDRESS = 'suspicious_address',
  RATE_LIMIT_WARNING = 'rate_limit_warning',
  SYSTEM_MAINTENANCE = 'system_maintenance',
  KEY_ROTATION_DUE = 'key_rotation_due',
  BLACKLIST_DETECTED = 'blacklist_detected'
}

export enum AlertPriority {
  CRITICAL = 'critical',
  HIGH = 'high',
  MEDIUM = 'medium',
  LOW = 'low'
}

export interface SecurityAlert {
  id: string;
  type: AlertType;
  priority: AlertPriority;
  title: string;
  message: string;
  timestamp: string;
  isRead: boolean;
  isDismissible: boolean;
  autoDismissAfter?: number; // milliseconds
  actions?: SecurityAlertAction[];
}

export interface SecurityAlertAction {
  id: string;
  label: string;
  type: 'primary' | 'secondary' | 'danger';
  handler: () => void;
}

// Security Settings Types
export interface SecuritySettings {
  riskTolerance: 'low' | 'medium' | 'high';
  autoBlockHighRisk: boolean;
  alertPreferences: {
    [key in AlertType]: boolean;
  };
  quantumSettings: {
    enableAutoRotation: boolean;
    rotationInterval: number; // days
    requireConfirmation: boolean;
  };
  monitoring: {
    realTimeAlerts: boolean;
    emailNotifications: boolean;
    webhookUrl?: string;
  };
}

// API Response Types
export interface SecurityStatusResponse {
  data: SecurityStatus;
  success: boolean;
  timestamp: string;
}

export interface RiskAnalysisResponse {
  data: RiskAnalysisResult;
  success: boolean;
  timestamp: string;
}

export interface UserRiskProfileResponse {
  data: {
    userId: string;
    currentRiskScore: RiskScore;
    riskHistory: RiskScore[];
    totalTransactions: number;
    avgRiskScore: number;
    thresholds: {
      lowRisk: number;
      mediumRisk: number;
      highRisk: number;
    };
  };
  success: boolean;
}

export interface SecuritySettingsResponse {
  data: SecuritySettings;
  success: boolean;
}

// Component Props Types
export interface SecurityIndicatorProps {
  quantumProtection: boolean;
  riskScore: number;
  isOnline: boolean;
  compact?: boolean;
  className?: string;
  quantumKeyId?: string;
  encryptionScheme?: string;
  lastKeyRotation?: string;
  transactionCount?: number;
}

export interface RiskAnalysisDisplayProps {
  riskData: RiskAnalysisResult;
  realTime?: boolean;
  showDetails?: boolean;
  onUpdate?: (data: RiskAnalysisResult) => void;
}

export interface SecurityAlertsProps {
  alerts: SecurityAlert[];
  maxVisible?: number;
  position?: 'top' | 'bottom' | 'floating';
  onDismiss: (alertId: string) => void;
  onAction: (alertId: string, actionId: string) => void;
}

export interface RiskScoreVisualizationProps {
  score: number;
  animated?: boolean;
  size?: 'small' | 'medium' | 'large';
  showLabel?: boolean;
  showTrend?: boolean;
  previousScore?: number;
}

export interface SecuritySettingsProps {
  settings: SecuritySettings;
  onUpdate: (settings: SecuritySettings) => void;
  isLoading?: boolean;
  readonly?: boolean;
}

// Utility Types
export type SecurityComponent = 
  | 'indicator' 
  | 'risk-display' 
  | 'alerts' 
  | 'score-viz' 
  | 'settings';

export interface SecurityMetrics {
  uptime: number;
  lastIncident: string | null;
  totalAlerts: number;
  resolvedAlerts: number;
  avgResponseTime: number;
}