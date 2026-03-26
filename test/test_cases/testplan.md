# Test Case Plan

## Overview
This directory contains shared test cases in JSON format, used by both
the C++ reference harness (`test/port_harness/`) and the Rust integration
tests (`tests/ratpack_integration.rs`).

## Test Case Files

| File | Category | Count | Level |
|------|----------|-------|-------|
| arithmetic.json | add/sub/mul/div/rem/mod/pow | 45 | 2 |
| conversion.json | string↔number, base conversion | 30 | 2 |
| exp_log.json | exp, ln, log10, pow, root | 28 | 2 |
| trig.json | sin, cos, tan + hyperbolic | 30 | 2 |
| itrig.json | asin, acos, atan + inv hyp | 24 | 2 |
| factorial.json | factorial, gamma | 15 | 2 |
| logic.json | AND, OR, XOR, shifts, mod | 25 | 2 |
| support.json | int_rat, frac_rat, gcd, etc. | 16 | 2 |

**Total: ~213 test cases**

## Tolerance Conventions
- `null` tolerance: exact match required (integers, exact rationals)
- Numeric tolerance: relative error bound (e.g., `1e-20` for 128-bit precision)
- String fields: exact match
- Error fields: exact CalcError variant match

## Tags
- `basic`: simple / happy-path test
- `edge`: boundary / corner case
- `error`: expected error condition
- `identity`: mathematical identity check
- `roundtrip`: input→transform→inverse should recover input
- `precision`: tests requiring specific precision behavior
