// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Number manipulation support functions.
//! Port of C++ Ratpack/num.cpp

use super::Number;
use crate::types::BASEX;

/// Compute GCD of two numbers.
/// Port of C++ `gcd`.
pub fn gcd(a: &Number, b: &Number) -> Number {
    // TODO: Port implementation from num.cpp
    Number::from_i32(1, BASEX)
}

/// Compute factorial of a number as a product: start * (start+1) * ... * stop.
/// Port of C++ `i32prodnum`.
pub fn i32_prod_num(start: i32, stop: i32, radix: u32) -> Number {
    // TODO: Port implementation from num.cpp
    Number::from_i32(1, radix)
}

/// Compute factorial as a Number.
/// Port of C++ `i32factnum`.
pub fn i32_fact_num(n: i32, radix: u32) -> Number {
    // TODO: Port implementation from num.cpp
    Number::from_i32(1, radix)
}
