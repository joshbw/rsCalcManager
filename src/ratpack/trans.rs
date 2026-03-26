// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Trigonometric and hyperbolic functions.
//! Port of C++ Ratpack/trans.cpp and transh.cpp

use crate::error::{CalcError, CalcResult};
use crate::types::{AngleType, BASEX};

use super::arithmetic::{
    add_num, add_rat, div_rat, mul_num_x, mul_rat, rat_ge, rat_gt, rat_le, rat_lt, sub_rat,
};
use super::constants::RatpackConstants;
use super::exp::exp_rat;
use super::support::{in_between, scale, scale_2pi, trim_rat};
use super::Number;
use super::Rational;

// ---------------------------------------------------------------------------
// Taylor-series helpers (same pattern as exp.rs)
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

// ---------------------------------------------------------------------------
// Scale for angle types
// ---------------------------------------------------------------------------

/// Route angle scaling by type. Port of C++ `scalerat`.
fn scale_rat(
    x: &mut Rational,
    angle_type: AngleType,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    match angle_type {
        AngleType::Radians => scale_2pi(x, &constants.two_pi, radix, precision, constants.ratio),
        AngleType::Degrees => scale(x, &constants.rat_360, radix, precision, constants.ratio),
        AngleType::Gradians => scale(x, &constants.rat_400, radix, precision, constants.ratio),
    }
}

/// Convert radians result to degrees/gradians. Port of C++ `ascalerat`.
pub fn a_scale_rat(
    x: &mut Rational,
    angle_type: AngleType,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    match angle_type {
        AngleType::Radians => {} // no conversion needed
        AngleType::Degrees => {
            *x = div_rat(x, &constants.two_pi, precision)?;
            *x = mul_rat(x, &constants.rat_360, precision);
        }
        AngleType::Gradians => {
            *x = div_rat(x, &constants.two_pi, precision)?;
            *x = mul_rat(x, &constants.rat_400, precision);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// _sinrat — core Taylor series for sin(x)
//
// Series: sin(x) = x - x³/3! + x⁵/5! - ...
// Recurrence: thisterm_{j+1} = thisterm_j * (-x²) / ((2j+2)(2j+3))
// ---------------------------------------------------------------------------

fn _sin_rat(
    x: &mut Rational,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    let ratio = constants.ratio;
    let ti = constants.true_infinite;

    // CREATETAYLOR: xx = x * x
    let mut xx = mul_rat_t(x, x, precision, ratio, ti);

    let mut pret = Rational::new(Number::from_i32(0, BASEX), Number::from_i32(0, BASEX));
    // pret = x (first term)
    *pret.p_mut() = x.p().clone();
    *pret.q_mut() = x.q().clone();
    let mut thisterm = pret.clone();
    let mut n2 = Number::from_i32(1, BASEX);

    // Negate xx for alternating sign: xx = -x²
    xx.p_mut().sign *= -1;

    loop {
        // NEXTTERM: thisterm *= xx
        thisterm = mul_rat_t(&thisterm, &xx, precision, ratio, ti);
        // INC(n2), DIVNUM(n2): thisterm.q *= (n2+1)
        inc_num(&mut n2);
        *thisterm.q_mut() = mul_num_x(thisterm.q(), &n2);
        // INC(n2), DIVNUM(n2): thisterm.q *= (n2+1)
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

    // Clamp to [-1, 1] (TRIMIT can cause epsilon overshoot)
    in_between(x, &constants.rat_one, precision);

    // Snap tiny values to zero
    if rat_le(x, &constants.rat_smallest, precision)
        && rat_ge(x, &constants.rat_neg_smallest, precision)
    {
        *x = constants.rat_zero.clone();
    }

    Ok(())
}

/// Compute sin(x) where x is in radians.
/// Port of C++ `sinrat`.
pub fn sin_rat(
    x: &mut Rational,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    scale_2pi(x, &constants.two_pi, radix, precision, constants.ratio)?;
    _sin_rat(x, precision, constants)
}

/// Compute sin(x) with angle type conversion.
/// Port of C++ `sinanglerat`.
pub fn sin_angle_rat(
    x: &mut Rational,
    angle_type: AngleType,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    scale_rat(x, angle_type, radix, precision, constants)?;
    match angle_type {
        AngleType::Degrees => {
            if rat_gt(x, &constants.rat_180, precision) {
                *x = sub_rat(x, &constants.rat_360, precision);
            }
            *x = div_rat(x, &constants.rat_180, precision)?;
            *x = mul_rat(x, &constants.pi, precision);
        }
        AngleType::Gradians => {
            if rat_gt(x, &constants.rat_200, precision) {
                *x = sub_rat(x, &constants.rat_400, precision);
            }
            *x = div_rat(x, &constants.rat_200, precision)?;
            *x = mul_rat(x, &constants.pi, precision);
        }
        AngleType::Radians => {} // already in radians after scale_2pi
    }
    _sin_rat(x, precision, constants)
}

// ---------------------------------------------------------------------------
// _cosrat — core Taylor series for cos(x)
//
// Series: cos(x) = 1 - x²/2! + x⁴/4! - ...
// Recurrence: same as sin but starts at 1, n2 starts at 0
// ---------------------------------------------------------------------------

fn _cos_rat(
    x: &mut Rational,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    let ratio = constants.ratio;
    let ti = constants.true_infinite;

    // CREATETAYLOR: xx = x * x
    let mut xx = mul_rat_t(x, x, precision, ratio, ti);

    // pret = 1/1 (first term of cosine)
    let mut pret = Rational::one();
    let mut thisterm = pret.clone();
    let mut n2 = Number::from_i32(0, BASEX);

    // Negate xx for alternating sign
    xx.p_mut().sign *= -1;

    loop {
        // NEXTTERM: thisterm *= xx
        thisterm = mul_rat_t(&thisterm, &xx, precision, ratio, ti);
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

    // DESTROYTAYLOR
    trimit(&mut pret, precision, ratio, ti);
    *x = pret;

    // Clamp to [-1, 1]
    in_between(x, &constants.rat_one, precision);

    // Snap tiny values to zero
    if rat_le(x, &constants.rat_smallest, precision)
        && rat_ge(x, &constants.rat_neg_smallest, precision)
    {
        *x = constants.rat_zero.clone();
    }

    Ok(())
}

/// Compute cos(x) where x is in radians.
/// Port of C++ `cosrat`.
pub fn cos_rat(
    x: &mut Rational,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    scale_2pi(x, &constants.two_pi, radix, precision, constants.ratio)?;
    _cos_rat(x, precision, constants)
}

/// Compute cos(x) with angle type conversion.
/// Port of C++ `cosanglerat`.
///
/// NOTE: For degrees, uses `360 - x` (not `x - 360`) when x > 180,
/// matching C++ `DUPRAT(ptmp, rat_360); _subrat(&ptmp, *pa, precision)`.
pub fn cos_angle_rat(
    x: &mut Rational,
    angle_type: AngleType,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    scale_rat(x, angle_type, radix, precision, constants)?;
    match angle_type {
        AngleType::Degrees => {
            if rat_gt(x, &constants.rat_180, precision) {
                // C++: ptmp = 360; ptmp -= x; x = ptmp  (i.e., x = 360 - x)
                *x = sub_rat(&constants.rat_360, x, precision);
            }
            *x = div_rat(x, &constants.rat_180, precision)?;
            *x = mul_rat(x, &constants.pi, precision);
        }
        AngleType::Gradians => {
            if rat_gt(x, &constants.rat_200, precision) {
                // C++: ptmp = 400; ptmp -= x; x = ptmp  (i.e., x = 400 - x)
                *x = sub_rat(&constants.rat_400, x, precision);
            }
            *x = div_rat(x, &constants.rat_200, precision)?;
            *x = mul_rat(x, &constants.pi, precision);
        }
        AngleType::Radians => {}
    }
    _cos_rat(x, precision, constants)
}

// ---------------------------------------------------------------------------
// _tanrat — tan(x) = sin(x) / cos(x)
// ---------------------------------------------------------------------------

fn _tan_rat(
    x: &mut Rational,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    let mut cos_x = x.clone();
    _sin_rat(x, precision, constants)?;
    _cos_rat(&mut cos_x, precision, constants)?;

    if cos_x.is_zero() {
        return Err(CalcError::Domain);
    }

    *x = div_rat(x, &cos_x, precision)?;
    Ok(())
}

/// Compute tan(x) where x is in radians.
/// Port of C++ `tanrat`.
pub fn tan_rat(
    x: &mut Rational,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    scale_2pi(x, &constants.two_pi, radix, precision, constants.ratio)?;
    _tan_rat(x, precision, constants)
}

/// Compute tan(x) with angle type conversion.
/// Port of C++ `tananglerat`.
pub fn tan_angle_rat(
    x: &mut Rational,
    angle_type: AngleType,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    scale_rat(x, angle_type, radix, precision, constants)?;
    match angle_type {
        AngleType::Degrees => {
            if rat_gt(x, &constants.rat_180, precision) {
                *x = sub_rat(x, &constants.rat_180, precision);
            }
            *x = div_rat(x, &constants.rat_180, precision)?;
            *x = mul_rat(x, &constants.pi, precision);
        }
        AngleType::Gradians => {
            if rat_gt(x, &constants.rat_200, precision) {
                *x = sub_rat(x, &constants.rat_200, precision);
            }
            *x = div_rat(x, &constants.rat_200, precision)?;
            *x = mul_rat(x, &constants.pi, precision);
        }
        AngleType::Radians => {}
    }
    _tan_rat(x, precision, constants)
}

// ===========================================================================
// Hyperbolic functions — Port of C++ transh.cpp
// ===========================================================================

/// Domain check for hyperbolic functions.
/// Port of C++ `IsValidForHypFunc`.
///
/// Returns true if x >= rat_min_exp / 10 (i.e., x is not extremely negative).
fn is_valid_for_hyp_func(
    x: &Rational,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<bool> {
    let threshold = div_rat(&constants.rat_min_exp, &constants.rat_ten, precision)?;
    Ok(!rat_lt(x, &threshold, precision))
}

// ---------------------------------------------------------------------------
// _sinhrat — Taylor series for sinh(x) (for |x| < 1)
//
// Series: sinh(x) = x + x³/3! + x⁵/5! + ...
// Same as sin but xx = +x² (no alternating sign)
// ---------------------------------------------------------------------------

fn _sinh_rat(
    x: &mut Rational,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    if !is_valid_for_hyp_func(x, precision, constants)? {
        return Err(CalcError::Domain);
    }

    let ratio = constants.ratio;
    let ti = constants.true_infinite;

    // CREATETAYLOR: xx = x² (NOT negated — no alternating sign for sinh)
    let xx = mul_rat_t(x, x, precision, ratio, ti);

    let mut pret = Rational::new(Number::from_i32(0, BASEX), Number::from_i32(0, BASEX));
    *pret.p_mut() = x.p().clone();
    *pret.q_mut() = x.q().clone();
    let mut thisterm = pret.clone();
    let mut n2 = Number::from_i32(1, BASEX);

    loop {
        // NEXTTERM: thisterm *= xx
        thisterm = mul_rat_t(&thisterm, &xx, precision, ratio, ti);
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

    // DESTROYTAYLOR
    trimit(&mut pret, precision, ratio, ti);
    *x = pret;
    Ok(())
}

/// Compute sinh(x).
/// Port of C++ `sinhrat`.
///
/// For x >= 1: uses (e^x - e^(-x)) / 2.
/// For |x| < 1: uses Taylor series.
pub fn sinh_rat(
    x: &mut Rational,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    if rat_ge(x, &constants.rat_one, precision) {
        // sinh(x) = (e^x - e^(-x)) / 2
        let mut neg_x = x.clone();
        exp_rat(x, radix, precision, constants)?;
        neg_x.p_mut().sign *= -1;
        exp_rat(&mut neg_x, radix, precision, constants)?;
        *x = sub_rat(x, &neg_x, precision);
        *x = div_rat(x, &constants.rat_two, precision)?;
    } else {
        _sinh_rat(x, precision, constants)?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// _coshrat — Taylor series for cosh(x) (for |x| < 1)
//
// Series: cosh(x) = 1 + x²/2! + x⁴/4! + ...
// Same as cos but xx = +x² (no alternating sign)
// ---------------------------------------------------------------------------

fn _cosh_rat(
    x: &mut Rational,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    if !is_valid_for_hyp_func(x, precision, constants)? {
        return Err(CalcError::Domain);
    }

    let ratio = constants.ratio;
    let ti = constants.true_infinite;

    // CREATETAYLOR: xx = x² (NOT negated)
    let xx = mul_rat_t(x, x, precision, ratio, ti);

    // pret = 1/1 (first term of cosh)
    let mut pret = Rational::one();
    let mut thisterm = pret.clone();
    let mut n2 = Number::from_i32(0, BASEX);

    loop {
        // NEXTTERM: thisterm *= xx
        thisterm = mul_rat_t(&thisterm, &xx, precision, ratio, ti);
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

    // DESTROYTAYLOR
    trimit(&mut pret, precision, ratio, ti);
    *x = pret;
    Ok(())
}

/// Compute cosh(x).
/// Port of C++ `coshrat`.
///
/// For |x| >= 1: uses (e^x + e^(-x)) / 2.
/// For |x| < 1: uses Taylor series.
/// Result is always >= 1 (cosh property).
pub fn cosh_rat(
    x: &mut Rational,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    // Take absolute value (cosh is even)
    x.p_mut().sign = 1;
    x.q_mut().sign = 1;

    if rat_ge(x, &constants.rat_one, precision) {
        // cosh(x) = (e^x + e^(-x)) / 2
        let mut neg_x = x.clone();
        exp_rat(x, radix, precision, constants)?;
        neg_x.p_mut().sign *= -1;
        exp_rat(&mut neg_x, radix, precision, constants)?;
        *x = add_rat(x, &neg_x, precision);
        *x = div_rat(x, &constants.rat_two, precision)?;
    } else {
        _cosh_rat(x, precision, constants)?;
    }

    // cosh(x) >= 1; clamp if epsilon below due to TRIMIT
    if rat_lt(x, &constants.rat_one, precision) {
        *x = constants.rat_one.clone();
    }
    Ok(())
}

/// Compute tanh(x).
/// Port of C++ `tanhrat`.
///
/// Uses tanh(x) = sinh(x) / cosh(x) via cross-multiplication of
/// numerator and denominator components.
pub fn tanh_rat(
    x: &mut Rational,
    radix: u32,
    precision: i32,
    constants: &RatpackConstants,
) -> CalcResult<()> {
    let mut cos_x = x.clone();
    sinh_rat(x, radix, precision, constants)?;
    cosh_rat(&mut cos_x, radix, precision, constants)?;

    // Cross-multiply: (sinh_p / sinh_q) / (cosh_p / cosh_q) =
    //   (sinh_p * cosh_q) / (sinh_q * cosh_p)
    let new_p = mul_num_x(x.p(), cos_x.q());
    let new_q = mul_num_x(x.q(), cos_x.p());
    *x.p_mut() = new_p;
    *x.q_mut() = new_q;

    Ok(())
}
