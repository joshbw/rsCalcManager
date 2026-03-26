// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Base conversion operations.
//! Port of C++ Ratpack/basex.cpp
//!
//! Numbers are stored internally in base `BASEX` (2^31). These routines convert
//! between that internal representation and a human-readable radix (2, 8, 10, 16).

use super::arithmetic::{add_num, mul_num, num_pow_i32};
use super::Number;
use crate::types::BASEX;

/// Convert a number from internal `BASEX` representation to an external radix.
///
/// Port of C++ `nRadixxtonum`.
///
/// Algorithm: extract every bit of the BASEX mantissa from MSD to LSD, doubling
/// the running sum each time (in the target radix) and OR-ing in the bit value.
/// Then scale by `BASEX^(exp_offset)`.
pub fn num_to_radix(num: &Number, radix: u32, precision: i32) -> Number {
    let mut sum = Number::from_i32(0, radix);

    // Limit digits to avoid costly conversion of invisible digits.
    let mut cdigits = (precision + 1) as usize;
    if cdigits > num.mantissa.len() {
        cdigits = num.mantissa.len();
    }

    // Compute the exponent offset for the LSD we're actually processing.
    let exp_offset = num.exp + (num.cdigit() - cdigits as i32);

    // Create BASEX as a Number in the target radix, raised to exp_offset.
    let pow_of_radix = {
        let base_in_radix = Number::from_u32(BASEX, radix);
        num_pow_i32(&base_in_radix, exp_offset, radix, precision)
    };

    // Loop over digits from MSD to LSD.
    // mantissa is little-endian, so MSD is at index (mantissa.len() - 1).
    let start = num.mantissa.len() - 1;
    let end = num.mantissa.len() - cdigits;
    for idx in (end..=start).rev() {
        let digit = num.mantissa[idx];
        // Loop over all 31 bits from MSB to LSB within each BASEX digit.
        // BASEX = 2^31, so BASEX/2 = 2^30.
        let mut bitmask: u32 = BASEX / 2; // 0x4000_0000
        while bitmask > 0 {
            // Double the sum: sum = sum + sum
            sum = add_num(&sum, &sum, radix);
            // If bit is set, set LSB of result
            if digit & bitmask != 0 {
                sum.mantissa[0] |= 1;
            }
            bitmask /= 2;
        }
    }

    // Scale answer by power of internal exponent.
    sum = mul_num(&sum, &pow_of_radix, radix);

    // Propagate sign.
    sum.sign = num.sign;
    sum
}

/// Convert a number from external radix to internal `BASEX` representation.
///
/// Port of C++ `numtonRadixx`.
///
/// Algorithm: Horner's method — for each digit from MSD to LSD:
///   result = result × radix + digit
/// Then scale by `radix^exp`.
pub fn num_from_radix(num: &Number, radix: u32) -> Number {
    let mut result = Number::from_i32(0, BASEX);
    let num_radix = Number::from_u32(radix, BASEX);

    // Digits are little-endian; iterate from MSD (last) to LSD (first).
    for idx in (0..num.mantissa.len()).rev() {
        // result = result * radix
        result = mul_num(&result, &num_radix, BASEX);
        // result += digit
        let this_digit = Number::from_u32(num.mantissa[idx], BASEX);
        result = add_num(&result, &this_digit, BASEX);
    }

    // Scale by radix^exp.
    let scale = {
        let base = Number::from_u32(radix, BASEX);
        num_pow_i32_x(&base, num.exp)
    };
    result = mul_num(&result, &scale, BASEX);

    // Propagate sign.
    result.sign = num.sign;
    result
}

/// Optimized power function in BASEX.
/// Port of C++ `numpowi32x` — binary exponentiation.
fn num_pow_i32_x(base: &Number, power: i32) -> Number {
    use super::arithmetic::num_pow_i32_x as pow_x;
    pow_x(base, power)
}

/// Convert a Number in external radix to a Rational (p/q in BASEX).
///
/// Port of C++ `numtorat`.
///
/// The rational is constructed so that p and q are integers in BASEX:
/// - If the number has a negative exponent (fractional part), the denominator
///   absorbs that exponent so both p and q are integers.
pub fn num_to_rat(num: &Number, radix: u32) -> super::Rational {
    let mut p_radix = num.clone();
    let mut q_radix = Number::from_i32(1, radix);

    // Ensure p and q start as integers.
    if p_radix.exp < 0 {
        q_radix.exp -= p_radix.exp;
        p_radix.exp = 0;
    }

    // Convert both to internal BASEX representation.
    let p = num_from_radix(&p_radix, radix);
    let q = num_from_radix(&q_radix, radix);

    super::Rational::new(p, q)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_num_from_radix_simple() {
        // 42 in base 10 → BASEX
        let n10 = Number::from_i32(42, 10);
        let n_basex = num_from_radix(&n10, 10);
        assert_eq!(n_basex.to_i32(BASEX), Some(42));
    }

    #[test]
    fn test_num_to_radix_simple() {
        // 42 in BASEX → base 10
        let n_basex = Number::from_i32(42, BASEX);
        let n10 = num_to_radix(&n_basex, 10, 32);
        assert_eq!(n10.to_i32(10), Some(42));
    }

    #[test]
    fn test_roundtrip_base10() {
        // 12345 in base 10 → BASEX → base 10
        let original = Number::from_i32(12345, 10);
        let basex = num_from_radix(&original, 10);
        let back = num_to_radix(&basex, 10, 32);
        assert_eq!(back.to_i32(10), Some(12345));
    }

    #[test]
    fn test_roundtrip_base16() {
        // 0xFF (255) in base 16 → BASEX → base 16
        let original = Number::from_i32(255, 16);
        let basex = num_from_radix(&original, 16);
        let back = num_to_radix(&basex, 16, 32);
        assert_eq!(back.to_i32(16), Some(255));
    }

    #[test]
    fn test_negative() {
        let n10 = Number::from_i32(-99, 10);
        let basex = num_from_radix(&n10, 10);
        let back = num_to_radix(&basex, 10, 32);
        assert_eq!(back.to_i32(10), Some(-99));
    }

    #[test]
    fn test_zero() {
        let z = Number::zero();
        let basex = num_from_radix(&z, 10);
        assert!(basex.is_zero());
        let back = num_to_radix(&basex, 10, 32);
        assert!(back.is_zero());
    }

    #[test]
    fn test_num_to_rat() {
        // "3.14" would be: mantissa [4,1,3], exp = -2 in radix 10
        // p should represent 314, q should represent 100
        let n = Number::new(1, -2, vec![4, 1, 3]);
        let rat = num_to_rat(&n, 10);
        // p should be 314 in BASEX, q should be 100 in BASEX
        assert_eq!(rat.p().to_i32(BASEX), Some(314));
        assert_eq!(rat.q().to_i32(BASEX), Some(100));
    }
}
