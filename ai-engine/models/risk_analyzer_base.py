from abc import ABC, abstractmethod
from typing import Dict


class RiskAnalyzerBase(ABC):
    """Abstract base class for risk analyzers."""

    @abstractmethod
    def analyze_risk(self, transaction_data: Dict) -> Dict:
        """Analyze transaction risk and return a result dictionary."""
        raise NotImplementedError 