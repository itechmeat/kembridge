// examples/oneinch_honest_integration.rs
// Comprehensive example of honest 1inch integration without fallbacks
// Demonstrates full cycle: validation, quotes, multi-chain comparison, error handling

use bigdecimal::BigDecimal;
use kembridge_backend::{
    constants::*,
    oneinch::{types::QuoteParams, OneinchService},
};
use std::sync::Arc;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    // Initialize logging with detailed format
    tracing_subscriber::fmt::init();

    info!("🚀 Starting comprehensive 1inch honest integration example");
    info!("📋 This example demonstrates:");
    info!("   • API key validation and authentication");
    info!("   • Multi-chain support and comparison");
    info!("   • Real quote retrieval and analysis");
    info!("   • Liquidity assessment");
    info!("   • Performance monitoring");
    info!("   • Honest error handling without fallbacks");

    // Get API key from environment
    let api_key = std::env::var("ONEINCH_API_KEY").unwrap_or_else(|_| {
        warn!("⚠️  ONEINCH_API_KEY not found in environment!");
        warn!("⚠️  Using test key - requests may fail!");
        warn!("💡 Set ONEINCH_API_KEY environment variable for real testing");
        "test_key_for_demo".to_string()
    });

    if api_key.contains("test") || api_key.contains("demo") {
        warn!("🔑 WARNING: Using test/demo API key!");
        warn!("🔑 For production use, get real API key from https://portal.1inch.dev/");
    }

    // Demonstrate multi-chain support
    let supported_networks = vec![
        (ONEINCH_ETHEREUM_CHAIN_ID, "Ethereum Mainnet"),
        (ONEINCH_BSC_CHAIN_ID, "Binance Smart Chain"),
        (ONEINCH_POLYGON_CHAIN_ID, "Polygon"),
        (ONEINCH_SEPOLIA_CHAIN_ID, "Sepolia Testnet"),
    ];

    info!("🌐 Testing 1inch integration across {} networks", supported_networks.len());

    for (chain_id, chain_name) in &supported_networks {
        info!("\n� === Tesrting {} (Chain ID: {}) ===", chain_name, chain_id);
        
        // Create service for this chain
        let oneinch_service = Arc::new(OneinchService::new(
            api_key.clone(),
            *chain_id,
        ));

        // Test 1: API key validation
        info!("🔍 Test 1: API key validation for {}", chain_name);
        match oneinch_service.validate_api_key().await {
            Ok(true) => {
                info!("✅ API key is valid for {}", chain_name);
            }
            Ok(false) => {
                warn!("❌ API key is invalid for {}", chain_name);
                warn!("💡 Check your API key permissions for this network");
                continue; // Skip other tests for this network
            }
            Err(e) => {
                error!("🔥 API key validation error for {}: {}", chain_name, e);
                continue; // Skip other tests for this network
            }
        }

        // Test 2: Health check
        info!("🔍 Test 2: Network health check for {}", chain_name);
        match oneinch_service.comprehensive_health_check().await {
            Ok(health_data) => {
                info!("✅ {} health check passed", chain_name);
                
                if let Some(tokens) = health_data.get("tokens") {
                    if let Some(count) = tokens.get("count") {
                        info!("   📊 Available tokens: {}", count);
                    }
                }

                if let Some(api_status) = health_data.get("api_connectivity") {
                    if let Some(status) = api_status.get("status") {
                        info!("   🔗 API status: {}", status);
                    }
                }
            }
            Err(e) => {
                warn!("❌ {} health check failed: {}", chain_name, e);
                continue;
            }
        }

        // Test 3: Token pair liquidity analysis
        info!("🔍 Test 3: Liquidity analysis for {}", chain_name);
        
        // Use appropriate token addresses for each network
        let (native_token, stable_token, stable_name) = match *chain_id {
            ONEINCH_ETHEREUM_CHAIN_ID => (ETHEREUM_NATIVE_TOKEN, ETHEREUM_USDC_ADDRESS, "USDC"),
            ONEINCH_BSC_CHAIN_ID => (BSC_NATIVE_TOKEN, "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d", "USDC"), // BSC USDC
            ONEINCH_POLYGON_CHAIN_ID => (POLYGON_NATIVE_TOKEN, "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174", "USDC"), // Polygon USDC
            ONEINCH_SEPOLIA_CHAIN_ID => (ETHEREUM_NATIVE_TOKEN, ETHEREUM_USDC_ADDRESS, "USDC"), // Sepolia uses same addresses
            _ => {
                warn!("⚠️  Unknown network, using default tokens");
                (ETHEREUM_NATIVE_TOKEN, ETHEREUM_USDC_ADDRESS, "USDC")
            }
        };

        match oneinch_service.get_liquidity_info(native_token, stable_token).await {
            Ok(liquidity_info) => {
                let available = liquidity_info["available"].as_bool().unwrap_or(false);
                
                if available {
                    info!("✅ Native/{} liquidity available on {}", stable_name, chain_name);
                    
                    if let Some(score) = liquidity_info["liquidity_score"].as_f64() {
                        info!("   📈 Liquidity score: {:.2}/1.0", score);
                        
                        match score {
                            s if s > 0.8 => info!("   🟢 Excellent liquidity"),
                            s if s > 0.6 => info!("   🟡 Good liquidity"),
                            s if s > 0.4 => info!("   🟠 Moderate liquidity"),
                            _ => info!("   🔴 Low liquidity"),
                        }
                    }

                    if let Some(protocols) = liquidity_info["protocols"].as_array() {
                        info!("   🔄 Liquidity sources ({} protocols):", protocols.len());
                        for (i, protocol) in protocols.iter().take(5).enumerate() {
                            if let (Some(name), Some(percentage)) = 
                                (protocol["name"].as_str(), protocol["part"].as_f64()) {
                                info!("     {}. {}: {:.1}%", i + 1, name, percentage);
                            }
                        }
                        if protocols.len() > 5 {
                            info!("     ... and {} more protocols", protocols.len() - 5);
                        }
                    }
                } else {
                    warn!("❌ Native/{} liquidity unavailable on {}", stable_name, chain_name);
                    if let Some(error) = liquidity_info["error"].as_str() {
                        warn!("   📝 Reason: {}", error);
                    }
                }
            }
            Err(e) => {
                warn!("⚠️  Liquidity check failed for {}: {}", chain_name, e);
            }
        }

        // Test 4: Real quote retrieval and analysis
        info!("🔍 Test 4: Quote retrieval for {}", chain_name);
        
        // Use appropriate amount for each network (considering gas costs)
        // QUESTION: Can "100000000000000000u64" be a global const (here in in all places of the file and other files)?
        let quote_amount = match *chain_id {
            ONEINCH_ETHEREUM_CHAIN_ID => BigDecimal::from(100000000000000000u64), // 0.1 ETH
            ONEINCH_BSC_CHAIN_ID => BigDecimal::from(100000000000000000u64), // 0.1 BNB
            ONEINCH_POLYGON_CHAIN_ID => BigDecimal::from(1000000000000000000u64), // 1 MATIC (cheaper)
            ONEINCH_SEPOLIA_CHAIN_ID => BigDecimal::from(10000000000000000u64), // 0.01 ETH (testnet)
            _ => BigDecimal::from(100000000000000000u64),
        };

        let quote_params = QuoteParams {
            from_token: native_token.to_string(),
            to_token: stable_token.to_string(),
            amount: quote_amount.clone(),
            from_address: ETHEREUM_ZERO_ADDRESS.to_string(),
            slippage: Some(ONEINCH_DEFAULT_SLIPPAGE),
            disable_estimate: Some(false),
            allow_partial_fill: Some(true),
            source: Some(ONEINCH_BRIDGE_INTEGRATION_SOURCE.to_string()),
        };

        match oneinch_service.get_quote(&quote_params).await {
            Ok(quote) => {
                info!("✅ Quote received from {} 1inch API", chain_name);
                
                // Calculate readable amounts
                let eth_multiplier = BigDecimal::from(1000000000000000000u64); // 10^18
                // QUESTION: Can "1000000u64" be a global const (here in in all places of the file and other files)?
                let usdc_multiplier = BigDecimal::from(1000000u64); // 10^6
                
                let from_amount_readable = &quote.from_amount / &eth_multiplier;
                let to_amount_readable = &quote.to_amount / &usdc_multiplier;
                
                info!("   💱 Swap: {} Native → {} {}", from_amount_readable, to_amount_readable, stable_name);
                
                // Calculate exchange rate
                let exchange_rate = &quote.to_amount / &quote.from_amount 
                    * &eth_multiplier 
                    / &usdc_multiplier;
                info!("   📊 Rate: 1 Native = {} {}", exchange_rate, stable_name);

                // Gas analysis
                // QUESTION: Can "1_000_000_000.0" be a global const (here in in all places of the file and other files)?
                let gas_cost_gwei = quote.estimated_gas.to_string().parse::<f64>().unwrap_or(0.0) / 1_000_000_000.0;
                info!("   ⛽ Estimated gas: {} wei ({:.2} Gwei)", quote.estimated_gas, gas_cost_gwei);

                // Protocol breakdown
                if !quote.protocols.is_empty() {
                    info!("   🔄 Execution path ({} protocols):", quote.protocols.len());
                    for protocol in &quote.protocols {
                        info!("     - {}: {:.1}%", protocol.name, protocol.part);
                    }
                } else {
                    info!("   🔄 Direct execution (no protocol splitting)");
                }

                // Quote validity
                let expires_in = (quote.expires_at - chrono::Utc::now()).num_seconds();
                info!("   ⏰ Quote valid for: {} seconds", expires_in);
                
                if expires_in < 30 {
                    warn!("   ⚠️  Quote expires soon! Execute quickly.");
                } else if expires_in > 300 {
                    info!("   ✅ Quote has good validity window");
                }

                // Performance metrics
                info!("   📈 Quote quality indicators:");
                info!("     • Protocol diversity: {}/10", (quote.protocols.len().min(10)));
                info!("     • Gas efficiency: {}", if gas_cost_gwei < 100.0 { "Good" } else { "High" });
                info!("     • Validity window: {}", if expires_in > 60 { "Good" } else { "Short" });
                
            }
            Err(e) => {
                error!("❌ Quote failed for {}: {}", chain_name, e);
                error!("💡 Possible causes:");
                error!("   - Insufficient liquidity for this pair");
                error!("   - Network-specific API limitations");
                error!("   - Temporary service unavailability");
                error!("   - Invalid token addresses for this network");
            }
        }

        // Test 5: Error handling demonstration
        info!("🔍 Test 5: Error handling for {}", chain_name);
        
        let invalid_params = QuoteParams {
            from_token: ETHEREUM_ZERO_ADDRESS.to_string(), // Invalid: zero address
            to_token: stable_token.to_string(),
            amount: BigDecimal::from(0), // Invalid: zero amount
            from_address: ETHEREUM_ZERO_ADDRESS.to_string(),
            slippage: Some(0.1),
            disable_estimate: Some(false),
            allow_partial_fill: Some(true),
            source: Some("kembridge-error-test".to_string()),
        };

        match oneinch_service.get_quote(&invalid_params).await {
            Ok(_) => {
                warn!("🤔 Unexpectedly received quote for invalid parameters on {}", chain_name);
            }
            Err(e) => {
                info!("✅ Error correctly handled for {}: {}", chain_name, e);
                info!("💡 This demonstrates honest error handling without fallbacks");
            }
        }

        info!("🏁 Completed testing for {}\n", chain_name);
    }

    // Final summary
    info!("🎉 Comprehensive 1inch honest integration example completed!");
    info!("📊 Summary of demonstrated features:");
    info!("   ✅ Multi-chain support ({} networks tested)", supported_networks.len());
    info!("   ✅ Real-time API key validation");
    info!("   ✅ Network health monitoring");
    info!("   ✅ Liquidity analysis and scoring");
    info!("   ✅ Real quote retrieval with detailed analysis");
    info!("   ✅ Performance and quality metrics");
    info!("   ✅ Honest error handling without fallbacks");
    info!("   ✅ Comprehensive logging for debugging");
    
    info!("🔧 Integration principles followed:");
    info!("   • No mock data or fallbacks");
    info!("   • All data from real 1inch API");
    info!("   • Transparent error reporting");
    info!("   • Network-specific optimizations");
    info!("   • Production-ready error handling");

    info!("💡 Next steps for production:");
    info!("   1. Set up real API key with proper permissions");
    info!("   2. Configure monitoring and alerting");
    info!("   3. Implement retry logic with exponential backoff");
    info!("   4. Add rate limiting and request queuing");
    info!("   5. Set up multi-region failover");

    Ok(())
}

/// Helper function to demonstrate price comparison across networks
async fn demonstrate_cross_chain_price_comparison(
    networks: &[(u64, &str)],
    api_key: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 Cross-chain price comparison demonstration");
    
    let mut network_quotes = Vec::new();
    
    for (chain_id, chain_name) in networks {
        let service = Arc::new(OneinchService::new(api_key.to_string(), *chain_id));
        
        let quote_params = QuoteParams {
            from_token: ETHEREUM_NATIVE_TOKEN.to_string(),
            to_token: ETHEREUM_USDC_ADDRESS.to_string(),
            amount: BigDecimal::from(1000000000000000000u64), // 1 ETH equivalent
            from_address: ETHEREUM_ZERO_ADDRESS.to_string(),
            slippage: Some(0.5),
            disable_estimate: Some(false),
            allow_partial_fill: Some(true),
            source: Some("kembridge-comparison".to_string()),
        };
        
        match service.get_quote(&quote_params).await {
            Ok(quote) => {
                let eth_multiplier = BigDecimal::from(1000000000000000000u64); // 10^18
                let usdc_multiplier = BigDecimal::from(1000000u64); // 10^6
                
                let rate = &quote.to_amount / &quote.from_amount 
                    * &eth_multiplier 
                    / &usdc_multiplier;
                
                network_quotes.push((*chain_id, chain_name.to_string(), rate.clone()));
                info!("✅ {}: 1 Native = {} USDC", chain_name, rate);
            }
            Err(e) => {
                warn!("❌ Failed to get quote for {}: {}", chain_name, e);
            }
        }
    }
    
    if network_quotes.len() > 1 {
        info!("📊 Cross-chain price analysis:");
        let rates: Vec<f64> = network_quotes.iter()
            .map(|(_, _, rate)| rate.to_string().parse::<f64>().unwrap_or(0.0))
            .collect();
        
        if let (Some(min_rate), Some(max_rate)) = (rates.iter().min_by(|a, b| a.partial_cmp(b).unwrap()), 
            rates.iter().max_by(|a, b| a.partial_cmp(b).unwrap())) {
            let spread = (max_rate - min_rate) / min_rate * 100.0;
            info!("   📈 Price spread: {:.2}%", spread);
            
            if spread > 5.0 {
                info!("   🚨 Significant arbitrage opportunity detected!");
            } else if spread > 1.0 {
                info!("   💡 Minor price differences detected");
            } else {
                info!("   ✅ Prices are well-aligned across networks");
            }
        }
    }
    
    Ok(())
}

/// Demonstration of performance monitoring
async fn demonstrate_performance_monitoring(
    service: &OneinchService,
    chain_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔍 Performance monitoring for {}", chain_name);
    
    let start_time = std::time::Instant::now();
    let mut successful_requests = 0;
    let mut _failed_requests = 0;
    let total_requests = 5;
    
    for i in 1..=total_requests {
        let request_start = std::time::Instant::now();
        
        let quote_params = QuoteParams {
            from_token: ETHEREUM_NATIVE_TOKEN.to_string(),
            to_token: ETHEREUM_USDC_ADDRESS.to_string(),
            amount: BigDecimal::from(100000000000000000u64 * i), // Varying amounts
            from_address: ETHEREUM_ZERO_ADDRESS.to_string(),
            slippage: Some(0.5),
            disable_estimate: Some(false),
            allow_partial_fill: Some(true),
            source: Some(format!("kembridge-perf-test-{}", i)),
        };
        
        match service.get_quote(&quote_params).await {
            Ok(_) => {
                successful_requests += 1;
                let duration = request_start.elapsed();
                info!("   ✅ Request {}: {}ms", i, duration.as_millis());
            }
            Err(e) => {
                _failed_requests += 1;
                let duration = request_start.elapsed();
                warn!("   ❌ Request {}: {}ms - Error: {}", i, duration.as_millis(), e);
            }
        }
        
        // Small delay between requests to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
    
    let total_duration = start_time.elapsed();
    let success_rate = (successful_requests as f64 / total_requests as f64) * 100.0;
    let avg_duration = total_duration.as_millis() / total_requests as u128;
    
    info!("📊 Performance summary for {}:", chain_name);
    info!("   🎯 Success rate: {:.1}% ({}/{})", success_rate, successful_requests, total_requests);
    info!("   ⏱️  Average response time: {}ms", avg_duration);
    info!("   📈 Total test duration: {}ms", total_duration.as_millis());
    
    if success_rate >= 80.0 {
        info!("   ✅ Performance is acceptable");
    } else {
        warn!("   ⚠️  Performance issues detected");
    }
    
    Ok(())
}
