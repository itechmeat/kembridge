# KEMBridge AI Risk Engine

## Overview

The AI Risk Engine is a machine learning-powered risk assessment system integrated into KEMBridge to analyze and evaluate the security and trustworthiness of cross-chain transactions in real-time. It combines behavioral analytics, transaction pattern recognition, and quantum-secure verification to provide intelligent fraud detection and risk mitigation for cross-chain bridge operations.

## Architecture

The AI Risk Engine consists of two main components:

### 1. Python FastAPI Service (`/ai-engine`)
- **Machine Learning Models**: Scikit-learn based models for risk scoring
- **Real-time Analysis**: FastAPI endpoints for instant risk assessment
- **User Profiling**: Behavioral analysis and risk trend tracking
- **Model Training**: Continuous learning from transaction patterns

### 2. Rust Integration Service (`/backend/src/services/risk_*`)
- **HTTP Client**: Rust reqwest-based client for AI Engine communication
- **Bridge Integration**: Risk analysis workflow integrated into bridge operations
- **Threshold Management**: Configurable risk thresholds for automated decisions
- **Fail-safe Mechanisms**: Fail-open/fail-closed strategies for service availability

## How It Works

### Risk Analysis Process

1. **Transaction Initiation**: When a user initiates a cross-chain swap, the system automatically triggers risk analysis
2. **Data Collection**: The system collects transaction metadata:
   - Source and destination chains
   - Transaction amount and token types
   - User historical behavior
   - Time patterns and frequency
   - Wallet address reputation

3. **AI Analysis**: The Python ML service processes the data using:
   - **Anomaly Detection**: Identifies unusual transaction patterns
   - **Behavioral Modeling**: Analyzes user's historical transaction behavior
   - **Network Analysis**: Evaluates wallet address reputation and connections
   - **Risk Scoring**: Generates a numerical risk score (0.0 - 1.0)

4. **Decision Making**: Based on risk thresholds:
   - **Allow** (< 0.6): Transaction proceeds automatically
   - **Manual Review** (0.6 - 0.9): Transaction requires administrator approval
   - **Block** (> 0.9): Transaction is automatically rejected

### Risk Factors Analyzed

- **Transaction Amount**: Large amounts receive higher scrutiny
- **Frequency Patterns**: Rapid consecutive transactions raise flags
- **Cross-chain Behavior**: Unusual chain combinations or patterns
- **Time-based Patterns**: Transactions at unusual times or intervals
- **Address Reputation**: Known malicious or high-risk addresses
- **User History**: Account age, previous transaction success rate
- **Quantum Signature Verification**: Post-quantum cryptographic validation

## Implementation Status

### ✅ Phase 5.1: AI Risk Analysis Module (Completed)
- FastAPI service with scikit-learn ML models
- Risk scoring endpoints (`/analyze`, `/profile/{user_id}`)
- Behavioral analysis and user profiling
- Model training and evaluation pipeline
- Health check and monitoring endpoints

### ✅ Phase 5.2.1: HTTP Client Setup (Completed)
- Rust HTTP client with retry logic and timeout handling
- Request/response models for AI Engine communication
- Configuration management for AI Engine URL and API keys
- Error handling and fallback mechanisms

### ✅ Phase 5.2.2: Bridge Workflow Integration (Completed)
- Risk analysis integrated into bridge service workflow
- Automatic risk assessment before transaction execution
- Risk-based decision making (Allow/Review/Block)
- REST API endpoints for risk management:
  - `GET /api/v1/risk/profile/{user_id}` - User risk profile
  - `GET /api/v1/risk/thresholds` - Current risk thresholds
  - `PUT /api/v1/risk/thresholds` - Update risk thresholds (admin)
  - `GET /api/v1/risk/health` - AI Engine health check
  - `POST /api/v1/risk/test` - Risk analysis testing (dev only)

### ✅ Phase 5.2.3: Risk Thresholds & Auto-blocking (Completed)
- **Configurable Risk Thresholds**: All thresholds configurable via environment variables
- **Automatic Blocking Logic**: Transactions automatically blocked when risk score exceeds threshold
- **Admin Bypass Mechanisms**: Administrators can bypass blocks for legitimate high-risk transactions
- **Comprehensive Audit Logging**: All risk decisions logged with detailed context for compliance
- **Environment Configuration**:
  - `RISK_LOW_THRESHOLD=0.3` - Low risk threshold
  - `RISK_MEDIUM_THRESHOLD=0.6` - Medium risk threshold  
  - `RISK_HIGH_THRESHOLD=0.8` - High risk threshold
  - `RISK_AUTO_BLOCK_THRESHOLD=0.9` - Automatic blocking threshold
  - `RISK_MANUAL_REVIEW_THRESHOLD=0.7` - Manual review threshold
  - `RISK_ADMIN_BYPASS_ENABLED=false` - Admin bypass feature (disabled by default)

### ✅ Phase 5.2.4: Manual Review Workflow (Completed)
- **Manual Review Queue Management**: Comprehensive queue system for transactions requiring manual administrator review
- **Review Assignment and Decision Tracking**: Administrators can assign reviews to themselves and make approval/rejection decisions  
- **Escalation Rules Implementation**: Automatic escalation of reviews based on priority levels and timeout periods
- **Integration with Bridge Workflow**: Automatic addition of high-risk transactions to review queue when risk analysis indicates manual review needed
- **REST API Endpoints**:
  - `GET /api/v1/admin/review/queue` - Get review queue with filtering and pagination
  - `POST /api/v1/admin/review/queue` - Add transaction to review queue
  - `GET /api/v1/admin/review/{review_id}` - Get review details
  - `PUT /api/v1/admin/review/{review_id}/assign` - Assign review to admin
  - `PUT /api/v1/admin/review/{review_id}/decision` - Make review decision (approve/reject)
  - `PUT /api/v1/admin/review/{review_id}/escalate` - Escalate review manually
  - `POST /api/v1/admin/review/check-escalations` - Check for escalations (cron job)
- **Comprehensive Audit Logging**: All review actions logged with detailed context for compliance
- **Priority-based Processing**: Critical, High, Medium, Low priority levels with corresponding escalation timeouts
- **Notification System**: Admin notifications for high-priority reviews and escalations

### ✅ Phase 5.2.5: Risk Scores in Database (Completed)
- **Real-time Risk Score Updates**: Automatic risk score updates in PostgreSQL transactions table during bridge operations
- **Database Schema Integration**: risk_score, risk_factors, ai_analysis_version fields fully integrated with BigDecimal support
- **TransactionService Implementation**: Comprehensive service for risk score management with methods for:
  - `update_risk_score()` - Real-time updates during risk analysis
  - `get_risk_score()` - Retrieve current risk scores
  - `get_transactions_by_risk_range()` - Query transactions by risk score range
  - `get_high_risk_transactions()` - Get transactions requiring review
  - `get_risk_statistics()` - Analytics for risk monitoring
- **Historical Risk Tracking**: risk_score_history table with automatic triggers for audit trail
- **Analytics Optimization**: Specialized indexes, materialized views, and optimized PostgreSQL functions for dashboard performance
- **Bridge Service Integration**: RiskIntegrationService automatically updates database with risk scores from AI analysis
- **Type Safety**: Full BigDecimal integration for precise risk score storage and PostgreSQL compatibility

### ✅ Phase 5.2.6: User Risk Profile Endpoint (Completed)
- **HTTP API Endpoint**: GET `/api/v1/risk/profile/{user_id}` fully implemented with comprehensive OpenAPI documentation
- **Authorization Security**: Users can only access their own profiles unless they have admin privileges
- **Query Parameters**: Optional `include_history_days` parameter for filtering historical data
- **Response Format**: Complete user risk profile with current risk thresholds for client-side decision making
- **AI Engine Integration**: Direct integration with Python FastAPI service for real-time profile retrieval
- **Error Handling**: Comprehensive error handling with proper HTTP status codes and detailed error messages
- **Performance Optimization**: Efficient caching and minimal database queries for optimal response times

### ✅ Phase 5.2.7: User Profile Updates Integration (Completed)
- **Automatic Profile Updates**: Seamless integration of risk profile updates into user management and bridge workflows
- **User Management Integration**: UserService automatically updates risk profiles when users modify their account information
- **Bridge Transaction Integration**: BridgeService automatically updates risk profiles after successful transaction completion
- **Non-blocking Updates**: Risk profile updates run asynchronously without blocking core operations
- **Comprehensive Error Handling**: Graceful handling of profile update failures with detailed logging
- **Bridge Service Enhancement**: Enhanced BridgeService with full risk integration wrapper around kembridge-bridge crate
- **Automatic Triggers**: Risk profile updates triggered by:
  - User profile modifications
  - Successful bridge transaction completion
  - Account changes and wallet updates
- **Transaction Data Format**: Structured JSON data passed to AI Engine with complete transaction context
- **Service Integration**: Full integration with AppState dependency injection for seamless risk profile management

## API Reference

### AI Engine Endpoints (Python FastAPI)

#### POST `/analyze`
Analyzes transaction risk and returns risk score.

```json
{
  "transaction_id": "uuid",
  "user_id": "uuid", 
  "source_chain": "ethereum",
  "destination_chain": "near",
  "amount": 1000.0,
  "source_address": "0x...",
  "destination_address": "user.near",
  "metadata": {}
}
```

#### GET `/profile/{user_id}`
Returns user risk profile and behavioral analysis.

#### GET `/health`
Health check endpoint for monitoring.

### Bridge API Endpoints (Rust Axum)

#### GET `/api/v1/risk/profile/{user_id}`
Retrieves user risk profile with optional history filtering.

**Parameters:**
- `user_id`: UUID of the user
- `include_history_days`: Optional number of days of history to include

**Response:**
```json
{
  "user_risk_profile": {
    "user_id": "uuid",
    "overall_risk_score": 0.35,
    "risk_level": "low",
    "transaction_count": 42,
    "total_volume": 15000.0,
    "risk_trend": "stable",
    "behavioral_flags": []
  },
  "current_thresholds": {
    "low_threshold": 0.3,
    "medium_threshold": 0.6,
    "high_threshold": 0.8,
    "auto_block_threshold": 0.9,
    "manual_review_threshold": 0.6
  }
}
```

#### PUT `/api/v1/risk/thresholds`
Updates risk analysis thresholds (admin only).

#### GET `/api/v1/risk/health`
Checks AI Engine availability and configuration.

### Manual Review API Endpoints (Rust Axum)

#### GET `/api/v1/admin/review/queue`
Retrieves the manual review queue with filtering and pagination options.

**Parameters:**
- `status`: Optional filter by review status (pending, in_review, approved, rejected, escalated, expired)
- `priority`: Optional filter by priority level (low, medium, high, critical)
- `page`: Optional page number (1-based, default: 1)
- `per_page`: Optional items per page (max 100, default: 20)

**Response:**
```json
{
  "success": true,
  "data": {
    "reviews": [
      {
        "review": {
          "id": "uuid",
          "transaction_id": "uuid",
          "user_id": "uuid",
          "risk_score": 0.75,
          "status": "pending",
          "priority": "high",
          "created_at": "2024-01-01T12:00:00Z",
          "expires_at": "2024-01-01T18:00:00Z",
          "review_reason": "High risk score requires manual review"
        },
        "transaction_details": {
          "source_chain": "ethereum",
          "destination_chain": "near",
          "amount_in": 1000.0
        },
        "user_risk_profile": {
          "overall_risk_score": 0.65,
          "risk_level": "medium"
        }
      }
    ],
    "pagination": {
      "current_page": 1,
      "per_page": 20,
      "total_items": 42,
      "total_pages": 3
    },
    "statistics": {
      "total_pending": 15,
      "total_in_review": 5,
      "critical_count": 1
    }
  }
}
```

#### PUT `/api/v1/admin/review/{review_id}/assign`
Assigns a pending review to the authenticated administrator.

#### PUT `/api/v1/admin/review/{review_id}/decision`
Makes a final decision on a review (approve or reject).

**Request Body:**
```json
{
  "status": "approved",
  "reason": "Transaction verified as legitimate",
  "metadata": {}
}
```

#### PUT `/api/v1/admin/review/{review_id}/escalate`
Manually escalates a review to higher priority.

## Configuration

### Environment Variables

**AI Engine (Python):**
```env
MODEL_PATH=/app/models
TRAINING_DATA_PATH=/app/data
LOG_LEVEL=INFO
```

**Backend (Rust):**
```env
AI_ENGINE_URL=http://ai-engine:8000
AI_ENGINE_API_KEY=optional_api_key
AI_ENGINE_TIMEOUT_MS=5000
AI_ENGINE_MAX_RETRIES=3
ENABLE_AI_RISK_ANALYSIS=true
```

### Risk Thresholds

Default risk thresholds can be configured:

- **Low Risk**: 0.0 - 0.3 (Auto-approve)
- **Medium Risk**: 0.3 - 0.6 (Auto-approve with logging)
- **High Risk**: 0.6 - 0.8 (Manual review required)
- **Critical Risk**: 0.8 - 0.9 (Manual review required)
- **Auto-block**: > 0.9 (Automatic rejection)

## Security Considerations

1. **Data Privacy**: User transaction data is analyzed but not stored persistently in the AI engine
2. **Fail-safe Design**: If AI Engine is unavailable, system can fail-open or fail-closed based on configuration
3. **Admin Controls**: Risk thresholds can only be modified by authenticated administrators
4. **Audit Trail**: All risk decisions are logged for compliance and debugging
5. **Model Security**: ML models are validated and regularly updated to prevent adversarial attacks

## Future Enhancements

- **Advanced ML Models**: Integration of neural networks and deep learning models
- **Real-time Learning**: Continuous model updates based on transaction outcomes
- **Cross-chain Intelligence**: Enhanced analysis across multiple blockchain networks
- **Reputation Networks**: Integration with external reputation and blacklist services
- **Regulatory Compliance**: AML/KYC integration and regulatory reporting

## Development and Testing

### Local Development

1. Start the AI Engine:
```bash
cd ai-engine
python -m uvicorn src.main:app --reload --host 0.0.0.0 --port 8000
```

2. Start the backend with AI integration:
```bash
cd backend
ENABLE_AI_RISK_ANALYSIS=true cargo run
```

### Testing Risk Analysis

Use the development endpoint to test risk analysis:

```bash
curl -X POST http://localhost:4000/api/v1/risk/test \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json"
```

This endpoint creates a mock transaction and runs it through the full risk analysis pipeline.

## Monitoring and Observability

- **Health Checks**: Regular monitoring of AI Engine availability
- **Metrics**: Risk score distributions, decision rates, and processing times
- **Logging**: Comprehensive logging of risk decisions and AI Engine interactions
- **Alerts**: Automatic notifications for high-risk transactions and system issues

The AI Risk Engine provides KEMBridge with intelligent, automated risk assessment capabilities while maintaining the security and reliability required for cross-chain financial operations.