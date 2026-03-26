// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Basic arithmetic operations on Numbers and Rationals.
//!
//! Ports the arithmetic from C++ ratpack: addnum, mulnum, divnum, etc.

use crate::error::{CalcError, CalcResult};
use crate::types::{MantType, TwoMantType, BASEX};

use super::number::Number;
use super::rational::Rational;

/// Add two numbers in the given radix: *pa += b.
/// Port of C++ `addnum`.
pub fn add_num(a: &Number, b: &Number, radix: u32) -> Number {
    if a.is_zero() {
        return b.clone();
    }
    if b.is_zero() {
        return a.clone();
    }

    // Same sign: add magnitudes
    if a.sign == b.sign {
        let result = add_magnitudes(a, b, radix);
        return Number::new(a.sign, result.exp, result.mantissa);
    }

    // Different signs: subtract the smaller magnitude from the larger
    let cmp = compare_magnitudes(a, b);
    match cmp {
        0 => Number::zero(),
        1 => {
            // |a| > |b|
            let result = sub_magnitudes(a, b, radix);
            Number::new(a.sign, result.exp, result.mantissa)
        }
        _ => {
            // |b| > |a|
            let result = sub_magnitudes(b, a, radix);
            Number::new(b.sign, result.exp, result.mantissa)
        }
    }
}

/// Multiply two numbers in the given radix.
/// Port of C++ `mulnum`.
pub fn mul_num(a: &Number, b: &Number, radix: u32) -> Number {
    if a.is_zero() || b.is_zero() {
        return Number::zero();
    }

    let sign = a.sign * b.sign;
    let exp = a.exp + b.exp;

    let a_len = a.mantissa.len();
    let b_len = b.mantissa.len();
    let result_len = a_len + b_len;

    let mut result = vec![0u32; result_len];
    let radix_u64 = u64::from(radix);

    for i in 0..a_len {
        let mut carry: u64 = 0;
        let a_digit = u64::from(a.mantissa[i]);
        for j in 0..b_len {
            let product = a_digit * u64::from(b.mantissa[j]) + u64::from(result[i + j]) + carry;
            result[i + j] = (product % radix_u64) as u32;
            carry = product / radix_u64;
        }
        if carry > 0 {
            result[i + b_len] = carry as u32;
        }
    }

    // Remove leading zeros
    while result.len() > 1 && result.last() == Some(&0) {
        result.pop();
    }

    Number::new(sign, exp, result)
}

/// Multiply two numbers in BASEX (internal radix).
/// Port of C++ `mulnumx`.
pub fn mul_num_x(a: &Number, b: &Number) -> Number {
    mul_num(a, b, BASEX)
}

/// Divide number a by number b in the given radix with specified precision.
/// Port of C++ `divnum`.
pub fn div_num(a: &Number, b: &Number, radix: u32, precision: i32) -> CalcResult<Number> {
    if b.is_zero() {
        return Err(CalcError::DivideByZero);
    }
    if a.is_zero() {
        return Ok(Number::zero());
    }

    let sign = a.sign * b.sign;

    // Long division algorithm
    let max_digits = (precision as usize) + b.mantissa.len() + 2;
    let mut quotient_digits: Vec<MantType> = Vec::with_capacity(max_digits);
    let radix_u64 = u64::from(radix);

    // Build dividend from a's mantissa
    let mut remainder: Vec<MantType> = a.mantissa.clone();

    // Normalize: shift until remainder >= divisor or we've done enough digits
    let b_val = &b.mantissa;
    let mut exp_adjust = a.exp - b.exp;
    let mut digits_computed = 0;

    // Simple long division: divide remainder by b, one digit at a time
    // For each quotient digit, we prepend a zero to remainder (shift left)
    // and find how many times b goes into the current remainder

    // For simplicity, use a trial division approach
    // This is a placeholder - full implementation will port the C++ algorithm
    let mut dividend_parts = remainder.clone();
    dividend_parts.reverse(); // big-endian for easier processing

    // Trial division producing quotient digits
    let mut rem: u64 = 0;
    for &d in &dividend_parts {
        rem = rem * radix_u64 + u64::from(d);
        // Simple single-digit divisor case
        if b.mantissa.len() == 1 {
            let divisor = u64::from(b.mantissa[0]);
            quotient_digits.push((rem / divisor) as MantType);
            rem %= divisor;
        }
    }

    if b.mantissa.len() > 1 {
        // Multi-digit division - needs full implementation
        // For now, fall back to a simpler approach
        // TODO: Port full divnum from C++
        return Err(CalcError::InvalidRange);
    }

    // Remove leading zeros from quotient
    while quotient_digits.len() > 1 && quotient_digits[0] == 0 {
        quotient_digits.remove(0);
        exp_adjust += 1;
    }

    quotient_digits.reverse(); // back to little-endian

    if quotient_digits.is_empty() {
        return Ok(Number::zero());
    }

    Ok(Number::new(sign, exp_adjust, quotient_digits))
}

/// Add two rationals with the given precision.
/// Port of C++ `addrat`.
pub fn add_rat(a: &Rational, b: &Rational, precision: i32) -> Rational {
    // a.p/a.q + b.p/b.q = (a.p*b.q + b.p*a.q) / (a.q*b.q)
    let new_p1 = mul_num_x(a.p(), b.q());
    let new_p2 = mul_num_x(b.p(), a.q());
    let new_p = add_num(&new_p1, &new_p2, BASEX);
    let new_q = mul_num_x(a.q(), b.q());

    Rational::new(new_p, new_q)
}

/// Subtract two rationals: a - b.
/// Port of C++ `subrat`.
pub fn sub_rat(a: &Rational, b: &Rational, precision: i32) -> Rational {
    let neg_b = b.negate();
    add_rat(a, &neg_b, precision)
}

/// Multiply two rationals.
/// Port of C++ `mulrat`.
pub fn mul_rat(a: &Rational, b: &Rational, precision: i32) -> Rational {
    let new_p = mul_num_x(a.p(), b.p());
    let new_q = mul_num_x(a.q(), b.q());
    Rational::new(new_p, new_q)
}

/// Divide two rationals: a / b.
/// Port of C++ `divrat`.
pub fn div_rat(a: &Rational, b: &Rational, precision: i32) -> CalcResult<Rational> {
    if b.p().is_zero() {
        return Err(CalcError::DivideByZero);
    }

    // a/b = (a.p * b.q) / (a.q * b.p)
    let new_p = mul_num_x(a.p(), b.q());
    let new_q = mul_num_x(a.q(), b.p());
    Ok(Rational::new(new_p, new_q))
}

/// Remainder: a % b.
/// Port of C++ `remrat`.
pub fn rem_rat(a: &Rational, b: &Rational) -> CalcResult<Rational> {
    if b.p().is_zero() {
        return Err(CalcError::DivideByZero);
    }

    // a % b = a - b * floor(a/b)
    // For integer rationals: (a.p * b.q) % (b.p * a.q) / (a.q * b.q)
    let cross_a = mul_num_x(a.p(), b.q());
    let cross_b = mul_num_x(b.p(), a.q());
    let new_q = mul_num_x(a.q(), b.q());

    // TODO: implement proper remainder via integer division
    // Placeholder
    Ok(Rational::zero())
}

/// Raise rational to an i32 power.
/// Port of C++ `ratpowi32`.
pub fn rat_pow_i32(base: &Rational, power: i32, precision: i32) -> CalcResult<Rational> {
    if power == 0 {
        return Ok(Rational::one());
    }

    let (base, power) = if power < 0 {
        // Negative power: invert base
        if base.is_zero() {
            return Err(CalcError::DivideByZero);
        }
        let inverted = Rational::new(base.q().clone(), base.p().clone());
        (inverted, -power)
    } else {
        (base.clone(), power)
    };

    // Binary exponentiation
    let mut result = Rational::one();
    let mut current = base;
    let mut exp = power as u32;

    while exp > 0 {
        if exp & 1 == 1 {
            result = mul_rat(&result, &current, precision);
        }
        exp >>= 1;
        if exp > 0 {
            current = mul_rat(&current, &current, precision);
        }
    }

    Ok(result)
}

/// Compare two rationals.
/// Returns true if a == b.
pub fn rat_equ(a: &Rational, b: &Rational, precision: i32) -> bool {
    let diff = sub_rat(a, b, precision);
    diff.is_zero()
}

/// Compare: a < b.
pub fn rat_lt(a: &Rational, b: &Rational, precision: i32) -> bool {
    let diff = sub_rat(a, b, precision);
    diff.sign() < 0 && !diff.is_zero()
}

/// Compare: a > b.
pub fn rat_gt(a: &Rational, b: &Rational, precision: i32) -> bool {
    let diff = sub_rat(a, b, precision);
    diff.sign() > 0 && !diff.is_zero()
}

/// Compare: a <= b.
pub fn rat_le(a: &Rational, b: &Rational, precision: i32) -> bool {
    !rat_gt(a, b, precision)
}

/// Compare: a >= b.
pub fn rat_ge(a: &Rational, b: &Rational, precision: i32) -> bool {
    !rat_lt(a, b, precision)
}

/// Compare: a != b.
pub fn rat_neq(a: &Rational, b: &Rational, precision: i32) -> bool {
    !rat_equ(a, b, precision)
}

// --- Internal helpers ---

/// Compare magnitudes of two numbers (ignoring sign).
fn compare_magnitudes(a: &Number, b: &Number) -> i32 {
    let a_log = a.cdigit() + a.exp;
    let b_log = b.cdigit() + b.exp;

    if a_log != b_log {
        return if a_log > b_log { 1 } else { -1 };
    }

    // Same order: compare digits from most significant
    let a_len = a.mantissa.len();
    let b_len = b.mantissa.len();
    let max_len = a_len.max(b_len);

    for i in (0..max_len).rev() {
        let ad = if i < a_len { a.mantissa[i] } else { 0 };
        let bd = if i < b_len { b.mantissa[i] } else { 0 };
        if ad != bd {
            return if ad > bd { 1 } else { -1 };
        }
    }

    0
}

/// Add magnitudes of two numbers (same sign assumed).
fn add_magnitudes(a: &Number, b: &Number, radix: u32) -> Number {
    // Align exponents
    let min_exp = a.exp.min(b.exp);

    let a_offset = (a.exp - min_exp) as usize;
    let b_offset = (b.exp - min_exp) as usize;

    let a_top = a_offset + a.mantissa.len();
    let b_top = b_offset + b.mantissa.len();
    let result_len = a_top.max(b_top) + 1; // +1 for carry

    let mut result = vec![0u32; result_len];
    let radix_u64 = u64::from(radix);
    let mut carry: u64 = 0;

    for i in 0..result_len {
        let a_digit = if i >= a_offset && i < a_top {
            u64::from(a.mantissa[i - a_offset])
        } else {
            0
        };
        let b_digit = if i >= b_offset && i < b_top {
            u64::from(b.mantissa[i - b_offset])
        } else {
            0
        };

        let sum = a_digit + b_digit + carry;
        result[i] = (sum % radix_u64) as u32;
        carry = sum / radix_u64;
    }

    // Remove leading zeros
    while result.len() > 1 && result.last() == Some(&0) {
        result.pop();
    }

    Number::new(1, min_exp, result)
}

/// Subtract magnitudes: |a| - |b| where |a| >= |b|.
fn sub_magnitudes(a: &Number, b: &Number, radix: u32) -> Number {
    let min_exp = a.exp.min(b.exp);

    let a_offset = (a.exp - min_exp) as usize;
    let b_offset = (b.exp - min_exp) as usize;

    let a_top = a_offset + a.mantissa.len();
    let b_top = b_offset + b.mantissa.len();
    let result_len = a_top.max(b_top);

    let mut result = vec![0i64; result_len];
    let radix_i64 = i64::from(radix);

    // Fill with a's digits
    for i in 0..a.mantissa.len() {
        result[i + a_offset] += i64::from(a.mantissa[i]);
    }
    // Subtract b's digits
    for i in 0..b.mantissa.len() {
        result[i + b_offset] -= i64::from(b.mantissa[i]);
    }

    // Propagate borrows
    for i in 0..result.len() - 1 {
        if result[i] < 0 {
            result[i] += radix_i64;
            result[i + 1] -= 1;
        }
    }

    // Convert to u32 and remove leading zeros
    let mut mantissa: Vec<u32> = result.iter().map(|&d| d as u32).collect();
    while mantissa.len() > 1 && mantissa.last() == Some(&0) {
        mantissa.pop();
    }

    Number::new(1, min_exp, mantissa)
}

// Implement standard ops for Rational using the arithmetic functions
impl std::ops::Add for &Rational {
    type Output = Rational;
    fn add(self, rhs: &Rational) -> Rational {
        add_rat(self, rhs, RATIONAL_PRECISION)
    }
}

impl std::ops::Sub for &Rational {
    type Output = Rational;
    fn sub(self, rhs: &Rational) -> Rational {
        sub_rat(self, rhs, RATIONAL_PRECISION)
    }
}

impl std::ops::Mul for &Rational {
    type Output = Rational;
    fn mul(self, rhs: &Rational) -> Rational {
        mul_rat(self, rhs, RATIONAL_PRECISION)
    }
}

impl std::ops::Div for &Rational {
    type Output = CalcResult<Rational>;
    fn div(self, rhs: &Rational) -> CalcResult<Rational> {
        div_rat(self, rhs, RATIONAL_PRECISION)
    }
}

use super::rational::RATIONAL_PRECISION;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_rat() {
        let a = Rational::from(3i32);
        let b = Rational::from(4i32);
        let result = &a + &b;
        // 3/1 + 4/1 = 7/1 in BASEX representation
        assert!(!result.is_zero());
        assert_eq!(result.sign(), 1);
    }

    #[test]
    fn test_mul_rat() {
        let a = Rational::from(3i32);
        let b = Rational::from(4i32);
        let result = &a * &b;
        assert!(!result.is_zero());
    }

    #[test]
    fn test_div_rat_by_zero() {
        let a = Rational::from(3i32);
        let b = Rational::zero();
        assert!((&a / &b).is_err());
    }

    #[test]
    fn test_sub_rat() {
        let a = Rational::from(5i32);
        let b = Rational::from(5i32);
        let result = &a - &b;
        assert!(result.is_zero());
    }

    #[test]
    fn test_add_num_basex() {
        let a = Number::from_i32(100, BASEX);
        let b = Number::from_i32(200, BASEX);
        let result = add_num(&a, &b, BASEX);
        assert!(!result.is_zero());
    }
}
