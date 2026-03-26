// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Base conversion operations.
//! Port of C++ Ratpack/basex.cpp

use crate::error::CalcResult;
use super::Number;

/// Convert a number from one radix to another.
/// Port of C++ `nRadixxtonum`.
pub fn num_to_radix(num: &Number, radix: u32, _precision: i32) -> CalcResult<Number> {
    // TODO: Port full implementation from basex.cpp
    Ok(num.clone())
}

/// Convert a number from external radix to internal BASEX.
/// Port of C++ `numtonRadixx`.
pub fn num_from_radix(num: &Number, radix: u32) -> Number {
    // TODO: Port full implementation from basex.cpp
    num.clone()
}
