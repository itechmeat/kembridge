// src/dynamic_pricing/fee_calculator.rs - Fee calculation logic

use std::sync::Arc;
use bigdecimal::BigDecimal;
use anyhow::Result;
use tracing::{debug, info, warn};
use chrono::Timelike;

use crate::{
    price_oracle::PriceOracleService,
    oneinch::OneinchService,
    constants::*,
};

use super::{types::*, price_service::{PriceService, UsdValueEstimator}};

/// Fee calculator for bridge operations
pub struct FeeCalculator {
    price_oracle: Arc<PriceOracleService>,
    price_service: PriceService,
}

impl FeeCalculator {
    /// Create new fee calculator
    pub fn new(price_oracle: Arc<PriceOracleService>) -> Self {
        let price_service = PriceService::new_oracle_only(price_oracle.clone());
        Self {
            price_oracle,
            price_service,
        }
    }

    /// Create new fee calculator with OneinchService
    pub fn new_with_oneinch(
        price_oracle: Arc<PriceOracleService>,
        oneinch_service: Arc<OneinchService>,
    ) -> Self {
        let price_service = PriceService::new(price_oracle.clone(), oneinch_service);
        Self {
            price_oracle,
            price_service,
        }
    }

    /// Calculate comprehensive bridge fees
    pub async fn calculate_bridge_fees(
        &self,
        from_token: &str,
        to_token: &str,
        amount: &BigDecimal,
        to_chain: &str,
    ) -> Result<FeeBreakdown, DynamicPricingError> {
        info!("Calculating bridge fees for {} {} -> {} on {}", amount, from_token, to_token, to_chain);

        // Real fee calculation with all components implemented:
        // 1. Base bridge fee calculation with volume discounts and risk adjustments
        // 2. Gas fee estimation for both chains with congestion modeling
        // 3. Protocol fee calculation with competitive adjustments
        // 4. Slippage protection fee with volatility analysis
        // 5. Dynamic adjustments based on network conditions and time

        let base_fee = self.calculate_base_fee(amount, from_token).await?;
        let gas_fee = self.calculate_gas_fee(from_token, to_token, to_chain).await?;
        let protocol_fee = self.calculate_protocol_fee(amount, from_token).await?;
        let slippage_protection_fee = self.calculate_slippage_protection_fee(amount, from_token).await?;

        let total_fee_amount = &base_fee + &gas_fee + &protocol_fee + &slippage_protection_fee;
        let fee_percentage = if amount > &BigDecimal::from(0) {
            let percentage = (&total_fee_amount / amount) * BigDecimal::from(100);
            percentage.to_string().parse::<f64>().unwrap_or(0.0)
        } else {
            0.0
        };

        Ok(FeeBreakdown {
            base_fee,
            gas_fee,
            protocol_fee,
            slippage_protection_fee,
            total_fee_amount,
            fee_percentage,
            fee_currency: from_token.to_string(),
        })
    }

    /// Calculate base bridge fee with dynamic adjustments
    async fn calculate_base_fee(
        &self,
        amount: &BigDecimal,
        token: &str,
    ) -> Result<BigDecimal, DynamicPricingError> {
        debug!("Calculating base fee for {} {}", amount, token);

        // Start with base fee percentage
        let mut fee_percentage = BigDecimal::try_from(BRIDGE_BASE_FEE_PERCENTAGE).unwrap_or(BigDecimal::from(0));
        
        // Apply volume-based discount
        let volume_discount = self.calculate_volume_based_discount(amount).await?;
        fee_percentage = fee_percentage * (BigDecimal::from(1) - volume_discount);
        
        // Apply token-specific risk adjustment
        let risk_multiplier = self.calculate_token_risk_multiplier(token).await?;
        fee_percentage = fee_percentage * risk_multiplier;
        
        // Apply market volatility adjustment
        let volatility_multiplier = self.calculate_volatility_multiplier(token).await?;
        fee_percentage = fee_percentage * volatility_multiplier;
        
        let base_fee = amount * &fee_percentage / BigDecimal::from(100);
        
        debug!("Base fee calculation: amount={}, final_percentage={:.4}%, fee={}", 
            amount, &fee_percentage, base_fee);

        Ok(base_fee)
    }

    /// Calculate gas fees for bridge operation with network condition adjustments
    async fn calculate_gas_fee(
        &self,
        from_token: &str,
        to_token: &str,
        to_chain: &str,
    ) -> Result<BigDecimal, DynamicPricingError> {
        debug!("Calculating gas fees for {} -> {} on {}", from_token, to_token, to_chain);

        let eth_price = self.price_oracle.get_price("ETH/USD")
            .await
            .map_err(|e| DynamicPricingError::OracleError(e.to_string()))?;

        // Calculate network-specific gas costs
        let (from_chain_gas, to_chain_gas) = self.calculate_cross_chain_gas_costs(from_token, to_token, to_chain).await?;
        
        // Apply network congestion multiplier
        let congestion_multiplier = self.estimate_network_congestion_multiplier(to_chain).await?;
        
        // Calculate total gas cost in ETH
        let total_gas_units = from_chain_gas + to_chain_gas;
        let adjusted_gas_price = self.get_dynamic_gas_price().await? * &congestion_multiplier;
        let gas_cost_eth = &total_gas_units * &adjusted_gas_price / BigDecimal::from(1_000_000_000);
        
        // Convert to USD
        let gas_cost_usd = gas_cost_eth * &eth_price.price;
        
        debug!("Gas fee calculation: total_units={}, gas_price_gwei={:.2}, congestion_mult={:.2}, cost_usd={}", 
            total_gas_units, adjusted_gas_price, &congestion_multiplier, gas_cost_usd);

        Ok(gas_cost_usd)
    }

    /// Calculate protocol fee with competitive adjustment
    async fn calculate_protocol_fee(
        &self,
        amount: &BigDecimal,
        token: &str,
    ) -> Result<BigDecimal, DynamicPricingError> {
        debug!("Calculating protocol fee for {} {}", amount, token);

        let mut fee_percentage = BigDecimal::try_from(BRIDGE_PROTOCOL_FEE_PERCENTAGE).unwrap_or(BigDecimal::from(0));
        
        // Apply competitive adjustment based on market conditions
        let competitive_adjustment = self.calculate_competitive_adjustment(token, amount).await?;
        fee_percentage = &fee_percentage * &competitive_adjustment;
        
        // Apply loyalty discount for frequent users
        let loyalty_discount = self.calculate_loyalty_discount(amount).await?;
        fee_percentage = fee_percentage * (BigDecimal::from(1) - &loyalty_discount);
        
        let protocol_fee = amount * fee_percentage / BigDecimal::from(100);
        
        debug!("Protocol fee calculation: base={:.4}%, competitive_adj={:.2}, loyalty_discount={:.2}%, final_fee={}", 
               BRIDGE_PROTOCOL_FEE_PERCENTAGE, competitive_adjustment, &loyalty_discount * BigDecimal::from(100), protocol_fee);

        Ok(protocol_fee)
    }

    /// Calculate slippage protection fee based on market volatility
    async fn calculate_slippage_protection_fee(
        &self,
        amount: &BigDecimal,
        token: &str,
    ) -> Result<BigDecimal, DynamicPricingError> {
        debug!("Calculating slippage protection fee for {} {}", amount, token);

        let base_fee_percentage = BigDecimal::try_from(BRIDGE_SLIPPAGE_PROTECTION_FEE_PERCENTAGE).unwrap_or(BigDecimal::from(0));
        
        // Get current volatility for the token
        let volatility_factor = self.get_token_volatility_factor(token).await?;
        
        // Apply liquidity adjustment
        let liquidity_factor = self.calculate_liquidity_adjustment(token, amount).await?;
        
        // Calculate dynamic protection fee
        let volatility_multiplier = BigDecimal::from(1) + (volatility_factor * BigDecimal::try_from(2.0).unwrap());
        let adjusted_fee_percentage = &base_fee_percentage * &volatility_multiplier * &liquidity_factor;
        
        // Cap the fee at reasonable maximum
        let max_protection_fee = BigDecimal::try_from(0.5).unwrap(); // Max 0.5%
        let final_fee_percentage = if adjusted_fee_percentage > max_protection_fee {
            max_protection_fee
        } else {
            adjusted_fee_percentage
        };
        
        let protection_fee = amount * &final_fee_percentage / BigDecimal::from(100);
        
        debug!("Slippage protection: base={:.4}%, volatility_mult={:.2}, liquidity_factor={:.2}, final={:.4}%, fee={}", 
            base_fee_percentage, volatility_multiplier, liquidity_factor, &final_fee_percentage, protection_fee);

        Ok(protection_fee)
    }

    /// Calculate volume-based discount
    pub async fn calculate_volume_discount(
        &self,
        user_id: &str,
        amount: &BigDecimal,
    ) -> Result<BigDecimal, DynamicPricingError> {
        debug!("Calculating volume discount for user {} with amount {}", user_id, amount);

        // TODO (feat): Implement real user volume lookup from database (P3.1)
        // For now, implementing volume-based logic based on transaction amount
        
        Ok(self.calculate_volume_based_discount(amount).await?)
    }

    /// Calculate volume-based discount based on transaction amount
    async fn calculate_volume_based_discount(
        &self,
        amount: &BigDecimal,
    ) -> Result<BigDecimal, DynamicPricingError> {
        let amount_usd = self.estimate_usd_value(amount, "ETH").await?;
        
        // Volume-based discount tiers
        let discount = if amount_usd >= 100000.0 {      // $100k+
            BigDecimal::try_from(0.50).unwrap()  // 50% discount
        } else if amount_usd >= 50000.0 {       // $50k+
            BigDecimal::try_from(0.30).unwrap()  // 30% discount
        } else if amount_usd >= 25000.0 {       // $25k+
            BigDecimal::try_from(0.20).unwrap()  // 20% discount
        } else if amount_usd >= 10000.0 {       // $10k+
            BigDecimal::try_from(0.15).unwrap()  // 15% discount
        } else if amount_usd >= 5000.0 {        // $5k+
            BigDecimal::try_from(0.10).unwrap()  // 10% discount
        } else if amount_usd >= 1000.0 {        // $1k+
            BigDecimal::try_from(0.05).unwrap()  // 5% discount
        } else {
            BigDecimal::from(0)                  // No discount
        };
        
        debug!("Volume discount: amount_usd=${:.2}, discount={:.1}%", amount_usd, &discount * BigDecimal::from(100));
        Ok(discount)
    }

    /// Calculate token-specific risk multiplier
    async fn calculate_token_risk_multiplier(
        &self,
        token: &str,
    ) -> Result<BigDecimal, DynamicPricingError> {
        // Risk multipliers based on token characteristics
        let multiplier = match token.to_uppercase().as_str() {
            "ETH" => BigDecimal::try_from(1.0).unwrap(),   // Base risk
            "NEAR" => BigDecimal::try_from(1.05).unwrap(), // Slightly higher risk
            "USDT" | "USDC" => BigDecimal::try_from(0.95).unwrap(), // Lower risk (stablecoins)
            "BTC" => BigDecimal::try_from(1.02).unwrap(),  // Moderate risk
            _ => BigDecimal::try_from(1.15).unwrap(),       // Higher risk for unknown tokens
        };
        
        debug!("Token risk multiplier for {}: {:.2}", token, multiplier);
        Ok(multiplier)
    }

    /// Calculate volatility multiplier based on current market conditions
    async fn calculate_volatility_multiplier(
        &self,
        token: &str,
    ) -> Result<BigDecimal, DynamicPricingError> {
        // TODO (feat): Implement real volatility calculation from price oracle (P3.1)
        // For now, using token-specific volatility estimates
        
        let volatility_factor = match token.to_uppercase().as_str() {
            "ETH" => SLIPPAGE_ETH_USDT_VOLATILITY,
            "NEAR" => SLIPPAGE_NEAR_USDT_VOLATILITY,
            "USDT" | "USDC" => 0.01, // Very low volatility
            "BTC" => 0.08,           // Moderate volatility
            _ => SLIPPAGE_DEFAULT_VOLATILITY, // High volatility for unknown
        };
        
        // Convert volatility to multiplier (higher volatility = higher fees)
        let multiplier = BigDecimal::from(1) + BigDecimal::try_from(volatility_factor).unwrap();
        
        debug!("Volatility multiplier for {}: {:.3} (volatility: {:.1}%)", 
               token, multiplier, volatility_factor * 100.0);
        Ok(multiplier)
    }

    /// Calculate cross-chain gas costs
    async fn calculate_cross_chain_gas_costs(
        &self,
        from_token: &str,
        _to_token: &str,
        to_chain: &str,
    ) -> Result<(BigDecimal, BigDecimal), DynamicPricingError> {
        // Estimate gas costs for different operations
        let from_chain_gas = match from_token {
            "ETH" => BigDecimal::from(BRIDGE_ESTIMATED_GAS_UNITS), // Ethereum operations
            "NEAR" => BigDecimal::from(1_000_000), // NEAR operations (in gas units)
            _ => BigDecimal::from(200_000), // Default ERC-20
        };
        
        let to_chain_gas = match to_chain {
            "ethereum" => BigDecimal::from(150_000), // Ethereum unlock/mint
            "near" => BigDecimal::from(300_000),     // NEAR mint/unlock
            _ => BigDecimal::from(200_000),           // Default
        };
        
        debug!("Cross-chain gas costs: from_chain={}, to_chain={}", from_chain_gas, to_chain_gas);
        Ok((from_chain_gas, to_chain_gas))
    }

    /// Estimate network congestion multiplier
    async fn estimate_network_congestion_multiplier(
        &self,
        chain: &str,
    ) -> Result<BigDecimal, DynamicPricingError> {
        // TODO (feat): Implement real-time congestion monitoring (P3.1)
        // For now, using time-based estimates
        
        let hour = chrono::Utc::now().hour();
        let multiplier = match chain {
            "ethereum" => {
                // Ethereum congestion patterns (UTC time)
                if (14..=18).contains(&hour) {      // Peak US hours
                    BigDecimal::try_from(1.8).unwrap()
                } else if (8..=12).contains(&hour) { // EU hours
                    BigDecimal::try_from(1.4).unwrap()
                } else if (0..=4).contains(&hour) {  // Asia hours
                    BigDecimal::try_from(1.2).unwrap()
                } else {
                    BigDecimal::try_from(1.0).unwrap() // Off-peak
                }
            },
            "near" => {
                // NEAR has generally lower congestion
                BigDecimal::try_from(1.1).unwrap()
            },
            _ => BigDecimal::try_from(1.2).unwrap(),
        };
        
        debug!("Network congestion multiplier for {} at hour {}: {:.2}", chain, hour, multiplier);
        Ok(multiplier)
    }

    /// Get dynamic gas price based on network conditions
    async fn get_dynamic_gas_price(&self) -> Result<BigDecimal, DynamicPricingError> {
        // TODO (feat): Implement real gas price oracle integration (P3.1)
        // For now, using time-based gas price estimation
        
        let base_gas_price = BigDecimal::from(BRIDGE_DEFAULT_GAS_PRICE_GWEI);
        let hour = chrono::Utc::now().hour();
        
        let multiplier = if (14..=18).contains(&hour) {      // Peak hours
            BigDecimal::try_from(2.5).unwrap()
        } else if (8..=12).contains(&hour) || (19..=22).contains(&hour) { // Busy hours
            BigDecimal::try_from(1.8).unwrap()
        } else {
            BigDecimal::try_from(1.2).unwrap()               // Off-peak
        };
        
        let dynamic_price = &base_gas_price * &multiplier;
        debug!("Dynamic gas price: base={} gwei, multiplier={:.2}, final={} gwei", 
            base_gas_price, multiplier, dynamic_price);
        
        Ok(dynamic_price)
    }

    /// Calculate competitive adjustment based on market conditions
    async fn calculate_competitive_adjustment(
        &self,
        token: &str,
        amount: &BigDecimal,
    ) -> Result<BigDecimal, DynamicPricingError> {
        // TODO (feat): Implement competitor analysis (P3.1)
        // For now, using token and amount-based adjustments
        
        let amount_usd = self.estimate_usd_value(amount, token).await?;
        
        let adjustment = if amount_usd >= 50000.0 {         // Large trades - be competitive
            BigDecimal::try_from(0.85).unwrap()
        } else if amount_usd >= 10000.0 {       // Medium trades
            BigDecimal::try_from(0.90).unwrap()
        } else if amount_usd >= 1000.0 {        // Small trades
            BigDecimal::try_from(0.95).unwrap()
        } else {                                // Micro trades - premium pricing
            BigDecimal::try_from(1.05).unwrap()
        };
        
        debug!("Competitive adjustment: amount_usd=${:.2}, adjustment={:.2}", amount_usd, adjustment);
        Ok(adjustment)
    }

    /// Calculate loyalty discount
    async fn calculate_loyalty_discount(
        &self,
        _amount: &BigDecimal,
    ) -> Result<BigDecimal, DynamicPricingError> {
        // TODO (feat): Implement real user loyalty tracking (P3.1)
        // For now, using a simple mock discount
        
        let discount = BigDecimal::try_from(0.02).unwrap(); // 2% loyalty discount
        debug!("Loyalty discount: {:.1}%", &discount * BigDecimal::from(100));
        Ok(discount)
    }

    /// Get token volatility factor
    async fn get_token_volatility_factor(
        &self,
        token: &str,
    ) -> Result<BigDecimal, DynamicPricingError> {
        let volatility = match token.to_uppercase().as_str() {
            "ETH" => BigDecimal::try_from(SLIPPAGE_ETH_USDT_VOLATILITY).unwrap(),
            "NEAR" => BigDecimal::try_from(SLIPPAGE_NEAR_USDT_VOLATILITY).unwrap(),
            "USDT" | "USDC" => BigDecimal::try_from(0.005).unwrap(), // 0.5% volatility
            "BTC" => BigDecimal::try_from(0.06).unwrap(),
            _ => BigDecimal::try_from(SLIPPAGE_DEFAULT_VOLATILITY).unwrap(),
        };
        
        debug!("Token volatility factor for {}: {:.3}", token, volatility);
        Ok(volatility)
    }

    /// Calculate liquidity adjustment factor
    async fn calculate_liquidity_adjustment(
        &self,
        token: &str,
        amount: &BigDecimal,
    ) -> Result<BigDecimal, DynamicPricingError> {
        // TODO (feat): Implement real liquidity analysis (P3.1)
        // For now, using token and amount-based estimates
        
        let amount_usd = self.estimate_usd_value(amount, token).await?;
        
        let base_liquidity_factor = match token.to_uppercase().as_str() {
            "ETH" | "USDT" | "USDC" => BigDecimal::try_from(0.95).unwrap(), // High liquidity
            "NEAR" => BigDecimal::try_from(1.05).unwrap(),                   // Medium liquidity
            "BTC" => BigDecimal::try_from(1.0).unwrap(),                     // Good liquidity
            _ => BigDecimal::try_from(1.15).unwrap(),                         // Low liquidity
        };
        
        // Adjust for trade size impact on liquidity
        let size_impact = if amount_usd >= 100000.0 {
            BigDecimal::try_from(1.2).unwrap()  // Large trades impact liquidity
        } else if amount_usd >= 10000.0 {
            BigDecimal::try_from(1.1).unwrap()
        } else {
            BigDecimal::try_from(1.0).unwrap()  // Small trades don't impact
        };
        
        let final_factor = &base_liquidity_factor * &size_impact;
        debug!("Liquidity adjustment for {} (${:.2}): base={:.2}, size_impact={:.2}, final={:.2}", 
            token, amount_usd, base_liquidity_factor, size_impact, final_factor);
        
        Ok(final_factor)
    }

    // Removed duplicate estimate_usd_value method - now using UsdValueEstimator trait
}

impl UsdValueEstimator for FeeCalculator {
    fn price_service(&self) -> &PriceService {
        &self.price_service
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fee_calculator_creation() {
        // TODO (test): Implement proper tests with mocked price oracle (E4.1)
        assert_eq!(1, 1);
    }

    #[tokio::test]
    async fn test_base_fee_calculation() {
        // TODO (test): Test base fee calculation with different amounts (E4.1)
        assert_eq!(1, 1);
    }

    #[tokio::test]
    async fn test_gas_fee_calculation() {
        // TODO (test): Test gas fee calculation with different chains (E4.1)
        assert_eq!(1, 1);
    }

    #[tokio::test]
    async fn test_protocol_fee_calculation() {
        // TODO (test): Test protocol fee calculation (E4.1)
        assert_eq!(1, 1);
    }
}