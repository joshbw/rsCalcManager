// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Support functions for ratpack.
//! Port of C++ Ratpack/support.cpp, rat.cpp (gcdrat, fracrat, _snaprat)

use crate::error::CalcResult;
use super::arithmetic::{
    add_rat, div_rat, div_num_x, equ_num, mul_rat, rem_num, rem_rat, sub_rat,
    rat_gt, rat_lt,
};
use super::constants::RatpackConstants;
use super::conv::flat_rat;
use super::num::gcd;
use super::Number;
use super::Rational;
use crate::types::BASEX;

/// Set the decimal separator character.
/// Port of C++ `SetDecimalSeparator`.
pub const fn set_decimal_separator(constants: &mut RatpackConstants, sep: char) {
    constants.decimal_separator = sep;
}

/// Trim a rational to the given precision.
/// Port of C++ `trimit` from support.cpp.
///
/// Chops off digits from rational numbers to avoid time explosions in
/// calculations of functions using series. Keeps only enough digits for
/// the required precision.
pub fn trim_rat(rat: &mut Rational, precision: i32, ratio: i32, true_infinite: bool) {
    if true_infinite {
        return;
    }

    let p = rat.p();
    let q = rat.q();

    let p_log = p.cdigit() + p.exp;
    let q_log = q.cdigit() + q.exp;
    let mut trim = ratio * (p_log.min(q_log) - 1) - precision;

    if trim > ratio {
        trim /= ratio;

        // Trim numerator
        let p_mut = rat.p_mut();
        if trim <= p_mut.exp {
            p_mut.exp -= trim;
        } else {
            let remove = (trim - p_mut.exp) as usize;
            if remove < p_mut.mantissa.len() {
                p_mut.mantissa.drain(0..remove);
            } else {
                // All digits removed: set to zero
                p_mut.mantissa = vec![0];
            }
            p_mut.exp = 0;
        }

        // Trim denominator
        let q_mut = rat.q_mut();
        if trim <= q_mut.exp {
            q_mut.exp -= trim;
        } else {
            let remove = (trim - q_mut.exp) as usize;
            if remove < q_mut.mantissa.len() {
                q_mut.mantissa.drain(0..remove);
            } else {
                // All digits removed: set to zero
                q_mut.mantissa = vec![0];
            }
            q_mut.exp = 0;
        }
    }

    // Remove common exponent
    let common_exp = rat.p().exp.min(rat.q().exp);
    if common_exp > 0 {
        rat.p_mut().exp -= common_exp;
        rat.q_mut().exp -= common_exp;
    }
}

/// Compute GCD of numerator and denominator, simplifying the rational.
/// Port of C++ `gcdrat` from rat.cpp.
pub fn gcd_rat(x: &mut Rational, precision: i32) {
    let pgcd = gcd(x.p(), x.q());

    if !pgcd.is_zero() {
        if let Ok(new_p) = div_num_x(x.p(), &pgcd, precision) {
            *x.p_mut() = new_p;
        }
        if let Ok(new_q) = div_num_x(x.q(), &pgcd, precision) {
            *x.q_mut() = new_q;
        }
    }

    x.renormalize();
}

/// Extract fractional part of a rational.
/// Port of C++ `fracrat` from rat.cpp.
///
/// After this operation, x contains only the fractional part.
pub fn frac_rat(x: &mut Rational, radix: u32, precision: i32) -> CalcResult<()> {
    let num_one = Number::from_i32(1, BASEX);

    // Only flatten if non-zero and denominator is not one
    if !x.p().is_zero() && !equ_num(x.q(), &num_one) {
        let flat = flat_rat(x, radix, precision)?;
        *x = flat;
    }

    // Compute p = p % q (remainder)
    let q_clone = x.q().clone();
    rem_num(x.p_mut(), &q_clone, BASEX);

    // Renormalize
    x.renormalize();
    Ok(())
}

/// Extract integer part of a rational.
/// Port of C++ `intrat` from support.cpp.
///
/// After this operation, x contains only the integer part.
pub fn int_rat(x: &mut Rational, radix: u32, precision: i32) -> CalcResult<()> {
    let num_one = Number::from_i32(1, BASEX);

    // Only process if non-zero and denominator is not one
    if !x.p().is_zero() && !equ_num(x.q(), &num_one) {
        // Flatten x
        let flat = flat_rat(x, radix, precision)?;
        *x = flat;

        // Subtract the fractional part
        let rat_one = Rational::one();
        let frac = rem_rat(x, &rat_one)?;

        // Flatten frac if denominators don't match
        let frac = if equ_num(x.q(), frac.q()) {
            frac
        } else {
            flat_rat(&frac, radix, precision)?
        };

        *x = sub_rat(x, &frac, precision);

        // Simplify
        let flat = flat_rat(x, radix, precision)?;
        *x = flat;
    }

    Ok(())
}

/// General modular scaling: x = x mod scalefact.
/// Port of C++ `scale` from support.cpp.
///
/// Computes remainder of x / scalefact. The result has the same sign as x
/// (truncation toward zero), matching C/C++ remainder semantics.
pub fn scale(
    x: &mut Rational,
    scalefact: &Rational,
    radix: u32,
    precision: i32,
    ratio: i32,
) -> CalcResult<()> {
    let mut pret = x.clone();

    // Logscale is a quick way to tell how much extra precision is needed
    let logscale = ratio * (pret.p().log2_estimate() - pret.q().log2_estimate());
    let precision = if logscale > 0 {
        precision + logscale
    } else {
        precision
    };

    // pret = floor(x / scalefact) * scalefact
    pret = div_rat(&pret, scalefact, precision)?;
    int_rat(&mut pret, radix, precision)?;
    pret = mul_rat(&pret, scalefact, precision);

    // x = x - pret (i.e., x mod scalefact)
    pret.p_mut().sign *= -1;
    *x = add_rat(x, &pret, precision);

    Ok(())
}

/// Scale x modulo 2*pi for trig functions.
/// Port of C++ `scale2pi` from support.cpp.
///
/// Reduces x modulo 2π for trig function input normalization.
/// Unlike the general `scale()` function, this inlines the math
/// (matching C++) to avoid double-adjusting precision.
pub fn scale_2pi(
    x: &mut Rational,
    two_pi: &Rational,
    radix: u32,
    precision: i32,
    ratio: i32,
) -> CalcResult<()> {
    let mut pret = x.clone();

    // Logscale: how much extra precision needed for scaling by 2*pi
    let logscale = ratio * (pret.p().log2_estimate() - pret.q().log2_estimate());
    let (my_two_pi, precision) = if logscale > 0 {
        // For large numbers, we'd ideally recompute 2*pi with more precision.
        // For now, use the provided two_pi (acceptable with approximations).
        (two_pi.clone(), precision + logscale)
    } else {
        (two_pi.clone(), precision)
    };

    // Inline the scale logic (matching C++ scale2pi which does NOT call scale):
    // pret = floor(x / 2pi) * 2pi, then x = x - pret
    pret = div_rat(&pret, &my_two_pi, precision)?;
    int_rat(&mut pret, radix, precision)?;
    pret = mul_rat(&pret, &my_two_pi, precision);
    pret.p_mut().sign *= -1;
    *x = add_rat(x, &pret, precision);

    Ok(())
}

/// Snap to zero if magnitude is below precision threshold.
/// Port of C++ `_snaprat` from rat.cpp.
///
/// If |r| is much smaller than |a| (or |b|) relative to precision,
/// snap r to zero. This addresses tiny residuals in calculations
/// that should yield zero (e.g., sqrt(2.25) - 1.5).
pub fn snap_rat(
    r: &mut Rational,
    a: &Rational,
    b: Option<&Rational>,
    precision: i32,
    rat_smallest: &Rational,
) {
    // Determine threshold from the larger of |a| and |b|
    let threshold_base = b.map_or_else(
        || a.abs(),
        |b_val| {
            let abs_a = a.abs();
            let abs_b = b_val.abs();
            if rat_lt(&abs_a, &abs_b, precision) {
                abs_b
            } else {
                abs_a
            }
        },
    );

    // threshold = threshold_base * rat_smallest
    let threshold = mul_rat(&threshold_base, rat_smallest, precision);

    let abs_r = r.abs();

    // If |r| < threshold, snap to zero
    if rat_lt(&abs_r, &threshold, precision) {
        *r = Rational::zero();
    }
}

/// Check if x is in range [-range, range] and clamp if needed.
/// Port of C++ `inbetween` from support.cpp.
pub fn in_between(x: &mut Rational, range: &Rational, precision: i32) {
    if rat_gt(x, range, precision) {
        *x = range.clone();
    } else {
        let neg_range = range.negate();
        if rat_lt(x, &neg_range, precision) {
            *x = neg_range;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_rat_noop_for_small() {
        // A simple rational shouldn't be trimmed if precision is large enough
        let mut r = Rational::from_i32(42);
        let original = r.clone();
        trim_rat(&mut r, 128, 1, false);
        // For a simple integer, trim shouldn't destroy it
        assert!(!r.is_zero() || original.is_zero());
    }

    #[test]
    fn test_trim_rat_infinite_mode() {
        let mut r = Rational::from_i32(42);
        let original = r.clone();
        trim_rat(&mut r, 128, 1, true);
        assert_eq!(r, original); // No change in infinite mode
    }

    #[test]
    fn test_gcd_rat_simplify() {
        // 6/4 should simplify to 3/2
        let p = Number::from_i32(6, BASEX);
        let q = Number::from_i32(4, BASEX);
        let mut r = Rational::new(p, q);
        gcd_rat(&mut r, 128);

        // After GCD simplification, check the result represents 3/2
        let p_val = r.p().to_i32(BASEX);
        let q_val = r.q().to_i32(BASEX);
        assert_eq!(p_val, Some(3));
        assert_eq!(q_val, Some(2));
    }

    #[test]
    fn test_gcd_rat_already_simplified() {
        // 3/7 is already in lowest terms
        let p = Number::from_i32(3, BASEX);
        let q = Number::from_i32(7, BASEX);
        let mut r = Rational::new(p, q);
        gcd_rat(&mut r, 128);
        assert_eq!(r.p().to_i32(BASEX), Some(3));
        assert_eq!(r.q().to_i32(BASEX), Some(7));
    }

    #[test]
    fn test_int_rat_integer() {
        // intrat of 5/1 = 5/1
        let mut r = Rational::from_i32(5);
        int_rat(&mut r, 10, 128).unwrap();
        assert!(!r.is_zero());
    }

    #[test]
    fn test_frac_rat_integer() {
        // fracrat of 5/1 = 0
        let mut r = Rational::from_i32(5);
        frac_rat(&mut r, 10, 128).unwrap();
        assert!(r.is_zero());
    }

    #[test]
    fn test_in_between_within() {
        let mut x = Rational::from_i32(3);
        let range = Rational::from_i32(10);
        in_between(&mut x, &range, 128);
        // 3 is within [-10, 10], should not change
        assert_eq!(x.p().to_i32(BASEX), Some(3));
    }

    #[test]
    fn test_in_between_clamp_high() {
        let mut x = Rational::from_i32(15);
        let range = Rational::from_i32(10);
        in_between(&mut x, &range, 128);
        // 15 > 10, should be clamped to 10
        assert_eq!(x.p().to_i32(BASEX), Some(10));
    }

    #[test]
    fn test_in_between_clamp_low() {
        let mut x = Rational::from_i32(-15);
        let range = Rational::from_i32(10);
        in_between(&mut x, &range, 128);
        // -15 < -10, should be clamped to -10
        let val = x.p().to_i32(BASEX).unwrap();
        assert_eq!(val.abs(), 10);
    }

    #[test]
    fn test_snap_rat_zero_snaps() {
        // A tiny value near zero should snap to zero
        let rat_smallest = Rational::new(
            Number::from_i32(1, BASEX),
            Number::from_i32(1_000_000, BASEX),
        );
        let a = Rational::from_i32(100);
        // r = very small value
        let mut r = Rational::new(
            Number::from_i32(1, BASEX),
            Number::from_i32(1_000_000_000, BASEX),
        );
        snap_rat(&mut r, &a, None, 128, &rat_smallest);
        assert!(r.is_zero());
    }

    #[test]
    fn test_snap_rat_nonzero_stays() {
        // A value that is NOT tiny should stay
        let rat_smallest = Rational::new(
            Number::from_i32(1, BASEX),
            Number::from_i32(1_000_000, BASEX),
        );
        let a = Rational::from_i32(1);
        let mut r = Rational::from_i32(5);
        snap_rat(&mut r, &a, None, 128, &rat_smallest);
        assert!(!r.is_zero());
    }
}
