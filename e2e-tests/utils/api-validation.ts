/**
 * API Response Schema Validation Utilities
 */
import { expect } from '@playwright/test';
import { ApiResponse, NonceResponse, AuthResponse, BridgeTransaction } from '../types/test-types.js';

export interface ValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
}

export class ApiValidator {
  /**
   * Validate base API response structure
   */
  static validateBaseResponse(response: any): ValidationResult {
    const result: ValidationResult = { valid: true, errors: [], warnings: [] };
    
    // Check required fields
    if (typeof response.success !== 'boolean') {
      result.errors.push('Missing or invalid "success" field');
      result.valid = false;
    }
    
    if (!response.timestamp) {
      result.warnings.push('Missing "timestamp" field');
    } else if (typeof response.timestamp !== 'string') {
      result.errors.push('Invalid "timestamp" field type');
      result.valid = false;
    }
    
    // If success is false, should have error message
    if (response.success === false && !response.error) {
      result.warnings.push('Error response missing error message');
    }
    
    // If success is true, should have data
    if (response.success === true && response.data === undefined) {
      result.warnings.push('Success response missing data field');
    }
    
    return result;
  }
  
  /**
   * Validate nonce response
   */
  static validateNonceResponse(response: any): ValidationResult {
    const result = this.validateBaseResponse(response);
    
    if (!result.valid) return result;
    
    if (response.success && response.data) {
      const data = response.data;
      
      // Validate nonce
      if (!data.nonce) {
        result.errors.push('Missing "nonce" field');
        result.valid = false;
      } else if (typeof data.nonce !== 'string') {
        result.errors.push('Invalid "nonce" field type');
        result.valid = false;
      } else if (!/^[a-f0-9]{64}$/i.test(data.nonce)) {
        result.errors.push('Invalid nonce format (should be 64-char hex)');
        result.valid = false;
      }
      
      // Validate message
      if (!data.message) {
        result.errors.push('Missing "message" field');
        result.valid = false;
      } else if (typeof data.message !== 'string') {
        result.errors.push('Invalid "message" field type');
        result.valid = false;
      }
      
      // Validate expiresAt
      if (data.expiresAt && typeof data.expiresAt !== 'string') {
        result.errors.push('Invalid "expiresAt" field type');
        result.valid = false;
      }
    }
    
    return result;
  }
  
  /**
   * Validate authentication response
   */
  static validateAuthResponse(response: any): ValidationResult {
    const result = this.validateBaseResponse(response);
    
    if (!result.valid) return result;
    
    if (response.success && response.data) {
      const data = response.data;
      
      // Validate token
      if (!data.token) {
        result.errors.push('Missing "token" field');
        result.valid = false;
      } else if (typeof data.token !== 'string') {
        result.errors.push('Invalid "token" field type');
        result.valid = false;
      } else if (data.token.split('.').length !== 3) {
        result.warnings.push('Token does not appear to be a JWT');
      }
      
      // Validate user object
      if (!data.user) {
        result.errors.push('Missing "user" field');
        result.valid = false;
      } else {
        if (!data.user.address) {
          result.errors.push('Missing "user.address" field');
          result.valid = false;
        }
        
        if (!data.user.chainType || !['ethereum', 'near'].includes(data.user.chainType)) {
          result.errors.push('Invalid "user.chainType" field');
          result.valid = false;
        }
      }
      
      // Validate expiresAt
      if (!data.expiresAt) {
        result.warnings.push('Missing "expiresAt" field');
      } else if (typeof data.expiresAt !== 'string') {
        result.errors.push('Invalid "expiresAt" field type');
        result.valid = false;
      }
    }
    
    return result;
  }
  
  /**
   * Validate bridge transaction response
   */
  static validateBridgeTransactionResponse(response: any): ValidationResult {
    const result = this.validateBaseResponse(response);
    
    if (!result.valid) return result;
    
    if (response.success && response.data) {
      const data = response.data;
      
      // Validate basic fields
      if (!data.id) {
        result.errors.push('Missing "id" field');
        result.valid = false;
      }
      
      if (!data.fromChain || !['ethereum', 'near'].includes(data.fromChain)) {
        result.errors.push('Invalid "fromChain" field');
        result.valid = false;
      }
      
      if (!data.toChain || !['ethereum', 'near'].includes(data.toChain)) {
        result.errors.push('Invalid "toChain" field');
        result.valid = false;
      }
      
      if (!data.amount || typeof data.amount !== 'string') {
        result.errors.push('Invalid "amount" field');
        result.valid = false;
      }
      
      if (!data.status || !['pending', 'processing', 'completed', 'failed'].includes(data.status)) {
        result.errors.push('Invalid "status" field');
        result.valid = false;
      }
      
      // Validate timestamps
      if (!data.createdAt) {
        result.errors.push('Missing "createdAt" field');
        result.valid = false;
      }
      
      if (!data.updatedAt) {
        result.errors.push('Missing "updatedAt" field');
        result.valid = false;
      }
      
      // Validate optional fields
      if (data.txHashes) {
        if (typeof data.txHashes !== 'object') {
          result.errors.push('Invalid "txHashes" field type');
          result.valid = false;
        }
      }
      
      if (data.quantumSignature && typeof data.quantumSignature !== 'string') {
        result.errors.push('Invalid "quantumSignature" field type');
        result.valid = false;
      }
      
      if (data.riskScore !== undefined && typeof data.riskScore !== 'number') {
        result.errors.push('Invalid "riskScore" field type');
        result.valid = false;
      }
    }
    
    return result;
  }
  
  /**
   * Validate health check response
   */
  static validateHealthResponse(response: any): ValidationResult {
    const result = this.validateBaseResponse(response);
    
    if (!result.valid) return result;
    
    if (response.success && response.data) {
      const data = response.data;
      
      if (!data.service) {
        result.errors.push('Missing "service" field');
        result.valid = false;
      }
      
      if (!data.version) {
        result.warnings.push('Missing "version" field');
      }
      
      if (!data.uptime) {
        result.warnings.push('Missing "uptime" field');
      }
      
      if (data.dependencies) {
        if (!Array.isArray(data.dependencies)) {
          result.errors.push('Invalid "dependencies" field type');
          result.valid = false;
        } else {
          data.dependencies.forEach((dep: any, index: number) => {
            if (!dep.name || !dep.status) {
              result.errors.push(`Invalid dependency at index ${index}`);
              result.valid = false;
            }
          });
        }
      }
    }
    
    return result;
  }
  
  /**
   * Assert validation result is valid
   */
  static assertValid(result: ValidationResult, context: string): void {
    if (!result.valid) {
      throw new Error(`${context} validation failed: ${result.errors.join(', ')}`);
    }
    
    if (result.warnings.length > 0) {
      console.warn(`⚠️ ${context} validation warnings: ${result.warnings.join(', ')}`);
    }
  }
  
  /**
   * Validate common error response
   */
  static validateErrorResponse(response: any, expectedStatus?: number): ValidationResult {
    const result: ValidationResult = { valid: true, errors: [], warnings: [] };
    
    if (response.success !== false) {
      result.errors.push('Error response should have success: false');
      result.valid = false;
    }
    
    if (!response.error) {
      result.errors.push('Error response should have error message');
      result.valid = false;
    } else if (typeof response.error !== 'string') {
      result.errors.push('Error message should be a string');
      result.valid = false;
    }
    
    if (response.data !== undefined && response.data !== null) {
      result.warnings.push('Error response should not have data field');
    }
    
    return result;
  }
  
  /**
   * Validate transaction list response
   */
  static validateTransactionListResponse(response: any): ValidationResult {
    const result = this.validateBaseResponse(response);
    
    if (!result.valid) return result;
    
    if (response.success && response.data) {
      if (!Array.isArray(response.data)) {
        result.errors.push('Transaction list data should be an array');
        result.valid = false;
        return result;
      }
      
      response.data.forEach((transaction: any, index: number) => {
        const txResult = this.validateBridgeTransactionResponse({ 
          success: true, 
          data: transaction 
        });
        
        if (!txResult.valid) {
          result.errors.push(`Transaction at index ${index}: ${txResult.errors.join(', ')}`);
          result.valid = false;
        }
        
        result.warnings.push(...txResult.warnings.map(w => `Transaction ${index}: ${w}`));
      });
      
      // Validate pagination if present
      if (response.pagination) {
        const pagination = response.pagination;
        
        if (typeof pagination.total !== 'number') {
          result.errors.push('Invalid pagination.total field');
          result.valid = false;
        }
        
        if (typeof pagination.page !== 'number') {
          result.errors.push('Invalid pagination.page field');
          result.valid = false;
        }
        
        if (typeof pagination.limit !== 'number') {
          result.errors.push('Invalid pagination.limit field');
          result.valid = false;
        }
      }
    }
    
    return result;
  }
  
  /**
   * Validate balance response
   */
  static validateBalanceResponse(response: any): ValidationResult {
    const result = this.validateBaseResponse(response);
    
    if (!result.valid) return result;
    
    if (response.success && response.data) {
      const data = response.data;
      
      if (!data.balances || typeof data.balances !== 'object') {
        result.errors.push('Missing or invalid "balances" field');
        result.valid = false;
        return result;
      }
      
      // Validate each balance entry
      Object.entries(data.balances).forEach(([currency, balance]) => {
        if (typeof balance !== 'string') {
          result.errors.push(`Invalid balance for ${currency}: should be string`);
          result.valid = false;
        } else if (!/^\d+(\.\d+)?$/.test(balance)) {
          result.errors.push(`Invalid balance format for ${currency}: ${balance}`);
          result.valid = false;
        }
      });
      
      if (!data.address) {
        result.errors.push('Missing "address" field');
        result.valid = false;
      }
      
      if (!data.updatedAt) {
        result.warnings.push('Missing "updatedAt" field');
      }
    }
    
    return result;
  }
}

/**
 * Helper function to validate API response with proper error handling
 */
export async function validateApiResponse<T>(
  responsePromise: Promise<Response>,
  validator: (data: any) => ValidationResult,
  context: string
): Promise<T> {
  try {
    const response = await responsePromise;
    
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }
    
    const data = await response.json();
    const validation = validator(data);
    
    ApiValidator.assertValid(validation, context);
    
    return data;
  } catch (error) {
    throw new Error(`${context} failed: ${error}`);
  }
}