use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use lazy_static::lazy_static;

lazy_static! {
    static ref LOG_TABLE: [u8; 256] = gen_log_table();
    static ref EXP_TABLE: [Galois; 256] = gen_exp_table();
}

const PRIMITIVE_POLYNOMIAL: usize = 0b100011101;
const FIELD_SIZE: usize = 1 << 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Galois(u8);

impl Galois {
    pub fn new(v: u8) -> Self {
        Galois(v)
    }

    pub fn zero() -> Self {
        Galois(0)
    }

    pub fn identity() -> Self {
        Galois(1)
    }

    pub fn inv(self) -> Self {
        Galois::identity() / self
    }

    pub fn exp(self, n: u32) -> Self {
        if n == 0 {
            return Galois::identity();
        }

        if self == Galois::zero() {
            return self;
        }

        let log_a = LOG_TABLE[self.0 as usize] as u32;
        let mut log_res = log_a * n;
        while log_res >= 255 {
            log_res -= 255;
        }

        EXP_TABLE[log_res as usize]
    }
}

impl Add for Galois {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Galois(self.0 ^ rhs.0)
    }
}

impl Sub for Galois {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Galois(self.0 ^ rhs.0)
    }
}

impl Mul for Galois {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        if self == Galois::zero() || rhs == Galois(0) {
            return Galois::zero();
        }

        let pow_l = LOG_TABLE[self.0 as usize] as usize;
        let pow_r = LOG_TABLE[rhs.0 as usize] as usize;

        let mut pow_mul = pow_l + pow_r;

        if pow_mul >= 255 {
            pow_mul -= 255;
        }

        EXP_TABLE[pow_mul]
    }
}

impl Div for Galois {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        if self == Galois::zero() {
            return Galois::zero();
        }

        assert_ne!(rhs, Galois::zero(), "divide by zero");

        let pow_l = LOG_TABLE[self.0 as usize] as isize;
        let pow_r = LOG_TABLE[rhs.0 as usize] as isize;

        let mut pow_div = pow_l - pow_r;
        if pow_div < 0 {
            pow_div += (FIELD_SIZE - 1) as isize;
        }

        assert!(pow_div >= 0);
        EXP_TABLE[pow_div as usize]
    }
}

impl AddAssign for Galois {
    fn add_assign(&mut self, rhs: Self) {
        *self = Galois(self.0 ^ rhs.0);
    }
}

impl SubAssign for Galois {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Galois(self.0 ^ rhs.0);
    }
}

impl MulAssign for Galois {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl DivAssign for Galois {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

impl fmt::Display for Galois {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u8> for Galois {
    fn from(v: u8) -> Self {
        Galois(v)
    }
}

impl From<Galois> for u8 {
    fn from(v: Galois) -> u8 {
        v.0
    }
}

fn gen_exp_table() -> [Galois; 256] {
    let mut exps = [Galois::zero(); 256];
    exps[0] = Galois(1); // x ^ 0 = 1

    // x^1 - x^254
    for i in 1..FIELD_SIZE - 1 {
        let mut elem = (exps[i - 1].0 as usize) << 1;

        if elem > u8::MAX as usize {
            elem ^= PRIMITIVE_POLYNOMIAL;
            assert!(elem <= u8::MAX as usize);
        }

        exps[i] = Galois(elem as u8);
    }

    exps
}

fn gen_log_table() -> [u8; 256] {
    let exp_tables = gen_exp_table();

    let mut logs = [0u8; 256];

    for i in 0..FIELD_SIZE - 1 {
        // exp[i] = v
        // log[v] = i
        logs[exp_tables[i].0 as usize] = i as u8;
    }

    logs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        for i in 0..u8::MAX {
            let a = Galois(i);
            assert_eq!(a, a + Galois::zero());
            assert_eq!(a, a * Galois(1));
        }
    }

    #[test]
    fn test_single() {
        let l = Galois(2);
        assert_eq!(l, l + Galois::zero());
    }

    #[test]
    fn test_associativity() {
        for i in 0..FIELD_SIZE {
            let a = Galois(i as u8);
            for j in 0..FIELD_SIZE {
                let b = Galois(j as u8);
                for k in 0..FIELD_SIZE {
                    let c = Galois(k as u8);
                    assert_eq!(a + (b + c), (a + b) + c);
                    assert_eq!(a * (b * c), (a * b) * c);
                }
            }
        }
    }

    #[test]
    fn test_inverse() {
        for i in 0..FIELD_SIZE {
            let a = Galois(i as u8);
            {
                let b = Galois::zero() - a;
                assert_eq!(Galois::zero(), a + b);
            }

            if a != Galois::zero() {
                let b = Galois::identity() / a;
                assert_eq!(Galois::identity(), a * b);
            }
        }
    }

    #[test]
    fn test_logs_eq() {
        let logs: [u8; 255] = [
            0, 1, 25, 2, 50, 26, 198, 3, 223, 51, 238, 27, 104, 199, 75, 4, 100, 224, 14, 52, 141,
            239, 129, 28, 193, 105, 248, 200, 8, 76, 113, 5, 138, 101, 47, 225, 36, 15, 33, 53,
            147, 142, 218, 240, 18, 130, 69, 29, 181, 194, 125, 106, 39, 249, 185, 201, 154, 9,
            120, 77, 228, 114, 166, 6, 191, 139, 98, 102, 221, 48, 253, 226, 152, 37, 179, 16, 145,
            34, 136, 54, 208, 148, 206, 143, 150, 219, 189, 241, 210, 19, 92, 131, 56, 70, 64, 30,
            66, 182, 163, 195, 72, 126, 110, 107, 58, 40, 84, 250, 133, 186, 61, 202, 94, 155, 159,
            10, 21, 121, 43, 78, 212, 229, 172, 115, 243, 167, 87, 7, 112, 192, 247, 140, 128, 99,
            13, 103, 74, 222, 237, 49, 197, 254, 24, 227, 165, 153, 119, 38, 184, 180, 124, 17, 68,
            146, 217, 35, 32, 137, 46, 55, 63, 209, 91, 149, 188, 207, 205, 144, 135, 151, 178,
            220, 252, 190, 97, 242, 86, 211, 171, 20, 42, 93, 158, 132, 60, 57, 83, 71, 109, 65,
            162, 31, 45, 67, 216, 183, 123, 164, 118, 196, 23, 73, 236, 127, 12, 111, 246, 108,
            161, 59, 82, 41, 157, 85, 170, 251, 96, 134, 177, 187, 204, 62, 90, 203, 89, 95, 176,
            156, 169, 160, 81, 11, 245, 22, 235, 122, 117, 44, 215, 79, 174, 213, 233, 230, 231,
            173, 232, 116, 214, 244, 234, 168, 80, 88, 175,
        ];

        for i in 1..FIELD_SIZE {
            assert_eq!(logs[i - 1], LOG_TABLE[i]);
        }
    }

    #[test]
    fn test_pow_eq() {
        let exps: [i8; 255] = [
            1, 2, 4, 8, 16, 32, 64, -128, 29, 58, 116, -24, -51, -121, 19, 38, 76, -104, 45, 90,
            -76, 117, -22, -55, -113, 3, 6, 12, 24, 48, 96, -64, -99, 39, 78, -100, 37, 74, -108,
            53, 106, -44, -75, 119, -18, -63, -97, 35, 70, -116, 5, 10, 20, 40, 80, -96, 93, -70,
            105, -46, -71, 111, -34, -95, 95, -66, 97, -62, -103, 47, 94, -68, 101, -54, -119, 15,
            30, 60, 120, -16, -3, -25, -45, -69, 107, -42, -79, 127, -2, -31, -33, -93, 91, -74,
            113, -30, -39, -81, 67, -122, 17, 34, 68, -120, 13, 26, 52, 104, -48, -67, 103, -50,
            -127, 31, 62, 124, -8, -19, -57, -109, 59, 118, -20, -59, -105, 51, 102, -52, -123, 23,
            46, 92, -72, 109, -38, -87, 79, -98, 33, 66, -124, 21, 42, 84, -88, 77, -102, 41, 82,
            -92, 85, -86, 73, -110, 57, 114, -28, -43, -73, 115, -26, -47, -65, 99, -58, -111, 63,
            126, -4, -27, -41, -77, 123, -10, -15, -1, -29, -37, -85, 75, -106, 49, 98, -60, -107,
            55, 110, -36, -91, 87, -82, 65, -126, 25, 50, 100, -56, -115, 7, 14, 28, 56, 112, -32,
            -35, -89, 83, -90, 81, -94, 89, -78, 121, -14, -7, -17, -61, -101, 43, 86, -84, 69,
            -118, 9, 18, 36, 72, -112, 61, 122, -12, -11, -9, -13, -5, -21, -53, -117, 11, 22, 44,
            88, -80, 125, -6, -23, -49, -125, 27, 54, 108, -40, -83, 71, -114,
        ];

        for i in 0..FIELD_SIZE - 1 {
            assert_eq!(exps[i] as u8, EXP_TABLE[i].0);
        }
    }
}
