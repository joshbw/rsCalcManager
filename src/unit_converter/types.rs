// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Unit converter types.
//!
//! Port of C++ UnitConverter.h type definitions.

/// A unit for conversion.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Unit {
    /// Unique identifier.
    pub id: i32,
    /// Display name.
    pub name: String,
    /// Accessible name.
    pub accessible_name: String,
    /// Abbreviation.
    pub abbreviation: String,
    /// Whether this unit can be a conversion source.
    pub is_conversion_source: bool,
    /// Whether this unit can be a conversion target.
    pub is_conversion_target: bool,
    /// Whether this is a whimsical unit.
    pub is_whimsical: bool,
}

impl Unit {
    /// Create a new unit.
    #[must_use]
    pub fn new(
        id: i32,
        name: impl Into<String>,
        abbreviation: impl Into<String>,
        is_conversion_source: bool,
        is_conversion_target: bool,
        is_whimsical: bool,
    ) -> Self {
        let name = name.into();
        Self {
            id,
            accessible_name: name.clone(),
            name,
            abbreviation: abbreviation.into(),
            is_conversion_source,
            is_conversion_target,
            is_whimsical,
        }
    }

    /// Create an empty/null unit.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            id: -1,
            name: String::new(),
            accessible_name: String::new(),
            abbreviation: String::new(),
            is_conversion_source: true,
            is_conversion_target: true,
            is_whimsical: false,
        }
    }
}

impl Default for Unit {
    fn default() -> Self {
        Self::empty()
    }
}

/// A category of units.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Category {
    /// Unique identifier.
    pub id: i32,
    /// Display name.
    pub name: String,
    /// Whether this category supports negative values.
    pub supports_negative: bool,
}

impl Default for Category {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::new(),
            supports_negative: false,
        }
    }
}

/// Conversion data between two units.
#[derive(Debug, Clone, Copy)]
pub struct ConversionData {
    /// Conversion ratio.
    pub ratio: f64,
    /// Conversion offset.
    pub offset: f64,
    /// Whether to apply offset before ratio.
    pub offset_first: bool,
}

impl Default for ConversionData {
    fn default() -> Self {
        Self {
            ratio: 1.0,
            offset: 0.0,
            offset_first: false,
        }
    }
}

/// Trait for loading conversion data.
pub trait ConverterDataLoader {
    /// Prepare data if necessary.
    fn load_data(&mut self);
    /// Get ordered categories.
    fn get_ordered_categories(&self) -> Vec<Category>;
    /// Get ordered units for a category.
    fn get_ordered_units(&self, category: &Category) -> Vec<Unit>;
    /// Load conversion ratios for a unit.
    fn load_ordered_ratios(&self, unit: &Unit) -> std::collections::HashMap<i32, ConversionData>;
    /// Check if a category is supported.
    fn supports_category(&self, category: &Category) -> bool;
}

/// Trait for unit converter view model callbacks.
pub trait UnitConverterCallback {
    /// Called with conversion results.
    fn display_callback(&mut self, from: &str, to: &str);
    /// Called with suggested values.
    fn suggested_value_callback(&mut self, suggested_values: &[(String, Unit)]);
    /// Called when max digits are reached.
    fn max_digits_reached(&mut self);
}
