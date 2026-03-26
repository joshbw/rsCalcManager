// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Factorial function.
//! Port of C++ Ratpack/fact.cpp

use crate::error::CalcResult;
use super::Rational;

/// Compute n! (factorial).
/// Port of C++ `factrat`.
pub fn fact_rat(x: &Rational, radix: u32, precision: i32) -> CalcResult<Rational> {
    // TODO: Port implementation from fact.cpp
    Ok(Rational::zero())
}
