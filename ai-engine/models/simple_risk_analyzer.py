"""
Simple Risk Analyzer Module (без ML для первоначального тестирования)
"""

import logging
from typing import Dict, List
from datetime import datetime
from .blacklist_checker import BlacklistChecker

logger = logging.getLogger(__name__)

class SimpleRiskAnalyzer:
    def __init__(self):
        self.blacklist_checker = BlacklistChecker()
    
    def analyze_risk(self, transaction_data: Dict) -> Dict:
        """Простой анализ рисков без ML"""
        try:
            # Blacklist проверка
            blacklist_result = self._check_blacklist(transaction_data)
            
            # Если адрес в blacklist, сразу высокий риск
            if blacklist_result['is_blacklisted']:
                return {
                    "risk_score": 1.0,
                    "risk_level": "high",
                    "is_anomaly": True,
                    "anomaly_score": -1.0,
                    "ml_confidence": 1.0,
                    "risk_factors": [blacklist_result['reason']],
                    "recommended_action": "block_transaction",
                    "approved": False,
                    "blacklist_check": blacklist_result
                }
            
            # Rule-based анализ
            return self._rule_based_analysis(transaction_data, blacklist_result)
                
        except Exception as e:
            logger.error(f"Risk analysis failed: {e}")
            return self._emergency_fallback()
    
    def _check_blacklist(self, transaction_data: Dict) -> Dict:
        """Проверка blacklist"""
        results = {
            "is_blacklisted": False,
            "reason": None,
            "source": None,
            "risk_score_increase": 0.0
        }
        
        # Проверяем user_address если доступен
        if 'user_address' in transaction_data and transaction_data['user_address']:
            source_chain = transaction_data.get('source_chain', 'ethereum')
            check_result = self.blacklist_checker.check_address(
                transaction_data['user_address'], 
                source_chain
            )
            if check_result['is_blacklisted']:
                return check_result
        
        return results
    
    def _rule_based_analysis(self, transaction_data: Dict, blacklist_result: Dict) -> Dict:
        """Rule-based risk analysis"""
        risk_score = 0.1  # Base risk
        risk_factors = []
        
        amount = float(transaction_data.get('amount_in', 0))
        
        # Amount-based risk
        if amount > 100:
            risk_score += 0.4
            risk_factors.append("Very large transaction amount (>100)")
        elif amount > 10:
            risk_score += 0.2
            risk_factors.append("Large transaction amount (>10)")
        elif amount > 1:
            risk_score += 0.1
            risk_factors.append("Medium transaction amount (>1)")
        
        # Cross-chain risk
        if (transaction_data.get('source_chain') != 
            transaction_data.get('destination_chain')):
            risk_score += 0.1
            risk_factors.append("Cross-chain transaction")
        
        # User history risk
        user_history = transaction_data.get('user_history', {})
        if user_history.get('is_new_user', True):
            risk_score += 0.2
            risk_factors.append("New user with no transaction history")
        elif user_history.get('high_risk_ratio', 0) > 0.3:
            risk_score += 0.2
            risk_factors.append("User has history of high-risk transactions")
        
        # Velocity check
        if (not user_history.get('is_new_user', True) and 
            user_history.get('avg_transaction_size', 0) > 0):
            if amount > user_history.get('avg_transaction_size', 0) * 5:
                risk_score += 0.3
                risk_factors.append("Transaction significantly larger than user average")
        
        # Add blacklist penalty
        risk_score += blacklist_result['risk_score_increase']
        if blacklist_result['reason']:
            risk_factors.append(blacklist_result['reason'])
        
        risk_score = min(risk_score, 1.0)
        
        # Determine risk level and approval
        if risk_score < 0.3:
            risk_level = "low"
            approved = True
            recommended_action = "proceed"
        elif risk_score < 0.6:
            risk_level = "medium"
            approved = True
            recommended_action = "proceed_with_monitoring"
        elif risk_score < 0.8:
            risk_level = "high"
            approved = False
            recommended_action = "manual_review_required"
        else:
            risk_level = "critical"
            approved = False
            recommended_action = "block_transaction"
        
        if not risk_factors:
            risk_factors = ["Transaction appears normal"]
        
        return {
            "risk_score": risk_score,
            "risk_level": risk_level,
            "is_anomaly": risk_score > 0.7,
            "anomaly_score": None,
            "ml_confidence": 0.7,  # Rule-based confidence
            "risk_factors": risk_factors,
            "recommended_action": recommended_action,
            "approved": approved,
            "blacklist_check": blacklist_result
        }
    
    def _emergency_fallback(self) -> Dict:
        """Emergency fallback при полном отказе анализа"""
        return {
            "risk_score": 0.5,
            "risk_level": "medium",
            "is_anomaly": False,
            "anomaly_score": None,
            "ml_confidence": 0.0,
            "risk_factors": ["Risk analysis system error - manual review required"],
            "recommended_action": "manual_review_required",
            "approved": False,
            "blacklist_check": {"is_blacklisted": False, "reason": None}
        }