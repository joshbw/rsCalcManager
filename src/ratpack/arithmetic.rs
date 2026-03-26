// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Basic arithmetic operations on Numbers and Rationals.
//!
//! Ports the arithmetic from C++ ratpack: addnum, mulnum, divnum, etc.

use crate::error::{CalcError, CalcResult};
use crate::types::{MantType, BASEX, BASEX_PWR};

use super::number::Number;
use super::rational::Rational;

// ---------------------------------------------------------------------------
// addnum — Port of C++ addnum / _addnum (num.cpp)
//
// Uses radix-complement for opposite-sign addition, exactly matching the
// C++ algorithm: complement the negative operand's digits as
// (radix-1 - digit), add with initial carry=1, and re-complement if the
// final carry is 0 (meaning the result is negative).
// ---------------------------------------------------------------------------

/// Add two numbers in the given radix: returns a + b.
/// Port of C++ `addnum`.
pub fn add_num(a: &Number, b: &Number, radix: u32) -> Number {
    // If b is zero, return a.
    if zer_num(b) {
        return a.clone();
    }
    // If a is zero, return b.
    if zer_num(a) {
        return b.clone();
    }

    // Both non-zero — delegate to the inner algorithm.
    _add_num(a, b, radix)
}

fn _add_num(a: &Number, b: &Number, radix: u32) -> Number {
    let a_top = a.cdigit() + a.exp; // cdigit + exp
    let b_top = b.cdigit() + b.exp;
    let c_exp = a.exp.min(b.exp);
    let cdigits_total = a_top.max(b_top) - c_exp;

    let mut result = vec![0u32; (cdigits_total + 1) as usize]; // +1 for potential carry
    let c_cdigit = cdigits_total; // will adjust later

    // Determine complement flags for different-sign addition
    let mut cy: MantType = 0;
    let mut fcompla = false;
    let mut fcomplb = false;
    if a.sign != b.sign {
        cy = 1;
        fcompla = a.sign == -1;
        fcomplb = b.sign == -1;
    }

    let mut pcha_idx: usize = 0; // index into a.mantissa
    let mut pchb_idx: usize = 0; // index into b.mantissa
    let mut pchc_idx: usize = 0; // index into result
    let mut mexp = c_exp;

    let a_cdigit = a.cdigit();
    let b_cdigit = b.cdigit();
    let radix_m1 = radix - 1;

    for _ in 0..cdigits_total {
        // Get digit from a with padding
        let mut da: MantType = if mexp >= a.exp
            && (cdigits_total - (pchc_idx as i32) + a.exp - c_exp) > (c_cdigit - a_cdigit)
        {
            let d = a.mantissa[pcha_idx];
            pcha_idx += 1;
            d
        } else {
            0
        };

        // Get digit from b with padding
        let mut db: MantType = if mexp >= b.exp
            && (cdigits_total - (pchc_idx as i32) + b.exp - c_exp) > (c_cdigit - b_cdigit)
        {
            let d = b.mantissa[pchb_idx];
            pchb_idx += 1;
            d
        } else {
            0
        };

        // Complement
        if fcompla {
            da = radix_m1 - da;
        }
        if fcomplb {
            db = radix_m1 - db;
        }

        // Add with carry
        cy = da + db + cy;
        result[pchc_idx] = cy % radix;
        cy /= radix;
        pchc_idx += 1;
        mexp += 1;
    }

    let mut actual_cdigit = c_cdigit;
    let sign;

    // Handle carry from the last sum
    if !(fcompla || fcomplb) {
        // Same sign addition
        if cy != 0 {
            result[pchc_idx] = cy;
            actual_cdigit += 1;
        }
        sign = a.sign;
    } else {
        // Different sign (complement) addition
        if cy != 0 {
            sign = 1;
        } else {
            // Overflow/underflow: re-complement all digits
            sign = -1;
            let mut recomp_cy: MantType = 1;
            for digit in result.iter_mut().take(c_cdigit as usize) {
                recomp_cy = radix_m1 - *digit + recomp_cy;
                *digit = recomp_cy % radix;
                recomp_cy /= radix;
            }
        }
    }

    // Trim to actual_cdigit
    result.truncate(actual_cdigit as usize);

    // Remove leading zeros (digits are in increasing significance order)
    while result.len() > 1 && result.last() == Some(&0) {
        result.pop();
    }

    Number::new(sign, c_exp, result)
}

// ---------------------------------------------------------------------------
// mulnum — Port of C++ mulnum / _mulnum (num.cpp)
// ---------------------------------------------------------------------------

/// Multiply two numbers in the given radix: returns a * b.
/// Port of C++ `mulnum`.
pub fn mul_num(a: &Number, b: &Number, radix: u32) -> Number {
    // If b is one (mant=[1], exp=0), just adjust sign
    if b.mantissa.len() == 1 && b.mantissa[0] == 1 && b.exp == 0 {
        let mut r = a.clone();
        r.sign *= b.sign;
        return r;
    }
    // If a is one, copy b and adjust sign
    if a.mantissa.len() == 1 && a.mantissa[0] == 1 && a.exp == 0 {
        let mut r = b.clone();
        r.sign *= a.sign;
        return r;
    }

    _mul_num(a, b, radix)
}

fn _mul_num(a: &Number, b: &Number, radix: u32) -> Number {
    let a_cdigit = a.mantissa.len();
    let b_cdigit = b.mantissa.len();
    let c_cdigit_init = a_cdigit + b_cdigit - 1;

    let sign = a.sign * b.sign;
    let exp = a.exp + b.exp;

    // Allocate result with one extra slot for potential overflow
    let mut c_mant = vec![0u32; c_cdigit_init + 1];
    let mut c_cdigit = c_cdigit_init;
    let radix_u64 = u64::from(radix);

    for ia in 0..a_cdigit {
        let da = u64::from(a.mantissa[ia]);
        let mut c_offset = ia; // pchcoffset

        for ib in 0..b_cdigit {
            let mut cy: u64 = 0;
            let mut mcy: u64 = da * u64::from(b.mantissa[ib]);

            if mcy != 0 && ib == b_cdigit - 1 && ia == a_cdigit - 1 {
                c_cdigit = c_cdigit_init + 1;
            }

            let mut ic: usize = 0;
            while mcy != 0 || cy != 0 {
                cy += u64::from(c_mant[c_offset + ic]) + (mcy % radix_u64);
                c_mant[c_offset + ic] = (cy % radix_u64) as MantType;
                ic += 1;
                mcy /= radix_u64;
                cy /= radix_u64;
            }

            c_offset += 1;
        }
    }

    // Trim to c_cdigit
    c_mant.truncate(c_cdigit);

    // Strip leading zeros
    while c_mant.len() > 1 && c_mant.last() == Some(&0) {
        c_mant.pop();
    }

    Number::new(sign, exp, c_mant)
}

// ---------------------------------------------------------------------------
// mulnumx — Port of C++ mulnumx / _mulnumx (basex.cpp)
//
// Identical structure to mulnum but uses bit-shift and mask for BASEX radix
// since BASEX = 2^31.
// ---------------------------------------------------------------------------

/// Multiply two numbers in BASEX (internal radix).
/// Port of C++ `mulnumx` / `_mulnumx`.
pub fn mul_num_x(a: &Number, b: &Number) -> Number {
    // Short-circuit for one
    if b.mantissa.len() == 1 && b.mantissa[0] == 1 && b.exp == 0 {
        let mut r = a.clone();
        r.sign *= b.sign;
        return r;
    }
    if a.mantissa.len() == 1 && a.mantissa[0] == 1 && a.exp == 0 {
        let mut r = b.clone();
        r.sign *= a.sign;
        return r;
    }

    _mul_num_x(a, b)
}

fn _mul_num_x(a: &Number, b: &Number) -> Number {
    let a_cdigit = a.mantissa.len();
    let b_cdigit = b.mantissa.len();
    let c_cdigit_init = a_cdigit + b_cdigit - 1;

    let sign = a.sign * b.sign;
    let exp = a.exp + b.exp;
    let mask = u64::from(!BASEX); // lower 31 bits mask

    let mut c_mant = vec![0u32; c_cdigit_init + 1];
    let mut c_cdigit = c_cdigit_init;

    for ia in 0..a_cdigit {
        let da = u64::from(a.mantissa[ia]);
        let mut c_offset = ia;

        for ib in 0..b_cdigit {
            let mut cy: u64 = 0;
            let mut mcy: u64 = da * u64::from(b.mantissa[ib]);

            if mcy != 0 && ib == b_cdigit - 1 && ia == a_cdigit - 1 {
                c_cdigit = c_cdigit_init + 1;
            }

            let mut ic: usize = 0;
            while mcy != 0 || cy != 0 {
                // Use bit-ops for BASEX: low 31 bits via mask, shift by BASEXPWR
                cy += u64::from(c_mant[c_offset + ic]) + (mcy & mask);
                c_mant[c_offset + ic] = (cy & mask) as MantType;
                ic += 1;
                mcy >>= BASEX_PWR;
                cy >>= BASEX_PWR;
            }

            c_offset += 1;
        }
    }

    c_mant.truncate(c_cdigit);
    while c_mant.len() > 1 && c_mant.last() == Some(&0) {
        c_mant.pop();
    }

    Number::new(sign, exp, c_mant)
}

// ---------------------------------------------------------------------------
// divnum — Port of C++ divnum / _divnum (num.cpp)
//
// Long division using a pre-built table of divisor multiples [0..radix-1].
// For each quotient digit, walks the table from (radix-1) downward to find
// the largest multiple ≤ remainder.
// ---------------------------------------------------------------------------

/// Divide number a by b in the given radix with specified precision.
/// Port of C++ `divnum`.
pub fn div_num(a: &Number, b: &Number, radix: u32, precision: i32) -> CalcResult<Number> {
    if zer_num(b) {
        return Err(CalcError::DivideByZero);
    }
    if zer_num(a) {
        return Ok(Number::zero());
    }

    // Short-circuit: if b is 1 (single digit, exp 0), just adjust sign
    if b.mantissa.len() == 1 && b.mantissa[0] == 1 && b.exp == 0 {
        let mut r = a.clone();
        r.sign *= b.sign;
        return Ok(r);
    }

    Ok(_div_num(a, b, radix, precision))
}

fn _div_num(a: &Number, b: &Number, radix: u32, precision: i32) -> Number {
    let mut thismax = precision + 2;
    if thismax < a.cdigit() {
        thismax = a.cdigit();
    }
    if thismax < b.cdigit() {
        thismax = b.cdigit();
    }

    let sign = a.sign * b.sign;
    let c_exp_init = (a.cdigit() + a.exp) - (b.cdigit() + b.exp) + 1;

    let mut c_mant = vec![0u32; (thismax + 1) as usize];

    // Set up remainder
    let mut rem = a.clone();
    rem.exp = b.cdigit() + b.exp - rem.cdigit();

    // Temporary divisor with a's sign (for subtraction to work)
    let mut tmp_b = b.clone();
    tmp_b.sign = a.sign;

    // Build multiplication table: table[i] represents tmp_b * (radix-1-i)
    // Walking from front: table[0] = tmp_b*(radix-1), table[1] = tmp_b*(radix-2), ...
    // This matches the C++ which iterates the list from front (largest) to back.
    let zero_num = Number::from_i32(0, radix);
    let mut table_asc: Vec<Number> = vec![zero_num]; // table_asc[0] = 0
    for _i in 1..radix {
        let prev = table_asc.last().unwrap().clone();
        let next = add_num(&prev, &tmp_b, radix);
        table_asc.push(next);
    }
    // Reverse so index 0 is the largest multiple (radix-1)*b
    let table: Vec<Number> = table_asc.into_iter().rev().collect();

    let mut ptrc = thismax as usize;
    let mut cdigits: i32 = 0;

    while cdigits < thismax && !zer_num(&rem) {
        cdigits += 1;
        let mut digit = (radix - 1) as i32;

        // Walk table from largest to smallest to find the right multiple
        let mut found_multiple = None;
        for entry in &table {
            if !less_num(&rem, entry) || digit == 0 {
                found_multiple = Some(entry);
                break;
            }
            digit -= 1;
        }

        if digit != 0 {
            if let Some(multiple) = found_multiple {
                // Subtract: rem -= multiple
                let mut neg_mult = multiple.clone();
                neg_mult.sign *= -1;
                rem = add_num(&rem, &neg_mult, radix);
            }
        }

        rem.exp += 1;
        c_mant[ptrc] = digit as MantType;
        ptrc = ptrc.wrapping_sub(1);
    }

    // The digits were written from high index to low.
    // ptrc+1 is the start, cdigits is the count.
    let start = ptrc + 1;
    if cdigits == 0 {
        return Number::zero();
    }

    let actual_mant: Vec<u32> = c_mant[start..start + cdigits as usize].to_vec();
    let actual_exp = c_exp_init - cdigits;

    let mut result = Number::new(sign, actual_exp, actual_mant);

    // Strip leading zeros
    while result.mantissa.len() > 1 && result.mantissa.last() == Some(&0) {
        result.mantissa.pop();
    }

    result
}

// ---------------------------------------------------------------------------
// divnumx — Port of C++ divnumx / _divnumx (basex.cpp)
//
// Division in BASEX using binary search (doubling) for each quotient digit.
// Each "digit" in BASEX is a 31-bit value, so the quotient digit is found
// by doubling the divisor until it exceeds the remainder, then backing off.
// ---------------------------------------------------------------------------

/// Divide number a by b in BASEX with specified precision.
/// Port of C++ `divnumx`.
pub fn div_num_x(a: &Number, b: &Number, precision: i32) -> CalcResult<Number> {
    if zer_num(b) {
        return Err(CalcError::DivideByZero);
    }
    if zer_num(a) {
        return Ok(Number::zero());
    }

    // Short-circuit for b == 1
    if b.mantissa.len() == 1 && b.mantissa[0] == 1 && b.exp == 0 {
        let mut r = a.clone();
        r.sign *= b.sign;
        return Ok(r);
    }

    Ok(_div_num_x(a, b, precision))
}

fn _div_num_x(a: &Number, b: &Number, precision: i32) -> Number {
    // Use precision + 2 as a safe approximation for thismax
    let mut thismax = precision + 2;
    if thismax < a.cdigit() {
        thismax = a.cdigit();
    }
    if thismax < b.cdigit() {
        thismax = b.cdigit();
    }

    let sign = a.sign * b.sign;
    let c_exp_init = (a.cdigit() + a.exp) - (b.cdigit() + b.exp) + 1;

    let mut c_mant = vec![0u32; (thismax + 1) as usize];
    let mut ptrc = thismax as usize;

    let mut rem = a.clone();
    rem.sign = b.sign;
    rem.exp = b.cdigit() + b.exp - rem.cdigit();

    let mut cdigits: i32 = 0;

    while cdigits < thismax && !zer_num(&rem) {
        cdigits += 1;
        let mut digit: u32 = 0;
        c_mant[ptrc] = 0;

        while !less_num(&rem, b) {
            digit = 1;
            let mut tmp = b.clone();
            let mut lasttmp = Number::zero();

            while less_num(&tmp, &rem) {
                lasttmp = tmp.clone();
                tmp = add_num(&tmp, &tmp, BASEX);
                digit *= 2;
            }

            if less_num(&rem, &tmp) {
                // Went too far, back up
                digit /= 2;
                tmp = lasttmp;
            }

            tmp.sign *= -1;
            rem = add_num(&rem, &tmp, BASEX);
            c_mant[ptrc] |= digit;
        }
        rem.exp += 1;
        ptrc = ptrc.wrapping_sub(1);
    }

    let start = ptrc + 1;
    if cdigits == 0 {
        return Number::zero();
    }

    let actual_mant: Vec<u32> = c_mant[start..start + cdigits as usize].to_vec();
    let actual_exp = c_exp_init - cdigits;

    let mut result = Number::new(sign, actual_exp, actual_mant);

    while result.mantissa.len() > 1 && result.mantissa.last() == Some(&0) {
        result.mantissa.pop();
    }

    result
}

// ---------------------------------------------------------------------------
// remnum — Port of C++ remnum (num.cpp)
//
// Computes *pa %= b by repeatedly subtracting scaled powers-of-2 of b.
// ---------------------------------------------------------------------------

/// Compute remainder: a %= b in the given radix. Mutates a in place.
/// Port of C++ `remnum`.
pub fn rem_num(a: &mut Number, b: &Number, radix: u32) {
    // Once a < b, a is the remainder.
    while !less_num(a, b) {
        let mut tmp = b.clone();
        if less_num(&tmp, a) {
            // Start close to the right answer
            tmp.exp = a.cdigit() + a.exp - tmp.cdigit();
            if a.msd() <= tmp.msd() {
                tmp.exp -= 1;
            }
        }

        let mut lasttmp = Number::from_i32(0, radix);

        while less_num(&tmp, a) {
            lasttmp = tmp.clone();
            tmp = add_num(&tmp, &tmp, radix);
        }

        if less_num(a, &tmp) {
            // Too far, back up
            tmp = lasttmp;
        }

        // Subtract
        tmp.sign = -a.sign;
        *a = add_num(a, &tmp, radix);
    }
}

// ---------------------------------------------------------------------------
// numpowi32 / numpowi32x — Port of C++ numpowi32 (conv.cpp) and
// numpowi32x (basex.cpp)
//
// Binary exponentiation: decomposes power into sums of powers of 2.
// ---------------------------------------------------------------------------

/// Raise number to integer power in the given radix.
/// Port of C++ `numpowi32`.
pub fn num_pow_i32(root: &Number, power: i32, radix: u32, _precision: i32) -> Number {
    let mut lret = Number::from_i32(1, radix);
    let mut current = root.clone();
    let mut p = power;

    while p > 0 {
        if p & 1 != 0 {
            lret = mul_num(&lret, &current, radix);
        }
        current = mul_num(&current, &current, radix);
        // Note: C++ does TRIMNUM here for precision, but trimming is a
        // separate concern (support.rs) and not yet ported.
        p >>= 1;
    }

    lret
}

/// Raise number to integer power in BASEX.
/// Port of C++ `numpowi32x`.
pub fn num_pow_i32_x(root: &Number, power: i32) -> Number {
    let mut lret = Number::from_i32(1, BASEX);
    let mut current = root.clone();
    let mut p = power;

    while p > 0 {
        if p & 1 != 0 {
            lret = mul_num_x(&lret, &current);
        }
        current = mul_num_x(&current, &current);
        p >>= 1;
    }

    lret
}

// ---------------------------------------------------------------------------
// equnum, lessnum, zernum — Port of C++ equnum, lessnum, zernum (num.cpp)
//
// These are unsigned (magnitude-only) comparisons, matching the C++ semantics
// where lessnum compares abs(a) < abs(b).
// ---------------------------------------------------------------------------

/// Check if a number is zero.
/// Port of C++ `zernum`.
pub fn zer_num(a: &Number) -> bool {
    a.mantissa.iter().all(|&d| d == 0)
}

/// Check equality of two numbers (same magnitude and exponent alignment).
/// Port of C++ `equnum`.
///
/// Note: This is a structural equality check like the C++, not a value equality
/// (it does NOT account for different representations of the same value with
/// different exponents unless they align the same way).
pub fn equ_num(a: &Number, b: &Number) -> bool {
    let diff = (a.cdigit() + a.exp) - (b.cdigit() + b.exp);
    if diff != 0 {
        return false;
    }

    let a_cdigit = a.mantissa.len();
    let b_cdigit = b.mantissa.len();
    let cdigits = a_cdigit.max(b_cdigit);

    let mut pa = a_cdigit.wrapping_sub(1);
    let mut pb = b_cdigit.wrapping_sub(1);

    for i in (0..cdigits).rev() {
        let da = if i + 1 > cdigits - a_cdigit {
            let d = a.mantissa[pa];
            pa = pa.wrapping_sub(1);
            d
        } else {
            0
        };
        let db = if i + 1 > cdigits - b_cdigit {
            let d = b.mantissa[pb];
            pb = pb.wrapping_sub(1);
            d
        } else {
            0
        };
        if da != db {
            return false;
        }
    }

    true
}

/// Check if |a| < |b| (unsigned/magnitude comparison).
/// Port of C++ `lessnum`.
pub fn less_num(a: &Number, b: &Number) -> bool {
    let diff = (a.cdigit() + a.exp) - (b.cdigit() + b.exp);
    if diff < 0 {
        return true;
    }
    if diff > 0 {
        return false;
    }

    let a_cdigit = a.mantissa.len();
    let b_cdigit = b.mantissa.len();
    let cdigits = a_cdigit.max(b_cdigit);

    let mut pa = a_cdigit.wrapping_sub(1);
    let mut pb = b_cdigit.wrapping_sub(1);

    for i in (0..cdigits).rev() {
        let da: i64 = if i + 1 > cdigits - a_cdigit {
            let d = a.mantissa[pa];
            pa = pa.wrapping_sub(1);
            i64::from(d)
        } else {
            0
        };
        let db: i64 = if i + 1 > cdigits - b_cdigit {
            let d = b.mantissa[pb];
            pb = pb.wrapping_sub(1);
            i64::from(d)
        } else {
            0
        };
        let d = da - db;
        if d != 0 {
            return d < 0;
        }
    }

    false // equal
}

// ---------------------------------------------------------------------------
// Rational arithmetic — these use the Number primitives above.
// ---------------------------------------------------------------------------

/// Add two rationals with the given precision.
/// Port of C++ `addrat`.
pub fn add_rat(a: &Rational, b: &Rational, _precision: i32) -> Rational {
    // a.p/a.q + b.p/b.q = (a.p*b.q + b.p*a.q) / (a.q*b.q)
    let new_p1 = mul_num_x(a.p(), b.q());
    let new_p2 = mul_num_x(b.p(), a.q());
    let new_p = add_num(&new_p1, &new_p2, BASEX);
    let new_q = mul_num_x(a.q(), b.q());

    Rational::new(new_p, new_q)
}

/// Subtract two rationals: a - b.
/// Port of C++ `subrat`.
pub fn sub_rat(a: &Rational, b: &Rational, precision: i32) -> Rational {
    let neg_b = b.negate();
    add_rat(a, &neg_b, precision)
}

/// Multiply two rationals.
/// Port of C++ `mulrat`.
pub fn mul_rat(a: &Rational, b: &Rational, _precision: i32) -> Rational {
    let new_p = mul_num_x(a.p(), b.p());
    let new_q = mul_num_x(a.q(), b.q());
    Rational::new(new_p, new_q)
}

/// Divide two rationals: a / b.
/// Port of C++ `divrat`.
pub fn div_rat(a: &Rational, b: &Rational, _precision: i32) -> CalcResult<Rational> {
    if b.p().is_zero() {
        return Err(CalcError::DivideByZero);
    }

    // a/b = (a.p * b.q) / (a.q * b.p)
    let new_p = mul_num_x(a.p(), b.q());
    let new_q = mul_num_x(a.q(), b.p());
    Ok(Rational::new(new_p, new_q))
}

/// Remainder: a % b for rationals.
/// Port of C++ `remrat`.
pub fn rem_rat(a: &Rational, b: &Rational) -> CalcResult<Rational> {
    if b.p().is_zero() {
        return Err(CalcError::DivideByZero);
    }

    // For integer rationals: (a.p * b.q) % (b.p * a.q) / (a.q * b.q)
    let mut cross_a = mul_num_x(a.p(), b.q());
    let cross_b = mul_num_x(b.p(), a.q());
    let new_q = mul_num_x(a.q(), b.q());

    rem_num(&mut cross_a, &cross_b, BASEX);

    Ok(Rational::new(cross_a, new_q))
}

/// Raise rational to an i32 power.
/// Port of C++ `ratpowi32`.
pub fn rat_pow_i32(base: &Rational, power: i32, precision: i32) -> CalcResult<Rational> {
    if power == 0 {
        return Ok(Rational::one());
    }

    let (base, power) = if power < 0 {
        // Negative power: invert base
        if base.is_zero() {
            return Err(CalcError::DivideByZero);
        }
        let inverted = Rational::new(base.q().clone(), base.p().clone());
        (inverted, -power)
    } else {
        (base.clone(), power)
    };

    // Binary exponentiation using numpowi32x on p and q separately
    let new_p = num_pow_i32_x(base.p(), power);
    let new_q = num_pow_i32_x(base.q(), power);

    Ok(Rational::new(new_p, new_q))
}

/// Compare two rationals.
/// Returns true if a == b.
pub fn rat_equ(a: &Rational, b: &Rational, precision: i32) -> bool {
    let diff = sub_rat(a, b, precision);
    diff.is_zero()
}

/// Compare: a < b.
pub fn rat_lt(a: &Rational, b: &Rational, precision: i32) -> bool {
    let diff = sub_rat(a, b, precision);
    diff.sign() < 0 && !diff.is_zero()
}

/// Compare: a > b.
pub fn rat_gt(a: &Rational, b: &Rational, precision: i32) -> bool {
    let diff = sub_rat(a, b, precision);
    diff.sign() > 0 && !diff.is_zero()
}

/// Compare: a <= b.
pub fn rat_le(a: &Rational, b: &Rational, precision: i32) -> bool {
    !rat_gt(a, b, precision)
}

/// Compare: a >= b.
pub fn rat_ge(a: &Rational, b: &Rational, precision: i32) -> bool {
    !rat_lt(a, b, precision)
}

/// Compare: a != b.
pub fn rat_neq(a: &Rational, b: &Rational, precision: i32) -> bool {
    !rat_equ(a, b, precision)
}

// Implement standard ops for Rational using the arithmetic functions
impl std::ops::Add for &Rational {
    type Output = Rational;
    fn add(self, rhs: &Rational) -> Rational {
        add_rat(self, rhs, RATIONAL_PRECISION)
    }
}

impl std::ops::Sub for &Rational {
    type Output = Rational;
    fn sub(self, rhs: &Rational) -> Rational {
        sub_rat(self, rhs, RATIONAL_PRECISION)
    }
}

impl std::ops::Mul for &Rational {
    type Output = Rational;
    fn mul(self, rhs: &Rational) -> Rational {
        mul_rat(self, rhs, RATIONAL_PRECISION)
    }
}

impl std::ops::Div for &Rational {
    type Output = CalcResult<Rational>;
    fn div(self, rhs: &Rational) -> CalcResult<Rational> {
        div_rat(self, rhs, RATIONAL_PRECISION)
    }
}

use super::rational::RATIONAL_PRECISION;

#[cfg(test)]
mod tests {
    use super::*;

    // --- addnum tests ---

    #[test]
    fn test_add_num_same_sign() {
        // 3 + 4 = 7 in base 10
        let a = Number::from_i32(3, 10);
        let b = Number::from_i32(4, 10);
        let r = add_num(&a, &b, 10);
        assert_eq!(r.to_i32(10), Some(7));
    }

    #[test]
    fn test_add_num_different_signs_pos_result() {
        // 10 + (-3) = 7
        let a = Number::from_i32(10, 10);
        let b = Number::from_i32(-3, 10);
        let r = add_num(&a, &b, 10);
        assert_eq!(r.to_i32(10), Some(7));
    }

    #[test]
    fn test_add_num_different_signs_neg_result() {
        // 3 + (-10) = -7
        let a = Number::from_i32(3, 10);
        let b = Number::from_i32(-10, 10);
        let r = add_num(&a, &b, 10);
        assert_eq!(r.to_i32(10), Some(-7));
    }

    #[test]
    fn test_add_num_cancel_to_zero() {
        // 5 + (-5) = 0
        let a = Number::from_i32(5, 10);
        let b = Number::from_i32(-5, 10);
        let r = add_num(&a, &b, 10);
        assert!(zer_num(&r));
    }

    #[test]
    fn test_add_num_both_negative() {
        // -3 + (-4) = -7
        let a = Number::from_i32(-3, 10);
        let b = Number::from_i32(-4, 10);
        let r = add_num(&a, &b, 10);
        assert_eq!(r.to_i32(10), Some(-7));
    }

    #[test]
    fn test_add_num_with_carry() {
        // 99 + 1 = 100 in base 10
        let a = Number::from_i32(99, 10);
        let b = Number::from_i32(1, 10);
        let r = add_num(&a, &b, 10);
        assert_eq!(r.to_i32(10), Some(100));
    }

    #[test]
    fn test_add_num_basex() {
        let a = Number::from_i32(100, BASEX);
        let b = Number::from_i32(200, BASEX);
        let result = add_num(&a, &b, BASEX);
        assert_eq!(result.to_i32(BASEX), Some(300));
    }

    #[test]
    fn test_add_num_basex_diff_sign() {
        let a = Number::from_i32(200, BASEX);
        let b = Number::from_i32(-50, BASEX);
        let result = add_num(&a, &b, BASEX);
        assert_eq!(result.to_i32(BASEX), Some(150));
    }

    // --- mulnum tests ---

    #[test]
    fn test_mul_num_basic() {
        // 6 * 7 = 42
        let a = Number::from_i32(6, 10);
        let b = Number::from_i32(7, 10);
        let r = mul_num(&a, &b, 10);
        assert_eq!(r.to_i32(10), Some(42));
    }

    #[test]
    fn test_mul_num_by_one() {
        let a = Number::from_i32(42, 10);
        let one = Number::from_i32(1, 10);
        let r = mul_num(&a, &one, 10);
        assert_eq!(r.to_i32(10), Some(42));
    }

    #[test]
    fn test_mul_num_mixed_sign() {
        // -6 * 7 = -42
        let a = Number::from_i32(-6, 10);
        let b = Number::from_i32(7, 10);
        let r = mul_num(&a, &b, 10);
        assert_eq!(r.to_i32(10), Some(-42));
    }

    #[test]
    fn test_mul_num_x_basic() {
        let a = Number::from_i32(100, BASEX);
        let b = Number::from_i32(200, BASEX);
        let r = mul_num_x(&a, &b);
        assert_eq!(r.to_i32(BASEX), Some(20000));
    }

    // --- divnum tests ---

    #[test]
    fn test_div_num_basic() {
        // 42 / 6 = 7
        let a = Number::from_i32(42, 10);
        let b = Number::from_i32(6, 10);
        let r = div_num(&a, &b, 10, 10).unwrap();
        assert_eq!(r.to_i32(10), Some(7));
    }

    #[test]
    fn test_div_num_by_zero() {
        let a = Number::from_i32(42, 10);
        let b = Number::zero();
        assert!(div_num(&a, &b, 10, 10).is_err());
    }

    #[test]
    fn test_div_num_exact() {
        // 100 / 10 = 10
        let a = Number::from_i32(100, 10);
        let b = Number::from_i32(10, 10);
        let r = div_num(&a, &b, 10, 10).unwrap();
        assert_eq!(r.to_i32(10), Some(10));
    }

    #[test]
    fn test_div_num_x_basic() {
        // 20000 / 100 = 200
        let a = Number::from_i32(20000, BASEX);
        let b = Number::from_i32(100, BASEX);
        let r = div_num_x(&a, &b, 32).unwrap();
        assert_eq!(r.to_i32(BASEX), Some(200));
    }

    // --- remnum tests ---

    #[test]
    fn test_rem_num_basic() {
        // 17 % 5 = 2
        let mut a = Number::from_i32(17, 10);
        let b = Number::from_i32(5, 10);
        rem_num(&mut a, &b, 10);
        assert_eq!(a.to_i32(10), Some(2));
    }

    #[test]
    fn test_rem_num_exact() {
        // 15 % 5 = 0
        let mut a = Number::from_i32(15, 10);
        let b = Number::from_i32(5, 10);
        rem_num(&mut a, &b, 10);
        assert!(zer_num(&a));
    }

    // --- numpowi32 tests ---

    #[test]
    fn test_num_pow_i32_basic() {
        // 2^10 = 1024
        let base = Number::from_i32(2, 10);
        let r = num_pow_i32(&base, 10, 10, 32);
        assert_eq!(r.to_i32(10), Some(1024));
    }

    #[test]
    fn test_num_pow_i32_zero() {
        // 5^0 = 1
        let base = Number::from_i32(5, 10);
        let r = num_pow_i32(&base, 0, 10, 32);
        assert_eq!(r.to_i32(10), Some(1));
    }

    #[test]
    fn test_num_pow_i32_x() {
        // 3^5 = 243
        let base = Number::from_i32(3, BASEX);
        let r = num_pow_i32_x(&base, 5);
        assert_eq!(r.to_i32(BASEX), Some(243));
    }

    // --- comparison tests ---

    #[test]
    fn test_less_num() {
        let a = Number::from_i32(5, 10);
        let b = Number::from_i32(10, 10);
        assert!(less_num(&a, &b));
        assert!(!less_num(&b, &a));
        assert!(!less_num(&a, &a));
    }

    #[test]
    fn test_equ_num() {
        let a = Number::from_i32(42, 10);
        let b = Number::from_i32(42, 10);
        let c = Number::from_i32(43, 10);
        assert!(equ_num(&a, &b));
        assert!(!equ_num(&a, &c));
    }

    #[test]
    fn test_zer_num() {
        assert!(zer_num(&Number::zero()));
        assert!(!zer_num(&Number::from_i32(1, 10)));
    }

    // --- i32tonum round-trip test ---

    #[test]
    fn test_i32tonum_roundtrip() {
        for val in [-1000, -1, 0, 1, 42, 1000, i32::MAX / 2] {
            let n = Number::from_i32(val, 10);
            assert_eq!(n.to_i32(10), Some(val), "round-trip failed for {val}");
        }
    }

    #[test]
    fn test_i32tonum_roundtrip_basex() {
        for val in [-1000, -1, 0, 1, 42, 1000, 100000] {
            let n = Number::from_i32(val, BASEX);
            assert_eq!(n.to_i32(BASEX), Some(val), "BASEX round-trip failed for {val}");
        }
    }

    // --- Rational tests ---

    #[test]
    fn test_add_rat() {
        let a = Rational::from(3i32);
        let b = Rational::from(4i32);
        let result = &a + &b;
        assert!(!result.is_zero());
        assert_eq!(result.sign(), 1);
    }

    #[test]
    fn test_mul_rat() {
        let a = Rational::from(3i32);
        let b = Rational::from(4i32);
        let result = &a * &b;
        assert!(!result.is_zero());
    }

    #[test]
    fn test_div_rat_by_zero() {
        let a = Rational::from(3i32);
        let b = Rational::zero();
        assert!((&a / &b).is_err());
    }

    #[test]
    fn test_sub_rat() {
        let a = Rational::from(5i32);
        let b = Rational::from(5i32);
        let result = &a - &b;
        assert!(result.is_zero());
    }
}
