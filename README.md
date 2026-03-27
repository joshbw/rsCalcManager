# calc_manager

A Rust port of Microsoft's Windows Calculator [CalcManager](https://github.com/microsoft/calculator/tree/main/src/CalcManager) engine — arbitrary-precision rational arithmetic with full scientific calculator support.

[![CI](https://github.com/joshbw/rsCalcManager/actions/workflows/ci.yml/badge.svg)](https://github.com/joshbw/rsCalcManager/actions/workflows/ci.yml)

## Features

- **Arbitrary-precision rational arithmetic** — exact `p/q` representation with configurable display precision
- **Zero dependencies** — pure Rust, no `unsafe`, no external crates required at runtime
- **Full scientific calculator math**:
  - Arithmetic: add, subtract, multiply, divide, remainder, power
  - Exponential/logarithmic: exp, ln, log₁₀, nth root
  - Trigonometric: sin, cos, tan (radians, degrees, gradians)
  - Inverse trig: asin, acos, atan
  - Hyperbolic: sinh, cosh, tanh, asinh, acosh, atanh
  - Factorial via gamma function
  - Bitwise logic: and, or, xor, shifts
  - Base conversion: binary, octal, decimal, hexadecimal
- **Cross-platform** — builds on Windows, macOS, Linux, iOS, and Android

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
calc_manager = { git = "https://github.com/joshbw/rsCalcManager" }
```

```rust
use calc_manager::ratpack::{Rational, arithmetic::*, constants::RatpackConstants};
use calc_manager::ratpack::exp::{exp_rat, log_rat};
use calc_manager::ratpack::trans::sin_rat;

let precision = 128;
let radix = 10;
let constants = RatpackConstants::new(radix, precision);

// Exact rational arithmetic
let a = Rational::from(355i32);
let b = Rational::from(113i32);
let pi_approx = div_rat(&a, &b, precision).unwrap();

// Transcendental functions
let mut x = Rational::from(1i32);
exp_rat(&mut x, radix, precision, &constants).unwrap();
// x ≈ e ≈ 2.71828...

// Trigonometry (input in radians)
let mut angle = div_rat(&constants.pi, &Rational::from(6i32), precision).unwrap();
sin_rat(&mut angle, radix, precision, &constants).unwrap();
// angle ≈ 0.5
```

## Architecture

| Module | Description |
|--------|-------------|
| `ratpack` | Arbitrary-precision rational number arithmetic (core math engine) |
| `engine` | Calculator state machine and command processing (stub) |
| `manager` | High-level calculator manager — mode switching, history, memory (stub) |
| `unit_converter` | Unit conversion engine (stub) |

The `ratpack` module is fully ported and tested. It provides the same mathematical operations as the C++ Ratpack library used in Windows Calculator.

## Building

```bash
cargo build          # Build the library
cargo test           # Run all tests (159 unit + 213 integration + 1 doc)
cargo doc --open     # Generate API documentation
```

## Testing

The test suite includes 373 tests validated against the original C++ implementation:

| Category | Tests |
|----------|-------|
| Arithmetic | 46 |
| Logic (bitwise) | 25 |
| Conversion | 30 |
| Support (int, frac, gcd) | 16 |
| Exp / Log | 28 |
| Factorial | 15 |
| Trigonometric | 29 |
| Inverse Trig | 24 |
| Unit tests | 159 |
| Doc tests | 1 |

## Design Decisions

- **No `unsafe`** — all C++ raw pointers replaced with `Vec<u32>` and owned types
- **`Result<T, CalcError>`** — C++ exceptions mapped to Rust `Result` types
- **Automatic memory management** — C++ `destroynum`/`destroyrat` replaced by `Drop`
- **Trait-based callbacks** — C++ virtual interfaces become Rust traits (`CalcDisplay`, `HistoryDisplay`)

## License

[MIT](LICENSE) — same as the [original Microsoft Calculator source](https://github.com/microsoft/calculator/blob/main/LICENSE).
