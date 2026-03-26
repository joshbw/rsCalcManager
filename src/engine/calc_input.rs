// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Calculator input handling.
//!
//! Port of C++ `CalcInput` class.

/// Handles accumulation of digit input for the calculator.
pub struct CalcInput {
    /// Whether the input has a base prefix (0x, 0o, 0b).
    has_prefix: bool,
    /// Whether the input has a decimal point.
    has_decimal: bool,
    /// Whether the input is negative.
    is_negative: bool,
    /// The integer part digits.
    integer_part: Vec<char>,
    /// The decimal part digits.
    decimal_part: Vec<char>,
    /// Whether there's an exponent.
    has_exponent: bool,
    /// Whether the exponent is negative.
    exponent_negative: bool,
    /// The exponent digits.
    exponent_part: Vec<char>,
}

impl CalcInput {
    /// Create a new empty input.
    #[must_use]
    pub fn new() -> Self {
        Self {
            has_prefix: false,
            has_decimal: false,
            is_negative: false,
            integer_part: Vec::new(),
            decimal_part: Vec::new(),
            has_exponent: false,
            exponent_negative: false,
            exponent_part: Vec::new(),
        }
    }

    /// Check if the input is empty (no digits entered).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.integer_part.is_empty() && self.decimal_part.is_empty()
    }

    /// Clear all input.
    pub fn clear(&mut self) {
        self.has_prefix = false;
        self.has_decimal = false;
        self.is_negative = false;
        self.integer_part.clear();
        self.decimal_part.clear();
        self.has_exponent = false;
        self.exponent_negative = false;
        self.exponent_part.clear();
    }

    /// Toggle the sign of the input.
    pub fn toggle_sign(&mut self) {
        if self.has_exponent {
            self.exponent_negative = !self.exponent_negative;
        } else {
            self.is_negative = !self.is_negative;
        }
    }

    /// Add a digit to the input.
    pub fn add_digit(&mut self, digit: char) {
        if self.has_exponent {
            self.exponent_part.push(digit);
        } else if self.has_decimal {
            self.decimal_part.push(digit);
        } else {
            self.integer_part.push(digit);
        }
    }

    /// Add a decimal point.
    pub fn add_decimal_point(&mut self) {
        if !self.has_decimal && !self.has_exponent {
            self.has_decimal = true;
        }
    }

    /// Start entering an exponent.
    pub fn start_exponent(&mut self) {
        if !self.has_exponent {
            self.has_exponent = true;
        }
    }

    /// Remove the last character.
    pub fn backspace(&mut self) {
        if self.has_exponent {
            if !self.exponent_part.is_empty() {
                self.exponent_part.pop();
            } else if self.exponent_negative {
                self.exponent_negative = false;
            } else {
                self.has_exponent = false;
            }
        } else if self.has_decimal {
            if !self.decimal_part.is_empty() {
                self.decimal_part.pop();
            } else {
                self.has_decimal = false;
            }
        } else if !self.integer_part.is_empty() {
            self.integer_part.pop();
        }
    }

    /// Get the string representation with the given decimal separator.
    #[must_use]
    pub fn to_string(&self, decimal_separator: char) -> String {
        let mut result = String::new();

        if self.is_negative {
            result.push('-');
        }

        if self.integer_part.is_empty() {
            result.push('0');
        } else {
            result.extend(&self.integer_part);
        }

        if self.has_decimal {
            result.push(decimal_separator);
            result.extend(&self.decimal_part);
        }

        if self.has_exponent {
            result.push('e');
            if self.exponent_negative {
                result.push('-');
            }
            result.extend(&self.exponent_part);
        }

        result
    }
}

impl Default for CalcInput {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let input = CalcInput::new();
        assert!(input.is_empty());
        assert_eq!(input.to_string('.'), "0");
    }

    #[test]
    fn test_digit_input() {
        let mut input = CalcInput::new();
        input.add_digit('1');
        input.add_digit('2');
        input.add_digit('3');
        assert!(!input.is_empty());
        assert_eq!(input.to_string('.'), "123");
    }

    #[test]
    fn test_decimal_input() {
        let mut input = CalcInput::new();
        input.add_digit('3');
        input.add_decimal_point();
        input.add_digit('1');
        input.add_digit('4');
        assert_eq!(input.to_string('.'), "3.14");
    }

    #[test]
    fn test_negative_input() {
        let mut input = CalcInput::new();
        input.add_digit('5');
        input.toggle_sign();
        assert_eq!(input.to_string('.'), "-5");
    }

    #[test]
    fn test_backspace() {
        let mut input = CalcInput::new();
        input.add_digit('1');
        input.add_digit('2');
        input.backspace();
        assert_eq!(input.to_string('.'), "1");
    }

    #[test]
    fn test_exponent() {
        let mut input = CalcInput::new();
        input.add_digit('1');
        input.start_exponent();
        input.add_digit('3');
        assert_eq!(input.to_string('.'), "1e3");
    }
}
