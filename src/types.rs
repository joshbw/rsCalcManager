// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Core type definitions for the calculator engine.

/// Number display format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NumberFormat {
    /// Floating point, or exponential if too big.
    #[default]
    Float,
    /// Always scientific notation.
    Scientific,
    /// Engineering notation (exponent is multiple of 3).
    Engineering,
}

/// Angle measurement type for trigonometric operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AngleType {
    /// 360 degrees per revolution.
    #[default]
    Degrees,
    /// 2π radians per revolution.
    Radians,
    /// 400 gradians per revolution.
    Gradians,
}

/// Radix (number base) for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RadixType {
    /// Base 16.
    Hex,
    /// Base 10.
    #[default]
    Decimal,
    /// Base 8.
    Octal,
    /// Base 2.
    Binary,
}

impl RadixType {
    /// Convert to numeric radix value.
    #[must_use]
    pub fn to_radix(self) -> u32 {
        match self {
            Self::Hex => 16,
            Self::Decimal => 10,
            Self::Octal => 8,
            Self::Binary => 2,
        }
    }
}

/// Integer word width for programmer mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NumWidth {
    /// 64-bit (default).
    #[default]
    QWord,
    /// 32-bit.
    DWord,
    /// 16-bit.
    Word,
    /// 8-bit.
    Byte,
}

impl NumWidth {
    /// Returns the bit width for this word size.
    #[must_use]
    pub fn bit_width(self) -> u32 {
        match self {
            Self::QWord => 64,
            Self::DWord => 32,
            Self::Word => 16,
            Self::Byte => 8,
        }
    }
}

/// Calculator operating mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CalculatorMode {
    /// Standard calculator.
    #[default]
    Standard,
    /// Scientific calculator.
    Scientific,
    /// Programmer calculator.
    Programmer,
}

/// Calculator precision by mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CalculatorPrecision {
    /// 16 digits for standard mode.
    Standard = 16,
    /// 32 digits for scientific mode.
    Scientific = 32,
    /// 64 digits for programmer mode.
    Programmer = 64,
}

/// Maximum depth for parenthesis/precedence stacks.
pub const MAX_PREC_DEPTH: usize = 25;

/// Internal radix used in ratpack calculations.
pub const BASEX: u32 = 0x8000_0000;

/// log2 of BASEX.
pub const BASEX_PWR: u32 = 31;

/// Type alias for mantissa digits (matching C++ MANTTYPE).
pub type MantType = u32;

/// Type alias for double-width mantissa operations.
pub type TwoMantType = u64;
