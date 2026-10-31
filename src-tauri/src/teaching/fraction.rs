//! Exact rational arithmetic (p / q) for symbolic and pedagogical calculations.

use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExactFraction {
    pub num: i64,
    pub den: i64,
}

impl ExactFraction {
    pub fn new(num: i64, den: i64) -> Self {
        assert!(den != 0, "Denominator cannot be zero");
        let mut frac = Self { num, den };
        frac.simplify();
        frac
    }

    pub fn zero() -> Self {
        Self { num: 0, den: 1 }
    }

    pub fn one() -> Self {
        Self { num: 1, den: 1 }
    }

    pub fn from_integer(n: i64) -> Self {
        Self { num: n, den: 1 }
    }

    pub fn from_f64_approx(val: f64) -> Self {
        if val.abs() < 1e-12 {
            return Self::zero();
        }
        // Continuous fraction approximation up to max denominator 1000
        let sign = if val < 0.0 { -1 } else { 1 };
        let x = val.abs();
        let mut m00 = 1i64;
        let mut m01 = 0i64;
        let mut m10 = 0i64;
        let mut m11 = 1i64;
        let mut rem = x;

        for _ in 0..15 {
            let a = rem.floor() as i64;
            let next_m00 = a * m00 + m01;
            let next_m10 = a * m10 + m11;
            if next_m10 > 1000 {
                break;
            }
            m01 = m00;
            m00 = next_m00;
            m11 = m10;
            m10 = next_m10;

            let frac_part = rem - a as f64;
            if frac_part.abs() < 1e-6 {
                break;
            }
            rem = 1.0 / frac_part;
        }

        Self::new(sign * m00, m10)
    }

    pub fn simplify(&mut self) {
        if self.den < 0 {
            self.num = -self.num;
            self.den = -self.den;
        }
        if self.num == 0 {
            self.den = 1;
            return;
        }
        let g = gcd(self.num.abs(), self.den);
        self.num /= g;
        self.den /= g;
    }

    pub fn to_f64(&self) -> f64 {
        self.num as f64 / self.den as f64
    }

    pub fn to_latex(&self) -> String {
        if self.num == 0 {
            return "0".to_string();
        }
        if self.den == 1 {
            return format!("{}", self.num);
        }
        if self.num < 0 {
            format!("-\\frac{{{}}}{{{}}}", -self.num, self.den)
        } else {
            format!("\\frac{{{}}}{{{}}}", self.num, self.den)
        }
    }
}

fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

impl Add for ExactFraction {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let num = self.num * rhs.den + rhs.num * self.den;
        let den = self.den * rhs.den;
        Self::new(num, den)
    }
}

impl Sub for ExactFraction {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        let num = self.num * rhs.den - rhs.num * self.den;
        let den = self.den * rhs.den;
        Self::new(num, den)
    }
}

impl Mul for ExactFraction {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::new(self.num * rhs.num, self.den * rhs.den)
    }
}

impl Div for ExactFraction {
    type Output = Self;
    fn div(self, rhs: Self) -> Self {
        assert!(rhs.num != 0, "Cannot divide by zero fraction");
        Self::new(self.num * rhs.den, self.den * rhs.num)
    }
}

impl Neg for ExactFraction {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.num, self.den)
    }
}

impl fmt::Display for ExactFraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

#[cfg(test)]
#[path = "fraction_test.rs"]
mod tests;
