// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Calculator history storage.
//!
//! Port of C++ `CalculatorHistory` class.

use crate::display::ExpressionToken;

/// A single history item.
#[derive(Debug, Clone)]
pub struct HistoryItem {
    /// The expression tokens.
    pub tokens: Vec<ExpressionToken>,
    /// The expression string.
    pub expression: String,
    /// The result string.
    pub result: String,
}

/// Stores calculator expression history.
pub struct CalculatorHistory {
    items: Vec<HistoryItem>,
    max_size: usize,
}

impl CalculatorHistory {
    /// Create a new history with the given maximum size.
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            items: Vec::new(),
            max_size,
        }
    }

    /// Add an item to history. Returns the index.
    pub fn add_item(&mut self, item: HistoryItem) -> usize {
        if self.items.len() >= self.max_size {
            self.items.remove(0);
        }
        self.items.push(item);
        self.items.len() - 1
    }

    /// Get all history items.
    #[must_use]
    pub fn items(&self) -> &[HistoryItem] {
        &self.items
    }

    /// Get a specific history item.
    #[must_use]
    pub fn get_item(&self, index: usize) -> Option<&HistoryItem> {
        self.items.get(index)
    }

    /// Remove a history item by index. Returns true if removed.
    pub fn remove_item(&mut self, index: usize) -> bool {
        if index < self.items.len() {
            self.items.remove(index);
            true
        } else {
            false
        }
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Get the maximum history size.
    #[must_use]
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}
