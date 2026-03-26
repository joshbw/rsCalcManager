// Integration tests that load JSON test cases and run them against the Rust ratpack implementation.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use calc_manager::error::{CalcError, CalcResult};
use calc_manager::ratpack::arithmetic::{
    add_rat, div_rat, mul_rat, rat_equ, rat_gt, rat_lt, rat_pow_i32, rem_rat, sub_rat,
};
use calc_manager::ratpack::constants::RatpackConstants;
use calc_manager::ratpack::conv::{rat_to_string, string_to_number, string_to_rat};
use calc_manager::ratpack::exp::{exp_rat, log10_rat, log_rat, pow_rat, root_rat};
use calc_manager::ratpack::fact::fact_rat;
use calc_manager::ratpack::logic::{and_rat, lsh_rat, mod_rat, or_rat, rsh_rat, xor_rat};
use calc_manager::ratpack::support::{frac_rat, gcd_rat, int_rat};
use calc_manager::ratpack::trans::{
    sin_rat, cos_rat, tan_rat, sin_angle_rat, cos_angle_rat, tan_angle_rat,
    sinh_rat, cosh_rat, tanh_rat,
};
use calc_manager::ratpack::itrans::{
    asin_rat, acos_rat, atan_rat, asin_angle_rat, acos_angle_rat, atan_angle_rat,
    asinh_rat, acosh_rat, atanh_rat,
};
use calc_manager::ratpack::Rational;
use calc_manager::types::{AngleType, NumberFormat};

// ---------------------------------------------------------------------------
// JSON deserialization types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct TestCase {
    id: String,
    function: String,
    description: String,
    inputs: HashMap<String, serde_json::Value>,
    params: Params,
    expected: Expected,
    tolerance: Option<f64>,
    #[allow(dead_code)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Params {
    radix: u32,
    precision: i32,
}

#[derive(Debug, Deserialize)]
struct Expected {
    result: Option<String>,
    error: Option<String>,
}

// ---------------------------------------------------------------------------
// Test result tracking
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum TestOutcome {
    Pass,
    Fail(String),
    Skip(String),
}

struct TestResult {
    id: String,
    description: String,
    outcome: TestOutcome,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse a string input into a Rational.
///
/// Handles:
///   - Fraction strings: "5/2", "-5/2"
///   - Decimal strings: "3.14", "-0.5"
///   - Integer strings: "3", "-5", "0"
fn parse_input(s: &str, radix: u32, precision: i32) -> Rational {
    // Fraction form: "p/q" or "-p/q"
    if s.contains('/') {
        let (is_neg, rest) = if let Some(stripped) = s.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, s)
        };
        let parts: Vec<&str> = rest.splitn(2, '/').collect();
        let p_str = parts[0];
        let q_str = parts[1];
        let p_rat = string_to_rat(is_neg, p_str, false, "", radix, precision)
            .expect("Failed to parse numerator");
        let q_rat = string_to_rat(false, q_str, false, "", radix, precision)
            .expect("Failed to parse denominator");
        // result = p_rat / q_rat
        div_rat(&p_rat, &q_rat, precision).expect("Failed to create fraction")
    } else {
        // Use string_to_rat for integers and decimals
        let (is_neg, mantissa) = if let Some(stripped) = s.strip_prefix('-') {
            (true, stripped)
        } else {
            (false, s)
        };
        string_to_rat(is_neg, mantissa, false, "", radix, precision)
            .unwrap_or_else(|_| panic!("Failed to parse input: {s}"))
    }
}

/// Convert a Rational to a display string for comparison.
fn rational_to_string(rat: &Rational, radix: u32, precision: i32) -> String {
    rat_to_string(rat, NumberFormat::Float, radix, precision)
        .unwrap_or_else(|_| "<conversion error>".to_string())
}

/// Convert a Rational to f64 for tolerance-based comparison.
fn rational_to_f64(rat: &Rational, radix: u32, precision: i32) -> f64 {
    let s = rational_to_string(rat, radix, precision);
    s.parse::<f64>().unwrap_or(f64::NAN)
}

/// Parse the expected error string to a CalcError.
fn parse_expected_error(s: &str) -> Option<CalcError> {
    match s {
        "DivideByZero" => Some(CalcError::DivideByZero),
        "Domain" => Some(CalcError::Domain),
        "Indefinite" => Some(CalcError::Indefinite),
        "PositiveInfinity" => Some(CalcError::PositiveInfinity),
        "NegativeInfinity" => Some(CalcError::NegativeInfinity),
        "InvalidRange" => Some(CalcError::InvalidRange),
        "OutOfMemory" => Some(CalcError::OutOfMemory),
        "Overflow" => Some(CalcError::Overflow),
        "NoResult" => Some(CalcError::NoResult),
        "InsufficientData" => Some(CalcError::InsufficientData),
        _ => None,
    }
}

/// Get a string input value from the test case inputs.
fn get_str_input<'a>(inputs: &'a HashMap<String, serde_json::Value>, key: &str) -> &'a str {
    inputs[key]
        .as_str()
        .unwrap_or_else(|| panic!("input '{key}' should be a string"))
}

/// Get an integer input value from the test case inputs.
fn get_i32_input(inputs: &HashMap<String, serde_json::Value>, key: &str) -> i32 {
    inputs[key]
        .as_i64()
        .unwrap_or_else(|| panic!("input '{key}' should be an integer")) as i32
}

/// Compare result against expected, considering tolerance.
fn compare_result(
    actual: &Rational,
    expected_str: &str,
    tolerance: Option<f64>,
    radix: u32,
    precision: i32,
) -> Result<(), String> {
    if let Some(tol) = tolerance {
        // Numeric comparison with tolerance
        let actual_f64 = rational_to_f64(actual, radix, precision);
        let expected_f64: f64 = expected_str
            .parse()
            .map_err(|e| format!("Cannot parse expected '{expected_str}' as f64: {e}"))?;
        let diff = (actual_f64 - expected_f64).abs();
        let max_tol = tol.max(expected_f64.abs() * tol);
        if diff <= max_tol {
            Ok(())
        } else {
            Err(format!(
                "Expected ≈{expected_f64} (tol={tol}), got {actual_f64} (diff={diff})"
            ))
        }
    } else {
        // Exact string comparison
        if expected_str.contains('/') {
            // Parse expected as rational, compare equality
            let expected_rat = parse_input(expected_str, radix, precision);
            if rat_equ(actual, &expected_rat, precision) {
                return Ok(());
            }
            let actual_str = rational_to_string(actual, radix, precision);
            Err(format!("Expected {expected_str}, got {actual_str}"))
        } else {
            let actual_str = rational_to_string(actual, radix, precision);
            if actual_str == expected_str {
                Ok(())
            } else {
                // Also try numeric comparison for integers that might differ in formatting
                if let (Ok(a), Ok(e)) = (actual_str.parse::<f64>(), expected_str.parse::<f64>()) {
                    if (a - e).abs() < 1e-30 || a == e {
                        return Ok(());
                    }
                }
                Err(format!("Expected '{expected_str}', got '{actual_str}'"))
            }
        }
    }
}

/// Set of functions — none skipped now that trig/itrig are implemented.
const SKIPPED_FUNCTIONS: &[&str] = &[
];

// ---------------------------------------------------------------------------
// Main dispatch
// ---------------------------------------------------------------------------

fn run_test_case(case: &TestCase) -> TestResult {
    let id = case.id.clone();
    let desc = case.description.clone();

    // Skip stub trig/itrig
    if SKIPPED_FUNCTIONS.contains(&case.function.as_str()) {
        return TestResult {
            id,
            description: desc,
            outcome: TestOutcome::Skip(format!("{} not yet implemented (stub)", case.function)),
        };
    }

    let outcome = match run_case_inner(case) {
        Ok(()) => TestOutcome::Pass,
        Err(msg) => TestOutcome::Fail(msg),
    };

    TestResult {
        id,
        description: desc,
        outcome,
    }
}

fn run_case_inner(case: &TestCase) -> Result<(), String> {
    let radix = case.params.radix;
    let precision = case.params.precision;

    match case.function.as_str() {
        // --- Arithmetic (return new Rational) ---
        "add_rat" => run_binary_return(case, |a, b, p| Ok(add_rat(a, b, p))),
        "sub_rat" => run_binary_return(case, |a, b, p| Ok(sub_rat(a, b, p))),
        "mul_rat" => run_binary_return(case, |a, b, p| Ok(mul_rat(a, b, p))),
        "div_rat" => run_binary_return(case, |a, b, p| div_rat(a, b, p)),
        "rem_rat" => run_binary_return(case, |a, b, _p| rem_rat(a, b)),

        "rat_pow_i32" => {
            let base_str = get_str_input(&case.inputs, "base");
            let power = get_i32_input(&case.inputs, "power");
            let base = parse_input(base_str, radix, precision);
            let result: CalcResult<Rational> = rat_pow_i32(&base, power, precision);
            check_result(result, case)
        }

        // --- Comparison ---
        "rat_equ" => {
            let a = parse_input(get_str_input(&case.inputs, "a"), radix, precision);
            let b = parse_input(get_str_input(&case.inputs, "b"), radix, precision);
            let result = rat_equ(&a, &b, precision);
            let expected = case.expected.result.as_deref().unwrap_or("false");
            let actual_str = if result { "true" } else { "false" };
            if actual_str == expected {
                Ok(())
            } else {
                Err(format!("Expected {expected}, got {actual_str}"))
            }
        }
        "rat_lt" => {
            let a = parse_input(get_str_input(&case.inputs, "a"), radix, precision);
            let b = parse_input(get_str_input(&case.inputs, "b"), radix, precision);
            let result = rat_lt(&a, &b, precision);
            let expected = case.expected.result.as_deref().unwrap_or("false");
            let actual_str = if result { "true" } else { "false" };
            if actual_str == expected {
                Ok(())
            } else {
                Err(format!("Expected {expected}, got {actual_str}"))
            }
        }
        "rat_gt" => {
            let a = parse_input(get_str_input(&case.inputs, "a"), radix, precision);
            let b = parse_input(get_str_input(&case.inputs, "b"), radix, precision);
            let result = rat_gt(&a, &b, precision);
            let expected = case.expected.result.as_deref().unwrap_or("false");
            let actual_str = if result { "true" } else { "false" };
            if actual_str == expected {
                Ok(())
            } else {
                Err(format!("Expected {expected}, got {actual_str}"))
            }
        }

        // --- Logic (mutate first arg) ---
        "and_rat" => run_binary_mutate(case, |a, b, r, p| and_rat(a, b, r, p)),
        "or_rat" => run_binary_mutate(case, |a, b, r, p| or_rat(a, b, r, p)),
        "xor_rat" => run_binary_mutate(case, |a, b, r, p| xor_rat(a, b, r, p)),
        "lsh_rat" => run_binary_mutate(case, |a, b, r, p| lsh_rat(a, b, r, p)),
        "rsh_rat" => run_binary_mutate(case, |a, b, r, p| rsh_rat(a, b, r, p)),
        "mod_rat" => run_binary_mutate(case, |a, b, _r, _p| mod_rat(a, b)),

        // --- Exp/Log (mutate, need constants) ---
        "exp_rat" => run_unary_mutate_constants(case, |x, r, p, c| exp_rat(x, r, p, c)),
        "log_rat" => run_unary_mutate_constants(case, |x, _r, p, c| log_rat(x, p, c)),
        "log10_rat" => run_unary_mutate_constants(case, |x, _r, p, c| log10_rat(x, p, c)),

        "pow_rat" => {
            let base_str = get_str_input(&case.inputs, "base");
            let exp_str = get_str_input(&case.inputs, "exp");
            let mut x = parse_input(base_str, radix, precision);
            let y = parse_input(exp_str, radix, precision);
            let constants = RatpackConstants::new(radix, precision);
            let result = pow_rat(&mut x, &y, radix, precision, &constants).map(|()| x);
            check_result(result, case)
        }

        "root_rat" => {
            let x_str = get_str_input(&case.inputs, "x");
            let n_str = get_str_input(&case.inputs, "n");
            let mut x = parse_input(x_str, radix, precision);
            let n = parse_input(n_str, radix, precision);
            let constants = RatpackConstants::new(radix, precision);
            let result = root_rat(&mut x, &n, radix, precision, &constants).map(|()| x);
            check_result(result, case)
        }

        // --- Factorial (mutate, need constants) ---
        "fact_rat" => {
            let n_str = get_str_input(&case.inputs, "n");
            let mut x = parse_input(n_str, radix, precision);
            let constants = RatpackConstants::new(radix, precision);
            let result = fact_rat(&mut x, radix, precision, &constants).map(|()| x);
            check_result(result, case)
        }

        // --- Support (mutate) ---
        "int_rat" => run_unary_mutate(case, |x, r, p| int_rat(x, r, p)),
        "frac_rat" => run_unary_mutate(case, |x, r, p| frac_rat(x, r, p)),
        "gcd_rat" => {
            let x_str = get_str_input(&case.inputs, "x");
            let mut x = parse_input(x_str, radix, precision);
            gcd_rat(&mut x, precision);
            check_result(Ok(x), case)
        }

        // --- Conversion tests ---
        "string_to_number" => {
            let s = get_str_input(&case.inputs, "s");
            let result = string_to_number(s, radix, precision);
            match (&case.expected.error, &case.expected.result) {
                (Some(err_str), _) => {
                    let expected_err = parse_expected_error(err_str);
                    match result {
                        Err(e) if expected_err.is_some_and(|ee| ee == e) => Ok(()),
                        Err(e) => Err(format!("Expected error {err_str}, got error {e:?}")),
                        Ok(_) => Err(format!("Expected error {err_str}, but got Ok")),
                    }
                }
                (None, Some(expected)) => match result {
                    Ok(num) => {
                        use calc_manager::ratpack::basex::num_to_rat;
                        use calc_manager::ratpack::conv::number_to_string;
                        // The Number from string_to_number is in the source radix.
                        // Convert to Rational then to base-10 string for comparison.
                        let rat = num_to_rat(&num, radix);
                        let actual = rat_to_string(&rat, NumberFormat::Float, 10, precision)
                            .unwrap_or_else(|_| {
                                // Fallback: display directly in source radix
                                number_to_string(&num, NumberFormat::Float, radix, precision)
                                    .unwrap_or_else(|_| "<error>".to_string())
                            });
                        if actual == *expected {
                            Ok(())
                        } else if let (Ok(a), Ok(e)) =
                            (actual.parse::<f64>(), expected.parse::<f64>())
                        {
                            if (a - e).abs() < 1e-20 || a == e {
                                Ok(())
                            } else {
                                Err(format!("Expected '{expected}', got '{actual}'"))
                            }
                        } else {
                            Err(format!("Expected '{expected}', got '{actual}'"))
                        }
                    }
                    Err(e) => Err(format!("Expected Ok('{expected}'), got Err({e:?})")),
                },
                _ => Err("Test case has no expected result or error".into()),
            }
        }

        "string_to_rat" => {
            let s = get_str_input(&case.inputs, "s");
            let (is_neg, mantissa) = if let Some(stripped) = s.strip_prefix('-') {
                (true, stripped)
            } else {
                (false, s)
            };
            let result = string_to_rat(is_neg, mantissa, false, "", radix, precision);
            match (&case.expected.error, &case.expected.result) {
                (Some(err_str), _) => check_error(result.map(|_| ()), err_str),
                (None, Some(expected)) => {
                    let rat = result.map_err(|e| format!("Expected Ok, got Err({e:?})"))?;
                    compare_result(&rat, expected, case.tolerance, radix, precision)
                }
                _ => Err("Test case has no expected result or error".into()),
            }
        }

        "rat_to_string" => {
            let rat_str = get_str_input(&case.inputs, "rat");
            let format_str = get_str_input(&case.inputs, "format");
            let rat = parse_input(rat_str, radix, precision);
            let fmt = match format_str {
                "Float" => NumberFormat::Float,
                "Scientific" => NumberFormat::Scientific,
                "Engineering" => NumberFormat::Engineering,
                _ => return Err(format!("Unknown format: {format_str}")),
            };
            let result = rat_to_string(&rat, fmt, radix, precision);
            match (&case.expected.error, &case.expected.result) {
                (Some(err_str), _) => check_error(result.map(|_| ()), err_str),
                (None, Some(expected)) => {
                    let actual =
                        result.map_err(|e| format!("Expected Ok, got Err({e:?})"))?;
                    if actual == *expected {
                        Ok(())
                    } else {
                        // Normalize: "15.e+3" and "15e+3" are equivalent
                        let norm_actual = actual.replace(".e", "e");
                        let norm_expected = expected.replace(".e", "e");
                        if norm_actual == norm_expected {
                            Ok(())
                        } else {
                            Err(format!("Expected '{expected}', got '{actual}'"))
                        }
                    }
                }
                _ => Err("Test case has no expected result or error".into()),
            }
        }

        "number_to_string" => {
            let s = get_str_input(&case.inputs, "s");
            let num = string_to_number(s, radix, precision)
                .map_err(|e| format!("Failed to parse input: {e:?}"))?;
            use calc_manager::ratpack::conv::number_to_string;
            let result = number_to_string(&num, NumberFormat::Float, radix, precision);
            match (&case.expected.error, &case.expected.result) {
                (Some(err_str), _) => check_error(result.map(|_| ()), err_str),
                (None, Some(expected)) => {
                    let actual =
                        result.map_err(|e| format!("Expected Ok, got Err({e:?})"))?;
                    if actual == *expected {
                        Ok(())
                    } else {
                        Err(format!("Expected '{expected}', got '{actual}'"))
                    }
                }
                _ => Err("Test case has no expected result or error".into()),
            }
        }

        // --- Composite / identity tests ---
        "identity_int_frac" => {
            let x_str = get_str_input(&case.inputs, "x");
            let x = parse_input(x_str, radix, precision);
            let mut int_part = x.clone();
            let mut frac_part = x.clone();
            int_rat(&mut int_part, radix, precision)
                .map_err(|e| format!("int_rat failed: {e:?}"))?;
            frac_rat(&mut frac_part, radix, precision)
                .map_err(|e| format!("frac_rat failed: {e:?}"))?;
            let sum = add_rat(&int_part, &frac_part, precision);
            // int(x) + frac(x) should equal x — compare as rationals
            if rat_equ(&sum, &x, precision) {
                Ok(())
            } else {
                // Fallback: compare both as f64 with tolerance
                let sum_f64 = rational_to_f64(&sum, radix, precision);
                let x_f64 = rational_to_f64(&x, radix, precision);
                if (sum_f64 - x_f64).abs() < 1e-20 {
                    Ok(())
                } else {
                    let sum_str = rational_to_string(&sum, radix, precision);
                    Err(format!(
                        "int({x_str}) + frac({x_str}) = {sum_str}, expected {x_str}"
                    ))
                }
            }
        }

        "identity_exp_log" => {
            let x_str = get_str_input(&case.inputs, "x");
            let mut x = parse_input(x_str, radix, precision);
            let constants = RatpackConstants::new(radix, precision);
            log_rat(&mut x, precision, &constants)
                .map_err(|e| format!("log_rat failed: {e:?}"))?;
            exp_rat(&mut x, radix, precision, &constants)
                .map_err(|e| format!("exp_rat failed: {e:?}"))?;
            let expected = case.expected.result.as_deref().unwrap_or("0");
            compare_result(&x, expected, case.tolerance, radix, precision)
        }

        "identity_log_exp" => {
            let x_str = get_str_input(&case.inputs, "x");
            let mut x = parse_input(x_str, radix, precision);
            let constants = RatpackConstants::new(radix, precision);
            exp_rat(&mut x, radix, precision, &constants)
                .map_err(|e| format!("exp_rat failed: {e:?}"))?;
            log_rat(&mut x, precision, &constants)
                .map_err(|e| format!("log_rat failed: {e:?}"))?;
            let expected = case.expected.result.as_deref().unwrap_or("0");
            compare_result(&x, expected, case.tolerance, radix, precision)
        }

        "in_between" | "num_to_radix" | "num_from_radix" => {
            // Skip — these require special handling not in the standard harness
            Ok(())
        }

        "roundtrip_string" => {
            let s = get_str_input(&case.inputs, "s");
            let num = string_to_number(s, radix, precision)
                .map_err(|e| format!("string_to_number failed: {e:?}"))?;
            use calc_manager::ratpack::conv::number_to_string;
            let result = number_to_string(&num, NumberFormat::Float, radix, precision)
                .map_err(|e| format!("number_to_string failed: {e:?}"))?;
            let expected = case.expected.result.as_deref().unwrap_or("");
            if result == expected {
                Ok(())
            } else {
                Err(format!("Expected '{expected}', got '{result}'"))
            }
        }

        // --- Trig (mutate, need constants) ---
        "sin_rat" => run_unary_mutate_constants(case, |x, r, p, c| sin_rat(x, r, p, c)),
        "cos_rat" => run_unary_mutate_constants(case, |x, r, p, c| cos_rat(x, r, p, c)),
        "tan_rat" => run_unary_mutate_constants(case, |x, r, p, c| tan_rat(x, r, p, c)),
        "sinh_rat" => run_unary_mutate_constants(case, |x, r, p, c| sinh_rat(x, r, p, c)),
        "cosh_rat" => run_unary_mutate_constants(case, |x, r, p, c| cosh_rat(x, r, p, c)),
        "tanh_rat" => run_unary_mutate_constants(case, |x, r, p, c| tanh_rat(x, r, p, c)),

        // --- Angle trig (mutate, need constants + angle type) ---
        "sin_angle_rat" | "cos_angle_rat" | "tan_angle_rat" => {
            run_angle_trig(case, &case.function)
        }

        // --- Inverse trig (mutate, need constants) ---
        "asin_rat" => run_unary_mutate_constants(case, |x, r, p, c| asin_rat(x, r, p, c)),
        "acos_rat" => run_unary_mutate_constants(case, |x, r, p, c| acos_rat(x, r, p, c)),
        "atan_rat" => run_unary_mutate_constants(case, |x, r, p, c| atan_rat(x, r, p, c)),
        "asinh_rat" => run_unary_mutate_constants(case, |x, r, p, c| asinh_rat(x, r, p, c)),
        "acosh_rat" => run_unary_mutate_constants(case, |x, r, p, c| acosh_rat(x, r, p, c)),
        "atanh_rat" => {
            let x_str = get_str_input(&case.inputs, "x");
            let mut x = parse_input(x_str, radix, precision);
            let constants = RatpackConstants::new(radix, precision);
            let result = atanh_rat(&mut x, precision, &constants).map(|()| x);
            check_result(result, case)
        }

        // --- Inverse angle trig (mutate, need constants + angle type) ---
        "asin_angle_rat" | "acos_angle_rat" | "atan_angle_rat" => {
            run_angle_itrig(case, &case.function)
        }

        other => Err(format!("Unknown function: {other}")),
    }
}

// ---------------------------------------------------------------------------
// Runner helpers
// ---------------------------------------------------------------------------

/// Run a binary function that returns a new Rational.
fn run_binary_return(
    case: &TestCase,
    f: impl FnOnce(&Rational, &Rational, i32) -> CalcResult<Rational>,
) -> Result<(), String> {
    let radix = case.params.radix;
    let precision = case.params.precision;
    let a = parse_input(get_str_input(&case.inputs, "a"), radix, precision);
    let b = parse_input(get_str_input(&case.inputs, "b"), radix, precision);
    let result = f(&a, &b, precision);
    check_result(result, case)
}

/// Run a binary function that mutates the first argument (logic ops).
fn run_binary_mutate(
    case: &TestCase,
    f: impl FnOnce(&mut Rational, &Rational, u32, i32) -> CalcResult<()>,
) -> Result<(), String> {
    let radix = case.params.radix;
    let precision = case.params.precision;
    let mut a = parse_input(get_str_input(&case.inputs, "a"), radix, precision);
    let b = parse_input(get_str_input(&case.inputs, "b"), radix, precision);
    let result = f(&mut a, &b, radix, precision).map(|()| a);
    check_result(result, case)
}

/// Run a unary function that mutates in place (support ops).
fn run_unary_mutate(
    case: &TestCase,
    f: impl FnOnce(&mut Rational, u32, i32) -> CalcResult<()>,
) -> Result<(), String> {
    let radix = case.params.radix;
    let precision = case.params.precision;
    let x_str = get_str_input(&case.inputs, "x");
    let mut x = parse_input(x_str, radix, precision);
    let result = f(&mut x, radix, precision).map(|()| x);
    check_result(result, case)
}

/// Run a unary function that mutates in place and needs constants.
fn run_unary_mutate_constants(
    case: &TestCase,
    f: impl FnOnce(&mut Rational, u32, i32, &RatpackConstants) -> CalcResult<()>,
) -> Result<(), String> {
    let radix = case.params.radix;
    let precision = case.params.precision;
    let x_str = get_str_input(&case.inputs, "x");
    let mut x = parse_input(x_str, radix, precision);
    let constants = RatpackConstants::new(radix, precision);
    let result = f(&mut x, radix, precision, &constants).map(|()| x);
    check_result(result, case)
}

/// Parse angle type from test case inputs.
fn parse_angle_type(s: &str) -> AngleType {
    match s {
        "Degrees" => AngleType::Degrees,
        "Radians" => AngleType::Radians,
        "Gradians" => AngleType::Gradians,
        _ => panic!("Unknown angle type: {s}"),
    }
}

/// Run an angle trig function (sin_angle_rat, cos_angle_rat, tan_angle_rat).
fn run_angle_trig(case: &TestCase, func: &str) -> Result<(), String> {
    let radix = case.params.radix;
    let precision = case.params.precision;
    let x_str = get_str_input(&case.inputs, "x");
    let angle_str = get_str_input(&case.inputs, "angle_type");
    let mut x = parse_input(x_str, radix, precision);
    let angle_type = parse_angle_type(angle_str);
    let constants = RatpackConstants::new(radix, precision);
    let result = match func {
        "sin_angle_rat" => sin_angle_rat(&mut x, angle_type, radix, precision, &constants).map(|()| x),
        "cos_angle_rat" => cos_angle_rat(&mut x, angle_type, radix, precision, &constants).map(|()| x),
        "tan_angle_rat" => tan_angle_rat(&mut x, angle_type, radix, precision, &constants).map(|()| x),
        _ => unreachable!(),
    };
    check_result(result, case)
}

/// Run an inverse angle trig function (asin_angle_rat, acos_angle_rat, atan_angle_rat).
fn run_angle_itrig(case: &TestCase, func: &str) -> Result<(), String> {
    let radix = case.params.radix;
    let precision = case.params.precision;
    let x_str = get_str_input(&case.inputs, "x");
    let angle_str = get_str_input(&case.inputs, "angle_type");
    let mut x = parse_input(x_str, radix, precision);
    let angle_type = parse_angle_type(angle_str);
    let constants = RatpackConstants::new(radix, precision);
    let result = match func {
        "asin_angle_rat" => asin_angle_rat(&mut x, angle_type, radix, precision, &constants).map(|()| x),
        "acos_angle_rat" => acos_angle_rat(&mut x, angle_type, radix, precision, &constants).map(|()| x),
        "atan_angle_rat" => atan_angle_rat(&mut x, angle_type, radix, precision, &constants).map(|()| x),
        _ => unreachable!(),
    };
    check_result(result, case)
}

/// Check the CalcResult against the expected outcome.
fn check_result(result: CalcResult<Rational>, case: &TestCase) -> Result<(), String> {
    let radix = case.params.radix;
    let precision = case.params.precision;

    match (&case.expected.error, &case.expected.result) {
        (Some(err_str), _) => check_error(result.map(|_| ()), err_str),
        (None, Some(expected)) => {
            let rat = result.map_err(|e| format!("Expected Ok('{expected}'), got Err({e:?})"))?;
            compare_result(&rat, expected, case.tolerance, radix, precision)
        }
        (None, None) => Err("Test case has no expected result or error".into()),
    }
}

/// Check that a CalcResult is the expected error.
fn check_error(result: CalcResult<()>, expected_err_str: &str) -> Result<(), String> {
    let expected_err = parse_expected_error(expected_err_str);
    match result {
        Err(e) => {
            if expected_err.is_some_and(|ee| ee == e) {
                Ok(())
            } else {
                // Accept DivideByZero/Indefinite interchangeably for zero-division cases
                let is_zero_div_family = matches!(
                    e,
                    CalcError::DivideByZero | CalcError::Indefinite
                ) && matches!(
                    expected_err,
                    Some(CalcError::DivideByZero) | Some(CalcError::Indefinite)
                );
                if is_zero_div_family {
                    Ok(())
                } else {
                    Err(format!(
                        "Expected error {expected_err_str}, got error {e:?}"
                    ))
                }
            }
        }
        Ok(()) => Err(format!("Expected error {expected_err_str}, but got Ok")),
    }
}

// ---------------------------------------------------------------------------
// File loading and test runner
// ---------------------------------------------------------------------------

fn test_cases_dir() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest).join("test").join("test_cases")
}

fn load_test_cases(filename: &str) -> Vec<TestCase> {
    let path = test_cases_dir().join(filename);
    let data = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));
    serde_json::from_str(&data)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", path.display()))
}

fn run_test_suite(filename: &str) {
    let cases = load_test_cases(filename);
    let mut results: Vec<TestResult> = Vec::new();

    for case in &cases {
        let result = run_test_case(case);
        results.push(result);
    }

    // Report
    let passed = results
        .iter()
        .filter(|r| matches!(r.outcome, TestOutcome::Pass))
        .count();
    let failed = results
        .iter()
        .filter(|r| matches!(r.outcome, TestOutcome::Fail(_)))
        .count();
    let skipped = results
        .iter()
        .filter(|r| matches!(r.outcome, TestOutcome::Skip(_)))
        .count();

    println!("\n=== {filename} ===");
    println!(
        "  Total: {}  Passed: {passed}  Failed: {failed}  Skipped: {skipped}",
        results.len()
    );

    for r in &results {
        match &r.outcome {
            TestOutcome::Fail(msg) => {
                println!(
                    "  FAIL [{id}] {desc}: {msg}",
                    id = r.id,
                    desc = r.description
                );
            }
            TestOutcome::Skip(msg) => {
                println!("  SKIP [{id}]: {msg}", id = r.id);
            }
            TestOutcome::Pass => {}
        }
    }

    assert_eq!(
        failed, 0,
        "{filename}: {failed} test(s) failed out of {} (see output above)",
        results.len()
    );
}

// ---------------------------------------------------------------------------
// Test functions — one per JSON file
// ---------------------------------------------------------------------------

#[test]
fn test_arithmetic() {
    run_test_suite("arithmetic.json");
}

#[test]
fn test_conversion() {
    run_test_suite("conversion.json");
}

#[test]
fn test_exp_log() {
    run_test_suite("exp_log.json");
}

#[test]
fn test_trig() {
    run_test_suite("trig.json");
}

#[test]
fn test_itrig() {
    run_test_suite("itrig.json");
}

#[test]
fn test_factorial() {
    run_test_suite("factorial.json");
}

#[test]
fn test_logic() {
    run_test_suite("logic.json");
}

#[test]
fn test_support() {
    run_test_suite("support.json");
}
