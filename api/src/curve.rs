use brine_fp::UnsignedNumeric;
use crate::consts::*;
use crate::table::*;

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

    // todo: validate implementation
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
        let end_supply = current_supply.checked_add(tokens)?;

        let start_step = current_supply
            .checked_div(&step_size)?
            .floor()?
            .to_imprecise()? as usize;
        let end_step = end_supply
            .checked_div(&step_size)?
            .floor()?
            .to_imprecise()? as usize;

        if end_step >= DISCRETE_PRICING_TABLE.len() {
            return None;
        }

        // Calculate partial tokens in start step (from current_supply to next step boundary)
        let start_step_boundary = UnsignedNumeric::new((start_step as u128 + 1) * DISCRETE_PRICING_STEP_SIZE)?;
        let tokens_in_start_step = if start_step_boundary.greater_than(&end_supply) {
            // All tokens are within the same step
            tokens.clone()
        } else {
            start_step_boundary.checked_sub(current_supply)?
        };

        // Calculate partial tokens in end step (from end step boundary to end_supply)
        let end_step_boundary = UnsignedNumeric::new(end_step as u128 * DISCRETE_PRICING_STEP_SIZE)?;
        let tokens_in_end_step = end_supply.checked_sub(&end_step_boundary)?;

        // Cost for partial start step
        let start_price = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[start_step]);
        let start_cost = tokens_in_start_step.checked_mul(&start_price)?;

        // If start and end are in the same step, we're done
        if start_step == end_step {
            return Some(start_cost);
        }

        // Cost for complete steps between start_step+1 and end_step-1 (inclusive)
        // Use cumulative table: cumulative[end_step] - cumulative[start_step + 1]
        let cumulative_start = UnsignedNumeric::from_scaled_u128(DISCRETE_CUMULATIVE_VALUE_TABLE[start_step + 1]);
        let cumulative_end = UnsignedNumeric::from_scaled_u128(DISCRETE_CUMULATIVE_VALUE_TABLE[end_step]);
        let middle_cost = cumulative_end.checked_sub(&cumulative_start)?;

        // Cost for partial end step
        let end_price = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[end_step]);
        let end_cost = tokens_in_end_step.checked_mul(&end_price)?;

        // Total cost
        start_cost.checked_add(&middle_cost)?.checked_add(&end_cost)
    }

    // todo: validate implementation
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

        // Get current step index and position within step
        let start_step = current_supply
            .checked_div(&step_size)?
            .floor()?
            .to_imprecise()? as usize;

        if start_step >= DISCRETE_PRICING_TABLE.len() - 1 {
            return None;
        }

        // Calculate cost to complete the current partial step
        let start_step_boundary = UnsignedNumeric::new((start_step as u128 + 1) * DISCRETE_PRICING_STEP_SIZE)?;
        let tokens_in_start_step = start_step_boundary.checked_sub(current_supply)?;
        let start_price = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[start_step]);
        let cost_to_complete_start_step = tokens_in_start_step.checked_mul(&start_price)?;

        // If we can't even complete the start step, just divide by price
        if value.less_than(&cost_to_complete_start_step) {
            return value.checked_div(&start_price)?.floor();
        }

        // We can at least complete the start step
        let remaining_after_start = value.checked_sub(&cost_to_complete_start_step)?;

        // Calculate the cumulative value at start_step + 1 (where we'll be after completing start step)
        let base_cumulative = UnsignedNumeric::from_scaled_u128(DISCRETE_CUMULATIVE_VALUE_TABLE[start_step + 1]);

        // Target cumulative = base_cumulative + remaining_value
        let target_cumulative = base_cumulative.checked_add(&remaining_after_start)?;

        // Binary search for the step where cumulative value exceeds or equals target
        let mut low = start_step + 1;
        let mut high = DISCRETE_CUMULATIVE_VALUE_TABLE.len() - 1;

        while low < high {
            let mid = (low + high + 1) / 2;
            let mid_cumulative = UnsignedNumeric::from_scaled_u128(DISCRETE_CUMULATIVE_VALUE_TABLE[mid]);

            if mid_cumulative.less_than_or_equal(&target_cumulative) {
                low = mid;
            } else {
                high = mid - 1;
            }
        }

        // low is now the last step where cumulative <= target
        let end_step = low;

        if end_step >= DISCRETE_PRICING_TABLE.len() {
            return None;
        }

        // Calculate tokens from complete steps
        let end_step_supply = UnsignedNumeric::new(end_step as u128 * DISCRETE_PRICING_STEP_SIZE)?;
        let tokens_from_complete_steps = end_step_supply.checked_sub(&start_step_boundary)?;

        // Calculate remaining value after complete steps
        let cumulative_at_end_step = UnsignedNumeric::from_scaled_u128(DISCRETE_CUMULATIVE_VALUE_TABLE[end_step]);
        let value_used_for_complete_steps = cumulative_at_end_step.checked_sub(&base_cumulative)?;
        let remaining_value = remaining_after_start.checked_sub(&value_used_for_complete_steps)?;

        // Buy partial tokens in end step with remaining value
        let end_price = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[end_step]);
        let tokens_in_end_step = remaining_value.checked_div(&end_price)?;

        // Total tokens
        tokens_in_start_step
            .checked_add(&tokens_from_complete_steps)?
            .checked_add(&tokens_in_end_step)
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
        let step_size = UnsignedNumeric::new(100).unwrap();
        let mut supply = zero.clone();

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
        let curve = ContinuousExponentialCurve::default();
        let step_size = UnsignedNumeric::new(DISCRETE_PRICING_STEP_SIZE).unwrap();

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
            let table_price = UnsignedNumeric::from_scaled_u128(table_price);

            // Assert they match within tolerance
            assert_approx_eq(
                &curve_price,
                &table_price,
                0,
            );
        }
    }

    #[test]
    #[ignore]
    fn export_discrete_cumulative_value_csv() {
        // CSV header
        println!("supply,spot_price");

        let step_size = UnsignedNumeric::new(DISCRETE_PRICING_STEP_SIZE).unwrap();
        let mut cumulative = UnsignedNumeric::zero();

        for i in 0..=DISCRETE_PRICING_TABLE.len()-1 {
            println!("{},{}", i * 100, cumulative.value.to_string());

            if i < DISCRETE_PRICING_TABLE.len() {
                let price = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[i]);
                let step_cost = price.checked_mul(&step_size).unwrap();
                cumulative = cumulative.checked_add(&step_cost).unwrap();
            }
        }
    }

    #[test]
    fn test_discrete_cumulative_table_matches_discrete_curve() {
        let curve = DiscreteExponentialCurve::default();
        let step_size = UnsignedNumeric::new(DISCRETE_PRICING_STEP_SIZE).unwrap();

        for (index, &cumulative_value) in DISCRETE_CUMULATIVE_VALUE_TABLE.iter().enumerate() {
            // Calculate supply for this index: supply = index * 100
            let supply = UnsignedNumeric::new(index as u128)
                .unwrap()
                .checked_mul(&step_size)
                .unwrap();

            // Calculate expected value from discrete curve
            let expected_value = curve.tokens_to_value(&UnsignedNumeric::zero(), &supply).unwrap();

            // Convert table value to UnsignedNumeric for comparison
            let actual_value = UnsignedNumeric::from_scaled_u128(cumulative_value);

            // Assert they match within tolerance
            assert_approx_eq(
                &expected_value,
                &actual_value,
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

    // todo: continue expanding on this
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

    // todo: continue expanding on this
    #[test]
    fn test_discrete_value_to_tokens() {
        let curve = DiscreteExponentialCurve::default();

        let price_0 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0]);
        let price_1 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[1]);

        let tokens_50 = &UnsignedNumeric::new(50).unwrap();
        let tokens_100 = &UnsignedNumeric::new(100).unwrap();
        let tokens_150 = &UnsignedNumeric::new(150).unwrap();

        // Test with 0 value
        let supply = UnsignedNumeric::new(0).unwrap();
        let value_0 = UnsignedNumeric::zero();
        let tokens = curve.value_to_tokens(&supply, &value_0).unwrap();
        assert_approx_eq(&tokens,  &UnsignedNumeric::zero(), 0);

        // Test buying approximately 50 tokens at price[0]
        let value_for_50 = price_0.checked_mul(tokens_50).unwrap();
        let tokens_result_50 = curve.value_to_tokens(&supply, &value_for_50).unwrap();
        assert_approx_eq(&tokens_result_50,  &tokens_50, 0);

        // Test buying exactly 100 tokens worth at price[0]
        let value_for_100 = price_0.checked_mul(&tokens_100).unwrap();
        let tokens_result_100 = curve.value_to_tokens(&supply, &value_for_100).unwrap();
        assert_approx_eq(&tokens_result_100,  &tokens_100, 50);

        // Test buying exactly 100 tokens worth at price[0] and 50 tokens at price[1]
        let value_for_first_100 = price_0.checked_mul(&tokens_100).unwrap();
        let value_for_next_50 = price_1.checked_mul(&tokens_50).unwrap();
        let value_for_150 =value_for_first_100.checked_add(&value_for_next_50) .unwrap();
        let tokens_result_150 = curve.value_to_tokens(&supply, &value_for_150).unwrap();
        assert_approx_eq(&tokens_result_150,  &tokens_150, 50);
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
