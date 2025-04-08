use crate::uint::U192;
use solana_program::msg;
use std::convert::*;

// Allows for easy swapping between different internal representations
pub type InnerUint = U192;

pub static ONE_PREC: PreciseNumber = PreciseNumber { value: one() };
pub static ZERO_PREC: PreciseNumber = PreciseNumber { value: zero() };
pub static TWO_PREC: PreciseNumber = PreciseNumber { value: two() };

/// The representation of the number one as a precise number as 10^18
pub const ONE: u128 = 1_000_000_000_000_000_000;

/// Struct encapsulating a fixed-point number that allows for decimal calculations
#[derive(Clone, Debug, PartialEq)]
pub struct PreciseNumber {
    /// Wrapper over the inner value, which is multiplied by ONE
    pub value: InnerUint,
}

/// The precise-number 1 as a InnerUint. 24 decimals of precision
#[inline]
pub const fn one() -> InnerUint {
    // InnerUint::from(1_000_000_000_000_000_000_000_000_u128)
    U192([1000000000000000000_u64, 0_u64, 0_u64])
    // InnerUint::from(ONE)
}

#[inline]
pub const fn two() -> InnerUint {
    // InnerUint::from(1_000_000_000_000_000_000_000_000_u128)
    U192([2000000000000000000_u64, 0_u64, 0_u64])
    // InnerUint::from(ONE)
}

// 0.693147180369123816490000
#[inline]
pub const fn ln2hi() -> InnerUint {
    U192([13974485815783726801_u64, 3_u64, 0_u64])
}
pub const LN2HI: PreciseNumber = PreciseNumber { value: ln2hi() };
#[inline]

pub const fn ln2hi_scale() -> InnerUint {
    U192([7766279631452241920_u64, 5_u64, 0_u64])
}

pub const LN2HI_SCALE: PreciseNumber = PreciseNumber {
    value: ln2hi_scale(),
};

// 1.90821492927058770002e-10 /* 3dea39ef 35793c76 */
// Note that ln2lo is lower than our max precision, so we store both it and the thirty zeroes to scale by
#[inline]
pub const fn ln2lo() -> InnerUint {
    U192([3405790746697269248_u64, 1034445385942222_u64, 0_u64])
}
pub const LN2LO: PreciseNumber = PreciseNumber { value: ln2lo() };

#[inline]
pub const fn ln2lo_scale() -> InnerUint {
    U192([80237960548581376_u64, 10841254275107988496_u64, 293873_u64])
}

pub const LN2LO_SCALE: PreciseNumber = PreciseNumber {
    value: ln2lo_scale(),
};

// 6.666666666666735130e-01
#[inline]
pub const fn l1() -> InnerUint {
    U192([666666666666673513_u64, 0_u64, 0_u64])
}
pub const L1: PreciseNumber = PreciseNumber { value: l1() };

#[inline]
pub const fn l2() -> InnerUint {
    U192([399999999994094190_u64, 0_u64, 0_u64])
}
pub const L2: PreciseNumber = PreciseNumber { value: l2() };

#[inline]
pub const fn l3() -> InnerUint {
    U192([285714287436623914_u64, 0_u64, 0_u64])
}
pub const L3: PreciseNumber = PreciseNumber { value: l3() };

#[inline]
pub const fn l4() -> InnerUint {
    U192([222221984321497839_u64, 0_u64, 0_u64])
}
pub const L4: PreciseNumber = PreciseNumber { value: l4() };

#[inline]
pub const fn l5() -> InnerUint {
    U192([181835721616180501_u64, 0_u64, 0_u64])
}
pub const L5: PreciseNumber = PreciseNumber { value: l5() };

pub const fn l6() -> InnerUint {
    U192([153138376992093733_u64, 0_u64, 0_u64])
}
pub const L6: PreciseNumber = PreciseNumber { value: l6() };

#[inline]
pub const fn l7() -> InnerUint {
    U192([147981986051165859_u64, 0_u64, 0_u64])
}
pub const L7: PreciseNumber = PreciseNumber { value: l7() };

#[inline]
pub const fn sqrt2overtwo() -> InnerUint {
    U192([707106781186547600_u64, 0_u64, 0_u64])
}
pub const SQRT2OVERTWO: PreciseNumber = PreciseNumber {
    value: sqrt2overtwo(),
};

#[inline]
pub const fn half() -> InnerUint {
    U192([500000000000000000_u64, 0_u64, 0_u64])
}
pub const HALF: PreciseNumber = PreciseNumber { value: half() };

/// The number 0 as a PreciseNumber, used for easier calculations.
#[inline]
pub const fn zero() -> InnerUint {
    U192([0_u64, 0_u64, 0_u64])
}

impl PreciseNumber {
    pub fn signed(&self) -> SignedPreciseNumber {
        SignedPreciseNumber {
            value: self.clone(),
            is_negative: false,
        }
    }

    /// Correction to apply to avoid truncation errors on division.  Since
    /// integer operations will always floor the result, we artifically bump it
    /// up by one half to get the expect result.
    fn rounding_correction() -> InnerUint {
        InnerUint::from(ONE / 2)
    }

    pub fn zero() -> Self {
        Self { value: zero() }
    }

    pub fn one() -> Self {
        Self { value: one() }
    }

    /// Create a precise number from an imprecise u128, should always succeed
    pub fn new(value: u128) -> Option<Self> {
        let value = InnerUint::from(value).checked_mul(one())?;
        Some(Self { value })
    }

    pub fn from_scaled_u128(value: u128) -> Self {
        Self {
            value: InnerUint::from(value),
        }
    }

    /// Convert a precise number back to u128
    pub fn to_imprecise(&self) -> Option<u128> {
        self.value
            .checked_add(Self::rounding_correction())?
            .checked_div(one())
            .map(|v| v.as_u128())
    }

    /// Checks that two PreciseNumbers are equal within some tolerance
    pub fn almost_eq(&self, rhs: &Self, precision: InnerUint) -> bool {
        let (difference, _) = self.unsigned_sub(rhs);
        difference.value < precision
    }

    /// Checks that a number is less than another
    pub fn less_than(&self, rhs: &Self) -> bool {
        self.value < rhs.value
    }

    /// Checks that a number is greater than another
    pub fn greater_than(&self, rhs: &Self) -> bool {
        self.value > rhs.value
    }

    /// Checks that a number is less than another
    pub fn less_than_or_equal(&self, rhs: &Self) -> bool {
        self.value <= rhs.value
    }

    /// Checks that a number is greater than another
    pub fn greater_than_or_equal(&self, rhs: &Self) -> bool {
        self.value >= rhs.value
    }

    /// Floors a precise value to a precision of ONE
    pub fn floor(&self) -> Option<Self> {
        let value = self.value.checked_div(one())?.checked_mul(one())?;
        Some(Self { value })
    }

    /// Ceiling a precise value to a precision of ONE
    pub fn ceiling(&self) -> Option<Self> {
        let value = self
            .value
            .checked_add(one().checked_sub(InnerUint::from(1))?)?
            .checked_div(one())?
            .checked_mul(one())?;
        Some(Self { value })
    }

    /// Performs a checked division on two precise numbers
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

    /// Performs a multiplication on two precise numbers
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

    /// Performs addition of two precise numbers
    pub fn checked_add(&self, rhs: &Self) -> Option<Self> {
        let value = self.value.checked_add(rhs.value)?;
        Some(Self { value })
    }

    /// Subtracts the argument from self
    pub fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        let value = self.value.checked_sub(rhs.value)?;
        Some(Self { value })
    }

    pub fn unsigned_sub(&self, rhs: &Self) -> (Self, bool) {
        match self.value.checked_sub(rhs.value) {
            None => {
                let value = rhs.value.checked_sub(self.value).unwrap();
                (Self { value }, true)
            }
            Some(value) => (Self { value }, false),
        }
    }

    // Frexp breaks f into a normalized fraction
    // and an integral power of two.
    // It returns frac and exp satisfying f == frac × 2**exp,
    // with the absolute value of frac in the interval [½, 1).
    //
    // Special cases are:
    //	Frexp(±0) = ±0, 0
    //	Frexp(±Inf) = ±Inf, 0
    //	Frexp(NaN) = NaN, 0
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

    // Modified from the original to support precise numbers instead of floats
    /*
      Floating-point logarithm.
      Borrowed from https://arm-software.github.io/golang-utils/src/math/log.go.html
    */
    // The original C code, the long comment, and the constants
    // below are from FreeBSD's /usr/src/lib/msun/src/e_log.c
    // and came with this notice. The go code is a simpler
    // version of the original C.
    //
    // ====================================================
    // Copyright (C) 1993 by Sun Microsystems, Inc. All rights reserved.
    //
    // Developed at SunPro, a Sun Microsystems, Inc. business.
    // Permission to use, copy, modify, and distribute this
    // software is freely granted, provided that this notice
    // is preserved.
    // ====================================================
    //
    // __ieee754_log(x)
    // Return the logarithm of x
    //
    // Method :
    //   1. Argument Reduction: find k and f such that
    //			x = 2**k * (1+f),
    //	   where  sqrt(2)/2 < 1+f < sqrt(2) .
    //
    //   2. Approximation of log(1+f).
    //	Let s = f/(2+f) ; based on log(1+f) = log(1+s) - log(1-s)
    //		 = 2s + 2/3 s**3 + 2/5 s**5 + .....,
    //	     	 = 2s + s*R
    //      We use a special Reme algorithm on [0,0.1716] to generate
    //	a polynomial of degree 14 to approximate R.  The maximum error
    //	of this polynomial approximation is bounded by 2**-58.45. In
    //	other words,
    //		        2      4      6      8      10      12      14
    //	    R(z) ~ L1*s +L2*s +L3*s +L4*s +L5*s  +L6*s  +L7*s
    //	(the values of L1 to L7 are listed in the program) and
    //	    |      2          14          |     -58.45
    //	    | L1*s +...+L7*s    -  R(z) | <= 2
    //	    |                             |
    //	Note that 2s = f - s*f = f - hfsq + s*hfsq, where hfsq = f*f/2.
    //	In order to guarantee error in log below 1ulp, we compute log by
    //		log(1+f) = f - s*(f - R)		(if f is not too large)
    //		log(1+f) = f - (hfsq - s*(hfsq+R)).	(better accuracy)
    //
    //	3. Finally,  log(x) = k*Ln2 + log(1+f).
    //			    = k*Ln2_hi+(f-(hfsq-(s*(hfsq+R)+k*Ln2_lo)))
    //	   Here Ln2 is split into two floating point number:
    //			Ln2_hi + Ln2_lo,
    //	   where n*Ln2_hi is always exact for |n| < 2000.
    //
    // Special cases:
    //	log(x) is NaN with signal if x < 0 (including -INF) ;
    //	log(+INF) is +INF; log(0) is -INF with signal;
    //	log(NaN) is that NaN with no signal.
    //
    // Accuracy:
    //	according to an error analysis, the error is always less than
    //	1 ulp (unit in the last place).
    //
    // Constants:
    // The hexadecimal values are the intended ones for the following
    // constants. The decimal values may be used, provided that the
    // compiler will convert from decimal to binary accurately enough
    // to produce the hexadecimal values shown.
    // Frexp breaks f into a normalized fraction
    // and an integral power of two.
    // It returns frac and exp satisfying f == frac × 2**exp,
    // with the absolute value of frac in the interval [½, 1).
    //
    // Log returns the natural logarithm of x.
    //
    // Special cases are:
    //	Log(+Inf) = +Inf
    //	Log(0) = -Inf
    //	Log(x < 0) = NaN
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

    /*
    b = pow/frac
    y = a^b
    ln (y) = bln (a)
    y = e^(b ln (a))
    */
    pub fn pow(&self, exp: &Self) -> Option<Self> {
        if self.eq(&ZERO_PREC) {
            return Some(ZERO_PREC.clone());
        }

        let lg = self.log()?;
        let x = exp.signed().checked_mul(&lg)?;
        x.exp()
    }

    pub fn print(&self) {
        let whole = self.floor().unwrap().to_imprecise().unwrap();
        let decimals = self
            .checked_sub(&PreciseNumber::new(whole).unwrap())
            .unwrap()
            .checked_mul(&PreciseNumber::new(ONE).unwrap())
            .unwrap()
            .to_imprecise()
            .unwrap();
        msg!("{}.{:0>width$}", whole, decimals, width = 18);
    }

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
}

/// Struct encapsulating a signed fixed-point number that allows for decimal calculations
#[derive(Clone, Debug, PartialEq)]
pub struct SignedPreciseNumber {
    pub value: PreciseNumber,
    pub is_negative: bool,
}

#[inline]
pub const fn p1() -> InnerUint {
    U192([166666666666666019_u64, 0_u64, 0_u64])
}
pub const P1: SignedPreciseNumber = SignedPreciseNumber {
    value: PreciseNumber { value: p1() },
    is_negative: false,
};

#[inline]
pub const fn p2() -> InnerUint {
    U192([2777777777701559_u64, 0_u64, 0_u64])
}
pub const P2: SignedPreciseNumber = SignedPreciseNumber {
    value: PreciseNumber { value: p2() },
    is_negative: true,
};

#[inline]
pub const fn p3() -> InnerUint {
    U192([66137563214379_u64, 0_u64, 0_u64])
}
pub const P3: SignedPreciseNumber = SignedPreciseNumber {
    value: PreciseNumber { value: p3() },
    is_negative: false,
};

#[inline]
pub const fn p4() -> InnerUint {
    U192([1653390220546_u64, 0_u64, 0_u64])
}
pub const P4: SignedPreciseNumber = SignedPreciseNumber {
    value: PreciseNumber { value: p4() },
    is_negative: true,
};

#[inline]
pub const fn p5() -> InnerUint {
    U192([41381367970_u64, 0_u64, 0_u64])
}
pub const P5: SignedPreciseNumber = SignedPreciseNumber {
    value: PreciseNumber { value: p5() },
    is_negative: false,
};

#[inline]
pub const fn halfln2() -> InnerUint {
    U192([346573590279972640_u64, 0_u64, 0_u64])
}
pub const HALFLN2: PreciseNumber = PreciseNumber { value: halfln2() };

#[inline]
pub const fn threehalfln2() -> InnerUint {
    U192([1039720770839917900_u64, 0_u64, 0_u64])
}
pub const THREEHALFLN2: PreciseNumber = PreciseNumber {
    value: threehalfln2(),
};

#[inline]
pub const fn invln2() -> InnerUint {
    U192([1442695040888963387_u64, 0_u64, 0_u64])
}
pub const INVLN2: PreciseNumber = PreciseNumber { value: invln2() };

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

    pub fn print(&self) {
        self.value.print()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::curve2::half;

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

#[derive(Debug, Clone)]
pub struct PreciseExponentialCurve {
    pub a: PreciseNumber,
    pub b: PreciseNumber,
    pub c: PreciseNumber,
}

impl PreciseExponentialCurve {
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
        let ab_over_c = self.a.checked_mul(&self.b)?.checked_div(&self.c)?;
        let exp_cs = self.c.checked_mul(current_supply)?.signed().exp()?;

        let term = value.checked_div(&ab_over_c)?.checked_add(&exp_cs)?;

        let ln_term = term.log()?.value;
        let result = ln_term.checked_div(&self.c)?.checked_sub(current_supply)?;

        Some(result)
    }
}

pub fn to_precise(amount: f64) -> PreciseNumber {
    let scaled = (amount * 1e18).round() as u128;
    PreciseNumber::from_scaled_u128(scaled)
}

pub fn print_precise(label: &str, value: &PreciseNumber) {
    let whole = value.floor().unwrap().to_imprecise().unwrap();
    let fractional = value
        .checked_sub(&PreciseNumber::new(whole).unwrap())
        .unwrap()
        .checked_mul(&PreciseNumber::new(1_000_000).unwrap())
        .unwrap()
        .to_imprecise()
        .unwrap();
    println!("{}: {}.{:06}", label, whole, fractional);
}

pub fn precise_to_str(value: &PreciseNumber) -> String {
    let whole = value.floor().unwrap().to_imprecise().unwrap();
    let fractional = value
        .checked_sub(&PreciseNumber::new(whole).unwrap())
        .unwrap()
        .checked_mul(&PreciseNumber::new(1_000_000).unwrap())
        .unwrap()
        .to_imprecise()
        .unwrap();
    //println!("{}: {}.{:06}", label, whole, fractional);
    format!("{}.{}", whole, fractional)
}

pub fn msg_precise(label: &str, value: &PreciseNumber) {
    let whole = value.floor().unwrap().to_imprecise().unwrap();
    let fractional = value
        .checked_sub(&PreciseNumber::new(whole).unwrap())
        .unwrap()
        .checked_mul(&PreciseNumber::new(1_000_000).unwrap())
        .unwrap()
        .to_imprecise()
        .unwrap();
    solana_program::msg!("{}: {}.{:06}", label, whole, fractional);
}

#[cfg(test)]
mod tests2 {
    use super::*;

    //#[test]
    //fn test_precise_exponential_curve() {
    //    let a = to_precise(11400.2301);
    //    let b = to_precise(0.00000087717527);
    //    let c = to_precise(0.00000087717527);
    //    let usdc = to_precise(2306.0); // $2306.00
    //
    //    let curve = PreciseExponentialCurve {
    //        a: a.clone(),
    //        b: b.clone(),
    //        c: c.clone(),
    //    };
    //
    //    let current_supply = PreciseNumber::zero();
    //
    //    println!("usdc: {:?}", usdc);
    //    println!("a: {:?}", a);
    //    println!("b: {:?}", b);
    //    println!("c: {:?}", c);
    //
    //    let tokens = curve.value_to_tokens(&current_supply, &usdc).unwrap();
    //    let actual_cost = curve.tokens_to_value(&current_supply, &tokens).unwrap();
    //    let spot_price = curve.spot_price_at_supply(&current_supply).unwrap();
    //
    //    print_precise("Tokens received for $2306.00", &tokens);
    //    print_precise("Actual cost for those tokens", &actual_cost);
    //    print_precise("Spot price at supply 0", &spot_price);
    //
    //    let tolerance = InnerUint::from(10_000_000_000_000u128); // 0.01 precision
    //    assert!(actual_cost.almost_eq(&usdc, tolerance));
    //}


    /* 
    |------|------------|---------------------|---------------------|
    | %    | S          | R(S)                | R'(S)               |
    |------|------------|---------------------|---------------------|
    | 0%   | 0          | $0                  | $0.010000           |
    | 1%   | 210000     | $2,306              | $0.012023           |
    | 2%   | 420000     | $5,078              | $0.014454           |
    | 3%   | 630000     | $8,411              | $0.017378           |
    | 4%   | 840000     | $12,418             | $0.020893           |
    | 5%   | 1050000    | $17,236             | $0.025119           |
    | 6%   | 1260000    | $23,028             | $0.030200           |
    | 7%   | 1470000    | $29,992             | $0.036308           |
    | 8%   | 1680000    | $38,364             | $0.043652           |
    | 9%   | 1890000    | $48,429             | $0.052481           |
    | 10%  | 2100000    | $60,530             | $0.063096           |
    | 11%  | 2310000    | $75,079             | $0.075858           |
    | 12%  | 2520000    | $92,571             | $0.091201           |
    | 13%  | 2730000    | $113,601            | $0.109648           |
    | 14%  | 2940000    | $138,884            | $0.131826           |
    | 15%  | 3150000    | $169,281            | $0.158489           |
    | 16%  | 3360000    | $205,827            | $0.190546           |
    | 17%  | 3570000    | $249,764            | $0.229087           |
    | 18%  | 3780000    | $302,588            | $0.275423           |
    | 19%  | 3990000    | $366,097            | $0.331131           |
    | 20%  | 4200000    | $442,451            | $0.398107           |
    | 21%  | 4410000    | $534,249            | $0.478630           |
    | 22%  | 4620000    | $644,615            | $0.575440           |
    | 23%  | 4830000    | $777,303            | $0.691831           |
    | 24%  | 5040000    | $936,830            | $0.831764           |
    | 25%  | 5250000    | $1,128,623          | $1.000000           |
    | 26%  | 5460000    | $1,359,209          | $1.202264           |
    | 27%  | 5670000    | $1,636,434          | $1.445440           |
    | 28%  | 5880000    | $1,969,733          | $1.737801           |
    | 29%  | 6090000    | $2,370,445          | $2.089296           |
    | 30%  | 6300000    | $2,852,208          | $2.511886           |
    | 31%  | 6510000    | $3,431,414          | $3.019952           |
    | 32%  | 6720000    | $4,127,773          | $3.630780           |
    | 33%  | 6930000    | $4,964,981          | $4.365158           |
    | 34%  | 7140000    | $5,971,525          | $5.248074           |
    | 35%  | 7350000    | $7,181,658          | $6.309573           |
    | 36%  | 7560000    | $8,636,558          | $7.585775           |
    | 37%  | 7770000    | $10,385,733         | $9.120108           |
    | 38%  | 7980000    | $12,488,703         | $10.964782          |
    | 39%  | 8190000    | $15,017,029         | $13.182567          |
    | 40%  | 8400000    | $18,056,746         | $15.848931          |
    | 41%  | 8610000    | $21,711,290         | $19.054606          |
    | 42%  | 8820000    | $26,105,017         | $22.908676          |
    | 43%  | 9030000    | $31,387,440         | $27.542286          |
    | 44%  | 9240000    | $37,738,308         | $33.113111          |
    | 45%  | 9450000    | $45,373,732         | $39.810715          |
    | 46%  | 9660000    | $54,553,530         | $47.863007          |
    | 47%  | 9870000    | $65,590,074         | $57.543991          |
    | 48%  | 10080000   | $78,858,920         | $69.183094          |
    | 49%  | 10290000   | $94,811,580         | $83.176373          |
    | 50%  | 10500000   | $113,990,897        | $99.999995          |
    | 51%  | 10710000   | $137,049,507        | $120.226438         |
    | 52%  | 10920000   | $164,772,053        | $144.543970         |
    | 53%  | 11130000   | $198,101,885        | $173.780075         |
    | 54%  | 11340000   | $238,173,157        | $208.929603         |
    | 55%  | 11550000   | $286,349,421        | $251.188631         |
    | 56%  | 11760000   | $344,270,031        | $301.995157         |
    | 57%  | 11970000   | $413,905,919        | $363.078036         |
    | 58%  | 12180000   | $497,626,671        | $436.515810         |
    | 59%  | 12390000   | $598,281,154        | $524.807433         |
    | 60%  | 12600000   | $719,294,459        | $630.957311         |
    | 61%  | 12810000   | $864,784,451        | $758.577534         |
    | 62%  | 13020000   | $1,039,701,894      | $912.010790         |
    | 63%  | 13230000   | $1,249,998,915      | $1,096.478136       |
    | 64%  | 13440000   | $1,502,831,544      | $1,318.256665       |
    | 65%  | 13650000   | $1,806,803,221      | $1,584.893103       |
    | 66%  | 13860000   | $2,172,257,557      | $1,905.460609       |
    | 67%  | 14070000   | $2,611,630,307      | $2,290.867520       |
    | 68%  | 14280000   | $3,139,872,538      | $2,754.228542       |
    | 69%  | 14490000   | $3,774,959,385      | $3,311.311018       |
    | 70%  | 14700000   | $4,538,501,714      | $3,981.071466       |
    | 71%  | 14910000   | $5,456,481,499      | $4,786.300632       |
    | 72%  | 15120000   | $6,560,135,945      | $5,754.399019       |
    | 73%  | 15330000   | $7,887,020,433      | $6,918.309278       |
    | 74%  | 15540000   | $9,482,286,460      | $8,317.637186       |
    | 75%  | 15750000   | $11,400,218,067     | $9,999.999361       |
    | 76%  | 15960000   | $13,706,079,025     | $12,022.643570      |
    | 77%  | 16170000   | $16,478,333,644     | $14,454.396763      |
    | 78%  | 16380000   | $19,811,316,773     | $17,378.007139      |
    | 79%  | 16590000   | $23,818,443,847     | $20,892.959912      |
    | 80%  | 16800000   | $28,636,070,210     | $25,118.862618      |
    | 81%  | 17010000   | $34,428,131,041     | $30,199.515141      |
    | 82%  | 17220000   | $41,391,719,776     | $36,307.802970      |
    | 83%  | 17430000   | $49,763,794,844     | $43,651.580178      |
    | 84%  | 17640000   | $59,829,242,934     | $52,480.742324      |
    | 85%  | 17850000   | $71,930,573,182     | $63,095.729952      |
    | 86%  | 18060000   | $86,479,572,140     | $75,857.752041      |
    | 87%  | 18270000   | $103,971,316,134    | $91,201.077302      |
    | 88%  | 18480000   | $125,001,017,822    | $109,647.811558     |
    | 89%  | 18690000   | $150,284,280,213    | $131,825.664072     |
    | 90%  | 18900000   | $180,681,447,354    | $158,489.307367     |
    | 91%  | 19110000   | $217,226,880,294    | $190,546.057373     |
    | 92%  | 19320000   | $261,164,154,533    | $229,086.747767     |
    | 93%  | 19530000   | $313,988,376,666    | $275,422.849079     |
    | 94%  | 19740000   | $377,497,060,175    | $331,131.095683     |
    | 95%  | 19950000   | $453,851,291,592    | $398,107.139242     |
    | 96%  | 20160000   | $545,649,268,388    | $478,630.054324     |
    | 97%  | 20370000   | $656,014,710,979    | $575,439.891227     |
    | 98%  | 20580000   | $788,703,157,318    | $691,830.914970     |
    | 99%  | 20790000   | $948,229,757,119    | $831,763.703223     |
    | 100% | 21000000   | $1,140,022,914,292  | $999,999.917651     |
    |------|------------|---------------------|---------------------|
*/


#[test]
fn generate_curve_table() {
    let a = to_precise(11400.2301);
    let b = to_precise(0.00000087717527);
    let c = to_precise(0.00000087717527);

    let curve = PreciseExponentialCurve {
        a: a.clone(),
        b: b.clone(),
        c: c.clone(),
    };


    println!("|------|------------|---------------------|---------------------|");
    println!("| %    | S          | R(S)                | R'(S)               |");
    println!("|------|------------|---------------------|---------------------|");

    let mut supply = PreciseNumber::zero();

    for i in 0..101 {
        let buy_amount = PreciseNumber::new(210000).unwrap();
        let cost = curve.tokens_to_value(&PreciseNumber::zero(), &supply).unwrap();
        supply = supply.checked_add(&buy_amount.clone()).unwrap();
        let spot_price = curve.spot_price_at_supply(&supply).unwrap();

        println!(
            "| {:<4} | {} | ${} | ${} |",
            format!("{}%", i),
            &supply.to_string(),
            &cost.to_string(),
            &spot_price.to_string()
        );
    }

    println!("|------|------------|---------------------|---------------------|");
    assert!(false, "Curve2 table generated");
    }
}
