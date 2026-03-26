// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Resource provider trait.
//!
//! Port of C++ `IResourceProvider` interface.

/// Provides localized strings to the calculator engine.
pub trait ResourceProvider {
    /// Get a localized string by its resource key.
    fn get_resource_string(&self, key: &str) -> String;
}

/// A simple resource provider that returns the key as-is (for testing).
pub struct DefaultResourceProvider;

impl ResourceProvider for DefaultResourceProvider {
    fn get_resource_string(&self, key: &str) -> String {
        key.to_string()
    }
}
