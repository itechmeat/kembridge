#!/bin/bash

# Test script for Rate Limiting functionality
# Check that the implemented rate limiting system works

set -e

API_BASE="http://localhost:4000"
ADMIN_TOKEN="your-admin-token-here"  # In real testing, a valid JWT token is needed

echo "🧪 Testing Rate Limiting system"
echo "======================================"

# Function to check API response
check_api() {
    local url=$1
    local expected_status=$2
    local description=$3
    
    echo "📡 Testing: $description"
    echo "   URL: $url"
    
    response=$(curl -s -w "HTTPSTATUS:%{http_code}" -X GET "$url" \
        -H "Authorization: Bearer $ADMIN_TOKEN" \
        -H "Content-Type: application/json")
    
    status=$(echo $response | grep -o "HTTPSTATUS:[0-9]*" | cut -d: -f2)
    body=$(echo $response | sed 's/HTTPSTATUS:[0-9]*$//')
    
    if [ "$status" -eq "$expected_status" ]; then
        echo "   ✅ Status: $status (expected)"
        if [ "$expected_status" -eq 200 ]; then
            echo "   📊 Response: $(echo $body | jq . 2>/dev/null || echo $body)"
        fi
    else
        echo "   ❌ Status: $status (expected $expected_status)"
        echo "   📄 Body: $body"
    fi
    echo ""
}

# Test 1: Check health endpoint (should work)
echo "1️⃣ Basic API check"
check_api "$API_BASE/health" 200 "Health check (without rate limiting)"

# Test 2: Check rate limiting dashboard (requires admin authentication)
echo "2️⃣ Rate Limiting monitoring endpoints"
check_api "$API_BASE/api/v1/monitoring/rate-limits" 401 "Rate limiting dashboard (without token - expected 401)"

# Test 3: Check endpoint statistics
check_api "$API_BASE/api/v1/monitoring/rate-limits/endpoints/health" 401 "Endpoint stats (without token - expected 401)"

# Test 4: Check top violators
check_api "$API_BASE/api/v1/monitoring/rate-limits/top-violators" 401 "Top violators (without token - expected 401)"

# Test 5: Check real-time metrics
check_api "$API_BASE/api/v1/monitoring/rate-limits/real-time" 401 "Real-time metrics (without token - expected 401)"

# Test 6: Check alerts
check_api "$API_BASE/api/v1/monitoring/rate-limits/alerts" 401 "Active alerts (without token - expected 401)"

# Test 7: Check Swagger UI documentation
echo "3️⃣ OpenAPI documentation"
check_api "$API_BASE/docs" 200 "Swagger UI (should be available)"

# Test 8: Check OpenAPI JSON
check_api "$API_BASE/api-docs/openapi.json" 200 "OpenAPI JSON schema"

echo "🎯 Testing completed!"
echo ""
echo "📝 For full rate limiting testing, you need:"
echo "   1. Running backend server (cargo run --bin kembridge-backend)"
echo "   2. Valid admin JWT token"
echo "   3. Configured Redis and PostgreSQL"
echo ""
echo "📚 Available endpoints for rate limiting monitoring:"
echo "   GET /api/v1/monitoring/rate-limits              - Dashboard"
echo "   GET /api/v1/monitoring/rate-limits/endpoints/*  - Stats by endpoint"
echo "   GET /api/v1/monitoring/rate-limits/top-violators - Top violators"
echo "   GET /api/v1/monitoring/rate-limits/real-time     - Real-time metrics"
echo "   GET /api/v1/monitoring/rate-limits/alerts        - Active alerts"
echo ""
echo "🔗 Swagger UI: http://localhost:4000/docs"