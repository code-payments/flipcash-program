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
            return value.checked_div(&start_price);
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
        let target_cumulative_raw = target_cumulative.value.as_u128();
        while low < high {
            let mid = (low + high + 1) / 2;
            let mid_cumulative = DISCRETE_CUMULATIVE_VALUE_TABLE[mid];

            if mid_cumulative <= target_cumulative_raw {
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

    #[test]
    fn test_discrete_spot_price_at_step_boundaries() {
        let curve = DiscreteExponentialCurve::default();

        // Test that prices change exactly at step boundaries (multiples of 100)
        for step in 0..10 {
            let boundary = step * DISCRETE_PRICING_STEP_SIZE;
            let expected_price = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[step as usize]);

            // At exact boundary
            let supply_at_boundary = UnsignedNumeric::new(boundary).unwrap();
            let price_at_boundary = curve.spot_price_at_supply(&supply_at_boundary).unwrap();
            assert_eq!(price_at_boundary.to_string(), expected_price.to_string(),
                "Price at step {} boundary should match table", step);

            // Just before next boundary (should still use current step's price)
            if boundary + 99 < DISCRETE_PRICING_TABLE.len() as u128 * DISCRETE_PRICING_STEP_SIZE {
                let supply_before_next = UnsignedNumeric::new(boundary + 99).unwrap();
                let price_before_next = curve.spot_price_at_supply(&supply_before_next).unwrap();
                assert_eq!(price_before_next.to_string(), expected_price.to_string(),
                    "Price at supply {} should still use step {}", boundary + 99, step);
            }
        }
    }

    #[test]
    fn test_discrete_spot_price_at_various_positions_within_step() {
        let curve = DiscreteExponentialCurve::default();

        // All positions within a step should return the same price
        let step_index = 5;
        let expected_price = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[step_index]);

        for offset in [0, 1, 25, 50, 75, 99] {
            let supply = UnsignedNumeric::new(step_index as u128 * DISCRETE_PRICING_STEP_SIZE + offset).unwrap();
            let price = curve.spot_price_at_supply(&supply).unwrap();
            assert_eq!(price.to_string(), expected_price.to_string(),
                "Price at offset {} within step {} should be the same", offset, step_index);
        }
    }

    #[test]
    fn test_discrete_spot_price_beyond_table_returns_none() {
        let curve = DiscreteExponentialCurve::default();

        // Supply beyond the table should return None
        let max_valid_supply = (DISCRETE_PRICING_TABLE.len() - 1) as u128 * DISCRETE_PRICING_STEP_SIZE;
        let supply_beyond = UnsignedNumeric::new(max_valid_supply + DISCRETE_PRICING_STEP_SIZE).unwrap();
        assert!(curve.spot_price_at_supply(&supply_beyond).is_none(),
            "Supply beyond table should return None");
    }

    #[test]
    fn test_discrete_spot_price_at_max_supply() {
        let curve = DiscreteExponentialCurve::default();

        // Test at maximum valid supply (last entry in table)
        let last_step = DISCRETE_PRICING_TABLE.len() - 1;
        let max_supply = UnsignedNumeric::new(last_step as u128 * DISCRETE_PRICING_STEP_SIZE).unwrap();
        let price = curve.spot_price_at_supply(&max_supply).unwrap();
        let expected = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[last_step]);
        assert_eq!(price.to_string(), expected.to_string());
    }

    #[test]
    fn test_discrete_tokens_to_value_zero_tokens() {
        let curve = DiscreteExponentialCurve::default();

        // Test buying 0 tokens from various supplies
        for supply_val in [0, 50, 100, 1000, 10000] {
            let supply = UnsignedNumeric::new(supply_val).unwrap();
            let tokens = UnsignedNumeric::zero();
            let cost = curve.tokens_to_value(&supply, &tokens).unwrap();
            assert_eq!(cost.to_string(), UnsignedNumeric::zero().to_string(),
                "Buying 0 tokens from supply {} should cost 0", supply_val);
        }
    }

    #[test]
    fn test_discrete_tokens_to_value_within_single_step() {
        let curve = DiscreteExponentialCurve::default();

        // Test buying tokens entirely within a single step
        let supply = UnsignedNumeric::new(0).unwrap();
        let tokens_50 = UnsignedNumeric::new(50).unwrap();
        let cost_50 = curve.tokens_to_value(&supply, &tokens_50).unwrap();
        let expected_50 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0])
            .checked_mul(&tokens_50).unwrap();
        assert_eq!(cost_50.to_string(), expected_50.to_string());

        // From mid-step to end of same step
        let supply_25 = UnsignedNumeric::new(25).unwrap();
        let tokens_75 = UnsignedNumeric::new(75).unwrap();
        let cost_75 = curve.tokens_to_value(&supply_25, &tokens_75).unwrap();
        let expected_75 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0])
            .checked_mul(&tokens_75).unwrap();
        assert_eq!(cost_75.to_string(), expected_75.to_string());

        // Middle of one step, still within same step
        let supply_10 = UnsignedNumeric::new(10).unwrap();
        let tokens_30 = UnsignedNumeric::new(30).unwrap();
        let cost_30 = curve.tokens_to_value(&supply_10, &tokens_30).unwrap();
        let expected_30 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0])
            .checked_mul(&tokens_30).unwrap();
        assert_eq!(cost_30.to_string(), expected_30.to_string());
    }

    #[test]
    fn test_discrete_tokens_to_value_exact_step() {
        let curve = DiscreteExponentialCurve::default();
        let tokens_100 = UnsignedNumeric::new(100).unwrap();

        // Test buying exactly 100 tokens from supply 0
        let supply = UnsignedNumeric::new(0).unwrap();
        let cost_100 = curve.tokens_to_value(&supply, &tokens_100).unwrap();
        let expected_100 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0])
            .checked_mul(&tokens_100).unwrap();
        assert_eq!(cost_100.to_string(), expected_100.to_string());
    }

    #[test]
    fn test_discrete_tokens_to_value_crossing_one_boundary() {
        let curve = DiscreteExponentialCurve::default();
        let tokens_100 = UnsignedNumeric::new(100).unwrap();

        // Test buying 200 tokens from supply 0 (crosses one boundary)
        let supply = UnsignedNumeric::new(0).unwrap();
        let tokens_200 = UnsignedNumeric::new(200).unwrap();
        let cost_200 = curve.tokens_to_value(&supply, &tokens_200).unwrap();
        let expected_200 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0])
            .checked_mul(&tokens_100).unwrap()
            .checked_add(
                &UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[1])
                    .checked_mul(&tokens_100).unwrap()
            ).unwrap();
        assert_eq!(cost_200.to_string(), expected_200.to_string());
    }

    #[test]
    fn test_discrete_tokens_to_value_partial_start_step() {
        let curve = DiscreteExponentialCurve::default();
        let tokens_100 = UnsignedNumeric::new(100).unwrap();

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
    fn test_discrete_tokens_to_value_partial_end_step() {
        let curve = DiscreteExponentialCurve::default();

        // Test buying 175 tokens from supply 0
        // 100 tokens at price[0], 75 tokens at price[1]
        let supply = UnsignedNumeric::new(0).unwrap();
        let tokens_175 = UnsignedNumeric::new(175).unwrap();
        let cost = curve.tokens_to_value(&supply, &tokens_175).unwrap();
        let expected = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0])
            .checked_mul(&UnsignedNumeric::new(100).unwrap()).unwrap()
            .checked_add(
                &UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[1])
                    .checked_mul(&UnsignedNumeric::new(75).unwrap()).unwrap()
            ).unwrap();
        assert_eq!(cost.to_string(), expected.to_string());
    }

    #[test]
    fn test_discrete_tokens_to_value_partial_both_ends() {
        let curve = DiscreteExponentialCurve::default();

        // Test buying 125 tokens from supply 50
        // 50 tokens at price[0], 75 tokens at price[1]
        let supply_50 = UnsignedNumeric::new(50).unwrap();
        let tokens_125 = UnsignedNumeric::new(125).unwrap();
        let cost = curve.tokens_to_value(&supply_50, &tokens_125).unwrap();
        let expected = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0])
            .checked_mul(&UnsignedNumeric::new(50).unwrap()).unwrap()
            .checked_add(
                &UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[1])
                    .checked_mul(&UnsignedNumeric::new(75).unwrap()).unwrap()
            ).unwrap();
        assert_eq!(cost.to_string(), expected.to_string());
    }

    #[test]
    fn test_discrete_tokens_to_value_multiple_full_steps() {
        let curve = DiscreteExponentialCurve::default();

        // Test buying 500 tokens from supply 0 (5 full steps)
        let supply = UnsignedNumeric::new(0).unwrap();
        let tokens_500 = UnsignedNumeric::new(500).unwrap();
        let cost = curve.tokens_to_value(&supply, &tokens_500).unwrap();

        // Calculate expected cost: sum of (price[i] * 100) for i in 0..5
        let mut expected = UnsignedNumeric::zero();
        for i in 0..5 {
            let step_cost = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[i])
                .checked_mul(&UnsignedNumeric::new(100).unwrap()).unwrap();
            expected = expected.checked_add(&step_cost).unwrap();
        }
        assert_eq!(cost.to_string(), expected.to_string());
    }

    #[test]
    fn test_discrete_tokens_to_value_multiple_steps_with_partials() {
        let curve = DiscreteExponentialCurve::default();

        // Test buying 350 tokens from supply 75
        // 25 tokens at price[0], 100 at price[1], 100 at price[2], 100 at price[3], 25 at price[4]
        let supply_75 = UnsignedNumeric::new(75).unwrap();
        let tokens_350 = UnsignedNumeric::new(350).unwrap();
        let cost = curve.tokens_to_value(&supply_75, &tokens_350).unwrap();

        let expected = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0])
            .checked_mul(&UnsignedNumeric::new(25).unwrap()).unwrap()  // partial start step
            .checked_add(
                &UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[1])
                    .checked_mul(&UnsignedNumeric::new(100).unwrap()).unwrap()
            ).unwrap()
            .checked_add(
                &UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[2])
                    .checked_mul(&UnsignedNumeric::new(100).unwrap()).unwrap()
            ).unwrap()
            .checked_add(
                &UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[3])
                    .checked_mul(&UnsignedNumeric::new(100).unwrap()).unwrap()
            ).unwrap()
            .checked_add(
                &UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[4])
                    .checked_mul(&UnsignedNumeric::new(25).unwrap()).unwrap()  // partial end step
            ).unwrap();
        assert_eq!(cost.to_string(), expected.to_string());
    }

    #[test]
    fn test_discrete_tokens_to_value_from_step_boundary() {
        let curve = DiscreteExponentialCurve::default();

        // Test buying from exact step boundary
        let supply_100 = UnsignedNumeric::new(100).unwrap();
        let tokens_150 = UnsignedNumeric::new(150).unwrap();
        let cost = curve.tokens_to_value(&supply_100, &tokens_150).unwrap();

        // 100 tokens at price[1], 50 tokens at price[2]
        let expected = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[1])
            .checked_mul(&UnsignedNumeric::new(100).unwrap()).unwrap()
            .checked_add(
                &UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[2])
                    .checked_mul(&UnsignedNumeric::new(50).unwrap()).unwrap()
            ).unwrap();
        assert_eq!(cost.to_string(), expected.to_string());
    }

    #[test]
    fn test_discrete_tokens_to_value_exceeds_table_returns_none() {
        let curve = DiscreteExponentialCurve::default();

        // Buying tokens that would exceed the table should return None
        let supply = UnsignedNumeric::new(0).unwrap();
        let tokens_too_many = UnsignedNumeric::new(
            (DISCRETE_PRICING_TABLE.len() as u128 + 1) * DISCRETE_PRICING_STEP_SIZE
        ).unwrap();
        assert!(curve.tokens_to_value(&supply, &tokens_too_many).is_none());
    }

    #[test]
    fn test_discrete_tokens_to_value_at_high_supply() {
        let curve = DiscreteExponentialCurve::default();

        // Test buying tokens at higher supply levels
        let supply = UnsignedNumeric::new(1_000_000).unwrap(); // step 10000
        let tokens_500 = UnsignedNumeric::new(500).unwrap();
        let cost = curve.tokens_to_value(&supply, &tokens_500).unwrap();

        // Manually calculate expected cost for 5 full steps starting at step 10000
        let mut expected = UnsignedNumeric::zero();
        for i in 10000..10005 {
            let step_cost = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[i])
                .checked_mul(&UnsignedNumeric::new(100).unwrap()).unwrap();
            expected = expected.checked_add(&step_cost).unwrap();
        }
        assert_eq!(cost.to_string(), expected.to_string());
    }

    #[test]
    fn test_discrete_tokens_to_value_is_additive() {
        let curve = DiscreteExponentialCurve::default();

        // Cost of buying A + B tokens should equal cost of A then cost of B from new supply
        let supply = UnsignedNumeric::new(50).unwrap();
        let tokens_a = UnsignedNumeric::new(200).unwrap();
        let tokens_b = UnsignedNumeric::new(150).unwrap();
        let tokens_total = tokens_a.checked_add(&tokens_b).unwrap();

        let cost_total = curve.tokens_to_value(&supply, &tokens_total).unwrap();

        let cost_a = curve.tokens_to_value(&supply, &tokens_a).unwrap();
        let new_supply = supply.checked_add(&tokens_a).unwrap();
        let cost_b = curve.tokens_to_value(&new_supply, &tokens_b).unwrap();
        let cost_sum = cost_a.checked_add(&cost_b).unwrap();

        assert_eq!(cost_total.to_string(), cost_sum.to_string(),
            "tokens_to_value should be additive");
    }

    #[test]
    fn test_discrete_tokens_to_value_small_amounts() {
        let curve = DiscreteExponentialCurve::default();

        // Test buying very small amounts (1 token)
        let supply = UnsignedNumeric::new(0).unwrap();
        let tokens_1 = UnsignedNumeric::new(1).unwrap();
        let cost_1 = curve.tokens_to_value(&supply, &tokens_1).unwrap();
        let expected_1 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0]);
        assert_eq!(cost_1.to_string(), expected_1.to_string());

        // Test buying 1 token at various positions
        for pos in [0, 50, 99, 100, 150] {
            let supply = UnsignedNumeric::new(pos).unwrap();
            let step_index = (pos / DISCRETE_PRICING_STEP_SIZE) as usize;
            let cost = curve.tokens_to_value(&supply, &tokens_1).unwrap();
            let expected = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[step_index]);
            assert_eq!(cost.to_string(), expected.to_string(),
                "Cost of 1 token at supply {} should be price at step {}", pos, step_index);
        }
    }

    #[test]
    fn test_discrete_value_to_tokens_zero_value() {
        let curve = DiscreteExponentialCurve::default();

        // Test with 0 value from various supplies
        for supply_val in [0, 50, 100, 1000, 10000] {
            let supply = UnsignedNumeric::new(supply_val).unwrap();
            let value = UnsignedNumeric::zero();
            let tokens = curve.value_to_tokens(&supply, &value).unwrap();
            assert_eq!(tokens.to_string(), UnsignedNumeric::zero().to_string(),
                "0 value from supply {} should yield 0 tokens", supply_val);
        }
    }

    #[test]
    fn test_discrete_value_to_tokens_within_single_step() {
        let curve = DiscreteExponentialCurve::default();
        let supply = UnsignedNumeric::new(0).unwrap();
        let price_0 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0]);

        // Test buying approximately 50 tokens at price[0]
        let tokens_50 = UnsignedNumeric::new(50).unwrap();
        let value_for_50 = price_0.checked_mul(&tokens_50).unwrap();
        let tokens_result_50 = curve.value_to_tokens(&supply, &value_for_50).unwrap();
        assert_approx_eq(&tokens_result_50, &tokens_50, 100);

        // Test buying 25 tokens
        let tokens_25 = UnsignedNumeric::new(25).unwrap();
        let value_for_25 = price_0.checked_mul(&tokens_25).unwrap();
        let tokens_result_25 = curve.value_to_tokens(&supply, &value_for_25).unwrap();
        assert_approx_eq(&tokens_result_25, &tokens_25, 100);

        // Test buying 99 tokens (just under step boundary)
        let tokens_99 = UnsignedNumeric::new(99).unwrap();
        let value_for_99 = price_0.checked_mul(&tokens_99).unwrap();
        let tokens_result_99 = curve.value_to_tokens(&supply, &value_for_99).unwrap();
        assert_approx_eq(&tokens_result_99, &tokens_99, 100);
    }

    #[test]
    fn test_discrete_value_to_tokens_exact_step() {
        let curve = DiscreteExponentialCurve::default();
        let supply = UnsignedNumeric::new(0).unwrap();
        let price_0 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0]);
        let tokens_100 = UnsignedNumeric::new(100).unwrap();

        // Test buying exactly 100 tokens worth at price[0]
        let value_for_100 = price_0.checked_mul(&tokens_100).unwrap();
        let tokens_result = curve.value_to_tokens(&supply, &value_for_100).unwrap();
        // Allow small tolerance since we cross a step boundary
        assert_approx_eq(&tokens_result, &tokens_100, 100);
    }

    #[test]
    fn test_discrete_value_to_tokens_crossing_boundary() {
        let curve = DiscreteExponentialCurve::default();
        let supply = UnsignedNumeric::new(0).unwrap();
        let price_0 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0]);
        let price_1 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[1]);

        let tokens_100 = UnsignedNumeric::new(100).unwrap();
        let tokens_50 = UnsignedNumeric::new(50).unwrap();
        let tokens_150 = UnsignedNumeric::new(150).unwrap();

        // Test buying 100 tokens at price[0] and 50 tokens at price[1]
        let value_for_first_100 = price_0.checked_mul(&tokens_100).unwrap();
        let value_for_next_50 = price_1.checked_mul(&tokens_50).unwrap();
        let value_for_150 = value_for_first_100.checked_add(&value_for_next_50).unwrap();
        let tokens_result = curve.value_to_tokens(&supply, &value_for_150).unwrap();
        assert_approx_eq(&tokens_result, &tokens_150, 100);
    }

    #[test]
    fn test_discrete_value_to_tokens_from_partial_step() {
        let curve = DiscreteExponentialCurve::default();
        let supply_50 = UnsignedNumeric::new(50).unwrap();
        let price_0 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0]);
        let price_1 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[1]);

        // From supply 50, buying 50 tokens at price[0] and 100 tokens at price[1]
        let tokens_50 = UnsignedNumeric::new(50).unwrap();
        let tokens_100 = UnsignedNumeric::new(100).unwrap();
        let tokens_150 = UnsignedNumeric::new(150).unwrap();

        let value = price_0.checked_mul(&tokens_50).unwrap()
            .checked_add(&price_1.checked_mul(&tokens_100).unwrap()).unwrap();
        let tokens_result = curve.value_to_tokens(&supply_50, &value).unwrap();
        assert_approx_eq(&tokens_result, &tokens_150, 100);
    }

    #[test]
    fn test_discrete_value_to_tokens_multiple_steps() {
        let curve = DiscreteExponentialCurve::default();
        let supply = UnsignedNumeric::new(0).unwrap();

        // Calculate value for 5 full steps (500 tokens)
        let mut value = UnsignedNumeric::zero();
        for i in 0..5 {
            let price = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[i]);
            let step_value = price.checked_mul(&UnsignedNumeric::new(100).unwrap()).unwrap();
            value = value.checked_add(&step_value).unwrap();
        }

        let tokens_result = curve.value_to_tokens(&supply, &value).unwrap();
        let tokens_500 = UnsignedNumeric::new(500).unwrap();
        assert_approx_eq(&tokens_result, &tokens_500, 100);
    }

    #[test]
    fn test_discrete_value_to_tokens_at_high_supply() {
        let curve = DiscreteExponentialCurve::default();
        let supply = UnsignedNumeric::new(1_000_000).unwrap(); // step 10000

        // Calculate value for buying tokens at high supply
        let price = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[10000]);
        let tokens_50 = UnsignedNumeric::new(50).unwrap();
        let value = price.checked_mul(&tokens_50).unwrap();

        let tokens_result = curve.value_to_tokens(&supply, &value).unwrap();
        assert_approx_eq(&tokens_result, &tokens_50, 100);
    }

    #[test]
    fn test_discrete_value_to_tokens_insufficient_for_step_completion() {
        let curve = DiscreteExponentialCurve::default();
        let supply_50 = UnsignedNumeric::new(50).unwrap();
        let price_0 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0]);

        // Value that can't complete the current step (only 25 tokens)
        let tokens_25 = UnsignedNumeric::new(25).unwrap();
        let value_for_25 = price_0.checked_mul(&tokens_25).unwrap();
        let tokens_result = curve.value_to_tokens(&supply_50, &value_for_25).unwrap();
        assert_approx_eq(&tokens_result, &tokens_25, 100);
    }

    #[test]
    fn test_discrete_value_to_tokens_just_enough_to_complete_step() {
        let curve = DiscreteExponentialCurve::default();
        let supply_50 = UnsignedNumeric::new(50).unwrap();
        let price_0 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0]);

        // Value exactly enough to complete the current step (50 tokens to reach 100)
        let tokens_50 = UnsignedNumeric::new(50).unwrap();
        let value_for_50 = price_0.checked_mul(&tokens_50).unwrap();
        let tokens_result = curve.value_to_tokens(&supply_50, &value_for_50).unwrap();
        // Allow small tolerance for floating point precision
        assert_approx_eq(&tokens_result, &tokens_50, 100);
    }

    #[test]
    fn test_discrete_value_to_tokens_beyond_max_returns_none() {
        let curve = DiscreteExponentialCurve::default();

        // Supply near the end of the table
        let near_max_supply = UnsignedNumeric::new(
            (DISCRETE_PRICING_TABLE.len() - 1) as u128 * DISCRETE_PRICING_STEP_SIZE
        ).unwrap();

        // This should return None since we're at the last step
        assert!(curve.value_to_tokens(&near_max_supply, &UnsignedNumeric::new(1000000000000000000).unwrap()).is_none());
    }

    #[test]
    fn test_discrete_value_to_tokens_small_value() {
        let curve = DiscreteExponentialCurve::default();
        let supply = UnsignedNumeric::new(0).unwrap();
        let price_0 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0]);

        // Value for exactly 1 token
        let value_for_1 = price_0.clone();
        let tokens_result = curve.value_to_tokens(&supply, &value_for_1).unwrap();
        let tokens_1 = UnsignedNumeric::new(1).unwrap();
        assert_approx_eq(&tokens_result, &tokens_1, 100);

        // Value for less than 1 token
        let half_price = price_0.checked_div(&UnsignedNumeric::new(2).unwrap()).unwrap();
        let tokens_result_half = curve.value_to_tokens(&supply, &half_price).unwrap();
        let tokens_half = tokens_1.checked_div(&UnsignedNumeric::new(2).unwrap()).unwrap();
        assert_approx_eq(&tokens_result_half, &tokens_half, 100);
    }

    #[test]
    fn test_discrete_value_to_tokens_partial_end_step() {
        let curve = DiscreteExponentialCurve::default();
        let supply = UnsignedNumeric::new(0).unwrap();
        let price_0 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0]);
        let price_1 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[1]);

        // Value for 100 tokens at price[0] + 75 tokens at price[1] = 175 tokens
        let tokens_100 = UnsignedNumeric::new(100).unwrap();
        let tokens_75 = UnsignedNumeric::new(75).unwrap();
        let tokens_175 = UnsignedNumeric::new(175).unwrap();

        let value = price_0.checked_mul(&tokens_100).unwrap()
            .checked_add(&price_1.checked_mul(&tokens_75).unwrap()).unwrap();
        let tokens_result = curve.value_to_tokens(&supply, &value).unwrap();
        assert_approx_eq(&tokens_result, &tokens_175, 100);
    }

    #[test]
    fn test_discrete_roundtrip_tokens_to_value_to_tokens() {
        let curve = DiscreteExponentialCurve::default();

        // Test roundtrip: tokens -> value -> tokens should be approximately equal
        let test_cases: Vec<(u128, u128)> = vec![
            (0, 100),
            (0, 250),
            (0, 500),
            (50, 150),
            (100, 200),
            (1000, 1000),
            (10000, 5000),
        ];

        for (supply_val, tokens_val) in test_cases {
            let supply = UnsignedNumeric::new(supply_val).unwrap();
            let tokens = UnsignedNumeric::new(tokens_val).unwrap();

            // Convert tokens to value
            let value = curve.tokens_to_value(&supply, &tokens).unwrap();

            // Convert value back to tokens
            let tokens_back = curve.value_to_tokens(&supply, &value).unwrap();

            // Should get approximately the same number of tokens back
            assert_approx_eq(&tokens_back, &tokens, 100);
        }
    }

    #[test]
    fn test_discrete_roundtrip_value_to_tokens_to_value() {
        let curve = DiscreteExponentialCurve::default();

        // Test roundtrip: value -> tokens -> value
        let price_0 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0]);

        let test_values: Vec<UnsignedNumeric> = vec![
            price_0.checked_mul(&UnsignedNumeric::new(50).unwrap()).unwrap(),
            price_0.checked_mul(&UnsignedNumeric::new(100).unwrap()).unwrap(),
            price_0.checked_mul(&UnsignedNumeric::new(250).unwrap()).unwrap(),
        ];

        for value in test_values {
            let supply = UnsignedNumeric::new(0).unwrap();

            // Convert value to tokens
            let tokens = curve.value_to_tokens(&supply, &value).unwrap();

            // Convert tokens back to value
            let value_back = curve.tokens_to_value(&supply, &tokens).unwrap();

            // Should get approximately the same number of tokens back
            assert_approx_eq(&value_back, &value, 100);
        }
    }

    #[test]
    fn test_discrete_spot_price_matches_tokens_to_value_for_small_amounts() {
        let curve = DiscreteExponentialCurve::default();

        // For very small purchases within a single step, the cost should be
        // exactly tokens * spot_price
        for step in 0..10 {
            let supply = UnsignedNumeric::new(step * DISCRETE_PRICING_STEP_SIZE).unwrap();
            let tokens_1 = UnsignedNumeric::new(1).unwrap();

            let spot_price = curve.spot_price_at_supply(&supply).unwrap();
            let cost = curve.tokens_to_value(&supply, &tokens_1).unwrap();

            assert_eq!(spot_price.to_string(), cost.to_string(),
                "Cost of 1 token at step {} should equal spot price", step);
        }
    }

    #[test]
    fn test_discrete_tokens_to_value_consistency_with_cumulative_table() {
        let curve = DiscreteExponentialCurve::default();

        // Verify that buying from 0 to step boundary equals cumulative table
        for step in [0, 10, 100, 1000, 10000] {
            if step >= DISCRETE_CUMULATIVE_VALUE_TABLE.len() {
                continue;
            }

            let supply = UnsignedNumeric::new(0).unwrap();
            let tokens = UnsignedNumeric::new(step as u128 * DISCRETE_PRICING_STEP_SIZE).unwrap();

            let value = curve.tokens_to_value(&supply, &tokens).unwrap();
            let cumulative = UnsignedNumeric::from_scaled_u128(DISCRETE_CUMULATIVE_VALUE_TABLE[step]);

            assert_eq!(value.to_string(), cumulative.to_string(),
                "tokens_to_value(0, {}) should match cumulative table at step {}",
                step as u128 * DISCRETE_PRICING_STEP_SIZE, step);
        }
    }

    #[test]
    fn test_discrete_methods_handle_step_boundaries_consistently() {
        let curve = DiscreteExponentialCurve::default();

        // Test that all methods transition correctly at step boundaries
        for boundary in [100, 200, 500, 1000] {
            let boundary_supply = UnsignedNumeric::new(boundary).unwrap();
            let just_before = UnsignedNumeric::new(boundary - 1).unwrap();
            let just_after = UnsignedNumeric::new(boundary + 1).unwrap();

            // spot_price should change at boundary
            let price_before = curve.spot_price_at_supply(&just_before).unwrap();
            let price_at = curve.spot_price_at_supply(&boundary_supply).unwrap();
            let price_after = curve.spot_price_at_supply(&just_after).unwrap();

            // Before and at boundary should be different (boundary is start of new step)
            assert!(price_at.greater_than(&price_before) || price_before.eq(&price_at),
                "Price at boundary {} should be >= price just before", boundary);

            // At boundary and just after should be the same
            assert_eq!(price_at.to_string(), price_after.to_string(),
                "Price at boundary {} should equal price just after", boundary);
        }
    }

    #[test]
    fn test_discrete_buying_in_parts_equals_buying_all_at_once() {
        let curve = DiscreteExponentialCurve::default();
        let supply = UnsignedNumeric::new(0).unwrap();

        // Buying 100 + 200 + 150 should equal buying 450
        let tokens_100 = UnsignedNumeric::new(100).unwrap();
        let tokens_200 = UnsignedNumeric::new(200).unwrap();
        let tokens_150 = UnsignedNumeric::new(150).unwrap();
        let tokens_450 = UnsignedNumeric::new(450).unwrap();

        let cost_100 = curve.tokens_to_value(&supply, &tokens_100).unwrap();
        let supply_after_100 = supply.checked_add(&tokens_100).unwrap();

        let cost_200 = curve.tokens_to_value(&supply_after_100, &tokens_200).unwrap();
        let supply_after_300 = supply_after_100.checked_add(&tokens_200).unwrap();

        let cost_150 = curve.tokens_to_value(&supply_after_300, &tokens_150).unwrap();

        let total_cost_parts = cost_100.checked_add(&cost_200).unwrap()
            .checked_add(&cost_150).unwrap();

        let total_cost_once = curve.tokens_to_value(&supply, &tokens_450).unwrap();

        assert_eq!(total_cost_parts.to_string(), total_cost_once.to_string(),
            "Buying in parts should equal buying all at once");
    }

    #[test]
    fn test_discrete_large_purchase_across_many_steps() {
        let curve = DiscreteExponentialCurve::default();
        let supply = UnsignedNumeric::new(1_234_567).unwrap();

        // Buy a large amount that spans many steps
        let large_tokens = UnsignedNumeric::new(10_000).unwrap(); // 100 steps
        let value = curve.tokens_to_value(&supply, &large_tokens).unwrap();

        // Verify it's positive
        assert!(value.greater_than(&UnsignedNumeric::zero()));

        // Verify roundtrip is close
        let tokens_back = curve.value_to_tokens(&supply, &value).unwrap();
        assert_approx_eq(&tokens_back, &large_tokens, 100);
    }

    #[test]
    fn test_discrete_fractional_tokens_handling() {
        let curve = DiscreteExponentialCurve::default();
        let supply = UnsignedNumeric::new(0).unwrap();
        let price_0 = UnsignedNumeric::from_scaled_u128(DISCRETE_PRICING_TABLE[0]);

        // Value for 10.5 tokens worth (fractional)
        let value_for_10_point_5 = price_0.checked_mul(&UnsignedNumeric::new(10).unwrap()).unwrap()
            .checked_add(&price_0.checked_div(&UnsignedNumeric::new(2).unwrap()).unwrap()).unwrap();

        // Should get 10.5 tokens
        let tokens = curve.value_to_tokens(&supply, &value_for_10_point_5).unwrap();
        let expected_10_5 = UnsignedNumeric::new(10).unwrap().
            checked_add(&UnsignedNumeric::one().checked_div(&UnsignedNumeric::new(2).unwrap()).unwrap()).unwrap();
        assert_approx_eq(&tokens, &expected_10_5, 100);
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
