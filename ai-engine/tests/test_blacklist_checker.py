import sys, os; sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
from models.blacklist_checker import BlacklistChecker


def test_blacklisted_address():
    checker = BlacklistChecker()
    result = checker.check_address("0x000000000000000000000000000000000000dead", "ethereum")
    assert result["is_blacklisted"] is True


def test_clean_address():
    checker = BlacklistChecker()
    result = checker.check_address("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd", "ethereum")
    assert result["is_blacklisted"] is False 