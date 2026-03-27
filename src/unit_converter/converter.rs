// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Unit converter implementation.
//!
//! Port of C++ `UnitConversionManager::UnitConverter` class.

use std::collections::HashMap;

use super::types::*;
use crate::commands::UnitConversionCommand;

/// Unit converter engine.
///
/// Manages unit conversion between different categories and unit types.
#[allow(dead_code)]
pub struct UnitConverter {
    data_loader: Box<dyn ConverterDataLoader>,
    callback: Option<Box<dyn UnitConverterCallback>>,
    categories: Vec<Category>,
    category_to_units: HashMap<i32, Vec<Unit>>,
    ratio_map: HashMap<i32, HashMap<i32, ConversionData>>,
    current_category: Category,
    from_type: Unit,
    to_type: Unit,
    current_display: String,
    return_display: String,
    current_has_decimal: bool,
    return_has_decimal: bool,
    switched_active: bool,
}

impl UnitConverter {
    /// Create a new unit converter with the given data loader.
    pub fn new(data_loader: Box<dyn ConverterDataLoader>) -> Self {
        Self {
            data_loader,
            callback: None,
            categories: Vec::new(),
            category_to_units: HashMap::new(),
            ratio_map: HashMap::new(),
            current_category: Category::default(),
            from_type: Unit::empty(),
            to_type: Unit::empty(),
            current_display: String::from("0"),
            return_display: String::from("0"),
            current_has_decimal: false,
            return_has_decimal: false,
            switched_active: false,
        }
    }

    /// Initialize the converter.
    pub fn initialize(&mut self) {
        self.data_loader.load_data();
        self.categories = self.data_loader.get_ordered_categories();
    }

    /// Get available categories.
    #[must_use]
    pub fn get_categories(&self) -> &[Category] {
        &self.categories
    }

    /// Set the current category.
    pub fn set_current_category(&mut self, category: &Category) -> (Vec<Unit>, Unit, Unit) {
        self.current_category = category.clone();
        let units = self.data_loader.get_ordered_units(category);
        let from = units.first().cloned().unwrap_or_default();
        let to = units.get(1).cloned().unwrap_or_default();
        self.from_type = from.clone();
        self.to_type = to.clone();
        self.category_to_units
            .insert(category.id, units.clone());
        (units, from, to)
    }

    /// Get the current category.
    #[must_use]
    pub fn current_category(&self) -> &Category {
        &self.current_category
    }

    /// Set the from and to unit types.
    pub fn set_current_unit_types(&mut self, from: &Unit, to: &Unit) {
        self.from_type = from.clone();
        self.to_type = to.clone();
        // Load ratios for the from type
        let ratios = self.data_loader.load_ordered_ratios(from);
        self.ratio_map.insert(from.id, ratios);
    }

    /// Switch which value is active (from ↔ to).
    pub fn switch_active(&mut self, new_value: &str) {
        self.switched_active = !self.switched_active;
        self.current_display = new_value.to_string();
    }

    /// Check if the active input is switched.
    #[must_use]
    pub fn is_switched_active(&self) -> bool {
        self.switched_active
    }

    /// Send a command (digit, decimal, etc.).
    pub fn send_command(&mut self, command: UnitConversionCommand) {
        match command {
            UnitConversionCommand::Clear | UnitConversionCommand::Reset => {
                self.current_display = String::from("0");
                self.current_has_decimal = false;
            }
            UnitConversionCommand::Backspace => {
                if self.current_display.len() > 1 {
                    let removed = self.current_display.pop();
                    if removed == Some('.') {
                        self.current_has_decimal = false;
                    }
                } else {
                    self.current_display = String::from("0");
                }
            }
            UnitConversionCommand::Decimal => {
                if !self.current_has_decimal {
                    self.current_display.push('.');
                    self.current_has_decimal = true;
                }
            }
            UnitConversionCommand::Negate => {
                if self.current_display.starts_with('-') {
                    self.current_display.remove(0);
                } else if self.current_display != "0" {
                    self.current_display.insert(0, '-');
                }
            }
            UnitConversionCommand::None => {}
            digit => {
                let ch = match digit {
                    UnitConversionCommand::Zero => '0',
                    UnitConversionCommand::One => '1',
                    UnitConversionCommand::Two => '2',
                    UnitConversionCommand::Three => '3',
                    UnitConversionCommand::Four => '4',
                    UnitConversionCommand::Five => '5',
                    UnitConversionCommand::Six => '6',
                    UnitConversionCommand::Seven => '7',
                    UnitConversionCommand::Eight => '8',
                    UnitConversionCommand::Nine => '9',
                    _ => return,
                };
                if self.current_display == "0" {
                    self.current_display = String::from(ch);
                } else {
                    self.current_display.push(ch);
                }
            }
        }

        self.calculate();
    }

    /// Perform the conversion calculation.
    pub fn calculate(&mut self) {
        let value: f64 = self.current_display.parse().unwrap_or(0.0);

        if let Some(ratios) = self.ratio_map.get(&self.from_type.id) {
            if let Some(conv) = ratios.get(&self.to_type.id) {
                let result = Self::convert(value, conv);
                self.return_display = format!("{result}");
            }
        }

        if let Some(callback) = &mut self.callback {
            callback.display_callback(&self.current_display, &self.return_display);
        }
    }

    /// Set the view model callback.
    pub fn set_callback(&mut self, callback: Box<dyn UnitConverterCallback>) {
        self.callback = Some(callback);
    }

    /// Perform a conversion calculation.
    fn convert(value: f64, data: &ConversionData) -> f64 {
        if data.offset_first {
            (value + data.offset) * data.ratio
        } else {
            value * data.ratio + data.offset
        }
    }
}
