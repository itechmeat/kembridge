from pydantic_settings import BaseSettings
from typing import List


class Settings(BaseSettings):
    """Application settings loaded from environment variables."""

    database_url: str = "postgresql://postgres:postgres@postgres:5432/kembridge"
    cors_origins: List[str] = [
        "http://localhost:4001",  # Frontend
        "http://localhost:4000",  # Backend
        "http://localhost:4100",  # Frontend dev
        "http://frontend:4001",   # Docker frontend
        "http://backend:4000",    # Docker backend
    ]
    port: int = 4003

    risk_threshold_low: float = 0.3
    risk_threshold_medium: float = 0.6
    risk_threshold_high: float = 0.8

    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"


settings = Settings()