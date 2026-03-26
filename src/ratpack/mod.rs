// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Arbitrary-precision rational number arithmetic.
//!
//! This module is a Rust port of the C++ `Ratpack` library, which provides
//! infinite-precision arithmetic using rational numbers (p/q pairs of
//! arbitrary-size integers).

mod number;
mod rational;
pub mod arithmetic;
pub mod basex;
pub mod conv;
pub mod constants;
pub mod exp;
pub mod fact;
pub mod logic;
pub mod num;
pub mod support;
pub mod trans;
pub mod itrans;

pub use number::Number;
pub use rational::Rational;
