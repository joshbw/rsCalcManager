// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Arbitrary-precision number type.
//!
//! Replaces the C++ `NUMBER` struct which used a flexible array member (`mant[]`)
//! with raw pointer manipulation. This Rust version uses `Vec<u32>` for the mantissa.

use std::fmt;

use crate::types::{MantType, BASEX};

/// An arbitrary-precision integer in a configurable radix.
///
/// Internally stored as:
/// - `sign`: +1 or -1
/// - `exp`: exponent (offset of mantissa digits from the radix point)
/// - `mantissa`: digits in little-endian order (least significant first)
///
/// The value represented is: sign × (Σ mantissa[i] × radix^(i + exp))
#[derive(Clone, PartialEq, Eq)]
pub struct Number {
    /// Sign: 1 for positive, -1 for negative.
    pub sign: i32,
    /// Exponent: offset from the radix point.
    pub exp: i32,
    /// Mantissa digits in little-endian order.
    pub mantissa: Vec<MantType>,
}

impl Number {
    /// Create a new number with the given sign, exponent, and mantissa.
    #[must_use]
    pub fn new(sign: i32, exp: i32, mantissa: Vec<MantType>) -> Self {
        Self {
            sign,
            exp,
            mantissa,
        }
    }

    /// Create a zero value.
    #[must_use]
    pub fn zero() -> Self {
        Self {
            sign: 1,
            exp: 0,
            mantissa: vec![0],
        }
    }

    /// Create a number from a single digit.
    #[must_use]
    pub fn from_digit(digit: MantType) -> Self {
        Self {
            sign: 1,
            exp: 0,
            mantissa: vec![digit],
        }
    }

    /// The number of digits in the mantissa (matches C++ `cdigit`).
    #[must_use]
    pub fn cdigit(&self) -> i32 {
        self.mantissa.len() as i32
    }

    /// The most significant digit.
    #[must_use]
    pub fn msd(&self) -> MantType {
        *self.mantissa.last().unwrap_or(&0)
    }

    /// Check if the number is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.mantissa.iter().all(|&d| d == 0)
    }

    /// Create a Number from an i32 value in the given radix.
    #[must_use]
    pub fn from_i32(value: i32, radix: u32) -> Self {
        let sign = if value < 0 { -1 } else { 1 };
        let mut abs_val = (value as i64).unsigned_abs();

        if abs_val == 0 {
            return Self {
                sign: 1,
                exp: 0,
                mantissa: vec![0],
            };
        }

        let mut digits = Vec::new();
        let radix_u64 = u64::from(radix);
        while abs_val > 0 {
            digits.push((abs_val % radix_u64) as MantType);
            abs_val /= radix_u64;
        }

        Self {
            sign,
            exp: 0,
            mantissa: digits,
        }
    }

    /// Create a Number from a u32 value in the given radix.
    #[must_use]
    pub fn from_u32(value: u32, radix: u32) -> Self {
        let mut val = u64::from(value);

        if val == 0 {
            return Self {
                sign: 1,
                exp: 0,
                mantissa: vec![0],
            };
        }

        let mut digits = Vec::new();
        let radix_u64 = u64::from(radix);
        while val > 0 {
            digits.push((val % radix_u64) as MantType);
            val /= radix_u64;
        }

        Self {
            sign: 1,
            exp: 0,
            mantissa: digits,
        }
    }

    /// Duplicate this number (explicit clone with matching C++ DUPNUM semantics).
    #[must_use]
    pub fn dup(&self) -> Self {
        self.clone()
    }

    /// Convert this number to an i32 in the given radix.
    /// Returns None if the number doesn't fit.
    pub fn to_i32(&self, radix: u32) -> Option<i32> {
        let radix_i64 = i64::from(radix);
        let mut result: i64 = 0;
        let mut place: i64 = 1;

        // Account for exponent by starting the place value higher
        for _ in 0..self.exp {
            place = place.checked_mul(radix_i64)?;
        }

        for &digit in &self.mantissa {
            result = result.checked_add(i64::from(digit).checked_mul(place)?)?;
            place = place.checked_mul(radix_i64)?;
        }

        result *= i64::from(self.sign);

        i32::try_from(result).ok()
    }

    /// Compute log2 estimate: cdigit + exp.
    /// Matches the C++ LOGNUM2 macro.
    #[must_use]
    pub fn log2_estimate(&self) -> i32 {
        self.cdigit() + self.exp
    }

    /// Strip trailing zero digits (least significant).
    pub fn strip_trailing_zeros(&mut self) {
        while self.mantissa.len() > 1 && self.mantissa[0] == 0 {
            self.mantissa.remove(0);
            self.exp += 1;
        }
    }

    /// Comparison between two numbers in the same radix.
    /// Returns -1, 0, or 1.
    #[must_use]
    pub fn compare(&self, other: &Self) -> i32 {
        // Handle zeros
        let self_zero = self.is_zero();
        let other_zero = other.is_zero();

        if self_zero && other_zero {
            return 0;
        }
        if self_zero {
            return -other.sign;
        }
        if other_zero {
            return self.sign;
        }

        // Different signs
        if self.sign != other.sign {
            return if self.sign > other.sign { 1 } else { -1 };
        }

        // Same sign: compare magnitudes
        let self_log = self.log2_estimate();
        let other_log = other.log2_estimate();

        if self_log != other_log {
            return if (self_log > other_log) == (self.sign > 0) {
                1
            } else {
                -1
            };
        }

        // Same order of magnitude: compare digit by digit from MSD
        let self_len = self.mantissa.len();
        let other_len = other.mantissa.len();
        let max_len = self_len.max(other_len);

        for i in (0..max_len).rev() {
            let self_digit = if i < self_len { self.mantissa[i] } else { 0 };
            let other_digit = if i < other_len {
                other.mantissa[i]
            } else {
                0
            };

            if self_digit != other_digit {
                return if (self_digit > other_digit) == (self.sign > 0) {
                    1
                } else {
                    -1
                };
            }
        }

        // Check trailing zeros due to different exp
        if self.exp != other.exp {
            return if (self.exp > other.exp) == (self.sign > 0) {
                -1
            } else {
                1
            };
        }

        0
    }
}

impl fmt::Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Number {{ sign: {}, exp: {}, cdigit: {}, mant: {:?} }}",
            self.sign,
            self.exp,
            self.cdigit(),
            &self.mantissa
        )
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.sign < 0 {
            write!(f, "-")?;
        }
        // Simple display in base BASEX
        for (i, &d) in self.mantissa.iter().enumerate().rev() {
            if i < self.mantissa.len() - 1 {
                write!(f, ",")?;
            }
            write!(f, "{d:08x}")?;
        }
        if self.exp != 0 {
            write!(f, "e{}", self.exp)?;
        }
        Ok(())
    }
}

impl Default for Number {
    fn default() -> Self {
        Self::zero()
    }
}

impl From<i32> for Number {
    fn from(value: i32) -> Self {
        Self::from_i32(value, BASEX)
    }
}

impl From<u32> for Number {
    fn from(value: u32) -> Self {
        Self::from_u32(value, BASEX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero() {
        let z = Number::zero();
        assert!(z.is_zero());
        assert_eq!(z.sign, 1);
        assert_eq!(z.exp, 0);
    }

    #[test]
    fn test_from_i32() {
        let n = Number::from_i32(42, 10);
        assert_eq!(n.sign, 1);
        assert_eq!(n.mantissa, vec![2, 4]);

        let n = Number::from_i32(-42, 10);
        assert_eq!(n.sign, -1);
        assert_eq!(n.mantissa, vec![2, 4]);

        let n = Number::from_i32(0, 10);
        assert!(n.is_zero());
    }

    #[test]
    fn test_from_u32() {
        let n = Number::from_u32(255, 16);
        assert_eq!(n.sign, 1);
        assert_eq!(n.mantissa, vec![15, 15]);
    }

    #[test]
    fn test_to_i32() {
        let n = Number::from_i32(42, 10);
        assert_eq!(n.to_i32(10), Some(42));

        let n = Number::from_i32(-100, 10);
        assert_eq!(n.to_i32(10), Some(-100));
    }

    #[test]
    fn test_compare() {
        let a = Number::from_i32(10, 10);
        let b = Number::from_i32(20, 10);
        assert_eq!(a.compare(&b), -1);
        assert_eq!(b.compare(&a), 1);
        assert_eq!(a.compare(&a), 0);

        let neg = Number::from_i32(-5, 10);
        assert_eq!(neg.compare(&a), -1);
    }
}
