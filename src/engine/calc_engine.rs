// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Calculator engine — the core state machine.
//!
//! Port of C++ CCalcEngine class from CalcEngine.h and related .cpp files.

use crate::commands::OpCode;
use crate::display::CalcDisplay;
use crate::ratpack::Rational;
use crate::types::*;

use super::calc_input::CalcInput;
use super::history::HistoryCollector;

/// The calculator engine state machine.
///
/// Processes commands and produces display output via the `CalcDisplay` trait.
/// Port of C++ `CCalcEngine` class.
#[allow(dead_code)]
pub struct CalcEngine {
    // Mode configuration
    precedence: bool,
    integer_mode: bool,

    // Current state
    op_code: OpCode,
    prev_op_code: OpCode,
    change_op: bool,
    record: bool,
    set_calc_state: bool,
    input: CalcInput,
    number_format: NumberFormat,
    error: bool,
    inv: bool,
    no_prev_equ: bool,

    // Values
    current_val: Rational,
    last_val: Rational,
    hold_val: Rational,
    memory_value: Option<Rational>,
    max_trig_num: Rational,

    // Stacks for parentheses and precedence
    paren_vals: Vec<Rational>,
    precedence_vals: Vec<Rational>,
    paren_ops: Vec<i32>,
    prec_ops: Vec<i32>,
    open_paren_count: usize,
    precedence_op_count: usize,

    // Display settings
    radix: u32,
    precision: i32,
    int_digits_sav: i32,
    dec_grouping: Vec<u32>,
    number_string: String,
    decimal_separator: char,
    group_separator: char,

    // State
    temp_com: i32,
    last_com: i32,
    angle_type: AngleType,
    num_width: NumWidth,
    word_bit_width: i32,
    carry_bit: u64,

    // History
    history_collector: HistoryCollector,
}

impl CalcEngine {
    /// Create a new calculator engine.
    pub fn new(
        precedence: bool,
        integer_mode: bool,
    ) -> Self {
        Self {
            precedence,
            integer_mode,
            op_code: 0,
            prev_op_code: 0,
            change_op: false,
            record: false,
            set_calc_state: false,
            input: CalcInput::new(),
            number_format: NumberFormat::Float,
            error: false,
            inv: false,
            no_prev_equ: true,
            current_val: Rational::zero(),
            last_val: Rational::zero(),
            hold_val: Rational::zero(),
            memory_value: None,
            max_trig_num: Rational::zero(),
            paren_vals: Vec::with_capacity(MAX_PREC_DEPTH),
            precedence_vals: Vec::with_capacity(MAX_PREC_DEPTH),
            paren_ops: Vec::with_capacity(MAX_PREC_DEPTH),
            prec_ops: Vec::with_capacity(MAX_PREC_DEPTH),
            open_paren_count: 0,
            precedence_op_count: 0,
            radix: 10,
            precision: 16,
            int_digits_sav: 0,
            dec_grouping: vec![3],
            number_string: String::new(),
            decimal_separator: '.',
            group_separator: ',',
            temp_com: 0,
            last_com: 0,
            angle_type: AngleType::Degrees,
            num_width: NumWidth::QWord,
            word_bit_width: 64,
            carry_bit: 0,
            history_collector: HistoryCollector::new(),
        }
    }

    /// Process a command (button press).
    /// Port of C++ `CCalcEngine::ProcessCommand`.
    pub fn process_command(&mut self, _op: OpCode, _display: &mut dyn CalcDisplay) {
        // TODO: Port full command processing from scicomm.cpp
    }

    /// Check if the engine is in an error state.
    #[must_use]
    pub fn is_in_error(&self) -> bool {
        self.error
    }

    /// Check if input is empty.
    #[must_use]
    pub fn is_input_empty(&self) -> bool {
        self.input.is_empty() && (self.number_string.is_empty() || self.number_string == "0")
    }

    /// Check if in recording state.
    #[must_use]
    pub fn is_recording(&self) -> bool {
        self.record
    }

    /// Get the current radix.
    #[must_use]
    pub fn current_radix(&self) -> u32 {
        self.radix
    }

    /// Change the precision.
    pub fn change_precision(&mut self, precision: i32) {
        self.precision = precision;
    }

    /// Get the persisted memory object.
    #[must_use]
    pub fn memory_value(&self) -> Option<&Rational> {
        self.memory_value.as_ref()
    }

    /// Set the persisted memory object.
    pub fn set_memory_value(&mut self, value: Rational) {
        self.memory_value = Some(value);
    }

    /// Get decimal separator.
    #[must_use]
    pub fn decimal_separator(&self) -> char {
        self.decimal_separator
    }
}
