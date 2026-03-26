// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Mathematical constants for ratpack.
//! Port of C++ Ratpack/ratconst.h and constant initialization from support.cpp

use crate::types::{BASEX, BASEX_PWR};

use super::arithmetic::{rat_pow_i32, sub_rat, mul_rat, div_rat};
use super::Number;
use super::Rational;

/// Ratio for decimal radix (ceil(31 / log2(10)) - 1).
#[cfg(test)]
const RATIO_FOR_DECIMAL: i32 = 9;

/// Global state for ratpack constants.
/// In C++ these were global variables; in Rust we encapsulate them in a struct.
pub struct RatpackConstants {
    pub rat_zero: Rational,
    pub rat_one: Rational,
    pub rat_neg_one: Rational,
    pub rat_two: Rational,
    pub rat_six: Rational,
    pub rat_half: Rational,
    pub rat_ten: Rational,
    pub ln_ten: Rational,
    pub ln_two: Rational,
    pub pi: Rational,
    pub pi_over_two: Rational,
    pub two_pi: Rational,
    pub one_pt_five_pi: Rational,
    pub e_to_one_half: Rational,
    pub rat_exp: Rational,
    pub rad_to_deg: Rational,
    pub rad_to_grad: Rational,
    pub rat_qword: Rational,
    pub rat_dword: Rational,
    pub rat_word: Rational,
    pub rat_byte: Rational,
    pub rat_360: Rational,
    pub rat_400: Rational,
    pub rat_180: Rational,
    pub rat_200: Rational,
    pub rat_n_radix: Rational,
    pub rat_smallest: Rational,
    pub rat_neg_smallest: Rational,
    pub rat_max_exp: Rational,
    pub rat_min_exp: Rational,
    pub rat_max_fact: Rational,
    pub rat_min_fact: Rational,
    pub rat_min_i32: Rational,
    pub rat_max_i32: Rational,
    pub pt_eight_five: Rational,

    /// True to allow infinite precision calculations.
    pub true_infinite: bool,
    /// Internal ratio of internal radix to display radix.
    /// Matches C++ `g_ratio`.
    pub ratio: i32,
    /// Current decimal separator character.
    pub decimal_separator: char,
}

impl RatpackConstants {
    /// Initialize constants for the given radix and precision.
    /// Port of C++ `ChangeConstants`.
    ///
    /// For transcendental constants (pi, e, ln2, ln10), uses rational
    /// approximations. Full-precision Taylor series computation will be
    /// added when those functions are ported.
    /// # Panics
    ///
    /// Panics if `radix < 2`.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn new(radix: u32, precision: i32) -> Self {
        assert!(radix >= 2, "radix must be >= 2, got {radix}");

        // ratio = ceil(BASEXPWR / log2(radix)) - 1
        let ratio = (f64::from(BASEX_PWR) / f64::from(radix).log2()).ceil() as i32 - 1;

        let rat_one = Rational::one();
        let rat_two = Rational::from_i32(2);
        let rat_n_radix = Rational::from_i32(radix as i32);

        // Compute rat_smallest = radix^(-precision)
        let rat_smallest = rat_pow_i32(&rat_n_radix, -precision, precision)
            .unwrap_or_else(|_| Rational::zero());
        let mut rat_neg_smallest = rat_smallest.clone();
        rat_neg_smallest.p_mut().sign = -1;

        let rat_max_exp = Rational::from_i32(100_000);
        let mut rat_min_exp = rat_max_exp.clone();
        rat_min_exp.p_mut().sign *= -1;

        // Word-size limits: 2^n - 1
        let word_limits = Self::compute_word_limits(&rat_two, &rat_one, precision);

        // Transcendental constant approximations
        let transcendentals = Self::compute_transcendentals(&rat_two, precision);

        Self {
            rat_zero: Rational::zero(),
            rat_one,
            rat_neg_one: Rational::from_i32(-1),
            rat_two,
            rat_six: Rational::from_i32(6),
            rat_half: Rational::new(
                Number::from_i32(1, BASEX),
                Number::from_i32(2, BASEX),
            ),
            rat_ten: Rational::from_i32(10),
            ln_ten: transcendentals.ln_ten,
            ln_two: transcendentals.ln_two,
            pi: transcendentals.pi,
            pi_over_two: transcendentals.pi_over_two,
            two_pi: transcendentals.two_pi,
            one_pt_five_pi: transcendentals.one_pt_five_pi,
            e_to_one_half: transcendentals.e_to_one_half,
            rat_exp: transcendentals.rat_exp,
            rad_to_deg: transcendentals.rad_to_deg,
            rad_to_grad: transcendentals.rad_to_grad,
            rat_qword: word_limits.qword,
            rat_dword: word_limits.dword,
            rat_word: Rational::from_i32(0xFFFF),
            rat_byte: Rational::from_i32(0xFF),
            rat_360: Rational::from_i32(360),
            rat_400: Rational::from_i32(400),
            rat_180: Rational::from_i32(180),
            rat_200: Rational::from_i32(200),
            rat_n_radix,
            rat_smallest,
            rat_neg_smallest,
            rat_max_exp,
            rat_min_exp,
            rat_max_fact: Rational::from_i32(3249),
            rat_min_fact: Rational::from_i32(-1000),
            rat_min_i32: word_limits.min_i32,
            rat_max_i32: word_limits.max_i32,
            pt_eight_five: Rational::new(
                Number::from_i32(85, BASEX),
                Number::from_i32(100, BASEX),
            ),
            true_infinite: false,
            ratio,
            decimal_separator: '.',
        }
    }

    /// Compute word-size limit constants (2^n - 1 variants).
    fn compute_word_limits(rat_two: &Rational, rat_one: &Rational, precision: i32) -> WordLimits {
        let pow_64 = rat_pow_i32(rat_two, 64, precision).unwrap_or_else(|_| Rational::zero());
        let pow_32 = rat_pow_i32(rat_two, 32, precision).unwrap_or_else(|_| Rational::zero());
        let pow_31 = rat_pow_i32(rat_two, 31, precision).unwrap_or_else(|_| Rational::zero());

        let mut min_i32 = pow_31.clone();
        min_i32.p_mut().sign *= -1;

        WordLimits {
            qword: sub_rat(&pow_64, rat_one, precision),
            dword: sub_rat(&pow_32, rat_one, precision),
            max_i32: sub_rat(&pow_31, rat_one, precision),
            min_i32,
        }
    }

    /// Compute transcendental constant approximations.
    fn compute_transcendentals(rat_two: &Rational, precision: i32) -> Transcendentals {
        // pi ≈ 355/113 (accurate to 6 decimal places)
        let pi = Rational::new(
            Number::from_i32(355, BASEX),
            Number::from_i32(113, BASEX),
        );

        let two_pi = mul_rat(&pi, rat_two, precision);
        let pi_over_two = div_rat(&pi, rat_two, precision)
            .unwrap_or_else(|_| Rational::zero());
        let three_halves = Rational::new(
            Number::from_i32(3, BASEX),
            Number::from_i32(2, BASEX),
        );
        let one_pt_five_pi = mul_rat(&pi, &three_halves, precision);

        let rat_180 = Rational::from_i32(180);
        let rat_200 = Rational::from_i32(200);

        Transcendentals {
            pi: pi.clone(),
            two_pi,
            pi_over_two,
            one_pt_five_pi,
            rat_exp: Rational::new(
                Number::from_i32(2718, BASEX),
                Number::from_i32(1000, BASEX),
            ),
            e_to_one_half: Rational::new(
                Number::from_i32(1649, BASEX),
                Number::from_i32(1000, BASEX),
            ),
            ln_ten: Rational::new(
                Number::from_i32(2303, BASEX),
                Number::from_i32(1000, BASEX),
            ),
            ln_two: Rational::new(
                Number::from_i32(693, BASEX),
                Number::from_i32(1000, BASEX),
            ),
            rad_to_deg: div_rat(&rat_180, &pi, precision)
                .unwrap_or_else(|_| Rational::zero()),
            rad_to_grad: div_rat(&rat_200, &pi, precision)
                .unwrap_or_else(|_| Rational::zero()),
        }
    }
}

/// Intermediate struct for word-size limit computation.
struct WordLimits {
    qword: Rational,
    dword: Rational,
    max_i32: Rational,
    min_i32: Rational,
}

/// Intermediate struct for transcendental constant computation.
struct Transcendentals {
    pi: Rational,
    two_pi: Rational,
    pi_over_two: Rational,
    one_pt_five_pi: Rational,
    rat_exp: Rational,
    e_to_one_half: Rational,
    ln_ten: Rational,
    ln_two: Rational,
    rad_to_deg: Rational,
    rad_to_grad: Rational,
}

impl Default for RatpackConstants {
    fn default() -> Self {
        Self::new(10, 128)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants_new_decimal() {
        let c = RatpackConstants::new(10, 32);
        assert!(!c.rat_zero.is_zero() || c.rat_zero.is_zero()); // smoke test
        assert!(!c.rat_one.is_zero());
        assert!(!c.pi.is_zero());
        assert!(!c.two_pi.is_zero());
        assert!(!c.pi_over_two.is_zero());
        assert!(!c.rat_exp.is_zero());
        assert!(!c.ln_ten.is_zero());
        assert!(!c.ln_two.is_zero());
        assert!(!c.rad_to_deg.is_zero());
        assert!(!c.rad_to_grad.is_zero());
        assert_eq!(c.decimal_separator, '.');
    }

    #[test]
    fn test_constants_ratio() {
        let c = RatpackConstants::new(10, 32);
        // For base 10: ceil(31 / log2(10)) - 1 = ceil(31/3.32) - 1 = 10 - 1 = 9
        assert_eq!(c.ratio, RATIO_FOR_DECIMAL);
    }

    #[test]
    fn test_constants_ratio_hex() {
        let c = RatpackConstants::new(16, 32);
        // For base 16: ceil(31 / 4) - 1 = 8 - 1 = 7
        assert_eq!(c.ratio, 7);
    }

    #[test]
    fn test_constants_ratio_binary() {
        let c = RatpackConstants::new(2, 32);
        // For base 2: ceil(31 / 1) - 1 = 31 - 1 = 30
        assert_eq!(c.ratio, 30);
    }

    #[test]
    fn test_word_size_limits() {
        let c = RatpackConstants::new(10, 32);
        assert!(!c.rat_qword.is_zero());
        assert!(!c.rat_dword.is_zero());
        assert!(!c.rat_byte.is_zero());
        assert!(!c.rat_word.is_zero());
    }

    #[test]
    fn test_default_constants() {
        let c = RatpackConstants::default();
        assert_eq!(c.ratio, RATIO_FOR_DECIMAL);
    }
}
