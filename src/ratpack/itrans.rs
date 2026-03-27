// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Inverse trigonometric and inverse hyperbolic functions.
//! Port of C++ Ratpack/itrans.cpp and itransh.cpp

use crate::error::{CalcError, CalcResult};
use crate::types::{AngleType, BASEX};

use super::arithmetic::{
    add_num, add_rat, div_rat, mul_num_x, mul_rat, rat_equ, rat_ge, rat_gt, rat_le, rat_lt,
    sub_rat,
};
use super::constants::RatpackConstants;
use super::exp::{_log_rat, root_rat};
use super::support::trim_rat;
use super::trans::a_scale_rat;
use super::Number;
use super::Rational;

// ---------------------------------------------------------------------------
// Taylor-series helpers (same pattern as exp.rs / trans.rs)
// ---------------------------------------------------------------------------

#[inline]
fn trimit(rat: &mut Rational, precision: i32, ratio: i32, true_infinite: bool) {
    trim_rat(rat, precision, ratio, true_infinite);
}

#[inline]
fn mul_rat_t(
    a: &Rational,
    b: &Rational,
    precision: i32,
    ratio: i32,
    true_infinite: bool,
) -> Rational {
    let mut result = mul_rat(a, b, precision);
    trimit(&mut result, precision, ratio, true_infinite);
    result
}

#[inline]
fn add_rat_t(
    a: &Rational,
    b: &Rational,
    precision: i32,
    ratio: i32,
    true_infinite: bool,
) -> Rational {
    let mut result = add_rat(a, b, precision);
    trimit(&mut result, precision, ratio, true_infinite);
    result
}

#[inline]
#[allow(dead_code)]
fn div_rat_t(
    a: &Rational,
    b: &Rational,
    precision: i32,
    ratio: i32,
    true_infinite: bool,
) -> CalcResult<Rational> {
    let mut result = div_rat(a, b, precision)?;
    trimit(&mut result, precision, ratio, true_infinite);
    Ok(result)
}

#[inline]
fn small_enough_rat(term: &Rational, precision: i32, ratio: i32) -> bool {
    term.p().is_zero()
        || ((term.q().cdigit() + term.q().exp - term.p().cdigit() - term.p().exp - 1) * ratio
            > precision)
}

#[inline]
fn inc_num(n: &mut Number) {
    if n.mantissa[0] < BASEX - 1 {
        n.mantissa[0] += 1;
    } else {
        let one = Number::from_i32(1, BASEX);
        *n = add_num(n, &one, BASEX);
    }
}

// ===========================================================================
// Inverse trigonometric functions — Port of C++ itrans.cpp
// ===========================================================================

// ---------------------------------------------------------------------------
// _asinrat — core Taylor series for asin(x) (for |x| <= 0.85)
//
// Series: asin(x) = x + (1²·x³)/(2·3) + (1²·3²·x⁵)/(2·3·4·5) + ...
// Recurrence: thisterm_{j+1} = thisterm_j * x² * (2j+1)² / ((2j+2)(2j+3))
// NEXTTERM ops: MULNUM(n2) MULNUM(n2) INC(n2) DIVNUM(n2) INC(n2) DIVNUM(n2)
// ---------------------------------------------------------------------------

fn _asin_rat(
    x: &mut Rational,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    let ratio = constants.ratio;
    let ti = constants.true_infinite;

    // CREATETAYLOR: xx = x²
    let xx = mul_rat_t(x, x, precision, ratio, ti);

    let mut pret = Rational::new(Number::from_i32(0, BASEX), Number::from_i32(0, BASEX));
    *pret.p_mut() = x.p().clone();
    *pret.q_mut() = x.q().clone();
    let mut thisterm = pret.clone();
    let mut n2 = Number::from_i32(1, BASEX);

    loop {
        // NEXTTERM: thisterm *= xx
        thisterm = mul_rat_t(&thisterm, &xx, precision, ratio, ti);
        // MULNUM(n2): thisterm.p *= n2
        *thisterm.p_mut() = mul_num_x(thisterm.p(), &n2);
        // MULNUM(n2): thisterm.p *= n2
        *thisterm.p_mut() = mul_num_x(thisterm.p(), &n2);
        // INC(n2), DIVNUM(n2): n2++; thisterm.q *= n2
        inc_num(&mut n2);
        *thisterm.q_mut() = mul_num_x(thisterm.q(), &n2);
        // INC(n2), DIVNUM(n2): n2++; thisterm.q *= n2
        inc_num(&mut n2);
        *thisterm.q_mut() = mul_num_x(thisterm.q(), &n2);
        // pret += thisterm
        pret = add_rat_t(&pret, &thisterm, precision, ratio, ti);

        if small_enough_rat(&thisterm, precision, ratio) {
            break;
        }
    }

    // DESTROYTAYLOR
    trimit(&mut pret, precision, ratio, ti);
    *x = pret;
    Ok(())
}

/// Bootstrap version of _asin_rat for computing π during constant initialization.
/// Does not depend on RatpackConstants (only needs ratio).
///
/// This is analogous to `_exp_rat_bootstrap` in exp.rs.
pub(crate) fn _asin_rat_bootstrap(
    x: &mut Rational,
    precision: i32,
    ratio: i32,
) -> CalcResult<()> {
    let ti = false; // never true_infinite during bootstrapping

    // CREATETAYLOR: xx = x²
    let xx = mul_rat_t(x, x, precision, ratio, ti);

    let mut pret = Rational::new(Number::from_i32(0, BASEX), Number::from_i32(0, BASEX));
    *pret.p_mut() = x.p().clone();
    *pret.q_mut() = x.q().clone();
    let mut thisterm = pret.clone();
    let mut n2 = Number::from_i32(1, BASEX);

    loop {
        thisterm = mul_rat_t(&thisterm, &xx, precision, ratio, ti);
        *thisterm.p_mut() = mul_num_x(thisterm.p(), &n2);
        *thisterm.p_mut() = mul_num_x(thisterm.p(), &n2);
        inc_num(&mut n2);
        *thisterm.q_mut() = mul_num_x(thisterm.q(), &n2);
        inc_num(&mut n2);
        *thisterm.q_mut() = mul_num_x(thisterm.q(), &n2);
        pret = add_rat_t(&pret, &thisterm, precision, ratio, ti);

        if small_enough_rat(&thisterm, precision, ratio) {
            break;
        }
    }

    trimit(&mut pret, precision, ratio, ti);
    *x = pret;
    Ok(())
}

/// Compute asin(x).
/// Port of C++ `asinrat`.
///
/// Domain: |x| <= 1.
/// For |x| near 1: snaps to ±π/2.
/// For |x| > 0.85: uses alternate form π/2 - asin(sqrt(1-x²)).
/// For |x| <= 0.85: uses Taylor series.
pub fn asin_rat(
    x: &mut Rational,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    let sgn = x.sign();
    x.p_mut().sign = 1;
    x.q_mut().sign = 1;

    // Check if |x| ≈ 1 (within epsilon)
    let mut hack = x.clone();
    hack = sub_rat(&hack, &constants.rat_one, precision);
    if rat_le(&hack, &constants.rat_smallest, precision)
        && rat_ge(&hack, &constants.rat_neg_smallest, precision)
    {
        // |x| ≈ 1 → asin(1) = π/2
        *x = constants.pi_over_two.clone();
    } else if rat_gt(x, &constants.pt_eight_five, precision) {
        // |x| > 0.85: use alternate form
        if rat_gt(x, &constants.rat_one, precision) {
            // |x| > 1: check if just epsilon above
            let diff = sub_rat(x, &constants.rat_one, precision);
            if rat_gt(&diff, &constants.rat_smallest, precision) {
                return Err(CalcError::Domain);
            }
            *x = constants.rat_one.clone();
        }
        // Compute sqrt(1 - x²)
        let x_sq = mul_rat(x, x, precision);
        let mut arg = sub_rat(&constants.rat_one, &x_sq, precision);
        root_rat(&mut arg, &constants.rat_two, radix, precision, constants)?;
        // asin(sqrt(1-x²))
        _asin_rat(&mut arg, precision, constants)?;
        // pi/2 - asin(sqrt(1-x²))
        arg.p_mut().sign *= -1;
        *x = add_rat(&constants.pi_over_two, &arg, precision);
    } else {
        // |x| <= 0.85: direct Taylor series
        _asin_rat(x, precision, constants)?;
    }

    // Restore original sign
    x.p_mut().sign = sgn;
    x.q_mut().sign = 1;
    Ok(())
}

/// Compute asin(x) with angle type conversion.
/// Port of C++ `asinanglerat`.
pub fn asin_angle_rat(
    x: &mut Rational,
    angle_type: AngleType,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    asin_rat(x, radix, precision, constants)?;
    a_scale_rat(x, angle_type, precision, constants)
}

// ---------------------------------------------------------------------------
// acosrat — acos(x) = π/2 - asin(x)
// ---------------------------------------------------------------------------

/// Compute acos(x).
/// Port of C++ `acosrat`.
///
/// For |x| = 1: returns 0 (positive) or π (negative).
/// Otherwise: uses π/2 - asin(x).
pub fn acos_rat(
    x: &mut Rational,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    let sgn = x.sign();
    x.p_mut().sign = 1;
    x.q_mut().sign = 1;

    if rat_equ(x, &constants.rat_one, precision) {
        if sgn == -1 {
            *x = constants.pi.clone();
        } else {
            *x = constants.rat_zero.clone();
        }
    } else {
        x.p_mut().sign = sgn;
        asin_rat(x, radix, precision, constants)?;
        x.p_mut().sign *= -1;
        *x = add_rat(x, &constants.pi_over_two, precision);
    }
    Ok(())
}

/// Compute acos(x) with angle type conversion.
/// Port of C++ `acosanglerat`.
pub fn acos_angle_rat(
    x: &mut Rational,
    angle_type: AngleType,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    acos_rat(x, radix, precision, constants)?;
    a_scale_rat(x, angle_type, precision, constants)
}

// ---------------------------------------------------------------------------
// _atanrat — core Taylor series for atan(x) (for |x| <= 0.85)
//
// Series: atan(x) = x - x³/3 + x⁵/5 - ...
// Recurrence: thisterm_{j+1} = thisterm_j * (-x²) * (2j+1) / (2j+3)
// NEXTTERM ops: MULNUM(n2) INC(n2) INC(n2) DIVNUM(n2)
// ---------------------------------------------------------------------------

fn _atan_rat(
    x: &mut Rational,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    let ratio = constants.ratio;
    let ti = constants.true_infinite;

    // CREATETAYLOR: xx = x²
    let mut xx = mul_rat_t(x, x, precision, ratio, ti);

    let mut pret = Rational::new(Number::from_i32(0, BASEX), Number::from_i32(0, BASEX));
    *pret.p_mut() = x.p().clone();
    *pret.q_mut() = x.q().clone();
    let mut thisterm = pret.clone();
    let mut n2 = Number::from_i32(1, BASEX);

    // Negate xx for alternating sign
    xx.p_mut().sign *= -1;

    loop {
        // NEXTTERM: thisterm *= xx
        thisterm = mul_rat_t(&thisterm, &xx, precision, ratio, ti);
        // MULNUM(n2): thisterm.p *= n2
        *thisterm.p_mut() = mul_num_x(thisterm.p(), &n2);
        // INC(n2): n2++
        inc_num(&mut n2);
        // INC(n2): n2++
        inc_num(&mut n2);
        // DIVNUM(n2): thisterm.q *= n2
        *thisterm.q_mut() = mul_num_x(thisterm.q(), &n2);
        // pret += thisterm
        pret = add_rat_t(&pret, &thisterm, precision, ratio, ti);

        if small_enough_rat(&thisterm, precision, ratio) {
            break;
        }
    }

    // DESTROYTAYLOR
    trimit(&mut pret, precision, ratio, ti);
    *x = pret;
    Ok(())
}

/// Compute atan(x).
/// Port of C++ `atanrat`.
///
/// For |x| <= 0.85: uses Taylor series.
/// For 0.85 < |x| <= 2: uses asin(x / sqrt(1+x²)).
/// For |x| > 2: uses π/2 - atan(1/x).
pub fn atan_rat(
    x: &mut Rational,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    let sgn = x.sign();
    x.p_mut().sign = 1;
    x.q_mut().sign = 1;

    if rat_gt(x, &constants.pt_eight_five, precision) {
        if rat_gt(x, &constants.rat_two, precision) {
            // |x| > 2: atan(x) = sgn(x) * (π/2 - atan(1/x))
            x.p_mut().sign = sgn;
            x.q_mut().sign = 1;
            let mut tmp = div_rat(&constants.rat_one, x, precision)?;
            _atan_rat(&mut tmp, precision, constants)?;
            tmp.p_mut().sign = sgn;
            tmp.q_mut().sign = 1;
            *x = sub_rat(&constants.pi_over_two, &tmp, precision);
        } else {
            // 0.85 < |x| <= 2: atan(x) = asin(x / sqrt(1+x²))
            x.p_mut().sign = sgn;
            let mut tmp = mul_rat(x, x, precision);
            tmp = add_rat(&tmp, &constants.rat_one, precision);
            root_rat(&mut tmp, &constants.rat_two, radix, precision, constants)?;
            *x = div_rat(x, &tmp, precision)?;
            asin_rat(x, radix, precision, constants)?;
            x.p_mut().sign = sgn;
            x.q_mut().sign = 1;
        }
    } else {
        // |x| <= 0.85: direct Taylor series
        x.p_mut().sign = sgn;
        x.q_mut().sign = 1;
        _atan_rat(x, precision, constants)?;
    }

    // Correction: if result > pi/2, subtract pi (matching C++)
    if rat_gt(x, &constants.pi_over_two, precision) {
        *x = sub_rat(x, &constants.pi, precision);
    }
    Ok(())
}

/// Compute atan(x) with angle type conversion.
/// Port of C++ `atananglerat`.
pub fn atan_angle_rat(
    x: &mut Rational,
    angle_type: AngleType,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    atan_rat(x, radix, precision, constants)?;
    a_scale_rat(x, angle_type, precision, constants)
}

// ===========================================================================
// Inverse hyperbolic functions — Port of C++ itransh.cpp
// ===========================================================================

/// Compute asinh(x).
/// Port of C++ `asinhrat`.
///
/// For |x| >= 0.85: uses asinh(x) = ln(x + sqrt(x²+1)).
/// For |x| < 0.85: uses Taylor series (same as asin but with xx negated).
pub fn asinh_rat(
    x: &mut Rational,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    let neg_pt85 = constants.pt_eight_five.negate();
    if rat_gt(x, &constants.pt_eight_five, precision)
        || rat_lt(x, &neg_pt85, precision)
    {
        // asinh(x) = ln(x + sqrt(x² + 1))
        let mut tmp = mul_rat(x, x, precision);
        tmp = add_rat(&tmp, &constants.rat_one, precision);
        root_rat(&mut tmp, &constants.rat_two, radix, precision, constants)?;
        *x = add_rat(x, &tmp, precision);
        _log_rat(x, precision, constants)?;
    } else {
        // Taylor series for |x| < 0.85
        let ratio = constants.ratio;
        let ti = constants.true_infinite;

        // xx = x², then negate for alternating sign
        let mut xx = mul_rat_t(x, x, precision, ratio, ti);
        xx.p_mut().sign *= -1;

        let mut pret = Rational::new(Number::from_i32(0, BASEX), Number::from_i32(0, BASEX));
        *pret.p_mut() = x.p().clone();
        *pret.q_mut() = x.q().clone();
        let mut thisterm = pret.clone();
        let mut n2 = Number::from_i32(1, BASEX);

        loop {
            // NEXTTERM: thisterm *= xx
            thisterm = mul_rat_t(&thisterm, &xx, precision, ratio, ti);
            // MULNUM(n2): thisterm.p *= n2
            *thisterm.p_mut() = mul_num_x(thisterm.p(), &n2);
            // MULNUM(n2): thisterm.p *= n2
            *thisterm.p_mut() = mul_num_x(thisterm.p(), &n2);
            // INC(n2), DIVNUM(n2)
            inc_num(&mut n2);
            *thisterm.q_mut() = mul_num_x(thisterm.q(), &n2);
            // INC(n2), DIVNUM(n2)
            inc_num(&mut n2);
            *thisterm.q_mut() = mul_num_x(thisterm.q(), &n2);
            // pret += thisterm
            pret = add_rat_t(&pret, &thisterm, precision, ratio, ti);

            if small_enough_rat(&thisterm, precision, ratio) {
                break;
            }
        }

        trimit(&mut pret, precision, ratio, ti);
        *x = pret;
    }
    Ok(())
}

/// Compute acosh(x).
/// Port of C++ `acoshrat`.
///
/// Formula: acosh(x) = ln(x + sqrt(x²-1)), domain x >= 1.
pub fn acosh_rat(
    x: &mut Rational,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    if rat_lt(x, &constants.rat_one, precision) {
        return Err(CalcError::Domain);
    }

    // acosh(x) = ln(x + sqrt(x² - 1))
    let mut tmp = mul_rat(x, x, precision);
    tmp = sub_rat(&tmp, &constants.rat_one, precision);
    root_rat(&mut tmp, &constants.rat_two, radix, precision, constants)?;
    *x = add_rat(x, &tmp, precision);
    _log_rat(x, precision, constants)?;
    Ok(())
}

/// Compute atanh(x).
/// Port of C++ `atanhrat`.
///
/// Formula: atanh(x) = (1/2) * ln((x+1)/(x-1)).
/// Note: C++ computes (x+1)/(x-1) then negates, then log, then /2.
pub fn atanh_rat(
    x: &mut Rational,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    // ptmp = x - 1
    let ptmp = sub_rat(x, &constants.rat_one, precision);
    // x = x + 1
    *x = add_rat(x, &constants.rat_one, precision);
    // x = (x+1) / (x-1)
    *x = div_rat(x, &ptmp, precision)?;
    // Negate (matches C++: px->pp->sign *= -1 after divrat)
    x.p_mut().sign *= -1;
    // ln
    _log_rat(x, precision, constants)?;
    // /2
    *x = div_rat(x, &constants.rat_two, precision)?;
    Ok(())
}
