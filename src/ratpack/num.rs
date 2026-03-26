// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

//! Number manipulation support functions.
//! Port of C++ Ratpack/num.cpp and conv.cpp (gcd, i32factnum, i32prodnum).

use super::arithmetic::{less_num, mul_num, rem_num, zer_num};
use super::Number;
use crate::types::BASEX;

/// Compute GCD of two numbers using the Euclidean algorithm.
/// Port of C++ `gcd` from conv.cpp.
///
/// Both numbers are assumed to be in BASEX radix.
pub fn gcd(a: &Number, b: &Number) -> Number {
    if zer_num(a) {
        return b.clone();
    }
    if zer_num(b) {
        return a.clone();
    }

    let (mut larger, mut smaller) = if less_num(a, b) {
        (b.clone(), a.clone())
    } else {
        (a.clone(), b.clone())
    };

    while !zer_num(&smaller) {
        rem_num(&mut larger, &smaller, BASEX);
        // swap
        std::mem::swap(&mut larger, &mut smaller);
    }

    // GCD is always positive
    larger.sign = 1;
    larger
}

/// Compute the product start * (start+1) * ... * stop as a Number.
/// Port of C++ `i32prodnum` from conv.cpp.
///
/// Skips zero values in the range (matching C++ behaviour where `if (start)` guards the multiply).
pub fn i32_prod_num(start: i32, stop: i32, radix: u32) -> Number {
    let mut result = Number::from_i32(1, radix);
    let mut cur = start;
    while cur <= stop {
        if cur != 0 {
            let tmp = Number::from_i32(cur, radix);
            result = mul_num(&result, &tmp, radix);
        }
        cur += 1;
    }
    result
}

/// Compute factorial n! as a Number.
/// Port of C++ `i32factnum` from conv.cpp.
pub fn i32_fact_num(n: i32, radix: u32) -> Number {
    let mut result = Number::from_i32(1, radix);
    let mut k = n;
    while k > 0 {
        let tmp = Number::from_i32(k, radix);
        result = mul_num(&result, &tmp, radix);
        k -= 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcd_basic() {
        let a = Number::from_i32(12, BASEX);
        let b = Number::from_i32(8, BASEX);
        let g = gcd(&a, &b);
        assert_eq!(g.to_i32(BASEX), Some(4));
    }

    #[test]
    fn test_gcd_coprime() {
        let a = Number::from_i32(7, BASEX);
        let b = Number::from_i32(13, BASEX);
        let g = gcd(&a, &b);
        assert_eq!(g.to_i32(BASEX), Some(1));
    }

    #[test]
    fn test_gcd_zero() {
        let a = Number::from_i32(0, BASEX);
        let b = Number::from_i32(5, BASEX);
        let g = gcd(&a, &b);
        assert_eq!(g.to_i32(BASEX), Some(5));
    }

    #[test]
    fn test_i32_prod_num() {
        // 2*3*4*5 = 120
        let p = i32_prod_num(2, 5, 10);
        assert_eq!(p.to_i32(10), Some(120));
    }

    #[test]
    fn test_i32_fact_num() {
        // 5! = 120
        let f = i32_fact_num(5, 10);
        assert_eq!(f.to_i32(10), Some(120));
    }

    #[test]
    fn test_i32_fact_num_zero() {
        // 0! = 1
        let f = i32_fact_num(0, 10);
        assert_eq!(f.to_i32(10), Some(1));
    }
}
