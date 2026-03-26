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
use super::exp::{_log_rat, exp_rat_in_place, pow_rat_comp};
use super::support::{frac_rat, int_rat};
use super::Number;
use super::Rational;

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
/// This function depends on exp.rs functions (`_log_rat`, `exp_rat_in_place`,
/// `pow_rat_comp`) which are currently stubs. Non-integer factorial results
/// will not be correct until exp.rs is fully ported.
fn _gamma(
    n: &mut Rational,
    radix: u32,
    mut precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    let num_one = Number::from_i32(1, BASEX);

    // Save original precision as rational (needed for error bound later)
    let mut ratprec = Rational::from_i32(precision);

    // ── Find best convergence parameter 'a' ──────────────────────────────
    // a = ln(radix) * precision + 2 + n * ln(a_intermediate) + 1
    let mut a = Rational::from_i32(radix as i32);
    _log_rat(&mut a, precision, constants)?;
    a = mul_rat(&a, &ratprec, precision);
    a = add_rat(&a, &constants.rat_two, precision);

    let mut tmp = a.clone();
    _log_rat(&mut tmp, precision, constants)?;
    tmp = mul_rat(&tmp, n, precision);
    a = add_rat(&a, &tmp, precision);
    a = add_rat(&a, &constants.rat_one, precision);

    // ── Calculate precision bump ─────────────────────────────────────────
    // term = ln(a^(n+1.5) * exp(a)) - ln(radix)
    tmp = n.clone();
    let one_pt_five = div_rat(&Rational::from_i32(3), &constants.rat_two, precision)?;
    tmp = add_rat(&tmp, &one_pt_five, precision);

    let mut term = a.clone();
    pow_rat_comp(&mut term, &tmp, radix, precision, constants)?;
    tmp = a.clone();
    exp_rat_in_place(&mut tmp, radix, precision, constants)?;
    term = mul_rat(&term, &tmp, precision);
    _log_rat(&mut term, precision, constants)?;

    let rat_radix = Rational::from_i32(radix as i32);
    tmp = rat_radix.clone();
    _log_rat(&mut tmp, precision, constants)?;
    term = sub_rat(&term, &tmp, precision);

    // Bump precision by the integer part of term
    if let Ok(bump) = rat_to_i32(&term, radix, precision) {
        precision += bump;
    }

    // ── Series computation ───────────────────────────────────────────────
    let mut factorial = constants.rat_one.clone();
    let mut count = Number::from_i32(0, BASEX);

    // mpy = a^n (computed with original n, before loop modifies it)
    let mut mpy = a.clone();
    pow_rat_comp(&mut mpy, n, radix, precision, constants)?;

    // a2 = a^2 (used to divide factorial each iteration)
    let a2 = mul_rat(&a, &a, precision);

    // Initial sum = 1/n - a/(n+1)
    let mut sum = div_rat(&constants.rat_one, n, precision)?;
    tmp = add_rat(n, &constants.rat_one, precision);
    term = div_rat(&a, &tmp, precision)?;
    sum = sub_rat(&sum, &term, precision);

    // ── Error bound ──────────────────────────────────────────────────────
    // err = radix^(-original_precision) / radix
    let mut err = rat_radix.clone();
    ratprec.negate_mut();
    pow_rat_comp(&mut err, &ratprec, radix, precision, constants)?;
    err = div_rat(&err, &rat_radix, precision)?;

    // ── Main loop ────────────────────────────────────────────────────────
    // Initialize term to 2 (non-zero) to enter the loop
    term = constants.rat_two.clone();

    while !term.is_zero() && rat_gt(&term, &err, precision) {
        // n += 2 (each iteration advances by 2)
        *n = add_rat(n, &constants.rat_two, precision);

        // factorial numerator *= (count+1) * (count+2), then divide by a^2
        count = add_num(&count, &num_one, BASEX);
        let new_p = mul_num_x(factorial.p(), &count);
        *factorial.p_mut() = new_p;

        count = add_num(&count, &num_one, BASEX);
        let new_p = mul_num_x(factorial.p(), &count);
        *factorial.p_mut() = new_p;

        factorial = div_rat(&factorial, &a2, precision)?;

        // tmp = n + 1
        tmp = add_rat(n, &constants.rat_one, precision);

        // term = (count + 1) * (n + 1)
        term = Rational::new(count.clone(), num_one.clone());
        term = add_rat(&term, &constants.rat_one, precision);
        term = mul_rat(&term, &tmp, precision);

        // tmp = a / ((count+1) * (n+1))
        tmp = div_rat(&a, &term, precision)?;

        // term = 1/n - a/((count+1)*(n+1))
        term = div_rat(&constants.rat_one, n, precision)?;
        term = sub_rat(&term, &tmp, precision);

        // term /= factorial
        term = div_rat(&term, &factorial, precision)?;

        // Accumulate
        sum = add_rat(&sum, &term, precision);

        // |term| for convergence check
        term.abs_mut();
    }

    // Result = sum * a^n
    sum = mul_rat(&sum, &mpy, precision);
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
        fact = mul_rat(&fact, x, precision);
        *x = sub_rat(x, &constants.rat_one, precision);
    }

    // ── Snap to integer if x ≈ 0 ────────────────────────────────────────
    if log_rat_radix(x, constants.ratio) <= -precision {
        *x = constants.rat_zero.clone();
        int_rat(&mut fact, radix, precision)?;
    }

    // ── Handle negative non-integer args: divide up toward −1 ────────
    while rat_lt(x, &neg_rat_one, precision) {
        *x = add_rat(x, &constants.rat_one, precision);
        fact = div_rat(&fact, x, precision)?;
    }

    // ── If fractional remainder, use gamma; otherwise return fact ─────
    if rat_neq(x, &constants.rat_zero, precision) {
        *x = add_rat(x, &constants.rat_one, precision);
        _gamma(x, radix, precision, constants)?;
        *x = mul_rat(x, &fact, precision);
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
