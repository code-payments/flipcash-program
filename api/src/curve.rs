use brine_fp::UnsignedNumeric;
use crate::consts::*;
use crate::discrete_pricing_table::*;
use crate::utils::*;

#[derive(Debug, Clone)]
pub struct ContinuousExponentialCurve {
    pub a: UnsignedNumeric,
    pub b: UnsignedNumeric,
    pub c: UnsignedNumeric,
}

impl ContinuousExponentialCurve {
    pub fn default() -> Self {
        Self {
            a: UnsignedNumeric::from_scaled_u128(CURVE_A),
            b: UnsignedNumeric::from_scaled_u128(CURVE_B),
            c: UnsignedNumeric::from_scaled_u128(CURVE_C),
        }
    }

    /// Calculate token price at a given supply
    pub fn spot_price_at_supply(&self, current_supply: &UnsignedNumeric) -> Option<UnsignedNumeric> {
        // R'(S) = a * b * e^(c * s)

        let c_times_s = self.c.checked_mul(current_supply)?;
        let exp = c_times_s.signed().exp()?;
        self.a.checked_mul(&self.b)?.checked_mul(&exp)
    }

    /// Calculate total cost to buy `num_tokens` starting at `current_supply`
    /// “How much does it cost to get X tokens?”
    pub fn tokens_to_value(
        &self,
        current_supply: &UnsignedNumeric,
        tokens: &UnsignedNumeric,
    ) -> Option<UnsignedNumeric> {
        // Integral of price function:
        // R(S) = ∫(a * b * e^(c * s)) ds = (a * b / c) * e^(c * s)
        // R(S) = (a * b / c) * (e^(c * S) - e^(c * S0))

        let new_supply = current_supply.checked_add(tokens)?;
        let cs = self.c.checked_mul(current_supply)?;
        let ns = self.c.checked_mul(&new_supply)?;

        let exp_cs = cs.signed().exp()?;
        let exp_ns = ns.signed().exp()?;

        let numerator = self.a.checked_mul(&self.b)?;
        let ab_over_c = numerator.checked_div(&self.c)?;

        exp_ns
            .checked_sub(&exp_cs)
            .and_then(|diff| ab_over_c.checked_mul(&diff))
    }

    /// Calculate number of tokens received for a `value` amount starting at `current_supply`
    /// “How many tokens can I get for Y value?”
    pub fn value_to_tokens(
        &self,
        current_supply: &UnsignedNumeric,
        value: &UnsignedNumeric,
    ) -> Option<UnsignedNumeric> {
        // num_tokens = (1/c) * ln(amount / (a * b / c) + e^(c * current_supply)) - current_supply

        let ab_over_c = self.a.checked_mul(&self.b)?.checked_div(&self.c)?;
        let exp_cs = self.c.checked_mul(current_supply)?.signed().exp()?;

        let term = value.checked_div(&ab_over_c)?.checked_add(&exp_cs)?;

        let ln_term = term.log()?.value;
        let result = ln_term.checked_div(&self.c)?.checked_sub(current_supply)?;

        Some(result)
    }
}

#[derive(Debug, Clone)]
pub struct DiscreteExponentialCurve;


/// Discrete implementation of ContinuousExponentialCurve
impl DiscreteExponentialCurve {
    pub fn default() -> Self {
        Self
    }

    pub fn spot_price_at_supply(&self, current_supply: &UnsignedNumeric) -> Option<UnsignedNumeric> {
        let step_size = UnsignedNumeric::new(DISCRETE_PRICING_STEP_SIZE).unwrap();
        let step_index = current_supply.
            checked_div(&step_size).
            unwrap().
            floor().
            unwrap().
            to_imprecise()
            .unwrap() as usize;
        if step_index >= DISCRETE_PRICING_TABLE.len() {
            return None;
        }

        let price = DISCRETE_PRICING_TABLE[step_index];
        Some(UnsignedNumeric::from_scaled_u128(price))
    }

    pub fn tokens_to_value(
        &self,
        current_supply: &UnsignedNumeric,
        tokens: &UnsignedNumeric,
    ) -> Option<UnsignedNumeric> {
        let zero = UnsignedNumeric::zero();

        if tokens.eq(&zero) {
            return Some(UnsignedNumeric::zero());
        }

        let step_size = UnsignedNumeric::new(DISCRETE_PRICING_STEP_SIZE).unwrap();

        let mut remaining_tokens = tokens.clone();
        let mut new_current_supply = current_supply.clone();
        let mut total_cost = UnsignedNumeric::zero();

        while remaining_tokens.greater_than(&zero) {
            let step_index = new_current_supply.
                checked_div(&step_size).
                unwrap().
                floor().
                unwrap().
                to_imprecise()
                .unwrap() as usize;
            if step_index >= DISCRETE_PRICING_TABLE.len() {
                return None;
            }

            let tokens_in_current_step = step_size.checked_sub(
                &modulo(&new_current_supply, &step_size).unwrap()
            ).unwrap();

            let mut tokens_to_buy = remaining_tokens.clone();
            if tokens_to_buy.greater_than(&tokens_in_current_step) {
                tokens_to_buy = tokens_in_current_step;
            }
            if tokens_to_buy.eq(&zero) {
                break;
            }

            let price = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[step_index]);
            let step_cost = tokens_to_buy.checked_mul(&price).unwrap();

            total_cost = total_cost.checked_add(&step_cost).unwrap();

            if tokens_to_buy.greater_than(&remaining_tokens) {
                break;
            }

            remaining_tokens = remaining_tokens.checked_sub(&tokens_to_buy).unwrap();
            new_current_supply = new_current_supply.checked_add(&tokens_to_buy).unwrap();
        }

        Some(total_cost)
    }

    pub fn value_to_tokens(
        &self,
        current_supply: &UnsignedNumeric,
        value: &UnsignedNumeric,
    ) -> Option<UnsignedNumeric> {
        let zero = UnsignedNumeric::zero();

        if value.eq(&zero) {
            return Some(UnsignedNumeric::zero());
        }

        let step_size = UnsignedNumeric::new(DISCRETE_PRICING_STEP_SIZE).unwrap();

        let mut remaining_value = value.clone();
        let mut new_current_supply = current_supply.clone();
        let mut total_tokens = UnsignedNumeric::zero();

        while !remaining_value.value.is_zero() {
           let step_index = new_current_supply.
                checked_div(&step_size).
                unwrap().
                floor().
                unwrap().
                to_imprecise()
                .unwrap() as usize;
            if step_index >= DISCRETE_PRICING_TABLE.len()-1 {
                return None;
            }

            let tokens_in_current_step = step_size.checked_sub(
                &modulo(&new_current_supply, &step_size).unwrap()
            ).unwrap();

            let price = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[step_index]);

            let step_total_cost = tokens_in_current_step.checked_mul(&price)?;

            if remaining_value.greater_than_or_equal(&step_total_cost) {
                total_tokens = total_tokens.checked_add(&tokens_in_current_step).unwrap();
                remaining_value = remaining_value.checked_sub(&step_total_cost)?;
                new_current_supply = new_current_supply.checked_add(&tokens_in_current_step).unwrap();
            } else {
                let tokens_affordable = remaining_value.checked_div(&price).unwrap();

                let mut tokens_to_buy = tokens_affordable;
                if tokens_to_buy.greater_than(&tokens_in_current_step) {
                    tokens_to_buy = tokens_in_current_step
                }
                if tokens_to_buy.eq(&zero) {
                    break;
                }

                total_tokens = total_tokens.checked_add(&tokens_to_buy).unwrap();

                let actual_cost = tokens_to_buy.checked_mul(&price).unwrap();
                if actual_cost.greater_than(&remaining_value) {
                    break
                }

                remaining_value = remaining_value.checked_sub(&actual_cost)?;
                new_current_supply = new_current_supply.checked_add(&tokens_to_buy).unwrap();
            }
        }

        Some(total_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use brine_fp::UnsignedNumeric;

    fn assert_approx_eq(actual: &UnsignedNumeric, expected: &UnsignedNumeric, tolerance: u128) {
        let (diff, _) = actual.unsigned_sub(expected);
        let tol = UnsignedNumeric::from_scaled_u128(tolerance);
        assert!(
            diff.less_than_or_equal(&tol),
            "Mismatch: got {}, expected {}, diff = {}",
            actual.to_string(),
            expected.to_string(),
            diff.to_string()
        );
    }

    #[test]
    fn test_calculate_continuous_curve_constants() {
        const ONE_PENNY: u128 = 10_000_000_000_000_000; // $0.01 (starting price)
        const ONE_MILLION: u128 = 1_000_000_000_000_000_000_000_000; // $1_000_000 (ending price)
        const TWENTY_ONE_MILLION: u128 = 21_000_000_000_000_000_000_000_000; // 21_000_000 tokens

        let price_start = UnsignedNumeric::from_scaled_u128(ONE_PENNY);
        let price_end = UnsignedNumeric::from_scaled_u128(ONE_MILLION);
        let supply_diff = UnsignedNumeric::from_scaled_u128(TWENTY_ONE_MILLION);

        println!("price_start: {}", price_start.to_string());
        println!("price_end: {}", price_end.to_string());
        println!("supply_diff: {}", supply_diff.to_string());

        // Step 1: Solve for c
        // exp(c * 21_000_000) = 100_000_000
        // c = ln(100_000_000) / 21_000_000
        let ratio = price_end.checked_div(&price_start).unwrap();
        let ln = ratio.log().unwrap(); // ln(100_000_000)
        let c = ln.value.checked_div(&supply_diff).unwrap();

        // Step 2: Solve for a
        // a * c = 0.01 → a = 0.01 / c
        let a = price_start.checked_div(&c).unwrap();

        // Step 3: Set b = c (for consistent use)
        let b = c.clone();

        // Check: R'(0) = a * b * exp(c * 0) = a * b
        let spot_0 = a.checked_mul(&b).unwrap();

        assert_approx_eq(
            &spot_0,
            &price_start,
            100_000_000_000_000, // acceptable error (0.0001)
        );

        // Check: R'(21_000_000) = a * b * exp(c * 21_000_000)
        let exp_term = c.checked_mul(&supply_diff).unwrap().signed().exp().unwrap();
        let spot_end = a.checked_mul(&b).unwrap().checked_mul(&exp_term).unwrap();

        assert_approx_eq(
            &spot_end,
            &price_end,
            100_000_000_000_000, // acceptable error (0.0001)
        );

        // Print values for manual inspection
        println!("a = {}", a.to_string());
        println!("b = {}", b.to_string());
        println!("c = {}", c.to_string());

        assert_approx_eq(&a, &UnsignedNumeric::from_scaled_u128(CURVE_A), 0);
        assert_approx_eq(&b, &UnsignedNumeric::from_scaled_u128(CURVE_B), 0);
        assert_approx_eq(&c, &UnsignedNumeric::from_scaled_u128(CURVE_C), 0);
    }

    #[test]
    #[ignore]
    fn generate_continuous_curve_table() {
        let a = UnsignedNumeric::from_scaled_u128(CURVE_A);
        let b = UnsignedNumeric::from_scaled_u128(CURVE_B);
        let c = UnsignedNumeric::from_scaled_u128(CURVE_C);

        let curve = ContinuousExponentialCurve {
            a: a.clone(),
            b: b.clone(),
            c: c.clone(),
        };

        println!("|------|----------------|-----------------------------------|-----------------------------|");
        println!("| %    | S              | R(S)                              | R'(S)                       |");
        println!("|------|----------------|-----------------------------------|-----------------------------|");

        let zero = UnsignedNumeric::zero();
        let buy_amount = UnsignedNumeric::new(210_000).unwrap(); // 1% at a time
        let mut supply = zero.clone();

        for i in 0..101 {
            let cost = curve.tokens_to_value(&zero, &supply).unwrap();
            let spot_price = curve.spot_price_at_supply(&supply).unwrap();

            println!(
                "| {:>3}% | {:>14.12} | {:>32.32} | {:>26.32} |",
                i,
                supply.to_string(),
                cost.to_string(),
                spot_price.to_string()
            );

            supply = supply.checked_add(&buy_amount.clone()).unwrap();
        }

        println!("|------|----------------|-----------------------------------|-----------------------------|");
        //assert!(false);
    }

    #[test]
    #[ignore]
    fn export_continuous_curve_pricing_csv() {
        let curve = ContinuousExponentialCurve::default();

        // CSV header
        println!("supply,spot_price");

        let zero = UnsignedNumeric::zero();
        let step_size = UnsignedNumeric::new(100).unwrap(); // 100 tokens per step
        let mut supply = zero.clone();

        // 0 to 21,000,000 in steps of 100 = 210,001 rows (including 0)
        for _ in 0..=210_000 {
            let spot_price = curve.spot_price_at_supply(&supply).unwrap();

            println!(
                "{},{}",
                supply.to_string(),
                spot_price.to_string(),
            );

            supply = supply.checked_add(&step_size).unwrap();
        }
    }

    #[test]
    fn test_discrete_pricing_table_matches_continuous_curve() {
        use crate::discrete_pricing_table::DISCRETE_PRICING_TABLE;

        let curve = ContinuousExponentialCurve::default();
        let step_size = UnsignedNumeric::new(100).unwrap(); // 100 tokens per step

        for (index, &table_price) in DISCRETE_PRICING_TABLE.iter().enumerate() {
            // Calculate supply for this index: supply = index * 100
            let supply = UnsignedNumeric::new(index as u128)
                .unwrap()
                .checked_mul(&step_size)
                .unwrap();

            // Get spot price from continuous curve
            let curve_price = curve.spot_price_at_supply(&supply)
                .expect(&format!("Failed to calculate spot price at supply {}", index * 100));

            // Convert table price to UnsignedNumeric for comparison
            let table_price_numeric = UnsignedNumeric::from_scaled_u128(table_price);

            // Assert they match within tolerance
            assert_approx_eq(
                &curve_price,
                &table_price_numeric,
                0,
            );
        }
    }

    #[test]
    fn test_discrete_spot_price() {
        let curve = DiscreteExponentialCurve::default();

        // Test at supply 0
        let supply_0 = UnsignedNumeric::new(0).unwrap();
        let price_0 = curve.spot_price_at_supply(&supply_0).unwrap();

        // Should be the first entry in the table
        let expected_0 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0]);
        assert_eq!(price_0.to_string(), expected_0.to_string());

        // Test at supply 50 (should use same price as supply 0, since both are in step 0)
        let supply_50 = UnsignedNumeric::new(50).unwrap();
        let price_50 = curve.spot_price_at_supply(&supply_50).unwrap();
        assert_eq!(price_50.to_string(), expected_0.to_string());

        // Test at supply 100 (should use price from step 1)
        let supply_100 = UnsignedNumeric::new(100).unwrap();
        let price_100 = curve.spot_price_at_supply(&supply_100).unwrap();
        let expected_100 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[1]);
        assert_eq!(price_100.to_string(), expected_100.to_string());
    }

    #[test]
    fn test_discrete_tokens_to_value() {
        let curve = DiscreteExponentialCurve::default();

        // Test buying 0 tokens
        let supply = UnsignedNumeric::new(0).unwrap();
        let tokens = UnsignedNumeric::new(0).unwrap();
        let cost = curve.tokens_to_value(&supply, &tokens).unwrap();
        assert_eq!(cost.to_string(), UnsignedNumeric::zero().to_string());

        // Test buying 100 tokens from supply 0
        // All tokens should cost price[0]
        let tokens_100 = UnsignedNumeric::new(100).unwrap();
        let cost_100 = curve.tokens_to_value(&supply, &tokens_100).unwrap();
        let expected_100 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0])
            .checked_mul(&tokens_100).unwrap();
        assert_eq!(cost_100.to_string(), expected_100.to_string());

        // Test buying 200 tokens from supply 0
        // First 100 tokens cost price[0], next 100 tokens cost price[1]
        let tokens_200 = UnsignedNumeric::new(200).unwrap();
        let cost_200 = curve.tokens_to_value(&supply, &tokens_200).unwrap();
        let expected_200 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0])
            .checked_mul(&tokens_100).unwrap()
            .checked_add(
                &UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[1])
                    .checked_mul(&tokens_100).unwrap()
            ).unwrap();
        assert_eq!(cost_200.to_string(), expected_200.to_string());

        // Test buying 150 tokens from supply 50
        // First 50 tokens cost price[0], next 100 tokens cost price[1]
        let supply_50 = UnsignedNumeric::new(50).unwrap();
        let tokens_150 = UnsignedNumeric::new(150).unwrap();
        let cost_150 = curve.tokens_to_value(&supply_50, &tokens_150).unwrap();
        let expected_150 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0])
            .checked_mul(&UnsignedNumeric::new(50).unwrap()).unwrap()
            .checked_add(
                &UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[1])
                    .checked_mul(&tokens_100).unwrap()
            ).unwrap();
        assert_eq!(cost_150.to_string(), expected_150.to_string());
    }

    #[test]
    fn test_discrete_value_to_tokens() {
        let curve = DiscreteExponentialCurve::default();

        // Test with 0 value
        let supply = UnsignedNumeric::new(0).unwrap();
        let value_0 = UnsignedNumeric::zero();
        let tokens = curve.value_to_tokens(&supply, &value_0).unwrap();
        assert_eq!(tokens.to_string(), UnsignedNumeric::zero().to_string());

        // Test buying exactly 100 tokens worth at price[0]
        let price_0 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0]);
        let tokens_100 = UnsignedNumeric::new(100).unwrap();
        let value_for_100 = price_0.checked_mul(&tokens_100).unwrap();
        let tokens_result = curve.value_to_tokens(&supply, &value_for_100).unwrap();
        assert_eq!(tokens_result.to_string(), tokens_100.to_string());

        // Test buying approximately 50 tokens at price[0]
        let value_for_50 = price_0.checked_mul(&UnsignedNumeric::new(50).unwrap()).unwrap();
        let tokens_result_50 = curve.value_to_tokens(&supply, &value_for_50).unwrap();
        assert_eq!(tokens_result_50.to_string(), UnsignedNumeric::new(50).unwrap().to_string());
    }

    #[test]
    fn test_curves_buy_and_sell_max() {
        let continuous_curve = ContinuousExponentialCurve::default();
        let discrete_curve = DiscreteExponentialCurve::default();

        let supply = UnsignedNumeric::zero();

        let tokens = UnsignedNumeric::new(21_000_000).unwrap();
        let value_continuous = continuous_curve.tokens_to_value(&supply, &tokens).unwrap();
        let value_discrete = discrete_curve.tokens_to_value(&supply, &tokens).unwrap();
        println!("Continuous Value:  {}", value_continuous.to_string());
        println!("Discrete Value:    {}", value_discrete.to_string());

        let tokens_continuous = continuous_curve.value_to_tokens(&supply, &UnsignedNumeric::new(1140023003583).unwrap()).unwrap();
        let tokens_discrete = discrete_curve.value_to_tokens(&supply, &UnsignedNumeric::new(1139973004315).unwrap()).unwrap();
        println!("Continuous Tokens: {}", tokens_continuous.to_string());
        println!("Discrete Tokens:   {}", tokens_discrete.to_string());
    }

    #[test]
    #[ignore]
    fn generate_discrete_curve_table() {
        let curve = DiscreteExponentialCurve::default();

        println!("|------|----------------|-----------------------------------|-----------------------------|");
        println!("| %    | S              | R(S)                              | R'(S)                       |");
        println!("|------|----------------|-----------------------------------|-----------------------------|");

        let zero = UnsignedNumeric::zero();
        let buy_amount = UnsignedNumeric::new(210_000).unwrap(); // 1% at a time
        let mut supply = zero.clone();

        for i in 0..101 {
            let cost = curve.tokens_to_value(&zero, &supply).unwrap();
            let spot_price = curve.spot_price_at_supply(&supply).unwrap();

            println!(
                "| {:>3}% | {:>14.12} | {:>32.32} | {:>26.32} |",
                i,
                supply.to_string(),
                cost.to_string(),
                spot_price.to_string()
            );

            supply = supply.checked_add(&buy_amount.clone()).unwrap();
        }

        println!("|------|----------------|-----------------------------------|-----------------------------|");
        //assert!(false);
    }
}
