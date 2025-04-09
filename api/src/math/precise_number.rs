use super::uint::U192;
use super::consts::*;
use std::convert::*;

pub type InnerUint = U192;

pub fn to_precise(amount: f64) -> PreciseNumber {
    let scaled = (amount * 1e18).round() as u128;
    PreciseNumber::from_scaled_u128(scaled)
}

/// A `PreciseNumber` is a 192-bit fixed-point number with 18 decimal places of precision.
///
/// ### Internal Representation
/// Internally, the value is stored as a [`U192`], which wraps a little-endian array `[u64; 3]`.
/// This means the layout is:
///
/// ```text
/// U192([lo, mid, hi])
/// // equivalent to:
/// // value = lo + (mid << 64) + (hi << 128)
/// ```
///
/// Each component contributes to the full 192-bit value:
///
/// ```text
/// value = (hi × 2^128) + (mid × 2^64) + lo
/// ```
///
/// ### Fixed-Point Scaling
/// All values are scaled by [`ONE`] (10^18). That is, the internal number is interpreted
/// as `raw / ONE` to recover its real-world value.
///
/// Examples:
/// - `U192([1_000_000_000_000_000_000, 0, 0])` → 1.0
/// - `U192([500_000_000_000_000_000, 0, 0])` → 0.5
/// - `U192([2_000_000_000_000_000_000, 0, 0])` → 2.0
///
/// ### Example: High-Bit Usage
///
/// When you write:
/// ```text
/// let a = PreciseNumber::from([0, 0, 1]);
/// ```
/// This initializes the internal 192-bit value with the array `[0, 0, 1]`.
/// In this representation:
///
/// - `0` is the least significant 64 bits (`lo`)
/// - `0` is the middle 64 bits (`mid`)
/// - `1` is the most significant 64 bits (`hi`)
///
/// The actual 192-bit value is computed as:
///
/// ```text
/// value = (1 × 2^128) + (0 × 2^64) + 0 = 2^128
///       = 340282366920938463463374607431768211456
/// ```
///
/// Since this is a fixed-point number, the real-world value is:
///
/// ```text
/// real_value = value / 10^18 = 340282366920938463463.374607431768211456
/// ```
///
/// This system allows for both extremely high precision and a vast dynamic range,
/// making [`PreciseNumber`] ideal for financial, scientific, or blockchain applications
/// where `f64` or even `u128` would lose accuracy or overflow.
#[derive(Clone, Debug, PartialEq)]
pub struct PreciseNumber {
    /// Internal value stored as a 192-bit integer, scaled by ONE (10^18).
    pub value: InnerUint,
}

impl PreciseNumber {

    /// Returns a `PreciseNumber` representing 0.0.
    pub fn zero() -> Self {
        Self { value: zero() }
    }

    /// Returns a `PreciseNumber` representing 1.0.
    pub fn one() -> Self {
        Self { value: one() }
    }

    /// Constructs a `PreciseNumber` from an integer value by scaling it by ONE (10^18).
    /// For example, `new(7)` produces `7.0`.
    /// Returns None on overflow during scaling.
    pub fn new(value: u128) -> Option<Self> {
        let value = InnerUint::from(value).checked_mul(one())?;
        Some(Self { value })
    }

    /// Constructs a `PreciseNumber` from a `u128` that is already scaled by ONE (i.e. in fixed-point space).
    /// This bypasses internal multiplication and is useful for constants or pre-scaled data.
    pub fn from_scaled_u128(value: u128) -> Self {
        Self {
            value: InnerUint::from(value),
        }
    }

    /// Constructs a `PreciseNumber` directly from a raw `[u64; 3]` value.
    /// The input is interpreted as already scaled (fixed-point).
    /// Layout is little-endian: `[lo, mid, hi]` = `lo + (mid << 64) + (hi << 128)`.
    pub fn from_u192(value: [u64; 3]) -> Self {
        Self {
            value: U192(value),
        }
    }

    /// Converts this `PreciseNumber` into a regular `u128` by dividing by ONE.
    /// Applies rounding correction to avoid always flooring the result.
    /// Returns `None` if the division would overflow or the result exceeds `u128::MAX`.
    pub fn to_imprecise(&self) -> Option<u128> {
        self.value
            .checked_add(Self::rounding_correction())?
            .checked_div(one())
            .map(|v| v.as_u128())
    }

    /// Converts this `PreciseNumber` into a signed version,
    /// wrapping it in a `SignedPreciseNumber` with `is_negative = false`.
    /// Useful when beginning arithmetic that may result in negative values.
    pub fn signed(&self) -> SignedPreciseNumber {
        SignedPreciseNumber {
            value: self.clone(),
            is_negative: false,
        }
    }

    /// Returns a rounding correction (0.5) used in division/multiplication
    /// to mitigate truncation from integer floor behavior.
    /// This simulates "round-to-nearest" by adding half the divisor.
    fn rounding_correction() -> InnerUint {
        InnerUint::from(ONE / 2)
    }

    /// Compares two `PreciseNumber`s for approximate equality,
    /// allowing for a configurable `precision` window.
    pub fn almost_eq(&self, rhs: &Self, precision: InnerUint) -> bool {
        let (difference, _) = self.unsigned_sub(rhs);
        difference.value < precision
    }

    /// Returns `true` if `self < rhs` in fixed-point terms.
    pub fn less_than(&self, rhs: &Self) -> bool {
        self.value < rhs.value
    }

    /// Returns `true` if `self > rhs`.
    pub fn greater_than(&self, rhs: &Self) -> bool {
        self.value > rhs.value
    }

    /// Returns `true` if `self <= rhs`.
    pub fn less_than_or_equal(&self, rhs: &Self) -> bool {
        self.value <= rhs.value
    }

    /// Returns `true` if `self >= rhs`.
    pub fn greater_than_or_equal(&self, rhs: &Self) -> bool {
        self.value >= rhs.value
    }

    /// Rounds down to the nearest whole number by truncating fractional digits.
    pub fn floor(&self) -> Option<Self> {
        let value = self.value.checked_div(one())?.checked_mul(one())?;
        Some(Self { value })
    }

    /// Rounds up to the nearest whole number.
    pub fn ceiling(&self) -> Option<Self> {
        let value = self
            .value
            .checked_add(one().checked_sub(InnerUint::from(1))?)?
            .checked_div(one())?
            .checked_mul(one())?;
        Some(Self { value })
    }

    /// Divides `self / rhs` in fixed-point space, maintaining precision.
    /// Applies rounding correction to minimize truncation error.
    /// Returns `None` on divide-by-zero or overflow.
    pub fn checked_div(&self, rhs: &Self) -> Option<Self> {
        if *rhs == Self::zero() {
            return None;
        }
        match self.value.checked_mul(one()) {
            Some(v) => {
                let value = v
                    .checked_add(Self::rounding_correction())?
                    .checked_div(rhs.value)?;
                Some(Self { value })
            }
            None => {
                let value = self
                    .value
                    .checked_add(Self::rounding_correction())?
                    .checked_div(rhs.value)?
                    .checked_mul(one())?;
                Some(Self { value })
            }
        }
    }

    /// Multiplies two `PreciseNumber`s and returns the result in fixed-point space.
    /// Automatically divides by ONE to maintain correct scaling, and applies rounding correction.
    /// Falls back to a reduced-precision path if full multiplication would overflow.
    pub fn checked_mul(&self, rhs: &Self) -> Option<Self> {
        match self.value.checked_mul(rhs.value) {
            Some(v) => {
                let value = v
                    .checked_add(Self::rounding_correction())?
                    .checked_div(one())?;
                Some(Self { value })
            }
            None => {
                let value = if self.value >= rhs.value {
                    self.value.checked_div(one())?.checked_mul(rhs.value)?
                } else {
                    rhs.value.checked_div(one())?.checked_mul(self.value)?
                };
                Some(Self { value })
            }
        }
    }

    /// Adds two precise numbers. Returns `None` on overflow.
    pub fn checked_add(&self, rhs: &Self) -> Option<Self> {
        let value = self.value.checked_add(rhs.value)?;
        Some(Self { value })
    }

    /// Subtracts `rhs` from `self`. Returns `None` if the result would be negative.
    pub fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        let value = self.value.checked_sub(rhs.value)?;
        Some(Self { value })
    }

    /// Computes the absolute difference between two numbers.
    /// Returns the result and a boolean indicating whether the result was originally negative.
    pub fn unsigned_sub(&self, rhs: &Self) -> (Self, bool) {
        match self.value.checked_sub(rhs.value) {
            None => {
                let value = rhs.value.checked_sub(self.value).unwrap();
                (Self { value }, true)
            }
            Some(value) => (Self { value }, false),
        }
    }

    /// Converts the precise number into a human-readable decimal string with full 18-digit precision.
    ///
    /// For example, a number representing 3.1415 will be displayed as:
    /// `"3.141500000000000000"`
    pub fn to_string(&self) -> String {
        let whole = self.floor().unwrap().to_imprecise().unwrap();
        let decimals = self
            .checked_sub(&PreciseNumber::new(whole).unwrap())
            .unwrap()
            .checked_mul(&PreciseNumber::new(ONE).unwrap())
            .unwrap()
            .to_imprecise()
            .unwrap();
        format!("{}.{:0>width$}", whole, decimals, width = 18)
    }

    /// Frexp breaks f into a normalized fraction and an integral power of two. It returns frac and
    /// exp satisfying f == frac × 2**exp, with the absolute value of frac in the interval [½, 1).
    ///
    /// Special cases are:
    ///	Frexp(±0) = ±0, 0
    ///	Frexp(±Inf) = ±Inf, 0
    ///	Frexp(NaN) = NaN, 0
    fn frexp(&self) -> Option<(Self, i64)> {
        if self.eq(&ZERO_PREC) {
            Some((ZERO_PREC.clone(), 0))
        } else if self.less_than(&ONE_PREC) {
            let first_leading = self.value.0[0].leading_zeros();
            let one_leading = ONE_PREC.value.0[0].leading_zeros();
            let bits = i64::from(first_leading.checked_sub(one_leading).unwrap());
            let frac = PreciseNumber {
                value: self.value << bits,
            };
            if frac.less_than(&HALF) {
                Some((frac.checked_mul(&TWO_PREC).unwrap(), -bits - 1))
            } else {
                Some((frac, -bits))
            }
        } else {
            let bits = 128_i64.checked_sub(i64::from(self.to_imprecise()?.leading_zeros()))?;
            let frac = PreciseNumber {
                value: self.value >> bits,
            };
            if frac.less_than(&HALF) {
                Some((frac.checked_mul(&TWO_PREC).unwrap(), bits - 1))
            } else {
                Some((frac, bits))
            }
        }
    }

    /// Modified from the original to support precise numbers instead of floats
    /// The original C code, the long comment, and the constants
    /// below are from FreeBSD's /usr/src/lib/msun/src/e_log.c
    /// and came with this notice. The go code is a simpler
    /// version of the original C.
    ///
    /// ====================================================
    /// Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
    ///
    /// Developed at SunPro, a Sun Microsystems, Inc. business.
    /// Permission to use, copy, modify, and distribute this
    /// software is freely granted, provided that this notice
    /// is preserved.
    /// ====================================================
    ///
    /// __ieee754_log(x)
    /// Return the logarithm of x
    ///
    /// Method :
    ///   1. Argument Reduction: find k and f such that
    ///			x = 2**k * (1+f),
    ///	   where  sqrt(2)/2 < 1+f < sqrt(2) .
    ///
    ///   2. Approximation of log(1+f).
    ///	Let s = f/(2+f) ; based on log(1+f) = log(1+s) - log(1-s)
    ///		 = 2s + 2/3 s**3 + 2/5 s**5 + .....,
    ///	     	 = 2s + s*R
    ///      We use a special Reme algorithm on [0,0.1716] to generate
    ///	a polynomial of degree 14 to approximate R.  The maximum error
    ///	of this polynomial approximation is bounded by 2**-58.45. In
    ///	other words,
    ///		        2      4      6      8      10      12      14
    ///	    R(z) ~ L1*s +L2*s +L3*s +L4*s +L5*s  +L6*s  +L7*s
    ///	(the values of L1 to L7 are listed in the program) and
    ///	    |      2          14          |     -58.45
    ///	    | L1*s +...+L7*s    -  R(z) | <= 2
    ///	    |                             |
    ///	Note that 2s = f - s*f = f - hfsq + s*hfsq, where hfsq = f*f/2.
    ///	In order to guarantee error in log below 1ulp, we compute log by
    ///		log(1+f) = f - s*(f - R)		(if f is not too large)
    ///		log(1+f) = f - (hfsq - s*(hfsq+R)).	(better accuracy)
    ///
    ///	3. Finally,  log(x) = k*Ln2 + log(1+f).
    ///			    = k*Ln2_hi+(f-(hfsq-(s*(hfsq+R)+k*Ln2_lo)))
    ///	   Here Ln2 is split into two floating point number:
    ///			Ln2_hi + Ln2_lo,
    ///	   where n*Ln2_hi is always exact for |n| < 2000.
    ///
    /// Special cases:
    ///	log(x) is NaN with signal if x < 0 (including -INF) ;
    ///	log(+INF) is +INF; log(0) is -INF with signal;
    ///	log(NaN) is that NaN with no signal.
    ///
    /// Accuracy:
    ///	according to an error analysis, the error is always less than
    ///	1 ulp (unit in the last place).
    ///
    /// Constants:
    /// The hexadecimal values are the intended ones for the following
    /// constants. The decimal values may be used, provided that the
    /// compiler will convert from decimal to binary accurately enough
    /// to produce the hexadecimal values shown.
    /// Frexp breaks f into a normalized fraction
    /// and an integral power of two.
    /// It returns frac and exp satisfying f == frac × 2**exp,
    /// with the absolute value of frac in the interval [½, 1).
    ///
    /// Log returns the natural logarithm of x.
    ///
    /// Special cases are:
    ///	Log(+Inf) = +Inf
    ///	Log(0) = -Inf
    ///	Log(x < 0) = NaN
    pub fn log(&self) -> Option<SignedPreciseNumber> {
        if self.eq(&ZERO_PREC) {
            return None;
        }

        if self.eq(&ONE_PREC) {
            return Some(SignedPreciseNumber {
                value: ZERO_PREC.clone(),
                is_negative: false,
            });
        }

        let (f1_init, ki_init) = self.frexp()?;

        let (f1, ki) = if f1_init.less_than(&SQRT2OVERTWO) {
            let new_f1 = f1_init.checked_mul(&TWO_PREC)?;
            let new_k1 = ki_init.checked_sub(1)?;
            (new_f1, new_k1)
        } else {
            (f1_init, ki_init)
        };

        let f = f1.signed().checked_sub(&PreciseNumber::one().signed())?;

        let s_divisor = PreciseNumber { value: two() }.signed().checked_add(&f)?;
        let s = &f.checked_div(&s_divisor)?;
        let s2 = s.checked_mul(s)?.value;
        let s4 = s2.checked_mul(&s2)?;
        // s2 * (L1 + s4*(L3+s4*(L5+s4*L7)))
        let t1 = s2.checked_mul(&L1.checked_add(&s4.checked_mul(
            &L3.checked_add(&s4.checked_mul(&L5.checked_add(&s4.checked_mul(&L7)?)?)?)?,
        )?)?)?;

        // s4 * (L2 + s4*(L4+s4*L6))
        let t2 = s4.checked_mul(
            &L2.checked_add(&s4.checked_mul(&L4.checked_add(&s4.checked_mul(&L6)?)?)?)?,
        )?;

        let r = t1.checked_add(&t2)?;
        let hfsq = f
            .checked_mul(&f)?
            .checked_div(&PreciseNumber { value: two() }.signed())?;
        let k = SignedPreciseNumber {
            value: PreciseNumber::new(u128::try_from(ki.abs()).ok()?)?,
            is_negative: ki < 0,
        };

        // k*Ln2Hi - ((hfsq - (s*(hfsq+R) + k*Ln2Lo)) - f)
        let kl2hi = k
            .checked_mul(&LN2HI.signed())?
            .checked_div(&LN2HI_SCALE.signed())?;
        let shfsqr = s.checked_mul(&hfsq.checked_add(&r.signed())?)?;
        let kl2lo = k
            .checked_mul(&LN2LO.signed())?
            .checked_div(&LN2LO_SCALE.signed())?;

        let shfsqr_kl2lo = shfsqr.checked_add(&kl2lo)?;
        let hfsq_shfsqr_kl2lo = hfsq.checked_sub(&shfsqr_kl2lo)?;
        let f_hfsq_shfsqr_kl2lo = hfsq_shfsqr_kl2lo.checked_sub(&f)?;

        kl2hi.checked_sub(&f_hfsq_shfsqr_kl2lo)
    }

    /// Raises `self` to the power of `exp`, returning the result as a new `PreciseNumber`.
    /// Returns `None` if the operation would overflow or if `self` is zero.
    ///
    /// b = pow/frac
    /// y = a^b
    /// ln (y) = bln (a)
    /// y = e^(b ln (a))
    pub fn pow(&self, exp: &Self) -> Option<Self> {
        if self.eq(&ZERO_PREC) {
            return Some(ZERO_PREC.clone());
        }

        let lg = self.log()?;
        let x = exp.signed().checked_mul(&lg)?;
        x.exp()
    }
}

/// Struct encapsulating a signed fixed-point number that allows for decimal calculations
#[derive(Clone, Debug, PartialEq)]
pub struct SignedPreciseNumber {
    pub value: PreciseNumber,
    pub is_negative: bool,
}

impl SignedPreciseNumber {
    pub fn negate(&self) -> SignedPreciseNumber {
        SignedPreciseNumber {
            value: self.value.clone(),
            is_negative: !self.is_negative,
        }
    }

    pub fn checked_mul(&self, rhs: &Self) -> Option<SignedPreciseNumber> {
        Some(SignedPreciseNumber {
            value: self.value.checked_mul(&rhs.value)?,
            is_negative: (self.is_negative || rhs.is_negative)
                && !(self.is_negative && rhs.is_negative),
        })
    }

    pub fn checked_div(&self, rhs: &Self) -> Option<SignedPreciseNumber> {
        Some(SignedPreciseNumber {
            value: self.value.checked_div(&rhs.value)?,
            is_negative: (self.is_negative || rhs.is_negative)
                && !(self.is_negative && rhs.is_negative),
        })
    }

    pub fn checked_add(&self, rhs: &Self) -> Option<SignedPreciseNumber> {
        let lhs_negative = self.is_negative;
        let rhs_negative = rhs.is_negative;

        if rhs_negative && lhs_negative {
            Some(Self {
                value: self.value.checked_add(&rhs.value)?,
                is_negative: true,
            })
        } else if rhs_negative {
            if rhs.value.greater_than(&self.value) {
                Some(Self {
                    value: rhs.value.checked_sub(&self.value)?,
                    is_negative: true,
                })
            } else {
                Some(Self {
                    value: self.value.checked_sub(&rhs.value)?,
                    is_negative: false,
                })
            }
        } else if lhs_negative {
            if self.value.greater_than(&rhs.value) {
                Some(Self {
                    value: self.value.checked_sub(&rhs.value)?,
                    is_negative: true,
                })
            } else {
                Some(Self {
                    value: rhs.value.checked_sub(&self.value)?,
                    is_negative: false,
                })
            }
        } else {
            Some(Self {
                value: self.value.checked_add(&rhs.value)?,
                is_negative: false,
            })
        }
    }

    pub fn checked_sub(&self, rhs: &Self) -> Option<SignedPreciseNumber> {
        self.checked_add(&rhs.clone().negate())
    }

    pub fn floor(&self) -> Option<SignedPreciseNumber> {
        Some(Self {
            value: self.value.floor()?,
            is_negative: self.is_negative,
        })
    }

    // Modified from the original to support precise numbers instead of floats
    /* origin: FreeBSD /usr/src/lib/msun/src/e_exp.c */
    /*
     * ====================================================
     * Copyright (C) 2004 by Sun Microsystems, Inc. All rights reserved.
     *
     * Permission to use, copy, modify, and distribute this
     * software is freely granted, provided that this notice
     * is preserved.
     * ====================================================
     */
    /* exp(x)
     * Returns the exponential of x.
     *
     * Method
     *   1. Argument reduction:
     *      Reduce x to an r so that |r| <= 0.5*ln2 ~ 0.34658.
     *      Given x, find r and integer k such that
     *
     *               x = k*ln2 + r,  |r| <= 0.5*ln2.
     *
     *      Here r will be represented as r = hi-lo for better
     *      accuracy.
     *
     *   2. Approximation of exp(r) by a special rational function on
     *      the interval [0,0.34658]:
     *      Write
     *          R(r**2) = r*(exp(r)+1)/(exp(r)-1) = 2 + r*r/6 - r**4/360 + ...
     *      We use a special Remez algorithm on [0,0.34658] to generate
     *      a polynomial of degree 5 to approximate R. The maximum error
     *      of this polynomial approximation is bounded by 2**-59. In
     *      other words,
     *          R(z) ~ 2.0 + P1*z + P2*z**2 + P3*z**3 + P4*z**4 + P5*z**5
     *      (where z=r*r, and the values of P1 to P5 are listed below)
     *      and
     *          |                  5          |     -59
     *          | 2.0+P1*z+...+P5*z   -  R(z) | <= 2
     *          |                             |
     *      The computation of exp(r) thus becomes
     *                              2*r
     *              exp(r) = 1 + ----------
     *                            R(r) - r
     *                                 r*c(r)
     *                     = 1 + r + ----------- (for better accuracy)
     *                                2 - c(r)
     *      where
     *                              2       4             10
     *              c(r) = r - (P1*r  + P2*r  + ... + P5*r   ).
     *
     *   3. Scale back to obtain exp(x):
     *      From step 1, we have
     *         exp(x) = 2^k * exp(r)
     *
     * Special cases:
     *      exp(INF) is INF, exp(NaN) is NaN;
     *      exp(-INF) is 0, and
     *      for finite argument, only exp(0)=1 is exact.
     *
     * Accuracy:
     *      according to an error analysis, the error is always less than
     *      1 ulp (unit in the last place).
     *
     * Misc. info.
     *      For IEEE double
     *          if x >  709.782712893383973096 then exp(x) overflows
     *          if x < -745.133219101941108420 then exp(x) underflows
     */

    /// Calculate the exponential of `x`, that is, *e* raised to the power `x`
    /// (where *e* is the base of the natural system of logarithms, approximately 2.71828).
    /// Note that precision can start to get inaccurate for larger numbers (> 20).
    pub fn exp(&self) -> Option<PreciseNumber> {
        let hi: Self;
        let lo: Self;
        let k: Self;
        let x: Self;

        /* argument reduction */
        /* if |x| > 0.5 ln2 */
        if self.value.greater_than(&HALFLN2) {
            /* if |x| >= 1.5 ln2 */
            if self.value.greater_than_or_equal(&THREEHALFLN2) {
                k = INVLN2
                    .signed()
                    .checked_mul(self)?
                    .checked_add(&Self {
                        value: HALF,
                        is_negative: self.is_negative,
                    })?
                    .floor()?;

                // A K larger than this value will cause less than 9 decimals of precision
                // if k.value.to_imprecise()? > 29 {
                //   return None
                // }
            } else {
                k = Self {
                    value: PreciseNumber::one(),
                    is_negative: self.is_negative,
                }
            }
            hi = self.checked_sub(
                &k.checked_mul(&LN2HI.signed())?
                    .checked_div(&LN2HI_SCALE.signed())?,
            )?;

            lo = k
                .checked_mul(&LN2LO.signed())?
                .checked_div(&LN2LO_SCALE.signed())?;
            x = hi.checked_sub(&lo)?
        } else {
            x = self.clone();
            k = PreciseNumber::zero().signed();
            hi = self.clone();
            lo = PreciseNumber::zero().signed()
        }

        /* x is now in primary range */
        let xx = x.checked_mul(&x)?;
        // c = x - xx * (P1 + xx * (P2 + xx * (P3 + xx * (P4 + xx * P5))));
        let p4p5 = P4.checked_add(&xx.checked_mul(&P5)?)?;
        let p3p4p5 = P3.checked_add(&xx.checked_mul(&p4p5)?)?;
        let p2p3p4p5 = P2.checked_add(&xx.checked_mul(&p3p4p5)?)?;
        let p1p2p3p4p5 = P1.checked_add(&xx.checked_mul(&p2p3p4p5)?)?;
        let c = x.checked_sub(&p1p2p3p4p5.checked_mul(&xx)?)?;

        // y = 1. + (x * c / (2. - c) - lo + hi);
        let y = ONE_PREC.signed().checked_add(
            &x.checked_mul(&c)?
                .checked_div(&TWO_PREC.signed().checked_sub(&c)?)?
                .checked_sub(&lo)?
                .checked_add(&hi)?,
        )?;

        if k.value.eq(&PreciseNumber::zero()) {
            Some(y.value)
        } else {
            let bits = k.value.to_imprecise()?;

            if k.is_negative {
                Some(PreciseNumber {
                    value: y.value.value >> bits,
                })
            } else {
                Some(PreciseNumber {
                    value: y.value.value << bits,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::precise_number::half;

    #[test]
    fn test_pow() {
        let precision = InnerUint::from(5_000_000_000_000_u128); // correct to at least 12 decimal places
        let test = PreciseNumber::new(8).unwrap();
        let sqrt = test.pow(&HALF).unwrap();
        let expected = PreciseNumber::new(28284271247461903)
            .unwrap()
            .checked_div(&PreciseNumber::new(10000000000000000).unwrap())
            .unwrap();
        assert!(sqrt.almost_eq(&expected, precision));

        let test2 = PreciseNumber::new(55)
            .unwrap()
            .checked_div(&PreciseNumber::new(100).unwrap())
            .unwrap();
        let squared = test2.pow(&TWO_PREC).unwrap();
        let expected = PreciseNumber::new(3025)
            .unwrap()
            .checked_div(&PreciseNumber::new(10000).unwrap())
            .unwrap();
        assert!(squared.almost_eq(&expected, precision));
    }

    #[test]
    fn test_log() {
        let precision = InnerUint::from(5_000_000_000_u128); // correct to at least 9 decimal places

        let test = PreciseNumber::new(9).unwrap();
        let log = test.log().unwrap().value;
        let expected = PreciseNumber::new(21972245773362196)
            .unwrap()
            .checked_div(&PreciseNumber::new(10000000000000000).unwrap())
            .unwrap();
        assert!(log.almost_eq(&expected, precision));

        let test2 = PreciseNumber::new(2).unwrap();
        assert!(test2.log().unwrap().value.almost_eq(
            &PreciseNumber::new(6931471805599453)
                .unwrap()
                .checked_div(&PreciseNumber::new(10000000000000000).unwrap())
                .unwrap(),
            precision
        ));

        let test3 = &PreciseNumber::new(12)
            .unwrap()
            .checked_div(&PreciseNumber::new(10).unwrap())
            .unwrap();
        assert!(test3.log().unwrap().value.almost_eq(
            &PreciseNumber::new(1823215567939546)
                .unwrap()
                .checked_div(&PreciseNumber::new(10000000000000000).unwrap())
                .unwrap(),
            precision
        ));

        let test5 = &PreciseNumber::new(15)
            .unwrap()
            .checked_div(&PreciseNumber::new(10).unwrap())
            .unwrap();
        assert!(test5.log().unwrap().value.almost_eq(
            &PreciseNumber::new(4054651081081644)
                .unwrap()
                .checked_div(&PreciseNumber::new(10000000000000000).unwrap())
                .unwrap(),
            precision
        ));

        let test6 = PreciseNumber::new(4)
            .unwrap()
            .checked_div(&PreciseNumber::new(1000000).unwrap())
            .unwrap();
        assert!(test6.log().unwrap().value.almost_eq(
            &PreciseNumber::new(12429216196844383)
                .unwrap()
                .checked_div(&PreciseNumber::new(1000000000000000).unwrap())
                .unwrap(),
            precision
        ));
    }

    #[test]
    fn test_floor() {
        let whole_number = PreciseNumber::new(2).unwrap();
        let mut decimal_number = PreciseNumber::new(2).unwrap();
        decimal_number.value += InnerUint::from(1);
        let floor = decimal_number.floor().unwrap();
        let floor_again = floor.floor().unwrap();
        assert_eq!(whole_number.value, floor.value);
        assert_eq!(whole_number.value, floor_again.value);
    }

    #[test]
    fn test_ceiling() {
        let whole_number = PreciseNumber::new(2).unwrap();
        let mut decimal_number = PreciseNumber::new(2).unwrap();
        decimal_number.value -= InnerUint::from(1);
        let ceiling = decimal_number.ceiling().unwrap();
        let ceiling_again = ceiling.ceiling().unwrap();
        assert_eq!(whole_number.value, ceiling.value);
        assert_eq!(whole_number.value, ceiling_again.value);
    }

    #[test]
    fn test_signed_exp() {
        let precision = InnerUint::from(1_000_000_000_u128); // correct to at least 9 decimal places

        let half = PreciseNumber { value: half() }.signed();
        assert!(half.exp().unwrap().almost_eq(
            &PreciseNumber::new(16487212707001282)
                .unwrap()
                .checked_div(&PreciseNumber::new(10000000000000000).unwrap())
                .unwrap(),
            precision
        ));

        let three_half = PreciseNumber::new(15)
            .unwrap()
            .checked_div(&PreciseNumber::new(10).unwrap())
            .unwrap()
            .signed();
        assert!(three_half.exp().unwrap().almost_eq(
            &PreciseNumber::new(44816890703380645)
                .unwrap()
                .checked_div(&PreciseNumber::new(10000000000000000).unwrap())
                .unwrap(),
            precision
        ));

        let point_one = PreciseNumber::new(1)
            .unwrap()
            .checked_div(&PreciseNumber::new(10).unwrap())
            .unwrap()
            .signed();
        assert!(point_one.exp().unwrap().almost_eq(
            &PreciseNumber::new(11051709180756477)
                .unwrap()
                .checked_div(&PreciseNumber::new(10000000000000000).unwrap())
                .unwrap(),
            precision
        ));

        let negative = PreciseNumber::new(55)
            .unwrap()
            .checked_div(&PreciseNumber::new(100).unwrap())
            .unwrap()
            .signed()
            .negate();
        assert!(negative.exp().unwrap().almost_eq(
            &PreciseNumber::new(5769498103804866)
                .unwrap()
                .checked_div(&PreciseNumber::new(10000000000000000).unwrap())
                .unwrap(),
            precision
        ));

        let test = PreciseNumber::new(19).unwrap().signed();
        assert!(test.exp().unwrap().almost_eq(
            &PreciseNumber::new(178482300963187260)
                .unwrap()
                .checked_div(&PreciseNumber::new(1000000000).unwrap())
                .unwrap(),
            precision
        ));
    }
}
