# CalcManager Rust Port — Copilot Instructions

## Project Overview

This is a Rust port of Microsoft's Windows Calculator `CalcManager` C++ library.
The original source is in `code2port/src/CalcManager/`.

## Directory Structure

```
rsCalcManager/
├── code2port/                  # Git submodule of microsoft/calculator
│   └── src/CalcManager/        # Original C++ source
│       ├── CEngine/            # Calculator engine implementation
│       ├── Header Files/       # Public headers
│       └── Ratpack/            # Arbitrary precision arithmetic
├── src/                        # Rust crate source
│   ├── lib.rs                  # Crate root, public API re-exports
│   ├── error.rs                # CalcError enum
│   ├── types.rs                # Core types (NumberFormat, AngleType, etc.)
│   ├── commands.rs             # Command enums and OpCode constants
│   ├── display.rs              # CalcDisplay/HistoryDisplay traits
│   ├── ratpack/                # Arbitrary precision rational arithmetic
│   │   ├── number.rs           # Number type (replaces C++ NUMBER/PNUMBER)
│   │   ├── rational.rs         # Rational type (replaces C++ RAT/PRAT)
│   │   ├── arithmetic.rs       # Basic arithmetic (+, -, *, /)
│   │   ├── basex.rs            # Base conversion
│   │   ├── conv.rs             # String conversion / parsing
│   │   ├── constants.rs        # Mathematical constants
│   │   ├── exp.rs              # Exponential / logarithmic functions
│   │   ├── fact.rs             # Factorial
│   │   ├── logic.rs            # Bitwise operations
│   │   ├── num.rs              # Number utilities (GCD, etc.)
│   │   ├── support.rs          # Support functions (trim, scale, snap)
│   │   ├── trans.rs            # Trigonometric functions
│   │   └── itrans.rs           # Inverse trigonometric functions
│   ├── engine/                 # Calculator engine
│   │   ├── calc_engine.rs      # Main engine (CCalcEngine port)
│   │   ├── calc_input.rs       # Input handling (CalcInput port)
│   │   ├── history.rs          # History collector
│   │   └── resource_provider.rs # Resource string provider trait
│   ├── manager/                # Calculator manager
│   │   ├── calculator_manager.rs  # High-level manager
│   │   ├── calculator_history.rs  # History storage
│   │   └── expression_command.rs  # Expression command types
│   └── unit_converter/         # Unit conversion
│       ├── types.rs            # Unit, Category, ConversionData
│       └── converter.rs        # UnitConverter implementation
├── test/                       # Test harnesses
│   ├── port_harness/           # Tests for original C++ code
│   ├── project_harness/        # Tests for Rust port
│   └── test_cases/             # Shared test cases
├── Cargo.toml
└── .github/
    └── copilot-instructions.md # This file
```

## Build Commands

```bash
cargo build              # Build the library
cargo test               # Run all tests
cargo clippy             # Run lints
cargo doc --open         # Generate and view documentation
```

## Key Design Decisions

1. **No `unsafe` code** — All C++ raw pointers replaced with `Vec<u32>` and standard Rust types
2. **`Result<T, CalcError>`** — C++ exceptions mapped to Rust `Result` types
3. **Traits for callbacks** — C++ virtual interfaces (`ICalcDisplay`, `IHistoryDisplay`) become Rust traits
4. **`String` over `wstring`** — Rust `String` (UTF-8) replaces C++ `std::wstring` (UTF-16)
5. **Module mirrors source** — Rust module structure mirrors the C++ directory structure

## Porting Status

- ✅ Types, enums, commands, error codes
- ✅ Basic Number and Rational types
- ✅ Basic arithmetic (add, sub, mul, div for rationals)
- 🔧 Ratpack advanced functions (trig, exp, log, conv) — stubs
- 🔧 CalcEngine command processing — stub
- 🔧 CalculatorManager — stub
- ✅ UnitConverter basic structure
- 🔧 CalcInput — implemented
- 🔧 History — stub

## C++ → Rust Mapping

| C++ Type | Rust Type |
|----------|-----------|
| `NUMBER*` (PNUMBER) | `Number` (owned, with `Vec<u32>` mantissa) |
| `RAT*` (PRAT) | `Rational` (owned, contains two `Number`s) |
| `std::wstring` | `String` |
| `ICalcDisplay*` | `&mut dyn CalcDisplay` |
| `IHistoryDisplay*` | `&mut dyn HistoryDisplay` |
| `IResourceProvider*` | `&dyn ResourceProvider` |
| `MANTTYPE` | `u32` |
| `TWO_MANTTYPE` | `u64` |
| `BASEX` (0x80000000) | `BASEX: u32` constant |
| C++ exceptions | `Result<T, CalcError>` |
| `destroynum/destroyrat` | Automatic via `Drop` |
| `DUPNUM/DUPRAT` | `.clone()` |

## License

MIT License (same as the original Microsoft Calculator source).
