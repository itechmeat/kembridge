"""
Blacklist Checker Module
Проверка адресов против known malicious addresses
"""

import logging
from typing import Set, List, Dict, Optional
import asyncio
import aiohttp
import json
from datetime import datetime, timedelta

logger = logging.getLogger(__name__)

class BlacklistChecker:
    def __init__(self):
        # Static blacklist для демонстрации
        self.static_blacklist: Set[str] = {
            # Известные мошеннические адреса Ethereum (примеры)
            "0x000000000000000000000000000000000000dead",
            "0x0000000000000000000000000000000000000000",
            # Fake addresses для тестирования
            "0x1234567890123456789012345678901234567890",
            "0xdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
        }
        
        # NEAR blacklist
        self.near_blacklist: Set[str] = {
            "scammer.near",
            "phishing.near", 
            "fake-wallet.near"
        }
        
        # Cache для external API results
        self.cache: Dict[str, Dict] = {}
        self.cache_ttl = timedelta(hours=1)
        
    def is_ethereum_address_blacklisted(self, address: str) -> Dict[str, any]:
        """Проверка Ethereum адреса в blacklist"""
        address_lower = address.lower()
        
        result = {
            "is_blacklisted": False,
            "reason": None,
            "source": None,
            "risk_score_increase": 0.0
        }
        
        # Проверка в static blacklist
        if address_lower in self.static_blacklist:
            result.update({
                "is_blacklisted": True,
                "reason": "Address in static blacklist",
                "source": "kembridge_static",
                "risk_score_increase": 1.0  # Максимальный риск
            })
            return result
        
        # Проверка паттернов подозрительных адресов
        suspicious_patterns = [
            "0x000000",  # Null-like addresses
            "0xffffff",  # Suspicious patterns
            "deadbeef",  # Common test patterns
        ]
        
        for pattern in suspicious_patterns:
            if pattern in address_lower:
                result.update({
                    "is_blacklisted": True,
                    "reason": f"Suspicious address pattern: {pattern}",
                    "source": "pattern_detection",
                    "risk_score_increase": 0.5
                })
                return result
        
        return result
    
    def is_near_address_blacklisted(self, address: str) -> Dict[str, any]:
        """Проверка NEAR адреса в blacklist"""
        result = {
            "is_blacklisted": False,
            "reason": None,
            "source": None,
            "risk_score_increase": 0.0
        }
        
        if address in self.near_blacklist:
            result.update({
                "is_blacklisted": True,
                "reason": "NEAR address in blacklist",
                "source": "kembridge_near_static",
                "risk_score_increase": 1.0
            })
            return result
        
        # Проверка подозрительных паттернов NEAR
        suspicious_keywords = ["scam", "phish", "fake", "fraud", "hack"]
        for keyword in suspicious_keywords:
            if keyword in address.lower():
                result.update({
                    "is_blacklisted": True,
                    "reason": f"Suspicious NEAR account name contains: {keyword}",
                    "source": "near_pattern_detection",
                    "risk_score_increase": 0.7
                })
                return result
        
        return result
    
    def check_address(self, address: str, chain: str) -> Dict[str, any]:
        """Универсальная проверка адреса"""
        try:
            if chain.lower() == "ethereum":
                return self.is_ethereum_address_blacklisted(address)
            elif chain.lower() == "near":
                return self.is_near_address_blacklisted(address)
            else:
                logger.warning(f"Unsupported chain for blacklist check: {chain}")
                return {
                    "is_blacklisted": False,
                    "reason": None,
                    "source": None,
                    "risk_score_increase": 0.0
                }
        except Exception as e:
            logger.error(f"Error checking blacklist for {address} on {chain}: {e}")
            return {
                "is_blacklisted": False,
                "reason": None,
                "source": None,
                "risk_score_increase": 0.0
            }
    
    async def check_external_api(self, address: str, chain: str) -> Dict[str, any]:
        """
        Проверка через внешние API (заглушка для будущей интеграции)
        В production можно интегрировать с Chainalysis, TRM Labs, etc.
        """
        cache_key = f"{chain}:{address}"
        
        # Проверка cache
        if cache_key in self.cache:
            cached_data = self.cache[cache_key]
            if datetime.now() - cached_data["timestamp"] < self.cache_ttl:
                return cached_data["result"]
        
        # Заглушка для external API call
        # В production здесь будет реальный API call
        result = {
            "is_blacklisted": False,
            "reason": None,
            "source": "external_api_placeholder",
            "risk_score_increase": 0.0
        }
        
        # Кеширование результата
        self.cache[cache_key] = {
            "result": result,
            "timestamp": datetime.now()
        }
        
        return result
    
    def add_to_blacklist(self, address: str, chain: str, reason: str) -> bool:
        """Добавление адреса в blacklist (admin функция)"""
        try:
            if chain.lower() == "ethereum":
                self.static_blacklist.add(address.lower())
            elif chain.lower() == "near":
                self.near_blacklist.add(address)
            else:
                return False
            
            logger.info(f"Added {address} to {chain} blacklist: {reason}")
            return True
            
        except Exception as e:
            logger.error(f"Failed to add {address} to blacklist: {e}")
            return False
    
    def get_blacklist_stats(self) -> Dict[str, any]:
        """Статистика blacklist"""
        return {
            "ethereum_blacklist_size": len(self.static_blacklist),
            "near_blacklist_size": len(self.near_blacklist),
            "cache_size": len(self.cache),
            "last_updated": datetime.now().isoformat()
        }