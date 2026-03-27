// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Arbitrary-precision rational number type.
//!
//! A rational number is represented as a pair p/q of `Number` values.

use std::fmt;

use crate::error::{CalcError, CalcResult};
use crate::types::BASEX;

use super::Number;

/// Default base/radix for rational calculations.
#[allow(dead_code)]
pub const RATIONAL_BASE: u32 = 10;

/// Default precision for rational calculations.
pub const RATIONAL_PRECISION: i32 = 128;

/// An arbitrary-precision rational number p/q.
#[derive(Clone, PartialEq, Eq)]
pub struct Rational {
    /// Numerator.
    p: Number,
    /// Denominator.
    q: Number,
}

impl Rational {
    /// Create a rational from numerator and denominator.
    #[must_use]
    pub fn new(p: Number, q: Number) -> Self {
        Self { p, q }
    }

    /// Create zero (0/1).
    #[must_use]
    pub fn zero() -> Self {
        Self {
            p: Number::zero(),
            q: Number::from_i32(1, BASEX),
        }
    }

    /// Create one (1/1).
    #[must_use]
    pub fn one() -> Self {
        Self {
            p: Number::from_i32(1, BASEX),
            q: Number::from_i32(1, BASEX),
        }
    }

    /// Get the numerator.
    #[must_use]
    pub fn p(&self) -> &Number {
        &self.p
    }

    /// Get the denominator.
    #[must_use]
    pub fn q(&self) -> &Number {
        &self.q
    }

    /// Get a mutable reference to the numerator.
    pub fn p_mut(&mut self) -> &mut Number {
        &mut self.p
    }

    /// Get a mutable reference to the denominator.
    pub fn q_mut(&mut self) -> &mut Number {
        &mut self.q
    }

    /// Check if this rational is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.p.is_zero()
    }

    /// Get the sign of the rational: +1 or -1.
    /// Matches C++ SIGN(prat) macro: pp->sign * pq->sign
    #[must_use]
    pub fn sign(&self) -> i32 {
        self.p.sign * self.q.sign
    }

    /// Make absolute value (both p and q signs become +1).
    pub fn abs_mut(&mut self) {
        self.p.sign = 1;
        self.q.sign = 1;
    }

    /// Return the absolute value.
    #[must_use]
    pub fn abs(&self) -> Self {
        let mut result = self.clone();
        result.abs_mut();
        result
    }

    /// Negate this rational.
    pub fn negate_mut(&mut self) {
        self.p.sign = -self.p.sign;
    }

    /// Return the negation.
    #[must_use]
    pub fn negate(&self) -> Self {
        let mut result = self.clone();
        result.negate_mut();
        result
    }

    /// Duplicate (explicit clone matching C++ DUPRAT semantics).
    #[must_use]
    pub fn dup(&self) -> Self {
        self.clone()
    }

    /// Renormalize: ensure exponents are non-negative.
    /// Matches C++ RENORMALIZE macro.
    pub fn renormalize(&mut self) {
        if self.p.exp < 0 {
            self.q.exp -= self.p.exp;
            self.p.exp = 0;
        }
        if self.q.exp < 0 {
            self.p.exp -= self.q.exp;
            self.q.exp = 0;
        }
    }

    /// Estimate log2 of the rational.
    /// Matches C++ LOGRAT2 macro: LOGNUM2(pp) - LOGNUM2(pq)
    #[must_use]
    pub fn log2_estimate(&self) -> i32 {
        self.p.log2_estimate() - self.q.log2_estimate()
    }

    /// Create from an i32 value.
    #[must_use]
    pub fn from_i32(value: i32) -> Self {
        Self {
            p: Number::from_i32(value, BASEX),
            q: Number::from_i32(1, BASEX),
        }
    }

    /// Create from a u32 value.
    #[must_use]
    pub fn from_u32(value: u32) -> Self {
        Self {
            p: Number::from_u32(value, BASEX),
            q: Number::from_i32(1, BASEX),
        }
    }

    /// Create from a u64 value.
    #[must_use]
    pub fn from_u64(value: u64) -> Self {
        if value == 0 {
            return Self::zero();
        }

        let mut val = value;
        let mut digits = Vec::new();
        let radix = u64::from(BASEX);
        while val > 0 {
            digits.push((val % radix) as u32);
            val /= radix;
        }

        Self {
            p: Number::new(1, 0, digits),
            q: Number::from_i32(1, BASEX),
        }
    }

    /// Try to convert to u64.
    pub fn to_u64(&self, _radix: u32, _precision: i32) -> CalcResult<u64> {
        // TODO: implement full conversion via ratpack conv
        // For now, simple case where q == 1
        if self.q.mantissa == vec![1] && self.q.exp == 0 {
            let mut result: u64 = 0;
            let mut place: u64 = 1;
            let basex = u64::from(BASEX);

            for _ in 0..self.p.exp {
                place = place.checked_mul(basex).ok_or(CalcError::Overflow)?;
            }

            for &digit in &self.p.mantissa {
                result = result
                    .checked_add(
                        u64::from(digit)
                            .checked_mul(place)
                            .ok_or(CalcError::Overflow)?,
                    )
                    .ok_or(CalcError::Overflow)?;
                place = place.checked_mul(basex).ok_or(CalcError::Overflow)?;
            }

            Ok(result)
        } else {
            // Need full rational-to-integer conversion
            Err(CalcError::InvalidRange)
        }
    }
}

impl fmt::Debug for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Rational {{ p: {:?}, q: {:?} }}", self.p, self.q)
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} / {})", self.p, self.q)
    }
}

impl Default for Rational {
    fn default() -> Self {
        Self::zero()
    }
}

impl From<i32> for Rational {
    fn from(value: i32) -> Self {
        Self::from_i32(value)
    }
}

impl From<u32> for Rational {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<u64> for Rational {
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

impl From<Number> for Rational {
    fn from(n: Number) -> Self {
        Self {
            p: n,
            q: Number::from_i32(1, BASEX),
        }
    }
}

impl std::ops::Neg for Rational {
    type Output = Self;
    fn neg(self) -> Self {
        self.negate()
    }
}

impl std::ops::Neg for &Rational {
    type Output = Rational;
    fn neg(self) -> Rational {
        self.negate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero() {
        let z = Rational::zero();
        assert!(z.is_zero());
        assert_eq!(z.sign(), 1);
    }

    #[test]
    fn test_one() {
        let o = Rational::one();
        assert!(!o.is_zero());
        assert_eq!(o.sign(), 1);
    }

    #[test]
    fn test_from_i32() {
        let r = Rational::from_i32(42);
        assert!(!r.is_zero());
        assert_eq!(r.sign(), 1);

        let r = Rational::from_i32(-7);
        assert_eq!(r.sign(), -1);
    }

    #[test]
    fn test_negate() {
        let r = Rational::from_i32(5);
        let neg = r.negate();
        assert_eq!(neg.sign(), -1);

        let dbl_neg = neg.negate();
        assert_eq!(dbl_neg.sign(), 1);
    }

    #[test]
    fn test_abs() {
        let r = Rational::from_i32(-5);
        let a = r.abs();
        assert_eq!(a.sign(), 1);
    }
}
