// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Number/rational to string conversion and parsing.
//! Port of C++ Ratpack/conv.cpp

use crate::error::{CalcError, CalcResult};
use crate::types::{MantType, NumberFormat, BASEX};

use super::arithmetic::{add_num, div_num, num_pow_i32_x};
use super::basex::{num_to_radix, num_to_rat};
use super::{Number, Rational};

/// Maximum leading zeros after decimal before switching to scientific notation.
const MAX_ZEROS_AFTER_DECIMAL: i32 = 2;

/// Digit characters for bases 2..64 (matching C++ DIGITS table).
const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_@";

/// Default decimal separator.
const DECIMAL_SEPARATOR: char = '.';

// ---------------------------------------------------------------------------
// State machine terminals and states for StringToNumber
// ---------------------------------------------------------------------------
const DP: usize = 0; // '.'
const ZR: usize = 1; // '0'
const NZ: usize = 2; // non-zero digit
const SG: usize = 3; // '+' or '-'
const EX: usize = 4; // exponent marker ('e' for base 10, '^' otherwise)

const START: u8 = 0;
const MANTS: u8 = 1;
const LZ: u8 = 2;
const LZDP: u8 = 3;
const LD: u8 = 4;
const DZ: u8 = 5;
const DD: u8 = 6;
const DDP: u8 = 7;
const EXPB: u8 = 8;
const EXPS: u8 = 9;
const EXPD: u8 = 10;
const EXPBZ: u8 = 11;
const EXPSZ: u8 = 12;
const EXPDZ: u8 = 13;
const ERR: u8 = 14;

/// State transition table: machine[state][terminal]
#[rustfmt::skip]
static MACHINE: [[u8; 5]; 15] = [
    //       DP,    ZR,    NZ,    SG,    EX
    /* START */ [LZDP,  LZ,    LD,    MANTS, ERR  ],
    /* MANTS */ [LZDP,  LZ,    LD,    ERR,   ERR  ],
    /* LZ    */ [LZDP,  LZ,    LD,    ERR,   EXPBZ],
    /* LZDP  */ [ERR,   DZ,    DD,    ERR,   EXPB ],
    /* LD    */ [DDP,   LD,    LD,    ERR,   EXPB ],
    /* DZ    */ [ERR,   DZ,    DD,    ERR,   EXPBZ],
    /* DD    */ [ERR,   DD,    DD,    ERR,   EXPB ],
    /* DDP   */ [ERR,   DD,    DD,    ERR,   EXPB ],
    /* EXPB  */ [ERR,   EXPD,  EXPD,  EXPS,  ERR  ],
    /* EXPS  */ [ERR,   EXPD,  EXPD,  ERR,   ERR  ],
    /* EXPD  */ [ERR,   EXPD,  EXPD,  ERR,   ERR  ],
    /* EXPBZ */ [ERR,   EXPDZ, EXPDZ, EXPSZ, ERR  ],
    /* EXPSZ */ [ERR,   EXPDZ, EXPDZ, ERR,   ERR  ],
    /* EXPDZ */ [ERR,   EXPDZ, EXPDZ, ERR,   ERR  ],
    /* ERR   */ [ERR,   ERR,   ERR,   ERR,   ERR  ],
];

/// Normalize a character for digit lookup.
const fn normalize_char_digit(c: char, radix: u32) -> char {
    if radix <= 36 {
        c.to_ascii_uppercase()
    } else {
        c
    }
}

/// Look up a character's digit value in the DIGITS table.
fn digit_value(c: char) -> Option<u32> {
    DIGITS.iter().position(|&d| d == c as u8).map(|p| p as u32)
}

// ---------------------------------------------------------------------------
// StringToNumber
// ---------------------------------------------------------------------------

/// Parse a string into a Number in the given radix.
///
/// Port of C++ `StringToNumber`. Uses a state machine to handle:
/// leading zeros, sign, decimal point, exponent notation.
///
/// The resulting Number's mantissa digits are each in `[0, radix)`.
pub fn string_to_number(s: &str, radix: u32, precision: i32) -> CalcResult<Number> {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    if len == 0 {
        return Err(CalcError::NoResult);
    }

    let mut ps = ParseState {
        sign: 1,
        exp: 0,
        exp_sign: 1,
        exp_value: 0,
        cdigit: 0,
    };
    let mut mant = vec![0u32; len];
    let mut mant_idx = len;
    let mut state: u8 = START;

    for &c in &chars {
        let cur_char = if c == DECIMAL_SEPARATOR { '.' } else { c };
        let terminal = classify_char(cur_char, radix);
        state = MACHINE[state as usize][terminal];

        state = process_state(state, cur_char, radix, &mut ps, &mut mant, &mut mant_idx)?;
    }

    if state == DZ || state == EXPDZ {
        return Ok(Number::zero());
    }

    // Reject invalid final states (incomplete exponent, error, etc.)
    if state == ERR || state == MANTS || state == EXPB || state == EXPS
        || state == EXPBZ || state == EXPSZ
    {
        return Err(CalcError::NoResult);
    }

    while ps.cdigit < len {
        ps.cdigit += 1;
        ps.exp -= 1;
    }
    ps.exp += ps.exp_sign * ps.exp_value;

    if ps.cdigit == 0 {
        return Err(CalcError::NoResult);
    }

    let digits_le = &mant[mant_idx..];
    let mut mantissa: Vec<MantType> = digits_le.to_vec();
    while mantissa.len() < ps.cdigit {
        mantissa.insert(0, 0);
    }

    let mut result = Number::new(ps.sign, ps.exp, mantissa);
    strip_zeroes(&mut result, precision);
    Ok(result)
}

/// Classify a character into a state-machine terminal.
const fn classify_char(c: char, radix: u32) -> usize {
    match c {
        '-' | '+' => SG,
        '.' => DP,
        '0' => ZR,
        '^' => EX,
        'e' if radix == 10 => EX,
        _ => NZ,
    }
}

/// Accumulators used during string-to-number parsing.
struct ParseState {
    sign: i32,
    exp: i32,
    exp_sign: i32,
    exp_value: i32,
    cdigit: usize,
}

/// Process a single state transition, updating accumulators.
/// Returns the (possibly updated) state, or an error.
fn process_state(
    mut state: u8,
    cur_char: char,
    radix: u32,
    ps: &mut ParseState,
    mant: &mut [u32],
    mant_idx: &mut usize,
) -> CalcResult<u8> {
    match state {
        MANTS => ps.sign = if cur_char == '-' { -1 } else { 1 },
        EXPS | EXPSZ => ps.exp_sign = if cur_char == '-' { -1 } else { 1 },
        EXPD | EXPDZ => {
            state = store_exp_digit(state, cur_char, radix, &mut ps.exp_value);
        }
        LD => {
            ps.exp += 1;
            state = store_mantissa_digit(
                state, cur_char, radix, &mut ps.exp, &mut ps.cdigit, mant, mant_idx,
            );
        }
        DD => {
            state = store_mantissa_digit(
                state, cur_char, radix, &mut ps.exp, &mut ps.cdigit, mant, mant_idx,
            );
        }
        DZ => ps.exp -= 1,
        ERR => return Err(CalcError::NoResult),
        _ => {} // LZ, LZDP, DDP, EXPB, etc.
    }
    Ok(state)
}

/// Store an exponent digit, returning ERR if invalid.
fn store_exp_digit(state: u8, c: char, radix: u32, exp_value: &mut i32) -> u8 {
    let nc = normalize_char_digit(c, radix);
    match digit_value(nc) {
        Some(v) if v < radix => {
            *exp_value = *exp_value * radix as i32 + v as i32;
            state
        }
        _ => ERR,
    }
}

/// Store a mantissa digit, returning ERR if invalid.
fn store_mantissa_digit(
    state: u8,
    c: char,
    radix: u32,
    exp: &mut i32,
    cdigit: &mut usize,
    mant: &mut [u32],
    mant_idx: &mut usize,
) -> u8 {
    let nc = normalize_char_digit(c, radix);
    match digit_value(nc) {
        Some(v) if v < radix => {
            *mant_idx -= 1;
            mant[*mant_idx] = v;
            *exp -= 1;
            *cdigit += 1;
            state
        }
        _ => ERR,
    }
}

/// Strip trailing zeros from a Number's mantissa (least significant end).
/// Port of C++ `stripzeroesnum`.
fn strip_zeroes(num: &mut Number, starting: i32) {
    // If starting is negative, don't strip anything (matches C++ signed comparison).
    if starting <= 0 {
        return;
    }
    let cdigit = num.mantissa.len();
    let limit = if cdigit > starting as usize {
        starting as usize
    } else {
        cdigit
    };

    let mut zeros = 0;
    while zeros < limit && num.mantissa[zeros] == 0 {
        zeros += 1;
    }

    if zeros > 0 && num.mantissa.len() > zeros {
        num.mantissa = num.mantissa[zeros..].to_vec();
        num.exp += zeros as i32;
    } else if zeros > 0 && num.mantissa.len() == zeros {
        *num = Number::zero();
    }
}

/// Strip zeroes and return whether stripping occurred.
fn strip_zeroes_check(num: &mut Number, starting: i32) -> bool {
    let before_len = num.mantissa.len();
    let before_exp = num.exp;
    strip_zeroes(num, starting);
    num.mantissa.len() != before_len || num.exp != before_exp
}

// ---------------------------------------------------------------------------
// StringToRat
// ---------------------------------------------------------------------------

/// Parse mantissa and exponent strings into a Rational.
///
/// Port of C++ `StringToRat`.
pub fn string_to_rat(
    mantissa_is_negative: bool,
    mantissa: &str,
    exponent_is_negative: bool,
    exponent: &str,
    radix: u32,
    precision: i32,
) -> CalcResult<Rational> {
    let mut result_rat = if mantissa.is_empty() {
        if exponent.is_empty() {
            Rational::zero()
        } else {
            Rational::one()
        }
    } else {
        let num_mant = string_to_number(mantissa, radix, precision)?;
        num_to_rat(&num_mant, radix)
    };

    // Deal with exponent
    let expt: i32 = if exponent.is_empty() {
        0
    } else {
        let num_exp = string_to_number(exponent, radix, precision)?;
        num_exp.to_i32(radix).ok_or(CalcError::InvalidRange)?
    };

    // Convert exponent to rational multiplier: radix^|expt|
    if expt != 0 || exponent_is_negative {
        let radix_num = Number::from_u32(radix, BASEX);
        let pow_num = num_pow_i32_x(&radix_num, expt.abs());
        let pow_rat = Rational::new(pow_num, Number::from_i32(1, BASEX));

        if exponent_is_negative {
            result_rat = div_rat_simple(&result_rat, &pow_rat)?;
        } else {
            result_rat = mul_rat_simple(&result_rat, &pow_rat);
        }
    }

    if mantissa_is_negative {
        result_rat.p_mut().sign *= -1;
    }

    Ok(result_rat)
}

use super::arithmetic::mul_num_x;

/// Simple rational multiplication: (a.p * b.p) / (a.q * b.q).
fn mul_rat_simple(a: &Rational, b: &Rational) -> Rational {
    let new_p = mul_num_x(a.p(), b.p());
    let new_q = mul_num_x(a.q(), b.q());
    Rational::new(new_p, new_q)
}

/// Simple rational division: (a.p * b.q) / (a.q * b.p).
fn div_rat_simple(a: &Rational, b: &Rational) -> CalcResult<Rational> {
    if b.p().is_zero() {
        return Err(CalcError::DivideByZero);
    }
    let new_p = mul_num_x(a.p(), b.q());
    let new_q = mul_num_x(a.q(), b.p());
    Ok(Rational::new(new_p, new_q))
}

// ---------------------------------------------------------------------------
// NumberToString
// ---------------------------------------------------------------------------

/// Result of rounding: either the rounded number is ready, or it needs
/// reformatting (structure changed after zero-stripping).
enum RoundResult {
    /// Number is ready for formatting.
    Ready(Number),
    /// Number changed structure; recurse with this number + old format.
    NeedsReformat(Number),
}

/// Convert a Number (in a given radix) to a string representation.
///
/// Port of C++ `NumberToString`.
pub fn number_to_string(
    num: &Number,
    format: NumberFormat,
    radix: u32,
    precision: i32,
) -> CalcResult<String> {
    let mut pnum = num.clone();
    strip_zeroes(&mut pnum, precision + 2);

    let old_format = format;
    let mut format = format;
    let exponent_raw = pnum.exp + pnum.cdigit();

    if exponent_raw > precision && format == NumberFormat::Float {
        format = NumberFormat::Scientific;
    }

    let length = pnum.cdigit().min(precision);

    // Apply rounding if needed.
    match apply_rounding(pnum, &mut format, radix, precision, length, exponent_raw)? {
        RoundResult::Ready(rounded) => pnum = rounded,
        RoundResult::NeedsReformat(rounded) => {
            // C++ recurses with the modified (rounded) pnum and the old format.
            return number_to_string(&rounded, old_format, radix, precision);
        }
    }

    Ok(format_number(&pnum, format, radix, precision))
}

/// Apply rounding to a number before formatting.
fn apply_rounding(
    mut pnum: Number,
    format: &mut NumberFormat,
    radix: u32,
    precision: i32,
    mut length: i32,
    exponent: i32,
) -> CalcResult<RoundResult> {
    let need_round = !pnum.is_zero()
        && (pnum.cdigit() >= precision
            || (length - exponent > precision && exponent >= -MAX_ZEROS_AFTER_DECIMAL));

    if !need_round {
        strip_zeroes(&mut pnum, precision);
        return Ok(RoundResult::Ready(pnum));
    }

    let mut round = Number::from_i32(radix as i32, radix);
    let two = Number::from_i32(2, radix);
    round = div_num(&round, &two, radix, precision)?;

    if exponent > 0 || *format == NumberFormat::Float {
        round.exp = pnum.exp + pnum.cdigit() - round.cdigit() - precision;
    } else {
        round.exp = pnum.exp + pnum.cdigit() - round.cdigit() - precision - exponent;
        length = precision + exponent;
    }

    if *format == NumberFormat::Float {
        if (length - exponent > precision) || (exponent > precision + 3) {
            if exponent >= -MAX_ZEROS_AFTER_DECIMAL {
                round.exp -= exponent;
            } else {
                *format = NumberFormat::Scientific;
            }
        } else if length + exponent.abs() < precision {
            round.exp -= exponent;
        }
    }

    round.sign = pnum.sign;
    pnum = add_num(&pnum, &round, radix);

    let offset = (pnum.cdigit() + pnum.exp) - (round.cdigit() + round.exp);
    if strip_zeroes_check(&mut pnum, offset) {
        // Rounding changed structure; return the rounded number for re-formatting.
        return Ok(RoundResult::NeedsReformat(pnum));
    }

    Ok(RoundResult::Ready(pnum))
}

/// Build the formatted string from a rounded Number.
fn format_number(pnum: &Number, format: NumberFormat, radix: u32, precision: i32) -> String {
    let length = pnum.cdigit().min(precision);
    let exponent_raw = pnum.exp + pnum.cdigit();

    let (use_sci_form, eout, mut exponent) = compute_exponent(format, exponent_raw);

    let mut result = String::new();

    if pnum.sign == -1 && length > 0 {
        result.push('-');
    }

    if exponent <= 0 && !use_sci_form {
        result.push('0');
        result.push(DECIMAL_SEPARATOR);
    }

    while exponent < 0 {
        result.push('0');
        exponent += 1;
    }

    // Emit digits from MSD to LSD
    let mut remaining = length;
    let mut digit_idx = pnum.mantissa.len() as i32 - 1;
    while remaining > 0 && digit_idx >= 0 {
        exponent -= 1;
        let d = pnum.mantissa[digit_idx as usize];
        result.push(if (d as usize) < DIGITS.len() {
            DIGITS[d as usize] as char
        } else {
            '?'
        });
        digit_idx -= 1;
        remaining -= 1;

        if exponent == 0 {
            result.push(DECIMAL_SEPARATOR);
        }
    }

    while exponent > 0 {
        result.push('0');
        exponent -= 1;
        if exponent == 0 {
            result.push(DECIMAL_SEPARATOR);
        }
    }

    if use_sci_form {
        append_exponent(&mut result, eout, radix);
    }

    if result.ends_with(DECIMAL_SEPARATOR) {
        result.pop();
    }

    result
}

/// Compute display exponent and digit positioning for sci/eng notation.
fn compute_exponent(format: NumberFormat, exponent_raw: i32) -> (bool, i32, i32) {
    let mut eout = exponent_raw - 1;
    let mut exponent = exponent_raw;

    if format == NumberFormat::Scientific || format == NumberFormat::Engineering {
        if eout != 0 {
            if format == NumberFormat::Engineering {
                exponent = eout % 3;
                eout -= exponent;
                exponent += 1;
                if exponent < 0 {
                    exponent += 3;
                    eout -= 3;
                }
            } else {
                exponent = 1;
            }
        }
        (true, eout, exponent)
    } else {
        (false, 0, exponent)
    }
}

/// Append scientific/engineering exponent notation to a result string.
fn append_exponent(result: &mut String, eout: i32, radix: u32) {
    result.push(if radix == 10 { 'e' } else { '^' });
    result.push(if eout < 0 { '-' } else { '+' });
    let mut exp_string = String::new();
    let mut ev = eout.unsigned_abs();
    loop {
        let d = (ev % radix) as usize;
        if d < DIGITS.len() {
            exp_string.push(DIGITS[d] as char);
        }
        ev /= radix;
        if ev == 0 {
            break;
        }
    }
    result.extend(exp_string.chars().rev());
}

// ---------------------------------------------------------------------------
// RatToNumber
// ---------------------------------------------------------------------------

/// Convert a Rational to a Number in the given radix by dividing p by q.
///
/// Port of C++ `RatToNumber`.
pub fn rat_to_number(rat: &Rational, radix: u32, precision: i32) -> CalcResult<Number> {
    let scale_by = rat.p().exp.min(rat.q().exp).max(0);

    let mut p_clone = rat.p().clone();
    let mut q_clone = rat.q().clone();
    p_clone.exp -= scale_by;
    q_clone.exp -= scale_by;

    let p_radix = num_to_radix(&p_clone, radix, precision);
    let q_radix = num_to_radix(&q_clone, radix, precision);

    div_num(&p_radix, &q_radix, radix, precision)
}

// ---------------------------------------------------------------------------
// RatToString
// ---------------------------------------------------------------------------

/// Convert a Rational to a string representation.
///
/// Port of C++ `RatToString`.
pub fn rat_to_string(
    rat: &Rational,
    format: NumberFormat,
    radix: u32,
    precision: i32,
) -> CalcResult<String> {
    let p = rat_to_number(rat, radix, precision)?;
    number_to_string(&p, format, radix, precision)
}

// ---------------------------------------------------------------------------
// flatrat
// ---------------------------------------------------------------------------

/// Flatten a rational by converting to a Number and back.
///
/// Port of C++ `flatrat`.
pub fn flat_rat(rat: &Rational, radix: u32, precision: i32) -> CalcResult<Rational> {
    let pnum = rat_to_number(rat, radix, precision)?;
    Ok(num_to_rat(&pnum, radix))
}

// ---------------------------------------------------------------------------
// numtoi32 — method on Number
// ---------------------------------------------------------------------------

impl Number {
    /// Convert this Number to an i32 using Horner's method.
    ///
    /// Port of C++ `numtoi32`.
    pub fn to_i32_horner(&self, radix: u32) -> i32 {
        let mut result: i64 = 0;
        let radix_i64 = i64::from(radix);

        let mut idx = self.mantissa.len() as i32 - 1;
        let mut remaining = self.cdigit();
        while remaining > 0 && remaining + self.exp > 0 {
            result = result * radix_i64 + i64::from(self.mantissa[idx as usize]);
            idx -= 1;
            remaining -= 1;
        }

        let mut expt = self.exp;
        while expt > 0 {
            result *= radix_i64;
            expt -= 1;
        }

        result *= i64::from(self.sign);
        result as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- string_to_number ---

    #[test]
    fn test_string_to_number_simple() {
        let n = string_to_number("123", 10, 32).unwrap();
        assert_eq!(n.sign, 1);
        assert_eq!(n.to_i32(10), Some(123));
    }

    #[test]
    fn test_string_to_number_negative() {
        let n = string_to_number("-42", 10, 32).unwrap();
        assert_eq!(n.sign, -1);
        assert_eq!(n.to_i32(10).unwrap().abs(), 42);
    }

    #[test]
    fn test_string_to_number_decimal() {
        let n = string_to_number("3.14", 10, 32).unwrap();
        assert_eq!(n.sign, 1);
        assert!(!n.is_zero());
    }

    #[test]
    fn test_string_to_number_hex() {
        let n = string_to_number("FF", 16, 32).unwrap();
        assert_eq!(n.to_i32(16), Some(255));
    }

    #[test]
    fn test_string_to_number_binary() {
        let n = string_to_number("1010", 2, 32).unwrap();
        assert_eq!(n.to_i32(2), Some(10));
    }

    #[test]
    fn test_string_to_number_zero() {
        let n = string_to_number("0", 10, 32).unwrap();
        assert!(n.is_zero() || n.to_i32(10) == Some(0));
    }

    #[test]
    fn test_string_to_number_leading_zeros() {
        let n = string_to_number("007", 10, 32).unwrap();
        assert_eq!(n.to_i32(10), Some(7));
    }

    // --- number_to_string ---

    #[test]
    fn test_number_to_string_simple() {
        let n = Number::from_i32(123, 10);
        let s = number_to_string(&n, NumberFormat::Float, 10, 32).unwrap();
        assert_eq!(s, "123");
    }

    #[test]
    fn test_number_to_string_negative() {
        let n = Number::from_i32(-42, 10);
        let s = number_to_string(&n, NumberFormat::Float, 10, 32).unwrap();
        assert_eq!(s, "-42");
    }

    #[test]
    fn test_number_to_string_zero() {
        let n = Number::zero();
        let s = number_to_string(&n, NumberFormat::Float, 10, 32).unwrap();
        assert!(s.starts_with('0'));
    }

    // --- roundtrip ---

    #[test]
    fn test_roundtrip_integer() {
        let n = string_to_number("456", 10, 32).unwrap();
        let s = number_to_string(&n, NumberFormat::Float, 10, 32).unwrap();
        assert_eq!(s, "456");
    }

    #[test]
    fn test_roundtrip_hex() {
        let n = string_to_number("1A", 16, 32).unwrap();
        let s = number_to_string(&n, NumberFormat::Float, 16, 32).unwrap();
        assert_eq!(s, "1A");
    }

    // --- string_to_rat ---

    #[test]
    fn test_string_to_rat_integer() {
        let r = string_to_rat(false, "42", false, "", 10, 32).unwrap();
        assert!(!r.is_zero());
        assert_eq!(r.p().to_i32(BASEX), Some(42));
        assert_eq!(r.q().to_i32(BASEX), Some(1));
    }

    #[test]
    fn test_string_to_rat_decimal() {
        let r = string_to_rat(false, "3.14", false, "", 10, 32).unwrap();
        assert!(!r.is_zero());
    }

    #[test]
    fn test_string_to_rat_negative() {
        let r = string_to_rat(true, "5", false, "", 10, 32).unwrap();
        assert_eq!(r.sign(), -1);
    }

    #[test]
    fn test_string_to_rat_with_exponent() {
        let r = string_to_rat(false, "1", false, "2", 10, 32).unwrap();
        assert!(!r.is_zero());
    }

    // --- rat_to_string ---

    #[test]
    fn test_rat_to_string_integer() {
        let r = Rational::from_i32(42);
        let s = rat_to_string(&r, NumberFormat::Float, 10, 32).unwrap();
        assert_eq!(s, "42");
    }

    #[test]
    fn test_rat_to_string_negative() {
        let r = Rational::from_i32(-7);
        let s = rat_to_string(&r, NumberFormat::Float, 10, 32).unwrap();
        assert_eq!(s, "-7");
    }

    #[test]
    fn test_rat_to_string_zero() {
        let r = Rational::zero();
        let s = rat_to_string(&r, NumberFormat::Float, 10, 32).unwrap();
        assert!(s.starts_with('0'));
    }

    // --- to_i32_horner ---

    #[test]
    fn test_to_i32_horner() {
        let n = Number::from_i32(42, 10);
        assert_eq!(n.to_i32_horner(10), 42);
        let n = Number::from_i32(-100, 10);
        assert_eq!(n.to_i32_horner(10), -100);
    }

    // --- flat_rat ---

    #[test]
    fn test_flat_rat_simple() {
        let r = Rational::from_i32(5);
        let flat = flat_rat(&r, 10, 32).unwrap();
        assert!(!flat.is_zero());
    }
}
