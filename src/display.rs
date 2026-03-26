// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Display callback traits for the calculator engine.
//!
//! These traits replace the C++ virtual interfaces `ICalcDisplay` and `IHistoryDisplay`.
//! Implement these traits to receive callbacks from the calculator engine.

use crate::commands::CommandType;

/// Expression token: a display string paired with a command ID.
pub type ExpressionToken = (String, i32);

/// Callback interface for calculator display updates.
///
/// Replaces the C++ `ICalcDisplay` pure virtual class.
pub trait CalcDisplay {
    /// Set the primary display text.
    fn set_primary_display(&mut self, text: &str, is_error: bool);

    /// Notify that the calculator is in/out of an error state.
    fn set_is_in_error(&mut self, is_in_error: bool);

    /// Set the expression display (token list).
    fn set_expression_display(
        &mut self,
        tokens: &[ExpressionToken],
        commands: &[Box<dyn ExpressionCommand>],
    );

    /// Set the count of open parentheses.
    fn set_parenthesis_number(&mut self, count: u32);

    /// Called when a closing parenthesis was not added.
    fn on_no_right_paren_added(&mut self);

    /// Called when maximum input digits are reached.
    fn max_digits_reached(&mut self);

    /// Called when a binary operator is received.
    fn binary_operator_received(&mut self);

    /// Called when a history item is added.
    fn on_history_item_added(&mut self, added_item_index: u32);

    /// Set the memorized numbers for display.
    fn set_memorized_numbers(&mut self, memorized_numbers: &[String]);

    /// Called when a memory item changes.
    fn memory_item_changed(&mut self, index_of_memory: u32);

    /// Called when input changes.
    fn input_changed(&mut self);
}

/// Callback interface for calculator history display.
///
/// Replaces the C++ `IHistoryDisplay` pure virtual class.
pub trait HistoryDisplay {
    /// Add an expression and result to history. Returns the index of the added item.
    fn add_to_history(
        &mut self,
        tokens: &[ExpressionToken],
        commands: &[Box<dyn ExpressionCommand>],
        result: &str,
    ) -> u32;
}

/// Expression command trait (replaces C++ `IExpressionCommand`).
pub trait ExpressionCommand {
    /// Get the type of this command.
    fn get_command_type(&self) -> CommandType;
}

/// Unary expression command.
pub trait UnaryCommand: ExpressionCommand {
    /// Get the command IDs.
    fn get_commands(&self) -> &[i32];
    /// Set a single command.
    fn set_command(&mut self, command: i32);
    /// Set two commands.
    fn set_commands(&mut self, command1: i32, command2: i32);
}

/// Binary expression command.
pub trait BinaryCommand: ExpressionCommand {
    /// Get the command ID.
    fn get_command(&self) -> i32;
    /// Set the command ID.
    fn set_command(&mut self, command: i32);
}

/// Operand expression command.
pub trait OperandCommand: ExpressionCommand {
    /// Get the command IDs.
    fn get_commands(&self) -> &[i32];
    /// Append a command.
    fn append_command(&mut self, command: i32);
    /// Toggle the sign.
    fn toggle_sign(&mut self);
    /// Remove last command.
    fn remove_from_end(&mut self);
    /// Check if negative.
    fn is_negative(&self) -> bool;
    /// Check if in scientific format.
    fn is_sci_fmt(&self) -> bool;
    /// Check if decimal point is present.
    fn is_decimal_present(&self) -> bool;
    /// Get the display token.
    fn get_token(&mut self, decimal_symbol: char) -> &str;
}

/// Parenthesis expression command.
pub trait ParenthesisCommand: ExpressionCommand {
    /// Get the command ID.
    fn get_command(&self) -> i32;
}
