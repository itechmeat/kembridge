/**
 * Quote Service
 * Advanced quote management and price calculations
 */

import { SwapQuote, SwapQuoteRequest } from "../api/bridgeService";

export interface QuoteComparison {
  bestPrice: boolean;
  priceImpactLevel: "low" | "medium" | "high";
  recommendationScore: number;
  warnings: string[];
}

class QuoteService {
  /**
   * Compares quote with market conditions
   */
  analyzeQuote(quote: SwapQuote): QuoteComparison {
    const priceImpact = quote.price_impact;
    let priceImpactLevel: "low" | "medium" | "high";
    let recommendationScore = 100;
    const warnings: string[] = [];

    // Analyze price impact
    if (priceImpact < 1) {
      priceImpactLevel = "low";
    } else if (priceImpact < 5) {
      priceImpactLevel = "medium";
      recommendationScore -= 20;
      warnings.push("Moderate price impact detected");
    } else {
      priceImpactLevel = "high";
      recommendationScore -= 50;
      warnings.push("High price impact - consider smaller amount");
    }

    // Check fees
    const totalFeeNum = parseFloat(quote.estimated_fees.total_fee);
    if (totalFeeNum > 0.01) {
      // 0.01 ETH threshold
      recommendationScore -= 10;
      warnings.push("High transaction fees");
    }

    // Check expiration
    const expiresAt = new Date(quote.expires_at);
    const now = new Date();
    const timeUntilExpiry = expiresAt.getTime() - now.getTime();

    if (timeUntilExpiry < 5 * 60 * 1000) {
      // 5 minutes
      recommendationScore -= 15;
      warnings.push("Quote expires soon");
    }

    return {
      bestPrice: recommendationScore > 80,
      priceImpactLevel,
      recommendationScore: Math.max(0, recommendationScore),
      warnings,
    };
  }

  /**
   * Validates quote parameters before request
   */
  validateQuoteParams(params: SwapQuoteRequest): {
    valid: boolean;
    errors: string[];
  } {
    const errors: string[] = [];

    if (!params.from_token) {
      errors.push("From token is required");
    }

    if (!params.to_token) {
      errors.push("To token is required");
    }

    if (
      params.from_token === params.to_token &&
      params.from_chain === params.to_chain
    ) {
      errors.push("Cannot swap same token on same chain");
    }

    if (!params.amount || parseFloat(params.amount) <= 0) {
      errors.push("Valid amount is required");
    }

    if (params.slippage && (params.slippage < 0.1 || params.slippage > 50)) {
      errors.push("Slippage must be between 0.1% and 50%");
    }

    return {
      valid: errors.length === 0,
      errors,
    };
  }

  /**
   * Calculates estimated USD value for quote
   */
  async calculateUsdValue(
    quote: SwapQuote,
    tokenPriceUsd: number
  ): Promise<number> {
    // TODO: Integrate with price oracle service
    const toAmountNum = parseFloat(quote.to_amount);
    return toAmountNum * tokenPriceUsd;
  }

  /**
   * Formats quote for display
   */
  formatQuote(quote: SwapQuote): {
    displayRate: string;
    displayFees: string;
    displayTime: string;
    riskLevel: string;
  } {
    const rate = quote.exchange_rate;
    const displayRate =
      rate > 1 ? `1 = ${rate.toFixed(4)}` : `${(1 / rate).toFixed(4)} = 1`;

    const totalFeeEth = parseFloat(quote.estimated_fees.total_fee) / 1e18;
    const displayFees =
      totalFeeEth < 0.001 ? "<0.001 ETH" : `${totalFeeEth.toFixed(4)} ETH`;

    const minutes = quote.estimated_time_minutes;
    const displayTime =
      minutes < 60
        ? `${minutes} min`
        : `${Math.ceil(minutes / 60)}h ${minutes % 60}min`;

    let riskLevel = "Low";
    if (quote.route_info.risk_score > 50) riskLevel = "Medium";
    if (quote.route_info.risk_score > 80) riskLevel = "High";

    return {
      displayRate,
      displayFees,
      displayTime,
      riskLevel,
    };
  }

  /**
   * Checks if quote is still valid
   */
  isQuoteValid(quote: SwapQuote): boolean {
    const expiresAt = new Date(quote.expires_at);
    return expiresAt.getTime() > Date.now();
  }

  /**
   * Gets time until quote expires
   */
  getTimeUntilExpiry(quote: SwapQuote): number {
    const expiresAt = new Date(quote.expires_at);
    return Math.max(0, expiresAt.getTime() - Date.now());
  }
}

export const quoteService = new QuoteService();
export default quoteService;
