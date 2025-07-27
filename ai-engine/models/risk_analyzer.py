"""
Risk Analyzer Module
ML-powered risk analysis для bridge транзакций
"""

import numpy as np
import logging
from typing import Dict, List, Tuple, Optional
from datetime import datetime
from sklearn.ensemble import IsolationForest
from sklearn.linear_model import LogisticRegression
from sklearn.preprocessing import StandardScaler
import joblib
import os
from .blacklist_checker import BlacklistChecker

logger = logging.getLogger(__name__)

class RiskAnalyzer:
    def __init__(self, model_path: str = "models/saved/"):
        self.model_path = model_path
        self.isolation_forest = None
        self.risk_classifier = None
        self.feature_scaler = None
        self.blacklist_checker = BlacklistChecker()
        self.is_trained = False
        
        # Убедимся что директория для моделей существует
        os.makedirs(self.model_path, exist_ok=True)
        
        # Загрузка сохраненных моделей
        self._load_models()
        
        # Если модели не загружены, создаем базовые
        if not self.is_trained:
            self._initialize_basic_models()
    
    def _load_models(self):
        """Загрузка обученных моделей"""
        try:
            isolation_path = os.path.join(self.model_path, "isolation_forest.joblib")
            classifier_path = os.path.join(self.model_path, "risk_classifier.joblib")
            scaler_path = os.path.join(self.model_path, "feature_scaler.joblib")
            
            if all(os.path.exists(p) for p in [isolation_path, classifier_path, scaler_path]):
                self.isolation_forest = joblib.load(isolation_path)
                self.risk_classifier = joblib.load(classifier_path)
                self.feature_scaler = joblib.load(scaler_path)
                self.is_trained = True
                logger.info("ML models loaded successfully")
            else:
                logger.info("No pre-trained models found, will initialize basic models")
                
        except Exception as e:
            logger.error(f"Error loading models: {e}")
            self.is_trained = False
    
    def _initialize_basic_models(self):
        """Инициализация базовых ML моделей с dummy data"""
        try:
            logger.info("Initializing basic ML models...")
            
            # Создаем dummy training data для демонстрации
            X_dummy = np.random.rand(100, 15)  # 100 samples, 15 features
            y_dummy = np.random.choice([0, 1], 100, p=[0.8, 0.2])  # 20% high risk
            
            # Инициализация моделей
            self.feature_scaler = StandardScaler()
            self.isolation_forest = IsolationForest(
                contamination=0.1,  # 10% outliers
                random_state=42
            )
            self.risk_classifier = LogisticRegression(
                random_state=42,
                class_weight='balanced'
            )
            
            # "Обучение" на dummy data
            X_scaled = self.feature_scaler.fit_transform(X_dummy)
            self.isolation_forest.fit(X_scaled)
            self.risk_classifier.fit(X_scaled, y_dummy)
            
            # Сохранение моделей
            self._save_models()
            
            self.is_trained = True
            logger.info("Basic ML models initialized successfully")
            
        except Exception as e:
            logger.error(f"Failed to initialize basic models: {e}")
            self.is_trained = False
    
    def _save_models(self):
        """Сохранение моделей"""
        try:
            joblib.dump(self.isolation_forest, os.path.join(self.model_path, "isolation_forest.joblib"))
            joblib.dump(self.risk_classifier, os.path.join(self.model_path, "risk_classifier.joblib"))
            joblib.dump(self.feature_scaler, os.path.join(self.model_path, "feature_scaler.joblib"))
            logger.info("Models saved successfully")
        except Exception as e:
            logger.error(f"Failed to save models: {e}")
    
    def extract_features(self, transaction_data: Dict) -> np.ndarray:
        """Извлечение признаков для ML анализа"""
        features = []
        
        # Transaction amount features
        amount = float(transaction_data.get('amount_in', 0))
        features.extend([
            amount,
            np.log1p(amount),  # Log transformation
            min(amount / 1000, 1.0),  # Normalized amount
        ])
        
        # Chain features (one-hot encoding)
        source_chain = transaction_data.get('source_chain', '').lower()
        dest_chain = transaction_data.get('destination_chain', '').lower()
        features.extend([
            1.0 if source_chain == 'ethereum' else 0.0,
            1.0 if source_chain == 'near' else 0.0,
            1.0 if dest_chain == 'ethereum' else 0.0,
            1.0 if dest_chain == 'near' else 0.0,
            1.0 if source_chain != dest_chain else 0.0,  # Cross-chain indicator
        ])
        
        # User history features
        user_history = transaction_data.get('user_history', {})
        features.extend([
            float(user_history.get('total_transactions', 0)),
            float(user_history.get('total_volume', 0)),
            float(user_history.get('avg_transaction_size', 0)),
            float(user_history.get('days_since_first_tx', 0)),
            float(user_history.get('high_risk_ratio', 0)),
            1.0 if user_history.get('is_new_user', True) else 0.0,
        ])
        
        # Time-based features (циклическое представление)
        hour_of_day = transaction_data.get('hour_of_day', datetime.now().hour)
        day_of_week = transaction_data.get('day_of_week', datetime.now().weekday())
        features.extend([
            np.sin(2 * np.pi * hour_of_day / 24),
            np.cos(2 * np.pi * hour_of_day / 24),
        ])
        
        return np.array(features).reshape(1, -1)
    
    def analyze_risk(self, transaction_data: Dict) -> Dict:
        """Основной метод анализа рисков"""
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
            
            # ML анализ если модели обучены
            if self.is_trained:
                return self._ml_analysis(transaction_data, blacklist_result)
            else:
                return self._fallback_analysis(transaction_data, blacklist_result)
                
        except Exception as e:
            logger.error(f"Risk analysis failed: {e}")
            return self._emergency_fallback()
    
    def _check_blacklist(self, transaction_data: Dict) -> Dict:
        """Проверка blacklist для всех адресов в транзакции"""
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
    
    def _ml_analysis(self, transaction_data: Dict, blacklist_result: Dict) -> Dict:
        """ML-powered risk analysis"""
        try:
            # Извлечение признаков
            features = self.extract_features(transaction_data)
            
            # Нормализация признаков
            features_scaled = self.feature_scaler.transform(features)
            
            # Anomaly detection
            anomaly_score = self.isolation_forest.decision_function(features_scaled)[0]
            is_anomaly = self.isolation_forest.predict(features_scaled)[0] == -1
            
            # Risk classification
            risk_proba = self.risk_classifier.predict_proba(features_scaled)[0]
            
            # Комбинированный risk score
            ml_risk_score = risk_proba[1] if len(risk_proba) > 1 else 0.5
            rule_based_score = self._rule_based_analysis(transaction_data)
            
            # Weighted combination
            combined_score = (
                0.4 * ml_risk_score +                    # ML prediction
                0.3 * max(0, (1 - (anomaly_score + 1) / 2)) +  # Anomaly (normalized)
                0.3 * rule_based_score +                 # Rule-based
                blacklist_result['risk_score_increase']  # Blacklist penalty
            )
            
            combined_score = min(max(combined_score, 0.0), 1.0)
            
            # Risk level determination
            risk_level, approved, recommended_action = self._determine_risk_level(combined_score)
            
            # Risk factors identification
            risk_factors = self._identify_risk_factors(transaction_data, features_scaled[0])
            if blacklist_result['reason']:
                risk_factors.append(blacklist_result['reason'])
            
            return {
                "risk_score": float(combined_score),
                "risk_level": risk_level,
                "is_anomaly": bool(is_anomaly),
                "anomaly_score": float(anomaly_score),
                "ml_confidence": float(max(risk_proba)),
                "risk_factors": risk_factors,
                "recommended_action": recommended_action,
                "approved": approved,
                "blacklist_check": blacklist_result
            }
            
        except Exception as e:
            logger.error(f"ML analysis failed: {e}")
            return self._fallback_analysis(transaction_data, blacklist_result)
    
    def _rule_based_analysis(self, transaction_data: Dict) -> float:
        """Rule-based risk analysis"""
        risk_score = 0.1  # Base risk
        
        amount = float(transaction_data.get('amount_in', 0))
        
        # Amount-based risk
        if amount > 100:
            risk_score += 0.4
        elif amount > 10:
            risk_score += 0.2
        elif amount > 1:
            risk_score += 0.1
        
        # Cross-chain risk
        if (transaction_data.get('source_chain') != 
            transaction_data.get('destination_chain')):
            risk_score += 0.1
        
        # User history risk
        user_history = transaction_data.get('user_history', {})
        if user_history.get('is_new_user', True):
            risk_score += 0.2
        elif user_history.get('high_risk_ratio', 0) > 0.3:
            risk_score += 0.2
        
        return min(risk_score, 1.0)
    
    def _fallback_analysis(self, transaction_data: Dict, blacklist_result: Dict) -> Dict:
        """Fallback analysis when ML models are not available"""
        risk_score = self._rule_based_analysis(transaction_data)
        risk_score += blacklist_result['risk_score_increase']
        risk_score = min(risk_score, 1.0)
        
        risk_level, approved, recommended_action = self._determine_risk_level(risk_score)
        
        risk_factors = []
        amount = float(transaction_data.get('amount_in', 0))
        
        if amount > 100:
            risk_factors.append("Very large transaction amount")
        elif amount > 10:
            risk_factors.append("Large transaction amount")
        
        if transaction_data.get('source_chain') != transaction_data.get('destination_chain'):
            risk_factors.append("Cross-chain transaction")
        
        user_history = transaction_data.get('user_history', {})
        if user_history.get('is_new_user', True):
            risk_factors.append("New user")
        
        if blacklist_result['reason']:
            risk_factors.append(blacklist_result['reason'])
        
        if not risk_factors:
            risk_factors = ["Transaction appears normal"]
        
        return {
            "risk_score": risk_score,
            "risk_level": risk_level,
            "is_anomaly": risk_score > 0.7,
            "anomaly_score": None,
            "ml_confidence": 0.6,  # Lower confidence for rule-based
            "risk_factors": risk_factors,
            "recommended_action": recommended_action,
            "approved": approved,
            "blacklist_check": blacklist_result
        }
    
    def _determine_risk_level(self, risk_score: float) -> Tuple[str, bool, str]:
        """Определение уровня риска и рекомендаций"""
        if risk_score < 0.3:
            return "low", True, "proceed"
        elif risk_score < 0.6:
            return "medium", True, "proceed_with_monitoring"
        elif risk_score < 0.8:
            return "high", False, "manual_review_required"
        else:
            return "critical", False, "block_transaction"
    
    def _identify_risk_factors(self, transaction_data: Dict, features: np.ndarray) -> List[str]:
        """Идентификация основных факторов риска"""
        factors = []
        
        amount = float(transaction_data.get('amount_in', 0))
        if amount > 50:
            factors.append(f"Large amount: {amount}")
        
        user_history = transaction_data.get('user_history', {})
        if user_history.get('is_new_user', True):
            factors.append("New user with no history")
        elif user_history.get('high_risk_ratio', 0) > 0.3:
            factors.append("User has history of high-risk transactions")
        
        # Velocity check
        if (user_history.get('total_transactions', 0) > 0 and 
            amount > user_history.get('avg_transaction_size', 0) * 5):
            factors.append("Transaction much larger than user average")
        
        return factors
    
    def _emergency_fallback(self) -> Dict:
        """Emergency fallback при полном отказе анализа"""
        return {
            "risk_score": 0.5,  # Medium risk когда неизвестно
            "risk_level": "medium",
            "is_anomaly": False,
            "anomaly_score": None,
            "ml_confidence": 0.0,
            "risk_factors": ["Risk analysis system error - manual review required"],
            "recommended_action": "manual_review_required",
            "approved": False,
            "blacklist_check": {"is_blacklisted": False, "reason": None}
        }
    
    def retrain_models(self, training_data: List[Dict]) -> Dict[str, any]:
        """Переобучение моделей на новых данных"""
        try:
            if len(training_data) < 10:
                return {"status": "error", "message": "Insufficient training data"}
            
            # Подготовка данных для обучения
            X = []
            y = []
            
            for sample in training_data:
                features = self.extract_features(sample)
                X.append(features[0])
                y.append(1 if sample.get('risk_score', 0) > 0.5 else 0)
            
            X = np.array(X)
            y = np.array(y)
            
            # Переобучение моделей
            X_scaled = self.feature_scaler.fit_transform(X)
            self.isolation_forest.fit(X_scaled)
            self.risk_classifier.fit(X_scaled, y)
            
            # Сохранение обновленных моделей
            self._save_models()
            
            logger.info(f"Models retrained on {len(training_data)} samples")
            return {
                "status": "success", 
                "samples_used": len(training_data),
                "timestamp": datetime.now().isoformat()
            }
            
        except Exception as e:
            logger.error(f"Model retraining failed: {e}")
            return {"status": "error", "message": str(e)}