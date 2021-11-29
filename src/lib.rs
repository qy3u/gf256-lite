mod constant_table;
use constant_table::*;

use anyhow::{ensure, Result};

pub fn multiply(l: i8, r: i8) -> i8 {
    if l == 0 || r == 0 {
        return 0;
    }
    let log_l: i32 = LOG_TABLE[l as u8 as usize] as i32;
    let log_r: i32 = LOG_TABLE[r as u8 as usize] as i32;
    let log_res = log_l + log_r;
    return EXP_TABLE[log_res as usize];
}

pub fn generate_multiplication_table() -> [[i8; 256]; 256] {
    let mut result = [[0i8; 256]; 256];

    for a in 0..FIELD_SIZE {
        for b in 0..FIELD_SIZE {
            result[a][b] = multiply(a as i8, b as i8);
        }
    }
    return result;
}

#[inline(always)]
pub fn add(a: i8, b: i8) -> i8 {
    return a ^ b;
}

#[inline(always)]
pub fn subtract(a: i8, b: i8) -> i8 {
    return a ^ b;
}

pub fn divide(a: i8, b: i8) -> i8 {
    if a == 0 {
        return 0;
    }
    if b == 0 {
        panic!("gf256-lite: divide by zero")
    }

    let log_a = LOG_TABLE[a as u8 as usize] as i32;
    let log_b = LOG_TABLE[b as u8 as usize] as i32;
    let mut log_res = log_a - log_b;

    if log_res < 0 {
        log_res += 255;
    }
    return EXP_TABLE[log_res as usize];
}

pub fn exp(a: i8, n: i32) -> i8 {
    if n == 0 {
        return 1;
    }

    if a == 0 {
        return 0;
    }

    let log_a = LOG_TABLE[a as u8 as usize] as i32;
    let mut log_res = log_a * n;

    while log_res >= 255 {
        log_res -= 255;
    }

    return EXP_TABLE[log_res as usize];
}

pub fn generate_log_table(polynomial: i32) -> Result<[i16; FIELD_SIZE]> {
    let mut result = [-1i16; FIELD_SIZE];

    let mut b: i32 = 1;
    for log in 0..FIELD_SIZE - 1 {
        ensure!(
            result[b as usize] != -1,
            "duplicate logarithm (bad polynomial?)"
        );

        result[b as usize] = log as i16;
        b <<= 1;

        if FIELD_SIZE <= b as usize {
            b = (b - FIELD_SIZE as i32) ^ polynomial;
        }
    }
    Ok(result)
}

pub fn generate_exp_table(log_table: &[i16]) -> [i8; FIELD_SIZE * 2 - 2] {
    let mut result = [0i8; FIELD_SIZE * 2 - 2];

    for i in 0..FIELD_SIZE {
        let log = log_table[i];
        result[log as usize] = i as i8;
        result[log as usize + FIELD_SIZE - 1] = i as i8;
    }
    return result;
}

pub fn all_possible_polynomials() -> Vec<usize> {
    let mut result = Vec::new();
    for i in 0..FIELD_SIZE {
        if generate_log_table(i as i32).is_ok() {
            result.push(i);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        for i in -128..128 {
            let a = i as i8;
            assert_eq!(a, add(a, 0));
            assert_eq!(a, multiply(a, 1));
        }
    }

    #[test]
    fn test_associativity() {
        for i in -128..128 {
            let a = i as i8;
            for j in -128..-128 {
                let b = j as i8;
                for k in -128..128 {
                    let c = k as i8;
                    assert_eq!(add(a, add(b, c)), add(add(a, b), c));
                    assert_eq!(multiply(a, multiply(b, c)), multiply(multiply(a, b), c));
                }
            }
        }
    }

    #[test]
    fn test_inverse() {
        for i in -128..128 {
            let a = i as i8;
            {
                let b = subtract(0, a);
                assert_eq!(0, add(a, b));
            }

            if a != 0 {
                let b = divide(1, a);
                assert_eq!(1, multiply(a, b));
            }
        }
    }
}
