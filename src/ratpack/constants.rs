// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Mathematical constants for ratpack.
//! Port of C++ Ratpack/ratconst.h and constant initialization from support.cpp

use super::Rational;

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

    /// True to allow infinite precision calculations.
    pub true_infinite: bool,
    /// Internal ratio of internal radix.
    pub ratio: i32,
    /// Current decimal separator character.
    pub decimal_separator: char,
}

impl RatpackConstants {
    /// Initialize constants for the given radix and precision.
    /// Port of C++ `ChangeConstants`.
    #[must_use]
    pub fn new(radix: u32, precision: i32) -> Self {
        // TODO: Port full constant initialization from support.cpp
        Self {
            rat_zero: Rational::zero(),
            rat_one: Rational::one(),
            rat_neg_one: Rational::from_i32(-1),
            rat_two: Rational::from_i32(2),
            rat_six: Rational::from_i32(6),
            rat_half: Rational::new(
                super::Number::from_i32(1, crate::types::BASEX),
                super::Number::from_i32(2, crate::types::BASEX),
            ),
            rat_ten: Rational::from_i32(10),
            ln_ten: Rational::zero(), // TODO: compute
            ln_two: Rational::zero(), // TODO: compute
            pi: Rational::zero(), // TODO: compute
            pi_over_two: Rational::zero(),
            two_pi: Rational::zero(),
            one_pt_five_pi: Rational::zero(),
            e_to_one_half: Rational::zero(),
            rat_exp: Rational::zero(),
            rad_to_deg: Rational::zero(),
            rad_to_grad: Rational::zero(),
            rat_qword: Rational::zero(), // TODO: compute as 2^64 - 1
            rat_dword: Rational::zero(),
            rat_word: Rational::zero(),
            rat_byte: Rational::zero(),
            rat_360: Rational::from_i32(360),
            rat_400: Rational::from_i32(400),
            rat_180: Rational::from_i32(180),
            rat_200: Rational::from_i32(200),
            true_infinite: false,
            ratio: 1,
            decimal_separator: '.',
        }
    }
}

impl Default for RatpackConstants {
    fn default() -> Self {
        Self::new(10, 128)
    }
}
