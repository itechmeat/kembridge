from fastapi import FastAPI, HTTPException, Depends
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
from typing import List, Optional, Dict, Any
import os
import asyncio
import asyncpg
import logging
from datetime import datetime
import numpy as np
from contextlib import asynccontextmanager
import json
from models.simple_risk_analyzer import SimpleRiskAnalyzer
from models.blacklist_checker import BlacklistChecker
from models.risk_analyzer_base import RiskAnalyzerBase
from config import settings

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Database connection management
class DatabaseManager:
    def __init__(self):
        self.database_url = settings.database_url
        self.pool = None
    
    async def create_pool(self):
        """Create connection pool"""
        try:
            self.pool = await asyncpg.create_pool(
                self.database_url,
                min_size=1,
                max_size=10,
                command_timeout=60
            )
            logger.info("Database pool created successfully")
        except Exception as e:
            logger.error(f"Failed to create database pool: {e}")
            raise
    
    @asynccontextmanager
    async def get_connection(self):
        """Get database connection from pool"""
        if not self.pool:
            await self.create_pool()
        async with self.pool.acquire() as connection:
            yield connection
    
    async def close_pool(self):
        """Close connection pool"""
        if self.pool:
            await self.pool.close()
            logger.info("Database pool closed")

# Global database manager
db_manager = DatabaseManager()

# Lifespan management
@asynccontextmanager
async def lifespan(app: FastAPI):
    # Startup
    logger.info("Starting KEMBridge AI Engine...")
    await db_manager.create_pool()
    
    yield
    
    # Shutdown
    logger.info("Shutting down KEMBridge AI Engine...")
    await db_manager.close_pool()

app = FastAPI(
    title="KEMBridge AI Risk Engine",
    description="Quantum-Secured AI Risk Analysis for Cross-Chain Bridge",
    version="0.1.0",
    docs_url="/docs",
    redoc_url="/redoc",
    lifespan=lifespan
)

# CORS middleware for frontend integration
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origins,
    allow_credentials=True,
    allow_methods=["GET", "POST", "PUT", "DELETE", "OPTIONS"],
    allow_headers=["*"],
)

class HealthResponse(BaseModel):
    status: str
    service: str
    version: str
    database_status: Optional[str] = None
    ml_models_status: Optional[str] = None
    blacklist_loaded: Optional[bool] = None
    timestamp: Optional[str] = None

class RiskAnalysisRequest(BaseModel):
    user_id: str = Field(..., description="User ID for risk analysis")
    transaction_id: Optional[str] = Field(None, description="Transaction ID if available")
    amount_in: float = Field(..., gt=0, description="Transaction amount")
    source_chain: str = Field(..., description="Source blockchain")
    destination_chain: str = Field(..., description="Destination blockchain")
    source_token: str = Field(..., description="Source token symbol")
    destination_token: str = Field(..., description="Destination token symbol")
    user_address: Optional[str] = Field(None, description="User wallet address")
    
    class Config:
        schema_extra = {
            "example": {
                "user_id": "123e4567-e89b-12d3-a456-426614174000",
                "transaction_id": "tx_789",
                "amount_in": 5.0,
                "source_chain": "ethereum",
                "destination_chain": "near",
                "source_token": "ETH",
                "destination_token": "NEAR",
                "user_address": "0x742d35Cc6Eba4C34aCe21Db51B0B87a9e1234567"
            }
        }

class RiskAnalysisResponse(BaseModel):
    risk_score: float = Field(..., ge=0, le=1, description="Risk score from 0.0 to 1.0")
    risk_level: str = Field(..., description="Risk level: low, medium, high")
    reasons: List[str] = Field(..., description="List of risk factors identified")
    approved: bool = Field(..., description="Whether transaction is approved")
    ml_confidence: Optional[float] = Field(None, description="ML model confidence score")
    is_anomaly: Optional[bool] = Field(None, description="Whether transaction is anomalous")
    recommended_action: str = Field(..., description="Recommended action")
    analysis_timestamp: str = Field(..., description="Analysis timestamp")
    
class UserRiskProfileResponse(BaseModel):
    user_id: str
    overall_risk_level: str
    transaction_count: int
    avg_risk_score: float
    high_risk_transactions: int
    last_analysis_date: str

class UserHistoryData(BaseModel):
    total_transactions: int = 0
    total_volume: float = 0.0
    avg_transaction_size: float = 0.0
    days_since_first_tx: int = 0
    high_risk_ratio: float = 0.0
    is_new_user: bool = True

@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint with database and ML models status"""
    db_status = "disconnected"
    ml_status = "not_loaded"
    
    try:
        async with db_manager.get_connection() as conn:
            await conn.fetchval("SELECT 1")
            db_status = "connected"
    except Exception as e:
        logger.error(f"Database health check failed: {e}")
    
    try:
        # SimpleRiskAnalyzer is always ready
        ml_status = "simple_analyzer_ready"
    except Exception as e:
        logger.error(f"ML models health check failed: {e}")
    
    overall_status = "healthy"
    if db_status != "connected":
        overall_status = "degraded"
    elif ml_status == "not_loaded":
        overall_status = "partial"
    
    return {
        "status": overall_status,
        "service": "kembridge-ai-engine",
        "version": "0.1.0",
        "database_status": db_status,
        "ml_models_status": ml_status,
        "blacklist_loaded": True,
        "timestamp": datetime.now().isoformat()
    }

@app.options("/health")
async def health_options():
    """OPTIONS endpoint for health check CORS preflight"""
    return {"message": "OK"}

# Dependency to get database connection
async def get_db_connection():
    """Dependency to get database connection"""
    async with db_manager.get_connection() as conn:
        yield conn

# Initialize ML models
risk_analyzer = SimpleRiskAnalyzer()
blacklist_checker = BlacklistChecker()

# Risk Analysis Service
class RiskAnalysisService:
    def __init__(self, analyzer: RiskAnalyzerBase):
        self.analyzer = analyzer

    async def get_user_history(self, conn: asyncpg.Connection, user_id: str) -> UserHistoryData:
        """Get user transaction history for risk analysis"""
        try:
            query = """
            SELECT 
                COUNT(*) as total_transactions,
                COALESCE(SUM(amount_in), 0) as total_volume,
                COALESCE(AVG(amount_in), 0) as avg_transaction_size,
                COALESCE(MAX(created_at), NOW()) as last_transaction,
                COALESCE(MIN(created_at), NOW()) as first_transaction,
                COUNT(CASE WHEN risk_score > 0.5 THEN 1 END) as high_risk_count
            FROM transactions 
            WHERE user_id = $1 
                AND created_at > NOW() - INTERVAL '30 days'
                AND status IN ('completed', 'confirmed')
            """
            
            result = await conn.fetchrow(query, user_id)
            
            if result and result['total_transactions'] > 0:
                first_tx = result['first_transaction']
                days_since_first = (datetime.now() - first_tx.replace(tzinfo=None)).days
                
                return UserHistoryData(
                    total_transactions=result['total_transactions'],
                    total_volume=float(result['total_volume']),
                    avg_transaction_size=float(result['avg_transaction_size']),
                    days_since_first_tx=days_since_first,
                    high_risk_ratio=result['high_risk_count'] / max(result['total_transactions'], 1),
                    is_new_user=False
                )
            else:
                return UserHistoryData(is_new_user=True)
                
        except Exception as e:
            logger.error(f"Error getting user history: {e}")
            return UserHistoryData(is_new_user=True)
    
    def analyze_transaction_risk(self, request: RiskAnalysisRequest, user_history: UserHistoryData) -> RiskAnalysisResponse:
        """ML-powered risk analysis with user history"""
        # Prepare data for ML analysis
        transaction_data = {
            "user_id": request.user_id,
            "transaction_id": request.transaction_id,
            "amount_in": request.amount_in,
            "source_chain": request.source_chain,
            "destination_chain": request.destination_chain,
            "source_token": request.source_token,
            "destination_token": request.destination_token,
            "user_address": request.user_address,
            "user_history": user_history.dict(),
            "hour_of_day": datetime.now().hour,
            "day_of_week": datetime.now().weekday()
        }
        
        # ML analysis using RiskAnalyzer
        analysis_result = self.analyzer.analyze_risk(transaction_data)
        
        return RiskAnalysisResponse(
            risk_score=analysis_result['risk_score'],
            risk_level=analysis_result['risk_level'],
            reasons=analysis_result['risk_factors'],
            approved=analysis_result['approved'],
            ml_confidence=analysis_result.get('ml_confidence', 0.8),
            is_anomaly=analysis_result.get('is_anomaly', False),
            recommended_action=analysis_result['recommended_action'],
            analysis_timestamp=datetime.now().isoformat()
        )

# Create risk service instance
risk_service = RiskAnalysisService(risk_analyzer)

@app.post("/api/risk/analyze", response_model=RiskAnalysisResponse)
async def analyze_transaction_risk(
    request: RiskAnalysisRequest,
    conn: asyncpg.Connection = Depends(get_db_connection)
):
    """Analyze transaction risk with user history integration"""
    try:
        logger.info(f"Analyzing risk for user {request.user_id}, amount: {request.amount_in}")
        
        # Get user history
        user_history = await risk_service.get_user_history(conn, request.user_id)
        
        # Perform risk analysis
        analysis = risk_service.analyze_transaction_risk(request, user_history)
        
        # Log the analysis result
        logger.info(f"Risk analysis completed: score={analysis.risk_score}, level={analysis.risk_level}")
        
        return analysis
        
    except Exception as e:
        logger.error(f"Risk analysis failed: {e}")
        raise HTTPException(status_code=500, detail=f"Risk analysis failed: {str(e)}")

@app.get("/api/risk/profile/{user_id}", response_model=UserRiskProfileResponse)
async def get_user_risk_profile(
    user_id: str,
    conn: asyncpg.Connection = Depends(get_db_connection)
):
    """Get user risk profile based on transaction history"""
    try:
        user_history = await risk_service.get_user_history(conn, user_id)
        
        # Determine overall risk level
        if user_history.is_new_user:
            overall_risk = "medium"  # New users are medium risk
        elif user_history.high_risk_ratio > 0.5:
            overall_risk = "high"
        elif user_history.high_risk_ratio > 0.2:
            overall_risk = "medium"
        else:
            overall_risk = "low"
        
        return UserRiskProfileResponse(
            user_id=user_id,
            overall_risk_level=overall_risk,
            transaction_count=user_history.total_transactions,
            avg_risk_score=user_history.high_risk_ratio,
            high_risk_transactions=int(user_history.total_transactions * user_history.high_risk_ratio),
            last_analysis_date=datetime.now().isoformat()
        )
        
    except Exception as e:
        logger.error(f"Failed to get user profile: {e}")
        raise HTTPException(status_code=500, detail=f"Failed to get user profile: {str(e)}")

@app.post("/api/risk/retrain")
async def retrain_models():
    """Retrain ML models (admin endpoint)."""
    try:
        # In production training data would come from the database
        # Currently this is a stub
        training_data = []  # Placeholder
        
        if len(training_data) == 0:
            return {
                "status": "skipped",
                "message": "No new training data available",
                "timestamp": datetime.now().isoformat()
            }
        
        # SimpleRiskAnalyzer does not support retraining
        return {
            "status": "skipped",
            "message": "Simple risk analyzer does not support retraining",
            "timestamp": datetime.now().isoformat()
        }
        
    except Exception as e:
        logger.error(f"Model retraining failed: {e}")
        raise HTTPException(status_code=500, detail=f"Model retraining failed: {str(e)}")

@app.get("/api/risk/blacklist/stats")
async def get_blacklist_stats():
    """Return basic statistics about the blacklist."""
    try:
        stats = blacklist_checker.get_blacklist_stats()
        return stats
    except Exception as e:
        logger.error(f"Failed to get blacklist stats: {e}")
        raise HTTPException(status_code=500, detail=f"Failed to get blacklist stats: {str(e)}")

@app.post("/api/risk/blacklist/check")
async def check_address_blacklist(address: str, chain: str):
    """Check a specific address against the blacklist."""
    try:
        result = blacklist_checker.check_address(address, chain)
        return {
            "address": address,
            "chain": chain,
            **result,
            "timestamp": datetime.now().isoformat()
        }
    except Exception as e:
        logger.error(f"Blacklist check failed: {e}")
        raise HTTPException(status_code=500, detail=f"Blacklist check failed: {str(e)}")

@app.get("/")
async def root():
    return {"message": "KEMBridge AI Engine is running"}

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=settings.port)
