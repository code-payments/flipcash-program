use super::InnerUint;
use super::unsigned::PreciseNumber;
use super::signed::SignedPreciseNumber;

// PreciseNumber constants

/// The representation of the number one as a precise number as 10^18
pub const ONE: u128 = 1_000_000_000_000_000_000;

pub static ONE_PREC: PreciseNumber = PreciseNumber { value: one() };
pub static ZERO_PREC: PreciseNumber = PreciseNumber { value: zero() };
pub static TWO_PREC: PreciseNumber = PreciseNumber { value: two() };

/// Returns the internal representation of 1.0 in fixed-point format.
#[inline]
pub const fn one() -> InnerUint {
    InnerUint([ONE as u64, 0, 0])
}

/// Returns the internal representation of 2.0 in fixed-point format.
#[inline]
pub const fn two() -> InnerUint {
    InnerUint([2 * ONE as u64, 0, 0])
}

/// High part of ln(2), used for logarithmic calculations. Stored as a fixed-point number.
#[inline]
pub const fn ln2hi() -> InnerUint {
    InnerUint([13974485815783726801_u64, 3_u64, 0])
}
pub const LN2HI: PreciseNumber = PreciseNumber { value: ln2hi() };

/// Scaled variant of ln2hi for internal use in higher-precision approximations.
#[inline]
pub const fn ln2hi_scale() -> InnerUint {
    InnerUint([7766279631452241920_u64, 5_u64, 0])
}
pub const LN2HI_SCALE: PreciseNumber = PreciseNumber { value: ln2hi_scale() };

/// Low part of ln(2). Very small value, stored separately for better precision.
#[inline]
pub const fn ln2lo() -> InnerUint {
    // Note that ln2lo is lower than our max precision, so we store both it and the thirty zeroes to scale by
    InnerUint([3405790746697269248_u64, 1034445385942222_u64, 0])
}
pub const LN2LO: PreciseNumber = PreciseNumber { value: ln2lo() };

/// Scaled low part of ln(2), for use in precision-sensitive computations.
#[inline]
pub const fn ln2lo_scale() -> InnerUint {
    InnerUint([80237960548581376_u64, 10841254275107988496_u64, 293873_u64])
}
pub const LN2LO_SCALE: PreciseNumber = PreciseNumber { value: ln2lo_scale() };

/// Constant for sqrt(2)/2, useful in trig/log calculations.
#[inline]
pub const fn sqrt2overtwo() -> InnerUint {
    InnerUint([707106781186547600_u64, 0, 0])
}
pub const SQRT2OVERTWO: PreciseNumber = PreciseNumber { value: sqrt2overtwo() };

/// Fixed-point representation of 0.5 (HALF).
#[inline]
pub const fn half() -> InnerUint {
    InnerUint([500000000000000000_u64, 0, 0])
}
pub const HALF: PreciseNumber = PreciseNumber { value: half() };

/// Zero value in fixed-point format.
#[inline]
pub const fn zero() -> InnerUint {
    InnerUint([0, 0, 0])
}

// 6.666666666666735130e-01
#[inline]
pub const fn l1() -> InnerUint {
    InnerUint([666666666666673513_u64, 0_u64, 0_u64])
}
pub const L1: PreciseNumber = PreciseNumber { value: l1() };

#[inline]
pub const fn l2() -> InnerUint {
    InnerUint([399999999994094190_u64, 0_u64, 0_u64])
}
pub const L2: PreciseNumber = PreciseNumber { value: l2() };

#[inline]
pub const fn l3() -> InnerUint {
    InnerUint([285714287436623914_u64, 0_u64, 0_u64])
}
pub const L3: PreciseNumber = PreciseNumber { value: l3() };

#[inline]
pub const fn l4() -> InnerUint {
    InnerUint([222221984321497839_u64, 0_u64, 0_u64])
}
pub const L4: PreciseNumber = PreciseNumber { value: l4() };

#[inline]
pub const fn l5() -> InnerUint {
    InnerUint([181835721616180501_u64, 0_u64, 0_u64])
}
pub const L5: PreciseNumber = PreciseNumber { value: l5() };

pub const fn l6() -> InnerUint {
    InnerUint([153138376992093733_u64, 0_u64, 0_u64])
}
pub const L6: PreciseNumber = PreciseNumber { value: l6() };

#[inline]
pub const fn l7() -> InnerUint {
    InnerUint([147981986051165859_u64, 0_u64, 0_u64])
}
pub const L7: PreciseNumber = PreciseNumber { value: l7() };

// SignedPreciseNumber constants

#[inline]
pub const fn p1() -> InnerUint {
    InnerUint([166666666666666019_u64, 0_u64, 0_u64])
}
pub const P1: SignedPreciseNumber = SignedPreciseNumber {
    value: PreciseNumber { value: p1() },
    is_negative: false,
};

#[inline]
pub const fn p2() -> InnerUint {
    InnerUint([2777777777701559_u64, 0_u64, 0_u64])
}
pub const P2: SignedPreciseNumber = SignedPreciseNumber {
    value: PreciseNumber { value: p2() },
    is_negative: true,
};

#[inline]
pub const fn p3() -> InnerUint {
    InnerUint([66137563214379_u64, 0_u64, 0_u64])
}
pub const P3: SignedPreciseNumber = SignedPreciseNumber {
    value: PreciseNumber { value: p3() },
    is_negative: false,
};

#[inline]
pub const fn p4() -> InnerUint {
    InnerUint([1653390220546_u64, 0_u64, 0_u64])
}
pub const P4: SignedPreciseNumber = SignedPreciseNumber {
    value: PreciseNumber { value: p4() },
    is_negative: true,
};

#[inline]
pub const fn p5() -> InnerUint {
    InnerUint([41381367970_u64, 0_u64, 0_u64])
}
pub const P5: SignedPreciseNumber = SignedPreciseNumber {
    value: PreciseNumber { value: p5() },
    is_negative: false,
};

#[inline]
pub const fn halfln2() -> InnerUint {
    InnerUint([346573590279972640_u64, 0_u64, 0_u64])
}
pub const HALFLN2: PreciseNumber = PreciseNumber { value: halfln2() };

#[inline]
pub const fn threehalfln2() -> InnerUint {
    InnerUint([1039720770839917900_u64, 0_u64, 0_u64])
}
pub const THREEHALFLN2: PreciseNumber = PreciseNumber {
    value: threehalfln2(),
};

#[inline]
pub const fn invln2() -> InnerUint {
    InnerUint([1442695040888963387_u64, 0_u64, 0_u64])
}
pub const INVLN2: PreciseNumber = PreciseNumber { value: invln2() };
