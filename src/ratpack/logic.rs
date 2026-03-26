// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Bitwise logic operations on rationals.
//! Port of C++ Ratpack/logic.cpp

use crate::error::{CalcError, CalcResult};
use crate::types::BASEX;

use super::arithmetic::{
    add_rat, div_rat, mul_num_x, mul_rat, rat_gt, rat_lt, rat_pow_i32, rem_num,
};
use super::support::int_rat;
use super::Number;
use super::Rational;

/// Maximum shift exponent (matches C++ `rat_max_exp`).
const MAX_SHIFT_EXP: i32 = 100_000;

/// Selector for the bitwise operation in [`bool_num`].
#[derive(Debug, Clone, Copy)]
enum BoolFunc {
    And,
    Or,
    Xor,
}

// ---------------------------------------------------------------------------
// lshrat / rshrat — Port of C++ lshrat / rshrat (logic.cpp)
// ---------------------------------------------------------------------------

/// Left shift: `*a = trunc(*a) * 2^trunc(b)`.
///
/// Port of C++ `lshrat`.
pub fn lsh_rat(a: &mut Rational, b: &Rational, radix: u32, precision: i32) -> CalcResult<()> {
    int_rat(a, radix, precision)?;

    if !a.p().is_zero() {
        let rat_max_exp = Rational::from_i32(MAX_SHIFT_EXP);
        if rat_gt(b, &rat_max_exp, precision) {
            return Err(CalcError::Domain);
        }

        let int_b = rat_to_i32(b, radix, precision)?;
        let rat_two = Rational::from_i32(2);
        let pwr = rat_pow_i32(&rat_two, int_b, precision)?;
        *a = mul_rat(a, &pwr, precision);
    }

    Ok(())
}

/// Right shift: `*a = trunc(*a) / 2^trunc(b)`.
///
/// Port of C++ `rshrat`.
pub fn rsh_rat(a: &mut Rational, b: &Rational, radix: u32, precision: i32) -> CalcResult<()> {
    int_rat(a, radix, precision)?;

    if !a.p().is_zero() {
        let rat_min_exp = Rational::from_i32(-MAX_SHIFT_EXP);
        if rat_lt(b, &rat_min_exp, precision) {
            return Err(CalcError::Domain);
        }

        let int_b = rat_to_i32(b, radix, precision)?;
        let rat_two = Rational::from_i32(2);
        let pwr = rat_pow_i32(&rat_two, int_b, precision)?;
        *a = div_rat(a, &pwr, precision)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// andrat / orrat / xorrat — Port of C++ andrat / orrat / xorrat (logic.cpp)
// ---------------------------------------------------------------------------

/// Bitwise AND of two rationals (truncated to integers first).
///
/// Port of C++ `andrat`.
pub fn and_rat(a: &mut Rational, b: &Rational, radix: u32, precision: i32) -> CalcResult<()> {
    bool_rat(a, b, BoolFunc::And, radix, precision)
}

/// Bitwise OR of two rationals (truncated to integers first).
///
/// Port of C++ `orrat`.
pub fn or_rat(a: &mut Rational, b: &Rational, radix: u32, precision: i32) -> CalcResult<()> {
    bool_rat(a, b, BoolFunc::Or, radix, precision)
}

/// Bitwise XOR of two rationals (truncated to integers first).
///
/// Port of C++ `xorrat`.
pub fn xor_rat(a: &mut Rational, b: &Rational, radix: u32, precision: i32) -> CalcResult<()> {
    bool_rat(a, b, BoolFunc::Xor, radix, precision)
}

// ---------------------------------------------------------------------------
// boolrat — Port of C++ boolrat (logic.cpp)
// ---------------------------------------------------------------------------

/// Truncate both operands to integers and apply a bitwise operation on the
/// numerator mantissa digits.
///
/// Port of C++ `boolrat`.
fn bool_rat(
    a: &mut Rational,
    b: &Rational,
    func: BoolFunc,
    radix: u32,
    precision: i32,
) -> CalcResult<()> {
    int_rat(a, radix, precision)?;
    let mut tmp = b.clone();
    int_rat(&mut tmp, radix, precision)?;

    let new_p = bool_num(a.p(), tmp.p(), func);
    *a.p_mut() = new_p;

    Ok(())
}

// ---------------------------------------------------------------------------
// boolnum — Port of C++ boolnum (logic.cpp)
//
// Applies a bitwise operation digit-by-digit on the BASEX mantissa,
// accounting for different exponents (digit alignment).
// ---------------------------------------------------------------------------

/// Apply a bitwise operation on two `Number` mantissa arrays, aligning
/// digits by exponent.
///
/// Port of C++ `boolnum`.
fn bool_num(a: &Number, b: &Number, func: BoolFunc) -> Number {
    let a_top = a.cdigit() + a.exp; // highest position + 1
    let b_top = b.cdigit() + b.exp;
    let min_exp = a.exp.min(b.exp);
    let max_top = a_top.max(b_top);
    let total_digits = max_top - min_exp;

    if total_digits <= 0 {
        return Number::zero();
    }

    let mut result_mant = Vec::with_capacity(total_digits as usize);
    let mut pcha_idx: usize = 0;
    let mut pchb_idx: usize = 0;
    let mut mexp = min_exp;

    // Iterate from LSD to MSD, mirroring the C++ loop that counts
    // `cdigits` down from `total_digits` to 1.
    for remaining in (1..=total_digits).rev() {
        let da = if mexp >= a.exp && remaining > (max_top - a_top) {
            let d = a.mantissa[pcha_idx];
            pcha_idx += 1;
            d
        } else {
            0
        };

        let db = if mexp >= b.exp && remaining > (max_top - b_top) {
            let d = b.mantissa[pchb_idx];
            pchb_idx += 1;
            d
        } else {
            0
        };

        let digit = match func {
            BoolFunc::And => da & db,
            BoolFunc::Or => da | db,
            BoolFunc::Xor => da ^ db,
        };
        result_mant.push(digit);
        mexp += 1;
    }

    // Trim trailing zeros from MSD (C++: while (c->cdigit > 1 && *(--pchc) == 0))
    while result_mant.len() > 1 && *result_mant.last().unwrap() == 0 {
        result_mant.pop();
    }

    Number::new(a.sign, min_exp, result_mant)
}

// ---------------------------------------------------------------------------
// modrat — Port of C++ modrat (logic.cpp)
//
// Modulus with sign of divisor (as opposed to remrat which uses sign of
// dividend, i.e. C-style truncation remainder).
// ---------------------------------------------------------------------------

/// Modulus: `*a = a mod b` with the sign of the divisor.
///
/// Unlike [`rem_rat`](super::arithmetic::rem_rat) which follows C-style
/// truncation-toward-zero semantics (result sign matches dividend), `mod_rat`
/// adjusts the result so that its sign matches the divisor.
///
/// Port of C++ `modrat`.
pub fn mod_rat(a: &mut Rational, b: &Rational) -> CalcResult<()> {
    if b.is_zero() {
        // C++ modrat returns silently when b == 0.
        return Ok(());
    }

    let need_adjust = if a.sign() == -1 {
        b.sign() == 1
    } else {
        b.sign() == -1
    };

    // Compute remainder: same cross-multiply-and-rem as remrat.
    let tmp = b.clone();
    let new_p = mul_num_x(a.p(), tmp.q());
    *a.p_mut() = new_p;

    let cross_b = mul_num_x(tmp.p(), a.q());

    rem_num(a.p_mut(), &cross_b, BASEX);

    let new_q = mul_num_x(a.q(), tmp.q());
    *a.q_mut() = new_q;

    // If signs of a and b differ and remainder is nonzero, add b to make
    // the result's sign match the divisor.
    if need_adjust && !a.is_zero() {
        *a = add_rat(a, b, BASEX as i32);
    }

    a.renormalize();
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a rational to `i32` by truncating to integer first.
///
/// Equivalent to the C++ sequence: `intrat(&b, …); rattoi32(b, …)`.
fn rat_to_i32(b: &Rational, radix: u32, precision: i32) -> CalcResult<i32> {
    let mut tmp = b.clone();
    int_rat(&mut tmp, radix, precision)?;
    tmp.p().to_i32(BASEX).ok_or(CalcError::Overflow)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ratpack::arithmetic::rem_rat;

    const PRECISION: i32 = 128;
    const RADIX: u32 = 10;

    // ---- lsh_rat / rsh_rat ----

    #[test]
    fn test_lsh_rat_basic() {
        // 5 << 3 = 5 * 2^3 = 40
        let mut a = Rational::from_i32(5);
        let b = Rational::from_i32(3);
        lsh_rat(&mut a, &b, RADIX, PRECISION).unwrap();
        let val = a.p().to_i32(BASEX).unwrap();
        assert_eq!(val, 40);
    }

    #[test]
    fn test_lsh_rat_zero() {
        // 0 << 5 = 0
        let mut a = Rational::from_i32(0);
        let b = Rational::from_i32(5);
        lsh_rat(&mut a, &b, RADIX, PRECISION).unwrap();
        assert!(a.is_zero());
    }

    #[test]
    fn test_lsh_rat_by_zero() {
        // 7 << 0 = 7 * 2^0 = 7
        let mut a = Rational::from_i32(7);
        let b = Rational::from_i32(0);
        lsh_rat(&mut a, &b, RADIX, PRECISION).unwrap();
        let val = a.p().to_i32(BASEX).unwrap();
        assert_eq!(val, 7);
    }

    #[test]
    fn test_lsh_rat_negative_value() {
        // -3 << 2 = -3 * 4 = -12
        let mut a = Rational::from_i32(-3);
        let b = Rational::from_i32(2);
        lsh_rat(&mut a, &b, RADIX, PRECISION).unwrap();
        let val = a.p().to_i32(BASEX).unwrap();
        assert_eq!(val, -12);
    }

    #[test]
    fn test_lsh_rat_domain_error() {
        // Shift by > MAX_SHIFT_EXP should fail
        let mut a = Rational::from_i32(1);
        let b = Rational::from_i32(MAX_SHIFT_EXP + 1);
        let result = lsh_rat(&mut a, &b, RADIX, PRECISION);
        assert!(result.is_err());
    }

    #[test]
    fn test_rsh_rat_basic() {
        // 40 >> 3 = 40 / 8 = 5
        let mut a = Rational::from_i32(40);
        let b = Rational::from_i32(3);
        rsh_rat(&mut a, &b, RADIX, PRECISION).unwrap();
        // Result is 40/8 = 5/1 as a rational
        // Need to check the rational value, not just numerator
        // 40 / 2^3 = 5 as a rational (p=40, q=8 or simplified)
        let p = a.p().to_i32(BASEX).unwrap();
        let q = a.q().to_i32(BASEX).unwrap();
        // p/q should equal 5
        assert_eq!(p * 1, q * 5); // cross multiply
    }

    #[test]
    fn test_rsh_rat_zero() {
        // 0 >> 5 = 0
        let mut a = Rational::from_i32(0);
        let b = Rational::from_i32(5);
        rsh_rat(&mut a, &b, RADIX, PRECISION).unwrap();
        assert!(a.is_zero());
    }

    #[test]
    fn test_rsh_rat_by_zero() {
        // 7 >> 0 = 7 / 2^0 = 7
        let mut a = Rational::from_i32(7);
        let b = Rational::from_i32(0);
        rsh_rat(&mut a, &b, RADIX, PRECISION).unwrap();
        let val = a.p().to_i32(BASEX).unwrap();
        assert_eq!(val, 7);
    }

    #[test]
    fn test_rsh_rat_domain_error() {
        // Shift by < -MAX_SHIFT_EXP should fail
        let mut a = Rational::from_i32(1);
        let b = Rational::from_i32(-MAX_SHIFT_EXP - 1);
        let result = rsh_rat(&mut a, &b, RADIX, PRECISION);
        assert!(result.is_err());
    }

    // ---- and_rat / or_rat / xor_rat ----

    #[test]
    fn test_and_rat_basic() {
        // 0b1100 & 0b1010 = 0b1000 → 12 & 10 = 8
        let mut a = Rational::from_i32(12);
        let b = Rational::from_i32(10);
        and_rat(&mut a, &b, RADIX, PRECISION).unwrap();
        let val = a.p().to_i32(BASEX).unwrap();
        assert_eq!(val, 8);
    }

    #[test]
    fn test_and_rat_with_zero() {
        // x & 0 = 0
        let mut a = Rational::from_i32(255);
        let b = Rational::from_i32(0);
        and_rat(&mut a, &b, RADIX, PRECISION).unwrap();
        assert!(a.is_zero());
    }

    #[test]
    fn test_and_rat_identity() {
        // x & x = x
        let mut a = Rational::from_i32(42);
        let b = Rational::from_i32(42);
        and_rat(&mut a, &b, RADIX, PRECISION).unwrap();
        let val = a.p().to_i32(BASEX).unwrap();
        assert_eq!(val, 42);
    }

    #[test]
    fn test_or_rat_basic() {
        // 0b1100 | 0b1010 = 0b1110 → 12 | 10 = 14
        let mut a = Rational::from_i32(12);
        let b = Rational::from_i32(10);
        or_rat(&mut a, &b, RADIX, PRECISION).unwrap();
        let val = a.p().to_i32(BASEX).unwrap();
        assert_eq!(val, 14);
    }

    #[test]
    fn test_or_rat_with_zero() {
        // x | 0 = x
        let mut a = Rational::from_i32(42);
        let b = Rational::from_i32(0);
        or_rat(&mut a, &b, RADIX, PRECISION).unwrap();
        let val = a.p().to_i32(BASEX).unwrap();
        assert_eq!(val, 42);
    }

    #[test]
    fn test_xor_rat_basic() {
        // 0b1100 ^ 0b1010 = 0b0110 → 12 ^ 10 = 6
        let mut a = Rational::from_i32(12);
        let b = Rational::from_i32(10);
        xor_rat(&mut a, &b, RADIX, PRECISION).unwrap();
        let val = a.p().to_i32(BASEX).unwrap();
        assert_eq!(val, 6);
    }

    #[test]
    fn test_xor_rat_self_is_zero() {
        // x ^ x = 0
        let mut a = Rational::from_i32(42);
        let b = Rational::from_i32(42);
        xor_rat(&mut a, &b, RADIX, PRECISION).unwrap();
        assert!(a.is_zero());
    }

    #[test]
    fn test_xor_rat_with_zero() {
        // x ^ 0 = x
        let mut a = Rational::from_i32(42);
        let b = Rational::from_i32(0);
        xor_rat(&mut a, &b, RADIX, PRECISION).unwrap();
        let val = a.p().to_i32(BASEX).unwrap();
        assert_eq!(val, 42);
    }

    // ---- bool_num digit alignment ----

    #[test]
    fn test_bool_num_same_exp() {
        // Both at exp=0: simple case
        let a = Number::new(1, 0, vec![0xFF, 0x0F]);
        let b = Number::new(1, 0, vec![0x0F, 0xFF]);
        let result = bool_num(&a, &b, BoolFunc::And);
        assert_eq!(result.mantissa, vec![0x0F, 0x0F]);
        assert_eq!(result.exp, 0);
    }

    #[test]
    fn test_bool_num_different_exp() {
        // a at exp=1 (digits at positions 1..2), b at exp=0 (digits at positions 0..1)
        // a: [3] at position 1 → digit 3 at pos 1
        // b: [6, 2] at positions 0,1 → digits 6 at pos 0, 2 at pos 1
        // Combined positions 0..2:
        //   pos 0: a=0, b=6 → AND = 0
        //   pos 1: a=3, b=2 → AND = 2
        let a = Number::new(1, 1, vec![3]);
        let b = Number::new(1, 0, vec![6, 2]);
        let result = bool_num(&a, &b, BoolFunc::And);
        assert_eq!(result.mantissa, vec![0, 2]);
        assert_eq!(result.exp, 0);
    }

    #[test]
    fn test_bool_num_or_different_exp() {
        // a: [3] at exp=1 → pos 1 = 3
        // b: [6, 2] at exp=0 → pos 0 = 6, pos 1 = 2
        // OR: pos 0 = 0|6 = 6, pos 1 = 3|2 = 3
        let a = Number::new(1, 1, vec![3]);
        let b = Number::new(1, 0, vec![6, 2]);
        let result = bool_num(&a, &b, BoolFunc::Or);
        assert_eq!(result.mantissa, vec![6, 3]);
        assert_eq!(result.exp, 0);
    }

    #[test]
    fn test_bool_num_trailing_zero_trim() {
        // 0xFF AND 0xFF00 → result has MSD zeros trimmed
        // a: [0xFF] at exp=0
        // b: [0, 0xFF] at exp=0
        // AND: pos 0 = 0xFF & 0 = 0, pos 1 = 0 & 0xFF = 0
        let a = Number::new(1, 0, vec![0xFF]);
        let b = Number::new(1, 0, vec![0, 0xFF]);
        let result = bool_num(&a, &b, BoolFunc::And);
        // All zeros → should be single zero digit
        assert_eq!(result.mantissa, vec![0]);
    }

    // ---- mod_rat vs rem_rat ----

    #[test]
    fn test_mod_rat_positive() {
        // 7 mod 3 = 1 (same as rem for positive values)
        let mut a = Rational::from_i32(7);
        let b = Rational::from_i32(3);
        mod_rat(&mut a, &b).unwrap();
        let val = a.p().to_i32(BASEX).unwrap();
        assert_eq!(val, 1);
    }

    #[test]
    fn test_mod_rat_negative_dividend() {
        // -7 mod 3: rem = -1, mod = -1 + 3 = 2 (sign matches divisor)
        let mut a = Rational::from_i32(-7);
        let b = Rational::from_i32(3);
        mod_rat(&mut a, &b).unwrap();
        let val = a.p().to_i32(BASEX).unwrap();
        let q_val = a.q().to_i32(BASEX).unwrap();
        // The result should be +2 (sign matches divisor which is positive)
        assert!(val * q_val.signum() > 0 || a.is_zero(), "mod result sign should match divisor");
        // Check the value: p/q = 2
        assert_eq!(val.abs() * 1, q_val.abs() * 2);
    }

    #[test]
    fn test_mod_rat_negative_divisor() {
        // 7 mod (-3): rem = 1, mod = 1 + (-3) = -2 (sign matches divisor)
        let mut a = Rational::from_i32(7);
        let b = Rational::from_i32(-3);
        mod_rat(&mut a, &b).unwrap();
        let val = a.p().to_i32(BASEX).unwrap();
        let q_val = a.q().to_i32(BASEX).unwrap();
        // The result should be -2 (sign matches divisor which is negative)
        let result_sign = val.signum() * q_val.signum();
        assert!(result_sign < 0 || a.is_zero(), "mod result sign should match divisor");
    }

    #[test]
    fn test_mod_rat_both_negative() {
        // -7 mod -3: rem = -1, signs same so no adjust → -1
        let mut a = Rational::from_i32(-7);
        let b = Rational::from_i32(-3);
        mod_rat(&mut a, &b).unwrap();
        let val = a.p().to_i32(BASEX).unwrap();
        let q_val = a.q().to_i32(BASEX).unwrap();
        // result = -1 (sign matches divisor which is negative)
        assert_eq!(val.abs(), q_val.abs()); // |p/q| = 1
    }

    #[test]
    fn test_mod_rat_zero_dividend() {
        // 0 mod 5 = 0
        let mut a = Rational::from_i32(0);
        let b = Rational::from_i32(5);
        mod_rat(&mut a, &b).unwrap();
        assert!(a.is_zero());
    }

    #[test]
    fn test_mod_rat_zero_divisor() {
        // x mod 0 → returns Ok(()) per C++ (no-op)
        let mut a = Rational::from_i32(7);
        let b = Rational::from_i32(0);
        let result = mod_rat(&mut a, &b);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rem_rat_vs_mod_rat_sign_difference() {
        // rem(-7, 3) = -1 (sign matches dividend)
        // mod(-7, 3) = 2  (sign matches divisor)
        let a = Rational::from_i32(-7);
        let b = Rational::from_i32(3);

        let rem_result = rem_rat(&a, &b).unwrap();
        let rem_p = rem_result.p().to_i32(BASEX).unwrap();
        let rem_q = rem_result.q().to_i32(BASEX).unwrap();
        // rem sign matches dividend (negative)
        let rem_sign = rem_p.signum() * rem_q.signum();
        assert!(rem_sign < 0, "rem result should be negative (match dividend)");

        let mut mod_a = a;
        mod_rat(&mut mod_a, &b).unwrap();
        // mod sign matches divisor (positive)
        assert!(mod_a.sign() > 0 || mod_a.is_zero(), "mod result should be positive (match divisor)");
    }

    // ---- edge cases ----

    #[test]
    fn test_and_or_xor_truncate_fraction() {
        // 5.7 AND 3 should first truncate 5.7 → 5, then compute 5 & 3 = 1
        let a = Rational::new(
            Number::from_i32(57, BASEX),
            Number::from_i32(10, BASEX),
        );
        let mut a_and = a.clone();
        let b = Rational::from_i32(3);
        and_rat(&mut a_and, &b, RADIX, PRECISION).unwrap();
        let val = a_and.p().to_i32(BASEX).unwrap();
        assert_eq!(val, 5 & 3);
    }

    #[test]
    fn test_lsh_rsh_roundtrip() {
        // (x << n) >> n should give back x for integer x
        let mut a = Rational::from_i32(42);
        let shift = Rational::from_i32(5);
        lsh_rat(&mut a, &shift, RADIX, PRECISION).unwrap();
        // a is now 42 * 32 = 1344
        rsh_rat(&mut a, &shift, RADIX, PRECISION).unwrap();
        // After int_rat + divide, should be 42 again
        int_rat(&mut a, RADIX, PRECISION).unwrap();
        let val = a.p().to_i32(BASEX).unwrap();
        assert_eq!(val, 42);
    }
}
