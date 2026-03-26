// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.
// Rust port of Microsoft's Windows Calculator CalcManager engine.

//! # CalcManager
//!
//! A Rust port of Microsoft's Windows Calculator `CalcManager` engine, providing
//! arbitrary-precision rational arithmetic, scientific/standard/programmer calculator
//! modes, and unit conversion.
//!
//! ## Architecture
//!
//! - **`ratpack`** — Arbitrary-precision rational number arithmetic
//! - **`engine`** — Calculator engine (state machine, command processing)
//! - **`manager`** — High-level calculator manager (mode switching, history, memory)
//! - **`unit_converter`** — Unit conversion engine
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use calc_manager::prelude::*;
//!
//! // Create a Rational number
//! let a = Rational::from(42i32);
//! let b = Rational::from(7i32);
//! let result = (&a / &b).unwrap();
//! assert_eq!(result, Rational::from(6i32));
//! ```

pub mod commands;
pub mod display;
pub mod engine;
pub mod error;
pub mod manager;
pub mod ratpack;
pub mod types;
pub mod unit_converter;

/// Convenience re-exports for common types.
pub mod prelude {
    pub use crate::commands::Command;
    pub use crate::display::{CalcDisplay, HistoryDisplay};
    pub use crate::error::CalcError;
    pub use crate::ratpack::{Number, Rational};
    pub use crate::types::*;
}
