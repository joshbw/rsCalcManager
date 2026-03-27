// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Calculator manager — high-level coordinator.
//!
//! Port of C++ `CalculatorManager` class.

use crate::commands::Command;
use crate::display::CalcDisplay;
use crate::engine::calc_engine::CalcEngine;
use crate::ratpack::Rational;
use crate::types::*;

use super::calculator_history::{CalculatorHistory, HistoryItem};

#[allow(dead_code)]
const MAXIMUM_MEMORY_SIZE: usize = 100;

/// High-level calculator manager.
///
/// Manages engine lifecycle, mode switching, memory, and history.
/// Port of C++ `CalculationManager::CalculatorManager`.
#[allow(dead_code)]
pub struct CalculatorManager {
    current_mode: CalculatorMode,
    standard_engine: CalcEngine,
    scientific_engine: CalcEngine,
    programmer_engine: CalcEngine,

    memorized_numbers: Vec<Rational>,
    persisted_primary_value: Rational,
    is_exponential_format: bool,
    current_degree_mode: Command,
    in_history_item_load_mode: bool,

    std_history: CalculatorHistory,
    sci_history: CalculatorHistory,
}

impl CalculatorManager {
    /// Create a new calculator manager.
    pub fn new() -> Self {
        Self {
            current_mode: CalculatorMode::Standard,
            standard_engine: CalcEngine::new(false, false),
            scientific_engine: CalcEngine::new(true, false),
            programmer_engine: CalcEngine::new(true, true),
            memorized_numbers: Vec::new(),
            persisted_primary_value: Rational::zero(),
            is_exponential_format: false,
            current_degree_mode: Command::CommandDEG,
            in_history_item_load_mode: false,
            std_history: CalculatorHistory::new(20),
            sci_history: CalculatorHistory::new(20),
        }
    }

    /// Get a reference to the current engine based on mode.
    fn current_engine(&self) -> &CalcEngine {
        match self.current_mode {
            CalculatorMode::Standard => &self.standard_engine,
            CalculatorMode::Scientific => &self.scientific_engine,
            CalculatorMode::Programmer => &self.programmer_engine,
        }
    }

    /// Get a mutable reference to the current engine based on mode.
    fn current_engine_mut(&mut self) -> &mut CalcEngine {
        match self.current_mode {
            CalculatorMode::Standard => &mut self.standard_engine,
            CalculatorMode::Scientific => &mut self.scientific_engine,
            CalculatorMode::Programmer => &mut self.programmer_engine,
        }
    }

    /// Reset the calculator state.
    pub fn reset(&mut self, clear_memory: bool) {
        if clear_memory {
            self.memorized_numbers.clear();
        }
        // TODO: reset current engine
    }

    /// Switch to standard mode.
    pub fn set_standard_mode(&mut self) {
        self.current_mode = CalculatorMode::Standard;
    }

    /// Switch to scientific mode.
    pub fn set_scientific_mode(&mut self) {
        self.current_mode = CalculatorMode::Scientific;
    }

    /// Switch to programmer mode.
    pub fn set_programmer_mode(&mut self) {
        self.current_mode = CalculatorMode::Programmer;
    }

    /// Send a command to the current engine.
    pub fn send_command(&mut self, command: Command, display: &mut dyn CalcDisplay) {
        let op = command as i32;
        self.current_engine_mut().process_command(op, display);
    }

    /// Check if the engine is recording input.
    #[must_use]
    pub fn is_engine_recording(&self) -> bool {
        self.current_engine().is_recording()
    }

    /// Check if input is empty.
    #[must_use]
    pub fn is_input_empty(&self) -> bool {
        self.current_engine().is_input_empty()
    }

    /// Get the current calculator mode.
    #[must_use]
    pub fn current_mode(&self) -> CalculatorMode {
        self.current_mode
    }

    /// Get the current degree mode.
    #[must_use]
    pub fn current_degree_mode(&self) -> Command {
        self.current_degree_mode
    }

    /// Memorize the current number.
    pub fn memorize_number(&mut self) {
        // TODO: implement
    }

    /// Load a memorized number.
    pub fn memorized_number_load(&mut self, _index: usize) {
        // TODO: implement
    }

    /// Add to a memorized number.
    pub fn memorized_number_add(&mut self, _index: usize) {
        // TODO: implement
    }

    /// Subtract from a memorized number.
    pub fn memorized_number_subtract(&mut self, _index: usize) {
        // TODO: implement
    }

    /// Clear a specific memorized number.
    pub fn memorized_number_clear(&mut self, index: usize) {
        if index < self.memorized_numbers.len() {
            self.memorized_numbers.remove(index);
        }
    }

    /// Clear all memorized numbers.
    pub fn memorized_number_clear_all(&mut self) {
        self.memorized_numbers.clear();
    }

    /// Get history items for the current mode.
    #[must_use]
    pub fn history_items(&self) -> &[HistoryItem] {
        match self.current_mode {
            CalculatorMode::Standard => self.std_history.items(),
            CalculatorMode::Scientific => self.sci_history.items(),
            CalculatorMode::Programmer => &[],
        }
    }

    /// Clear history for the current mode.
    pub fn clear_history(&mut self) {
        match self.current_mode {
            CalculatorMode::Standard => self.std_history.clear(),
            CalculatorMode::Scientific => self.sci_history.clear(),
            CalculatorMode::Programmer => {}
        }
    }

    /// Get the decimal separator.
    #[must_use]
    pub fn decimal_separator(&self) -> char {
        self.current_engine().decimal_separator()
    }
}

impl Default for CalculatorManager {
    fn default() -> Self {
        Self::new()
    }
}
