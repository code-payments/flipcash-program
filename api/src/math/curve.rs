use super::precise_number::PreciseNumber;

// Constants for the default curve that goes from $0.01 to $1_000_000 over 21_000_000 tokens
pub const A: u128 = 11400_230149967394933471;
pub const B: u128 = 0_000000877175273521;
pub const C: u128 = 0_000000877175273521;

#[derive(Debug, Clone)]
pub struct PreciseExponentialCurve {
    pub a: PreciseNumber,
    pub b: PreciseNumber,
    pub c: PreciseNumber,
}

impl PreciseExponentialCurve {
    pub fn default() -> Self {
        Self {
            a: PreciseNumber::from_scaled_u128(A),
            b: PreciseNumber::from_scaled_u128(B),
            c: PreciseNumber::from_scaled_u128(C),
        }
    }

    /// Calculate token price at a given supply
    pub fn spot_price_at_supply(&self, current_supply: &PreciseNumber) -> Option<PreciseNumber> {
        // R'(S) = a * b * e^(c * s)

        let c_times_s = self.c.checked_mul(current_supply)?;
        let exp = c_times_s.signed().exp()?;
        self.a.checked_mul(&self.b)?.checked_mul(&exp)
    }

    /// Calculate total cost to buy `num_tokens` starting at `current_supply`
    /// “How much does it cost to get X tokens?”
    pub fn tokens_to_value(
        &self,
        current_supply: &PreciseNumber,
        tokens: &PreciseNumber,
    ) -> Option<PreciseNumber> {
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

    /// Calculate number of tokens received for a price `value` starting at `current_supply`
    /// “How many tokens can I get for Y value?”
    pub fn value_to_tokens(
        &self,
        current_supply: &PreciseNumber,
        value: &PreciseNumber,
    ) -> Option<PreciseNumber> {
        // num_tokens = (1/c) * ln(amount / (a * b / c) + e^(c * current_supply)) - current_supply

        let ab_over_c = self.a.checked_mul(&self.b)?.checked_div(&self.c)?;
        let exp_cs = self.c.checked_mul(current_supply)?.signed().exp()?;

        let term = value.checked_div(&ab_over_c)?.checked_add(&exp_cs)?;

        let ln_term = term.log()?.value;
        let result = ln_term.checked_div(&self.c)?.checked_sub(current_supply)?;

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::precise_number::PreciseNumber;

    fn assert_approx_eq(actual: &PreciseNumber, expected: &PreciseNumber, tolerance: u128) {
        let (diff, _) = actual.unsigned_sub(expected);
        let tol = PreciseNumber::from_scaled_u128(tolerance);
        assert!(
            diff.less_than_or_equal(&tol),
            "Mismatch: got {}, expected {}, diff = {}",
            actual.to_string(),
            expected.to_string(),
            diff.to_string()
        );
    }

    #[test]
    fn test_calculate_curve_constants() {
        const ONE_PENNY: u128 = 10_000_000_000_000_000; // $0.01 (starting price)
        const ONE_MILLION: u128 = 1_000_000_000_000_000_000_000_000; // $1_000_000 (ending price)
        const TWENTY_ONE_MILLION: u128 = 21_000_000_000_000_000_000_000_000; // 21_000_000 tokens

        let price_start = PreciseNumber::from_scaled_u128(ONE_PENNY);
        let price_end = PreciseNumber::from_scaled_u128(ONE_MILLION);
        let supply_diff = PreciseNumber::from_scaled_u128(TWENTY_ONE_MILLION);

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

        assert_approx_eq(&a, &PreciseNumber::from_scaled_u128(A), 0);
        assert_approx_eq(&b, &PreciseNumber::from_scaled_u128(B), 0);
        assert_approx_eq(&c, &PreciseNumber::from_scaled_u128(C), 0);
    }

    #[test]
    fn generate_curve_table() {
        let a = PreciseNumber::from_scaled_u128(A);
        let b = PreciseNumber::from_scaled_u128(B);
        let c = PreciseNumber::from_scaled_u128(C);

        let curve = PreciseExponentialCurve {
            a: a.clone(),
            b: b.clone(),
            c: c.clone(),
        };

        println!("|------|----------------|-----------------------------------|-----------------------------|");
        println!("| %    | S              | R(S)                              | R'(S)                       |");
        println!("|------|----------------|-----------------------------------|-----------------------------|");

        let zero = PreciseNumber::zero();
        let buy_amount = PreciseNumber::new(210000).unwrap();
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
        assert!(false);
    }
}
