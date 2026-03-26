// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Error types for the calculator engine.

use std::fmt;

/// Calculator error codes, matching the C++ CalcErr.h definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CalcError {
    /// Division by zero (CALC_E_DIVIDEBYZERO)
    DivideByZero,
    /// Input not within function domain (CALC_E_DOMAIN)
    Domain,
    /// Result is undefined (CALC_E_INDEFINITE)
    Indefinite,
    /// Result is positive infinity (CALC_E_POSINFINITY)
    PositiveInfinity,
    /// Result is negative infinity (CALC_E_NEGINFINITY)
    NegativeInfinity,
    /// Input beyond computable range (CALC_E_INVALIDRANGE)
    InvalidRange,
    /// Out of memory (CALC_E_OUTOFMEMORY)
    OutOfMemory,
    /// Arithmetic overflow (CALC_E_OVERFLOW)
    Overflow,
    /// No result (CALC_E_NORESULT)
    NoResult,
    /// Insufficient data for operation
    InsufficientData,
}

impl CalcError {
    /// Convert from the C++ error code value.
    #[must_use]
    pub fn from_error_code(code: u32) -> Option<Self> {
        match code {
            0x8000_0000 => Some(Self::DivideByZero),
            0x8000_0001 => Some(Self::Domain),
            0x8000_0002 => Some(Self::Indefinite),
            0x8000_0003 => Some(Self::PositiveInfinity),
            0x8000_0004 => Some(Self::NegativeInfinity),
            0x8000_0006 => Some(Self::InvalidRange),
            0x8000_0007 => Some(Self::OutOfMemory),
            0x8000_0008 => Some(Self::Overflow),
            0x8000_0009 => Some(Self::NoResult),
            _ => None,
        }
    }

    /// Convert to the C++ error code value.
    #[must_use]
    pub fn to_error_code(self) -> u32 {
        match self {
            Self::DivideByZero => 0x8000_0000,
            Self::Domain => 0x8000_0001,
            Self::Indefinite => 0x8000_0002,
            Self::PositiveInfinity => 0x8000_0003,
            Self::NegativeInfinity => 0x8000_0004,
            Self::InvalidRange => 0x8000_0006,
            Self::OutOfMemory => 0x8000_0007,
            Self::Overflow => 0x8000_0008,
            Self::NoResult => 0x8000_0009,
            Self::InsufficientData => 0x8000_000A,
        }
    }
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DivideByZero => write!(f, "Cannot divide by zero"),
            Self::Domain => write!(f, "Invalid input"),
            Self::Indefinite => write!(f, "Result is undefined"),
            Self::PositiveInfinity => write!(f, "Positive infinity"),
            Self::NegativeInfinity => write!(f, "Negative infinity"),
            Self::InvalidRange => write!(f, "Invalid range"),
            Self::OutOfMemory => write!(f, "Out of memory"),
            Self::Overflow => write!(f, "Overflow"),
            Self::NoResult => write!(f, "No result"),
            Self::InsufficientData => write!(f, "Insufficient data"),
        }
    }
}

impl std::error::Error for CalcError {}

/// Result type alias for calculator operations.
pub type CalcResult<T> = Result<T, CalcError>;
