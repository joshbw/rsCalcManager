// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! History collection for the calculator engine.
//!
//! Port of C++ `CHistoryCollector` class.

use crate::display::ExpressionToken;

/// Collects expression history as commands are processed.
pub struct HistoryCollector {
    tokens: Vec<ExpressionToken>,
}

impl HistoryCollector {
    /// Create a new history collector.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
        }
    }

    /// Clear the history.
    pub fn clear(&mut self) {
        self.tokens.clear();
    }

    /// Get the current tokens.
    #[must_use]
    pub fn tokens(&self) -> &[ExpressionToken] {
        &self.tokens
    }
}

impl Default for HistoryCollector {
    fn default() -> Self {
        Self::new()
    }
}
