use steel::*;

#[derive(Debug, Copy, Clone)]
pub struct ParsedExponentialCurve {
    pub a: f64, // Overall scaling factor (e.g., 11400.2301)
    pub b: f64, // Multiplier outside the exponential (e.g., 0.00000087717527)
    pub c: f64, // Rate inside the exponent (e.g., 0.00000087717527)
}

impl ParsedExponentialCurve {
    /// Calculate price at a given supply
    pub fn price(&self, current_supply: f64) -> f64 {
        // R'(S) = a * b * e^(c * s)
        self.a * self.b * (self.c * current_supply).exp()
    }

    /// Calculate total cost to buy `num_tokens` starting at `current_supply` 
    /// (tokens_to_usdc)
    pub fn tokens_to_value(&self, current_supply: f64, tokens: f64) -> f64 {
        let new_supply = current_supply + tokens;
        // Integral of price function: 
        // R(S) = ∫(a * b * e^(c * s)) ds = (a * b / c) * e^(c * s)
        // R(S) = (a * b / c) * (e^(c * S) - e^(c * S0))
        (self.a * self.b / self.c) * 
            ((self.c * new_supply).exp() - (self.c * current_supply).exp())
    }

    /// Calculate number of tokens received for a price `value` starting at `current_supply`
    /// (usdc_to_tokens)
    pub fn value_to_tokens(&self, current_supply: f64, value: f64) -> f64 {
        // num_tokens = (1/c) * ln(amount / (a * b / c) + e^(c * current_supply)) - current_supply
        let term = value / (self.a * self.b / self.c) + 
            (self.c * current_supply).exp();
        (1.0 / self.c) * term.ln() - current_supply
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct ExponentialCurve {
    pub a: [u8; 8],
    pub b: [u8; 8],
    pub c: [u8; 8],
}

impl ExponentialCurve {
    pub fn from_struct(parsed: ParsedExponentialCurve) -> Self {
        Self {
            a: parsed.a.to_le_bytes(),
            b: parsed.b.to_le_bytes(),
            c: parsed.c.to_le_bytes(),
        }
    }

    pub fn to_struct(&self) -> Result<ParsedExponentialCurve, std::io::Error> {
        Ok(ParsedExponentialCurve {
            a: f64::from_le_bytes(self.a),
            b: f64::from_le_bytes(self.b),
            c: f64::from_le_bytes(self.c),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::to_decimal;

    #[test]
    fn test_curve() {
        let curve = ParsedExponentialCurve {
            a: 11400.2301,
            b: 0.00000087717527,
            c: 0.00000087717527,
        };

        let current_supply = 0.0; // starting supply
        let usdc = to_decimal(2306, 6); // dollars

        let tokens = curve.value_to_tokens(current_supply, usdc);
        let actual_cost = curve.tokens_to_value(current_supply, tokens);
        
        println!("Tokens received for {} dollars: {}", usdc, tokens);
        println!("Actual cost for {} tokens: {}", tokens, actual_cost);
        println!("Price at supply {}: {}", current_supply, curve.price(current_supply));

        // Extended tests from table
        struct TestCase {
            supply: f64,
            expected_price: f64,      // R'(S)
            expected_total: f64,      // R(S)
            expected_marketcap: f64,  // Spot Marketcap
        }

        let test_cases = vec![
            // 0% - Start
            TestCase {
                supply: 0.0,
                expected_price: 0.010000,
                expected_total: 0.0,
                expected_marketcap: 0.0,
            },
            // 25% - Quarter point
            TestCase {
                supply: 5_250_000.0,
                expected_price: 1.000000,
                expected_total: 1_128_623.0,
                expected_marketcap: 5_250_000.0,
            },
            // 50% - Half point
            TestCase {
                supply: 10_500_000.0,
                expected_price: 99.999995,
                expected_total: 113_990_897.0,
                expected_marketcap: 1_049_999_952.0,
            },
            // 75% - Three-quarter point
            TestCase {
                supply: 15_750_000.0,
                expected_price: 9_999.999361,
                expected_total: 11_400_218_067.0,
                expected_marketcap: 157_499_989_942.0,
            },
            // 100% - End
            TestCase {
                supply: 21_000_000.0,
                expected_price: 999_999.917651,
                expected_total: 1_140_022_914_292.0,
                expected_marketcap: 20_999_998_270_663.0,
            },
        ];

        let mut previous_supply = 0.0;
        let mut previous_total = 0.0;

        for (i, case) in test_cases.iter().enumerate() {
            // Test price (R'(S))
            let calculated_price = curve.price(case.supply);
            assert!(
                (calculated_price - case.expected_price).abs() < 0.0001,
                "Test case {}: Price at supply {} - expected {}, got {}",
                i, case.supply, case.expected_price, calculated_price
            );

            // Test total value (R(S)) from 0 to current supply
            let calculated_total = curve.tokens_to_value(0.0, case.supply);
            assert!(
                (calculated_total - case.expected_total).abs() / case.expected_total.max(1.0) < 0.01, // 1% tolerance, avoid division by zero
                "Test case {}: Total value at supply {} - expected {}, got {}",
                i, case.supply, case.expected_total, calculated_total
            );

            // Test incremental value between points
            let incremental_value = curve.tokens_to_value(previous_supply, case.supply - previous_supply);
            let expected_increment = case.expected_total - previous_total;
            if i > 0 {  // Skip first case since no previous point
                assert!(
                    (incremental_value - expected_increment).abs() / expected_increment.max(1.0) < 0.01,
                    "Test case {}: Incremental value from {} to {} - expected {}, got {}",
                    i, previous_supply, case.supply, expected_increment, incremental_value
                );
            }

            // Test spot market cap (supply * current price)
            let calculated_marketcap = case.supply * calculated_price;
            assert!(
                (calculated_marketcap - case.expected_marketcap).abs() / case.expected_marketcap.max(1.0) < 0.01, // 1% tolerance, avoid division by zero
                "Test case {}: Market cap at supply {} - expected {}, got {}",
                i, case.supply, case.expected_marketcap, calculated_marketcap
            );

            previous_supply = case.supply;
            previous_total = case.expected_total;
        }
    }
}
