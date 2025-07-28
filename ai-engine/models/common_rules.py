from typing import Dict, List, Tuple
from config import settings


def rule_based_score(transaction_data: Dict) -> Tuple[float, List[str]]:
    """Calculate rule-based score and corresponding factors."""
    risk_score = 0.1
    factors: List[str] = []

    amount = float(transaction_data.get("amount_in", 0))
    if amount > 100:
        risk_score += 0.4
        factors.append("Very large transaction amount (>100)")
    elif amount > 10:
        risk_score += 0.2
        factors.append("Large transaction amount (>10)")
    elif amount > 1:
        risk_score += 0.1
        factors.append("Medium transaction amount (>1)")

    if transaction_data.get("source_chain") != transaction_data.get("destination_chain"):
        risk_score += 0.1
        factors.append("Cross-chain transaction")

    user_history = transaction_data.get("user_history", {})
    if user_history.get("is_new_user", True):
        risk_score += 0.2
        factors.append("New user with no transaction history")
    elif user_history.get("high_risk_ratio", 0) > 0.3:
        risk_score += 0.2
        factors.append("User has history of high-risk transactions")

    if (
        not user_history.get("is_new_user", True)
        and user_history.get("avg_transaction_size", 0) > 0
        and amount > user_history.get("avg_transaction_size", 0) * 5
    ):
        risk_score += 0.3
        factors.append("Transaction significantly larger than user average")

    return min(risk_score, 1.0), factors


def determine_risk_level(score: float) -> Tuple[str, bool, str]:
    """Map score to risk level, approval flag and recommended action."""
    if score < settings.risk_threshold_low:
        return "low", True, "proceed"
    if score < settings.risk_threshold_medium:
        return "medium", True, "proceed_with_monitoring"
    if score < settings.risk_threshold_high:
        return "high", False, "manual_review_required"
    return "critical", False, "block_transaction" 