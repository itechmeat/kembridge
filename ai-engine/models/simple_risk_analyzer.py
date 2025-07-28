"""Simple risk analyzer used for initial testing without ML."""

import logging
from typing import Dict, List
from datetime import datetime
from .blacklist_checker import BlacklistChecker
from .common_rules import rule_based_score, determine_risk_level
from .risk_analyzer_base import RiskAnalyzerBase

logger = logging.getLogger(__name__)

class SimpleRiskAnalyzer(RiskAnalyzerBase):
    def __init__(self):
        self.blacklist_checker = BlacklistChecker()
    
    def analyze_risk(self, transaction_data: Dict) -> Dict:
        """Perform a simple rule-based risk analysis without ML."""
        try:
            # Check user address against the blacklist
            blacklist_result = self._check_blacklist(transaction_data)
            
            # If the address is blacklisted, return high risk immediately
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
            
            # Otherwise run rule-based analysis
            return self._rule_based_analysis(transaction_data, blacklist_result)
                
        except Exception as e:
            logger.error(f"Risk analysis failed: {e}")
            return self._emergency_fallback()
    
    def _check_blacklist(self, transaction_data: Dict) -> Dict:
        """Check the blacklist for the provided addresses."""
        results = {
            "is_blacklisted": False,
            "reason": None,
            "source": None,
            "risk_score_increase": 0.0
        }
        
        # Check user_address if provided
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
        """Calculate risk score using predefined rules."""
        score, factors = rule_based_score(transaction_data)
        score += blacklist_result["risk_score_increase"]
        if blacklist_result.get("reason"):
            factors.append(blacklist_result["reason"])

        score = min(score, 1.0)
        risk_level, approved, action = determine_risk_level(score)

        if not factors:
            factors = ["Transaction appears normal"]
        
        return {
            "risk_score": score,
            "risk_level": risk_level,
            "is_anomaly": score > 0.7,
            "anomaly_score": None,
            "ml_confidence": 0.7,
            "risk_factors": factors,
            "recommended_action": action,
            "approved": approved,
            "blacklist_check": blacklist_result,
        }
    
    def _emergency_fallback(self) -> Dict:
        """Emergency fallback used when analysis completely fails."""
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