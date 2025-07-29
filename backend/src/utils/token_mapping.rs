// src/utils/token_mapping.rs - Token symbol to address mapping utilities

use crate::constants::*;

/// Convert token symbol to token address for 1inch API
pub fn symbol_to_token_address(symbol: &str) -> Result<String, String> {
    match symbol {
        "ETH" => Ok(ETHEREUM_NATIVE_TOKEN.to_string()),
        "NEAR" => Ok(NEAR_NATIVE_TOKEN.to_string()),
        "USDT" => Ok(ETHEREUM_USDT_ADDRESS.to_string()),
        "USDC" => Ok(ETHEREUM_USDC_ADDRESS.to_string()),
        "DAI" => Ok(ETHEREUM_DAI_ADDRESS.to_string()),
        "WBTC" => Ok(ETHEREUM_WBTC_ADDRESS.to_string()),
        _ => Err(format!("Unsupported token symbol: {}", symbol)),
    }
}

/// Convert token address to symbol for display purposes
pub fn token_address_to_symbol(address: &str) -> Result<String, String> {
    match address {
        addr if addr == ETHEREUM_NATIVE_TOKEN => Ok("ETH".to_string()),
        addr if addr == NEAR_NATIVE_TOKEN => Ok("NEAR".to_string()),
        addr if addr == ETHEREUM_USDT_ADDRESS => Ok("USDT".to_string()),
        addr if addr == ETHEREUM_USDC_ADDRESS => Ok("USDC".to_string()),
        addr if addr == ETHEREUM_DAI_ADDRESS => Ok("DAI".to_string()),
        addr if addr == ETHEREUM_WBTC_ADDRESS => Ok("WBTC".to_string()),
        _ => Err(format!("Unsupported token address: {}", address)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_to_address_mapping() {
        assert_eq!(symbol_to_token_address("ETH").unwrap(), ETHEREUM_NATIVE_TOKEN);
        assert_eq!(symbol_to_token_address("USDT").unwrap(), ETHEREUM_USDT_ADDRESS);
        assert_eq!(symbol_to_token_address("NEAR").unwrap(), NEAR_NATIVE_TOKEN);
        assert!(symbol_to_token_address("UNKNOWN").is_err());
    }

    #[test]
    fn test_address_to_symbol_mapping() {
        assert_eq!(token_address_to_symbol(ETHEREUM_NATIVE_TOKEN).unwrap(), "ETH");
        assert_eq!(token_address_to_symbol(ETHEREUM_USDT_ADDRESS).unwrap(), "USDT");
        assert_eq!(token_address_to_symbol(NEAR_NATIVE_TOKEN).unwrap(), "NEAR");
        assert!(token_address_to_symbol("0xinvalid").is_err());
    }
}