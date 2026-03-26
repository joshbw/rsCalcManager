// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Factorial function.
//!
//! For integer arguments, computes exact factorial by iterative multiplication.
//! For non-integer arguments, uses the gamma function: n! = Γ(n+1).
//!
//! Port of C++ Ratpack/fact.cpp

use crate::error::{CalcError, CalcResult};
use crate::types::BASEX;

use super::arithmetic::{
    add_num, add_rat, div_rat, mul_num_x, mul_rat, rat_gt, rat_lt, rat_neq, sub_rat,
};
use super::constants::RatpackConstants;
use super::exp::{_log_rat, exp_rat, pow_rat};
use super::support::{frac_rat, int_rat, trim_rat};
use super::Number;
use super::Rational;

/// Trimmed multiply: equivalent of C++ `mulrat` which internally calls `trimit`.
#[inline]
fn mul_rat_t(a: &Rational, b: &Rational, precision: i32, ratio: i32, ti: bool) -> Rational {
    let mut r = mul_rat(a, b, precision);
    trim_rat(&mut r, precision, ratio, ti);
    r
}

/// Trimmed add: equivalent of C++ `_addrat` which internally calls `trimit`.
#[inline]
fn add_rat_t(a: &Rational, b: &Rational, precision: i32, ratio: i32, ti: bool) -> Rational {
    let mut r = add_rat(a, b, precision);
    trim_rat(&mut r, precision, ratio, ti);
    r
}

/// Trimmed sub: equivalent of C++ `subrat` which internally calls `trimit`.
#[inline]
fn sub_rat_t(a: &Rational, b: &Rational, precision: i32, ratio: i32, ti: bool) -> Rational {
    let mut r = sub_rat(a, b, precision);
    trim_rat(&mut r, precision, ratio, ti);
    r
}

/// Trimmed div: equivalent of C++ `divrat` which internally calls `trimit`.
#[inline]
fn div_rat_t(a: &Rational, b: &Rational, precision: i32, ratio: i32, ti: bool) -> CalcResult<Rational> {
    let mut r = div_rat(a, b, precision)?;
    trim_rat(&mut r, precision, ratio, ti);
    Ok(r)
}

/// Estimate the base-`radix` logarithm of a rational's magnitude.
///
/// Matches C++ macro `LOGRATRADIX(r)`:
///   `(LOGNUMRADIX(pp) - LOGNUMRADIX(pq))`
/// where `LOGNUMRADIX(n) = (n->cdigit + n->exp) * ratio`.
///
/// Uses i64 arithmetic internally to avoid overflow when `cdigit + exp` is large.
fn log_rat_radix(x: &Rational, ratio: i32) -> i32 {
    let p_log = i64::from(x.p().cdigit() + x.p().exp);
    let q_log = i64::from(x.q().cdigit() + x.q().exp);
    ((p_log - q_log) * i64::from(ratio)) as i32
}

/// Convert a rational to i32 after extracting its integer part.
///
/// Equivalent to the C++ sequence: `intrat(&r, …); rattoi32(r, …)`.
fn rat_to_i32(r: &Rational, radix: u32, precision: i32) -> CalcResult<i32> {
    let mut tmp = r.clone();
    int_rat(&mut tmp, radix, precision)?;
    tmp.p().to_i32(BASEX).ok_or(CalcError::Overflow)
}

/// Internal gamma-like function using a Stirling series expansion.
///
/// Computes a value related to the reciprocal gamma function, used for
/// non-integer factorial arguments. The algorithm:
///
/// 1. Finds an optimal convergence parameter `a` based on precision and radix.
/// 2. Computes a precision bump to ensure sufficient accuracy for the series.
/// 3. Evaluates the series Σ_k [ (1/(n+2k) - a/((2k+1)*(n+2k+1))) / (2k)! * a^(2k) ].
/// 4. Multiplies by `a^n` to produce the final result.
///
/// Port of C++ `_gamma` from fact.cpp.
///
/// # Note
///
/// This function depends on exp.rs functions (`_log_rat`, `exp_rat`,
/// `pow_rat`). Non-integer factorial results require fully ported exp.rs.
fn _gamma(
    n: &mut Rational,
    radix: u32,
    mut precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    let num_one = Number::from_i32(1, BASEX);
    let ratio = constants.ratio;
    let ti = constants.true_infinite;

    // Save original precision as rational (needed for error bound later)
    let mut ratprec = Rational::from_i32(precision);

    // ── Find best convergence parameter 'a' ──────────────────────────────
    // a = ln(radix) * precision + 2 + n * ln(a_intermediate) + 1
    let mut a = Rational::from_i32(radix as i32);
    _log_rat(&mut a, precision, constants)?;
    a = mul_rat_t(&a, &ratprec, precision, ratio, ti);
    a = add_rat_t(&a, &constants.rat_two, precision, ratio, ti);

    let mut tmp = a.clone();
    _log_rat(&mut tmp, precision, constants)?;
    tmp = mul_rat_t(&tmp, n, precision, ratio, ti);
    a = add_rat_t(&a, &tmp, precision, ratio, ti);
    a = add_rat_t(&a, &constants.rat_one, precision, ratio, ti);

    // ── Calculate precision bump ─────────────────────────────────────────
    // term = ln(a^(n+1.5) * exp(a)) - ln(radix)
    tmp = n.clone();
    let one_pt_five = div_rat_t(&Rational::from_i32(3), &constants.rat_two, precision, ratio, ti)?;
    tmp = add_rat_t(&tmp, &one_pt_five, precision, ratio, ti);

    let mut term = a.clone();
    pow_rat(&mut term, &tmp, radix, precision, constants)?;
    tmp = a.clone();
    exp_rat(&mut tmp, radix, precision, constants)?;
    term = mul_rat_t(&term, &tmp, precision, ratio, ti);
    _log_rat(&mut term, precision, constants)?;

    let rat_radix = Rational::from_i32(radix as i32);
    tmp = rat_radix.clone();
    _log_rat(&mut tmp, precision, constants)?;
    term = sub_rat_t(&term, &tmp, precision, ratio, ti);

    // Bump precision by the integer part of term (matching C++ which throws on failure)
    precision += rat_to_i32(&term, radix, precision)?;

    // ── Series computation ───────────────────────────────────────────────
    let mut factorial = constants.rat_one.clone();
    let mut count = Number::from_i32(0, BASEX);

    // mpy = a^n (computed with original n, before loop modifies it)
    let mut mpy = a.clone();
    pow_rat(&mut mpy, n, radix, precision, constants)?;

    // a2 = a^2 (used to divide factorial each iteration)
    let a2 = mul_rat_t(&a, &a, precision, ratio, ti);

    // Initial sum = 1/n - a/(n+1)
    let mut sum = div_rat_t(&constants.rat_one, n, precision, ratio, ti)?;
    tmp = add_rat_t(n, &constants.rat_one, precision, ratio, ti);
    term = div_rat_t(&a, &tmp, precision, ratio, ti)?;
    sum = sub_rat_t(&sum, &term, precision, ratio, ti);

    // ── Error bound ──────────────────────────────────────────────────────
    // err = radix^(-original_precision) / radix
    let mut err = rat_radix.clone();
    ratprec.negate_mut();
    pow_rat(&mut err, &ratprec, radix, precision, constants)?;
    err = div_rat_t(&err, &rat_radix, precision, ratio, ti)?;

    // ── Main loop ────────────────────────────────────────────────────────
    // Initialize term to 2 (non-zero) to enter the loop
    term = constants.rat_two.clone();

    while !term.is_zero() && rat_gt(&term, &err, precision) {
        // n += 2 (each iteration advances by 2)
        *n = add_rat_t(n, &constants.rat_two, precision, ratio, ti);

        // factorial numerator *= (count+1) * (count+2), then divide by a^2
        count = add_num(&count, &num_one, BASEX);
        let new_p = mul_num_x(factorial.p(), &count);
        *factorial.p_mut() = new_p;

        count = add_num(&count, &num_one, BASEX);
        let new_p = mul_num_x(factorial.p(), &count);
        *factorial.p_mut() = new_p;

        factorial = div_rat_t(&factorial, &a2, precision, ratio, ti)?;

        // tmp = n + 1
        tmp = add_rat_t(n, &constants.rat_one, precision, ratio, ti);

        // term = (count + 1) * (n + 1)
        term = Rational::new(count.clone(), num_one.clone());
        term = add_rat_t(&term, &constants.rat_one, precision, ratio, ti);
        term = mul_rat_t(&term, &tmp, precision, ratio, ti);

        // tmp = a / ((count+1) * (n+1))
        tmp = div_rat_t(&a, &term, precision, ratio, ti)?;

        // term = 1/n - a/((count+1)*(n+1))
        term = div_rat_t(&constants.rat_one, n, precision, ratio, ti)?;
        term = sub_rat_t(&term, &tmp, precision, ratio, ti);

        // term /= factorial
        term = div_rat_t(&term, &factorial, precision, ratio, ti)?;

        // Accumulate
        sum = add_rat_t(&sum, &term, precision, ratio, ti);

        // |term| for convergence check
        term.abs_mut();
    }

    // Result = sum * a^n
    sum = mul_rat_t(&sum, &mpy, precision, ratio, ti);
    *n = sum;

    Ok(())
}

/// Compute n! (factorial).
///
/// For integer arguments, computes the exact factorial by iterative
/// multiplication. For non-integer arguments, uses the gamma function
/// relationship n! = Γ(n+1).
///
/// Port of C++ `factrat` from fact.cpp.
///
/// # Errors
///
/// - [`CalcError::Overflow`] if `x > 3249` or `x < -1000`.
/// - [`CalcError::Domain`] if `x` is a negative integer (factorial is undefined).
pub fn fact_rat(
    x: &mut Rational,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    // ── Bounds check ─────────────────────────────────────────────────────
    if rat_gt(x, &constants.rat_max_fact, precision)
        || rat_lt(x, &constants.rat_min_fact, precision)
    {
        return Err(CalcError::Overflow);
    }

    let ratio = constants.ratio;
    let ti = constants.true_infinite;

    let mut fact = constants.rat_one.clone();
    let neg_rat_one = constants.rat_neg_one.clone();

    // ── Negative-integer check ───────────────────────────────────────────
    let mut frac = x.clone();
    frac_rat(&mut frac, radix, precision)?;

    if (frac.is_zero() || log_rat_radix(&frac, constants.ratio) <= -precision)
        && x.sign() == -1
    {
        return Err(CalcError::Domain);
    }

    // ── Multiply down: fact *= x; x -= 1; while x > 0 ───────────────
    while rat_gt(x, &constants.rat_zero, precision)
        && log_rat_radix(x, constants.ratio) > -precision
    {
        fact = mul_rat_t(&fact, x, precision, ratio, ti);
        *x = sub_rat_t(x, &constants.rat_one, precision, ratio, ti);
    }

    // ── Snap to integer if x ≈ 0 ────────────────────────────────────────
    if log_rat_radix(x, constants.ratio) <= -precision {
        *x = constants.rat_zero.clone();
        int_rat(&mut fact, radix, precision)?;
    }

    // ── Handle negative non-integer args: divide up toward −1 ────────
    while rat_lt(x, &neg_rat_one, precision) {
        *x = add_rat_t(x, &constants.rat_one, precision, ratio, ti);
        fact = div_rat_t(&fact, x, precision, ratio, ti)?;
    }

    // ── If fractional remainder, use gamma; otherwise return fact ─────
    if rat_neq(x, &constants.rat_zero, precision) {
        *x = add_rat_t(x, &constants.rat_one, precision, ratio, ti);
        _gamma(x, radix, precision, constants)?;
        *x = mul_rat_t(x, &fact, precision, ratio, ti);
    } else {
        *x = fact;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::arithmetic::rat_equ;

    /// Helper to evaluate n! and return the result.
    fn eval_factorial(n: i32) -> CalcResult<Rational> {
        let constants = RatpackConstants::new(10, 32);
        let mut x = Rational::from_i32(n);
        fact_rat(&mut x, 10, 32, &constants)?;
        Ok(x)
    }

    #[test]
    fn test_factorial_zero() {
        // 0! = 1
        let result = eval_factorial(0).unwrap();
        assert!(rat_equ(&result, &Rational::from_i32(1), 32));
    }

    #[test]
    fn test_factorial_one() {
        // 1! = 1
        let result = eval_factorial(1).unwrap();
        assert!(rat_equ(&result, &Rational::from_i32(1), 32));
    }

    #[test]
    fn test_factorial_five() {
        // 5! = 120
        let result = eval_factorial(5).unwrap();
        assert!(rat_equ(&result, &Rational::from_i32(120), 32));
    }

    #[test]
    fn test_factorial_ten() {
        // 10! = 3_628_800
        let result = eval_factorial(10).unwrap();
        assert!(
            rat_equ(&result, &Rational::from_i32(3_628_800), 32),
            "10! should be 3628800, got {result}"
        );
    }

    #[test]
    fn test_factorial_negative_one_domain_error() {
        // (-1)! is undefined
        let result = eval_factorial(-1);
        assert!(
            matches!(result, Err(CalcError::Domain)),
            "(-1)! should be Domain error, got {result:?}"
        );
    }

    #[test]
    fn test_factorial_negative_two_domain_error() {
        // (-2)! is undefined
        let result = eval_factorial(-2);
        assert!(
            matches!(result, Err(CalcError::Domain)),
            "(-2)! should be Domain error, got {result:?}"
        );
    }

    #[test]
    fn test_factorial_overflow_large() {
        // 3250 > rat_max_fact (3249) → Overflow
        let result = eval_factorial(3250);
        assert!(
            matches!(result, Err(CalcError::Overflow)),
            "3250! should overflow, got {result:?}"
        );
    }

    #[test]
    fn test_factorial_overflow_very_negative() {
        // -1001 < rat_min_fact (-1000) → Overflow
        let result = eval_factorial(-1001);
        assert!(
            matches!(result, Err(CalcError::Overflow)),
            "-1001! should overflow, got {result:?}"
        );
    }

    #[test]
    fn test_log_rat_radix_one() {
        // 1/1 → (1+0 - (1+0)) * ratio = 0
        let one = Rational::one();
        assert_eq!(log_rat_radix(&one, 9), 0);
    }

    #[test]
    fn test_log_rat_radix_zero() {
        // 0/1 → (1+0 - (1+0)) * ratio = 0
        // (zero has mantissa [0], cdigit=1)
        let zero = Rational::zero();
        assert_eq!(log_rat_radix(&zero, 9), 0);
    }
}
