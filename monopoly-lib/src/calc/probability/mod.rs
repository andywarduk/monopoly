use std::{
    fmt::Display,
    iter::Sum,
    ops::{Add, AddAssign, Div, Mul, MulAssign, Sub, SubAssign},
};

use num_traits::{Num, NumCast, One, Zero};

macro_rules! p {
    ($numerator:literal/$denominator:literal) => {
        p!($numerator, $denominator)
    };
    ($numerator:expr, $denominator:expr) => {
        Probability::new($numerator, $denominator)
    };
}
pub(crate) use p;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Probability {
    numerator: i64,
    denominator: u64,
}

impl Probability {
    pub const NEVER: Probability = Self {
        numerator: 0,
        denominator: 1,
    };

    pub const ALWAYS: Probability = Self {
        numerator: 1,
        denominator: 1,
    };

    pub const fn new(numerator: i64, denominator: u64) -> Self {
        let mut p = Self { numerator, denominator };

        p.normalise();

        p
    }

    pub const fn reciprocal(&self) -> Self {
        let (numerator, denominator) = if self.numerator < 0 {
            (-(self.denominator as i64), (-self.numerator as u64))
        } else {
            (self.denominator as i64, self.numerator as u64)
        };

        Self { numerator, denominator }
    }

    pub const fn as_f64(&self) -> f64 {
        self.numerator as f64 / self.denominator as f64
    }

    const fn normalise(&mut self) {
        let gcd = gcd(self.numerator.unsigned_abs(), self.denominator);

        if gcd != 1 {
            self.numerator /= gcd as i64;
            self.denominator /= gcd;
        }
    }
}

impl Add for Probability {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.add(&other)
    }
}

impl Add<&Probability> for Probability {
    type Output = Self;

    fn add(self, other: &Self) -> Self {
        let (lcm, am, bm) = lcm(self.denominator, other.denominator);

        let mut result = Self {
            numerator: self.numerator * am as i64 + other.numerator * bm as i64,
            denominator: lcm,
        };

        result.normalise();

        result
    }
}

impl AddAssign for Probability {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sum for Probability {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::NEVER, |acc, x| acc + x)
    }
}

impl<'a> Sum<&'a Probability> for Probability {
    fn sum<I: Iterator<Item = &'a Probability>>(iter: I) -> Self {
        iter.fold(Probability::NEVER, |acc, x| acc + x)
    }
}

impl Sub for Probability {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.sub(&other)
    }
}

impl Sub<&Probability> for Probability {
    type Output = Self;

    fn sub(self, other: &Self) -> Self {
        let (lcm, am, bm) = lcm(self.denominator, other.denominator);

        let mut result = Self {
            numerator: self.numerator * am as i64 - other.numerator * bm as i64,
            denominator: lcm,
        };

        result.normalise();

        result
    }
}

impl SubAssign for Probability {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl Mul for Probability {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let (lcm, am, bm) = lcm(self.denominator, other.denominator);

        let mut ret = Self {
            numerator: (self.numerator * am as i64) * (other.numerator * bm as i64),
            denominator: lcm * lcm,
        };

        ret.normalise();

        ret
    }
}

impl MulAssign for Probability {
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl<I: Num + NumCast> Mul<I> for Probability {
    type Output = Self;

    fn mul(self, other: I) -> Self {
        let mut ret = Self {
            numerator: self.numerator * other.to_i64().unwrap(),
            denominator: self.denominator,
        };

        ret.normalise();

        ret
    }
}

impl Div for Probability {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, other: Self) -> Self {
        self * other.reciprocal()
    }
}

impl<I: Num + NumCast> Div<I> for Probability {
    type Output = Self;

    fn div(self, other: I) -> Self {
        let other = other.to_u64().unwrap();

        let (lcm, am, bm) = lcm(self.denominator, other);

        let mut ret = Self {
            numerator: (self.numerator * am as i64) * bm as i64,
            denominator: lcm * lcm,
        };

        ret.normalise();

        ret
    }
}

impl One for Probability {
    fn one() -> Self {
        Self::ALWAYS
    }
}

impl Zero for Probability {
    fn zero() -> Self {
        Self::NEVER
    }

    fn is_zero(&self) -> bool {
        *self == Self::NEVER
    }
}

impl Display for Probability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = if self.numerator == 0 {
            "0".to_string()
        } else {
            format!("{}/{}", self.numerator, self.denominator)
        };

        string.fmt(f)
    }
}

impl PartialOrd for Probability {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Probability {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let (_, am, bm) = lcm(self.denominator, other.denominator);

        (self.numerator * am as i64).cmp(&(other.numerator * bm as i64))
    }
}

const fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }

    a
}

const fn lcm(a: u64, b: u64) -> (u64, u64, u64) {
    let lcm = a / gcd(a, b) * b;

    (lcm, lcm / a, lcm / b)
}

#[cfg(test)]
mod tests;
