// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Expression command types.
//!
//! Port of C++ CParentheses, CUnaryCommand, CBinaryCommand, COpndCommand classes.

use crate::commands::CommandType;
use crate::display::ExpressionCommand;

/// Parenthesis command.
#[derive(Debug, Clone)]
pub struct ParenthesesCmd {
    command: i32,
}

impl ParenthesesCmd {
    #[must_use]
    pub fn new(command: i32) -> Self {
        Self { command }
    }

    #[must_use]
    pub fn get_command(&self) -> i32 {
        self.command
    }
}

impl ExpressionCommand for ParenthesesCmd {
    fn get_command_type(&self) -> CommandType {
        CommandType::Parentheses
    }
}

/// Unary command (one or two command IDs).
#[derive(Debug, Clone)]
pub struct UnaryCmd {
    commands: Vec<i32>,
}

impl UnaryCmd {
    #[must_use]
    pub fn new(command: i32) -> Self {
        Self {
            commands: vec![command],
        }
    }

    #[must_use]
    pub fn new_two(command1: i32, command2: i32) -> Self {
        Self {
            commands: vec![command1, command2],
        }
    }

    #[must_use]
    pub fn get_commands(&self) -> &[i32] {
        &self.commands
    }
}

impl ExpressionCommand for UnaryCmd {
    fn get_command_type(&self) -> CommandType {
        CommandType::UnaryCommand
    }
}

/// Binary command.
#[derive(Debug, Clone)]
pub struct BinaryCmd {
    command: i32,
}

impl BinaryCmd {
    #[must_use]
    pub fn new(command: i32) -> Self {
        Self { command }
    }

    #[must_use]
    pub fn get_command(&self) -> i32 {
        self.command
    }

    pub fn set_command(&mut self, command: i32) {
        self.command = command;
    }
}

impl ExpressionCommand for BinaryCmd {
    fn get_command_type(&self) -> CommandType {
        CommandType::BinaryCommand
    }
}

/// Operand command (digit sequence).
#[derive(Debug, Clone)]
pub struct OperandCmd {
    commands: Vec<i32>,
    negative: bool,
    sci_fmt: bool,
    decimal: bool,
    token: String,
}

impl OperandCmd {
    #[must_use]
    pub fn new(commands: Vec<i32>, negative: bool, decimal: bool, sci_fmt: bool) -> Self {
        Self {
            commands,
            negative,
            sci_fmt,
            decimal,
            token: String::new(),
        }
    }

    #[must_use]
    pub fn get_commands(&self) -> &[i32] {
        &self.commands
    }

    pub fn append_command(&mut self, command: i32) {
        self.commands.push(command);
    }

    pub fn toggle_sign(&mut self) {
        self.negative = !self.negative;
    }

    pub fn remove_from_end(&mut self) {
        self.commands.pop();
    }

    #[must_use]
    pub fn is_negative(&self) -> bool {
        self.negative
    }

    #[must_use]
    pub fn is_sci_fmt(&self) -> bool {
        self.sci_fmt
    }

    #[must_use]
    pub fn is_decimal_present(&self) -> bool {
        self.decimal
    }
}

impl ExpressionCommand for OperandCmd {
    fn get_command_type(&self) -> CommandType {
        CommandType::OperandCommand
    }
}
