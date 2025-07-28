import sys, os; sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
import pytest
from datetime import datetime
from main import RiskAnalysisService, UserHistoryData
from models.simple_risk_analyzer import SimpleRiskAnalyzer


class DummyConn:
    def __init__(self, row):
        self.row = row

    async def fetchrow(self, query, user_id):
        return self.row


@pytest.mark.asyncio
async def test_get_user_history_existing_user():
    analyzer = SimpleRiskAnalyzer()
    service = RiskAnalysisService(analyzer)
    row = {
        "total_transactions": 2,
        "total_volume": 5.0,
        "avg_transaction_size": 2.5,
        "last_transaction": datetime.now(),
        "first_transaction": datetime.now(),
        "high_risk_count": 1,
    }
    conn = DummyConn(row)
    history = await service.get_user_history(conn, "user1")
    assert history.is_new_user is False
    assert history.total_transactions == 2 