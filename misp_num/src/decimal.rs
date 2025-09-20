use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
    str::FromStr,
};

use crate::Sign;

#[derive(Debug, Clone, Copy, Eq)]
pub struct Decimal {
    value: u64,
    scale: i32,
    sign: Sign,
}

impl Decimal {
    pub const ZERO: Decimal = Decimal {
        value: 0,
        scale: 0,
        sign: Sign::Positive,
    };

    pub const ONE: Decimal = Decimal {
        value: 1,
        scale: 0,
        sign: Sign::Positive,
    };

    pub const PI: Decimal = Decimal {
        value: 3141592653589793238,
        scale: 18,
        sign: Sign::Positive,
    };

    pub const E: Decimal = Decimal {
        value: 2718281828459045235,
        scale: 18,
        sign: Sign::Positive,
    };

    pub fn new(value: u64, scale: i32, sign: Sign) -> Decimal {
        Decimal { value, scale, sign }
    }

    pub fn from_unsigned(value: impl Into<u64>) -> Decimal {
        Decimal::new(value.into(), 0, Sign::Positive)
    }

    pub fn from_signed(value: impl Into<i64>) -> Decimal {
        let signed: i64 = value.into();

        if signed.is_positive() {
            Decimal::new(signed as u64, 0, Sign::Positive)
        } else {
            Decimal::new(signed.unsigned_abs(), 0, Sign::Negative)
        }
    }

    /// Normalizes the Decimal value the best it can, rewriting to so that
    /// the scale is as close to 0 as possible.
    pub fn normalize(mut self) -> Decimal {
        match self.scale.cmp(&0) {
            std::cmp::Ordering::Less => {
                while self.value < (u64::MAX / 10) && self.scale < 0 {
                    self.value *= 10;
                    self.scale += 1;
                }
            }
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => {
                while self.value > 0 && self.value % 10 == 0 && self.scale > 0 {
                    self.value /= 10;
                    self.scale -= 1;
                }
            }
        }

        if self.value == 0 {
            self.scale = 0;
            self.sign = Sign::Positive;
        }

        self
    }

    pub fn rescale(self, scale: i32) -> Option<Decimal> {
        if self.scale == scale {
            return Some(self);
        }

        let diff = self.scale - scale;

        // Positive diff means we are reducing our scale. (/ 10)
        // Negative diff means we are increasing our scale. (x 10)

        let rescaled = if diff > 0 {
            // If our target scale is smaller than our current scale,
            // we need to shrink down our value.
            self.value.checked_div(10_u64.checked_pow(diff as u32)?)?
        } else {
            // If our target scale is larger than our current scale,
            // we need to expand our value.
            self.value
                .checked_mul(10_u64.checked_pow(diff.unsigned_abs())?)?
        };

        Some(Decimal {
            value: rescaled,
            scale,
            sign: self.sign,
        })
    }

    fn align_scales(a: Decimal, b: Decimal) -> (Decimal, Decimal) {
        if let Some(ar) = a.rescale(b.scale) {
            (ar, b)
        } else if let Some(br) = b.rescale(a.scale) {
            (a, br)
        } else {
            panic!("No uniform scaling");
        }
    }
}

macro_rules! impl_from_unsigned {
    ($($t: ty), *) => {
        $(
            impl From<$t> for Decimal {
                fn from(value: $t) -> Self {
                    Self::from_unsigned(value)
                }
            }
        )*
    }
}

macro_rules! impl_from_signed {
    ($($t: ty), *) => {
        $(
            impl From<$t> for Decimal {
                fn from(value: $t) -> Self {
                    Self::from_signed(value)
                }
            }
        )*
    }
}

impl_from_unsigned!(u8, u16, u32, u64);
impl_from_signed!(i8, i16, i32, i64);

impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        let (self_normal, other_normal) = (self.normalize(), other.normalize());
        self_normal.value == other_normal.value
            && self_normal.scale == other_normal.scale
            && self_normal.sign == other_normal.sign
    }
}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let (self_normal, other_normal) = (self.normalize(), other.normalize());

        match (self_normal.sign, other_normal.sign) {
            (Sign::Positive, Sign::Negative) => Some(std::cmp::Ordering::Greater),
            (Sign::Negative, Sign::Positive) => Some(std::cmp::Ordering::Less),
            (Sign::Positive, Sign::Positive) => {
                let (self_aligned, other_aligned) =
                    Decimal::align_scales(self_normal, other_normal);
                Some(self_aligned.value.cmp(&other_aligned.value))
            }
            (Sign::Negative, Sign::Negative) => {
                let (self_aligned, other_aligned) =
                    Decimal::align_scales(self_normal, other_normal);
                Some(other_aligned.value.cmp(&self_aligned.value))
            }
        }
    }
}

impl From<bool> for Decimal {
    fn from(value: bool) -> Self {
        if value { Decimal::ONE } else { Decimal::ZERO }
    }
}

impl FromStr for Decimal {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(());
        }

        let mut chars = s.chars().peekable();
        let sign = match chars.peek().unwrap() {
            '-' => {
                chars.next();
                Sign::Negative
            }
            '+' => {
                chars.next();
                Sign::Positive
            }
            x if x.is_ascii_digit() => Sign::Positive,
            _ => return Err(()),
        };

        let remaining: String = chars.collect();
        let parts: Vec<&str> = remaining.split('.').collect();

        match parts.as_slice() {
            [part] => {
                let value: u64 = part.parse().map_err(|_| ())?;
                Ok(Decimal::new(value, 0, sign))
            }
            [value_part, fractional_part] => {
                let value_str: &str = if value_part.is_empty() {
                    "0"
                } else {
                    value_part
                };

                let fractional_str: &str = if fractional_part.is_empty() {
                    ""
                } else {
                    fractional_part
                };

                let combined = format!("{value_str}{fractional_part}");
                let value: u64 = combined.parse().map_err(|_| ())?;

                let scale = fractional_str.len() as i32;

                Ok(Decimal::new(value, scale, sign))
            }
            _ => Err(()),
        }
    }
}

impl Display for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: Update this.
        write!(f, "{self:?}")
    }
}

impl Add for Decimal {
    type Output = Decimal;

    fn add(self, rhs: Decimal) -> Self::Output {
        fn add_values(first: u64, second: u64, scale: i32, sign: Sign) -> Decimal {
            if let Some(sum) = first.checked_add(second) {
                Decimal {
                    value: sum,
                    scale,
                    sign,
                }
            } else {
                let rescaled = (first / 10) + (second / 10);
                Decimal {
                    value: rescaled,
                    scale: scale - 1,
                    sign,
                }
            }
        }

        fn sub_values(first: u64, second: u64, scale: i32) -> Decimal {
            if first >= second {
                Decimal {
                    value: first - second,
                    scale,
                    sign: Sign::Positive,
                }
            } else {
                Decimal {
                    value: second - first,
                    scale,
                    sign: Sign::Negative,
                }
            }
        }

        let (first, second) = Decimal::align_scales(self, rhs);

        let sum = match (first.sign, second.sign) {
            (Sign::Positive, Sign::Positive) => {
                add_values(first.value, second.value, first.scale, Sign::Positive)
            }
            (Sign::Positive, Sign::Negative) => sub_values(first.value, second.value, first.scale),
            (Sign::Negative, Sign::Positive) => sub_values(second.value, first.value, first.scale),
            (Sign::Negative, Sign::Negative) => {
                add_values(first.value, second.value, first.scale, Sign::Negative)
            }
        };

        sum.normalize()
    }
}

impl Sub for Decimal {
    type Output = Decimal;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: Decimal) -> Self::Output {
        let negated_rhs = Decimal {
            value: rhs.value,
            scale: rhs.scale,
            sign: rhs.sign.negate(),
        };

        self + negated_rhs
    }
}

impl Mul for Decimal {
    type Output = Decimal;

    fn mul(self, rhs: Self) -> Self::Output {
        let sign = match (self.sign, rhs.sign) {
            (Sign::Positive, Sign::Positive) => Sign::Positive,
            (Sign::Positive, Sign::Negative) | (Sign::Negative, Sign::Positive) => Sign::Negative,
            (Sign::Negative, Sign::Negative) => Sign::Positive,
        };

        // first try raw mult, if fien then good
        // otherwise, try reducing the values (and changing the scale)
        // otherwise, accept precision loss?

        if let Some(mult) = self.value.checked_mul(rhs.value) {
            return Decimal {
                value: mult,
                scale: self.scale + rhs.scale,
                sign,
            }
            .normalize();
        };

        let (mut left, mut right) = (self, rhs);

        // while left.value > (u64::MAX / right.value) {
        while left.value.checked_mul(right.value).is_none() {
            if left.value >= right.value && left.value > 1 {
                left.value /= 10;
                left.scale -= 1;
            } else if right.value > 1 {
                right.value /= 10;
                right.scale -= 1;
            } else {
                unreachable!("Should never happen")
            };
        }

        Decimal {
            value: left.value * right.value,
            scale: left.scale + right.scale,
            sign,
        }
        .normalize()
    }
}

impl Div for Decimal {
    type Output = Decimal;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.value == 0 {
            panic!("Division by zero");
        }

        if self.value == 0 {
            return Decimal::ZERO;
        }

        let sign = match (self.sign, rhs.sign) {
            (Sign::Positive, Sign::Positive) | (Sign::Negative, Sign::Negative) => Sign::Positive,
            (Sign::Positive, Sign::Negative) | (Sign::Negative, Sign::Positive) => Sign::Negative,
        };

        let mut working_dividend = self.value;
        let mut working_scale = self.scale - rhs.scale;

        while working_dividend <= u64::MAX / 10 && working_dividend % rhs.value != 0 {
            working_dividend *= 10;
            working_scale += 1;
        }

        let quotient = working_dividend / rhs.value;

        Decimal {
            value: quotient,
            scale: working_scale,
            sign,
        }
        .normalize()
    }
}

impl Add<u64> for Decimal {
    type Output = Decimal;
    fn add(self, rhs: u64) -> Decimal {
        self + Decimal::from_unsigned(rhs)
    }
}

impl Add<i64> for Decimal {
    type Output = Decimal;
    fn add(self, rhs: i64) -> Decimal {
        self + Decimal::from_signed(rhs)
    }
}

impl Sub<u64> for Decimal {
    type Output = Decimal;

    fn sub(self, rhs: u64) -> Self::Output {
        self - Decimal::from_unsigned(rhs)
    }
}

impl Sub<i64> for Decimal {
    type Output = Decimal;

    fn sub(self, rhs: i64) -> Self::Output {
        self - Decimal::from_signed(rhs)
    }
}

impl Mul<u64> for Decimal {
    type Output = Decimal;

    fn mul(self, rhs: u64) -> Self::Output {
        self * Decimal::from_unsigned(rhs)
    }
}

impl Mul<i64> for Decimal {
    type Output = Decimal;

    fn mul(self, rhs: i64) -> Self::Output {
        self * Decimal::from_signed(rhs)
    }
}

impl Div<u64> for Decimal {
    type Output = Decimal;

    fn div(self, rhs: u64) -> Self::Output {
        self / Decimal::from_unsigned(rhs)
    }
}

impl Div<i64> for Decimal {
    type Output = Decimal;

    fn div(self, rhs: i64) -> Self::Output {
        self / Decimal::from_signed(rhs)
    }
}

macro_rules! impl_ref_combinations {
    (impl $imp:ident for $res:ty, $method:ident) => {
        impl $imp<&$res> for $res {
            type Output = $res;
            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(self, *other)
            }
        }

        impl $imp<$res> for &$res {
            type Output = $res;
            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(*self, other)
            }
        }

        impl $imp for &$res {
            type Output = $res;
            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(self, *other)
            }
        }
    };
}

macro_rules! impl_base_scalar_combinations {
    (impl $imp:ident<$scalar:ty> for $res:ty, $method:ident) => {
        impl $imp<&$scalar> for $res {
            type Output = $res;
            #[inline]
            fn $method(self, other: &$scalar) -> $res {
                $imp::$method(&self, *other)
            }
        }

        impl $imp<$scalar> for &$res {
            type Output = $res;
            #[inline]
            fn $method(self, other: $scalar) -> $res {
                $imp::$method(self, &other)
            }
        }

        impl $imp<&$scalar> for &$res {
            type Output = $res;
            #[inline]
            fn $method(self, other: &$scalar) -> $res {
                $imp::$method(self, *other)
            }
        }

        // Commutative: scalar + Decimal
        impl $imp<$res> for $scalar {
            type Output = $res;
            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(&other, self)
            }
        }

        impl $imp<&$res> for $scalar {
            type Output = $res;
            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(other, self)
            }
        }

        impl $imp<$res> for &$scalar {
            type Output = $res;
            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(&other, *self)
            }
        }

        impl $imp<&$res> for &$scalar {
            type Output = $res;
            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(other, *self)
            }
        }
    };
}

macro_rules! impl_scalar_combinations {
    (impl $imp:ident<$scalar:ty> for $res:ty, $method:ident, $promo:ty) => {
        impl $imp<$scalar> for $res {
            type Output = $res;
            #[inline]
            fn $method(self, other: $scalar) -> $res {
                $imp::$method(self, std::convert::Into::<$promo>::into(other))
            }
        }

        impl $imp<&$scalar> for $res {
            type Output = $res;
            #[inline]
            fn $method(self, other: &$scalar) -> $res {
                $imp::$method(self, std::convert::Into::<$promo>::into(*other))
            }
        }

        impl $imp<$scalar> for &$res {
            type Output = $res;
            #[inline]
            fn $method(self, other: $scalar) -> $res {
                $imp::$method(*self, std::convert::Into::<$promo>::into(other))
            }
        }

        impl $imp<&$scalar> for &$res {
            type Output = $res;
            #[inline]
            fn $method(self, other: &$scalar) -> $res {
                $imp::$method(*self, std::convert::Into::<$promo>::into(*other))
            }
        }

        // Commutative: scalar + Decimal
        impl $imp<$res> for $scalar {
            type Output = $res;
            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(other, std::convert::Into::<$promo>::into(self))
            }
        }

        impl $imp<&$res> for $scalar {
            type Output = $res;
            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(*other, std::convert::Into::<$promo>::into(self))
            }
        }

        impl $imp<$res> for &$scalar {
            type Output = $res;
            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(other, std::convert::Into::<$promo>::into(*self))
            }
        }

        impl $imp<&$res> for &$scalar {
            type Output = $res;
            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(*other, std::convert::Into::<$promo>::into(*self))
            }
        }
    };
}

macro_rules! impl_for_scalars {
    (impl $imp:ident<$promo:ty> for $res:ty, $method:ident, $( $scalar:ty ),*) => {
        $(
            impl_scalar_combinations!(impl $imp<$scalar> for Decimal, $method, $promo);
        )*
    }
}

impl_ref_combinations!(impl Add for Decimal, add);
impl_ref_combinations!(impl Sub for Decimal, sub);
impl_ref_combinations!(impl Mul for Decimal, mul);
impl_ref_combinations!(impl Div for Decimal, div);

impl_base_scalar_combinations!(impl Add<u64> for Decimal, add);
impl_base_scalar_combinations!(impl Add<i64> for Decimal, add);
impl_base_scalar_combinations!(impl Sub<u64> for Decimal, sub);
impl_base_scalar_combinations!(impl Sub<i64> for Decimal, sub);
impl_base_scalar_combinations!(impl Mul<u64> for Decimal, mul);
impl_base_scalar_combinations!(impl Mul<i64> for Decimal, mul);
impl_base_scalar_combinations!(impl Div<u64> for Decimal, div);
impl_base_scalar_combinations!(impl Div<i64> for Decimal, div);

impl_for_scalars!(impl Add<u64> for Decimal, add, u8, u16, u32);
impl_for_scalars!(impl Add<i64> for Decimal, add, i8, i16, i32);
impl_for_scalars!(impl Sub<u64> for Decimal, sub, u8, u16, u32);
impl_for_scalars!(impl Sub<i64> for Decimal, sub, i8, i16, i32);
impl_for_scalars!(impl Mul<u64> for Decimal, mul, u8, u16, u32);
impl_for_scalars!(impl Mul<i64> for Decimal, mul, i8, i16, i32);
impl_for_scalars!(impl Div<u64> for Decimal, div, u8, u16, u32);
impl_for_scalars!(impl Div<i64> for Decimal, div, i8, i16, i32);

impl AddAssign for Decimal {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl SubAssign for Decimal {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl MulAssign for Decimal {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl DivAssign for Decimal {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integers() {
        assert_eq!(
            "123".parse::<Decimal>().unwrap(),
            Decimal::new(123, 0, Sign::Positive)
        );
        assert_eq!(
            "-456".parse::<Decimal>().unwrap(),
            Decimal::new(456, 0, Sign::Negative)
        );
        assert_eq!(
            "+789".parse::<Decimal>().unwrap(),
            Decimal::new(789, 0, Sign::Positive)
        );
    }

    #[test]
    fn test_parse_decimals() {
        assert_eq!(
            "12.34".parse::<Decimal>().unwrap(),
            Decimal::new(1234, 2, Sign::Positive)
        );
        assert_eq!(
            "-0.567".parse::<Decimal>().unwrap(),
            Decimal::new(567, 3, Sign::Negative)
        );
        assert_eq!(
            "0.1".parse::<Decimal>().unwrap(),
            Decimal::new(1, 1, Sign::Positive)
        );
    }

    #[test]
    fn test_parse_edge_cases() {
        assert_eq!(
            "5.".parse::<Decimal>().unwrap(),
            Decimal::new(5, 0, Sign::Positive)
        );

        assert_eq!(
            "0".parse::<Decimal>().unwrap(),
            Decimal::new(0, 0, Sign::Positive)
        );
        assert_eq!(
            "0.0".parse::<Decimal>().unwrap(),
            Decimal::new(0, 1, Sign::Positive)
        );
    }

    #[test]
    fn test_parse_errors() {
        assert!("".parse::<Decimal>().is_err());
        assert!("abc".parse::<Decimal>().is_err());
        assert!("1.2.3".parse::<Decimal>().is_err());
        assert!("1.2a".parse::<Decimal>().is_err());
        assert!("-".parse::<Decimal>().is_err());
        assert!(".".parse::<Decimal>().is_err());
        assert!(".1".parse::<Decimal>().is_err());
    }

    #[test]
    fn test_rescale() {
        let first = Decimal::new(500, 2, Sign::Positive);
        assert_eq!(
            first.rescale(1).unwrap(),
            Decimal {
                value: 50,
                scale: 1,
                sign: Sign::Positive
            }
        );
        assert_eq!(
            first.rescale(0).unwrap(),
            Decimal {
                value: 5,
                scale: 0,
                sign: Sign::Positive
            }
        );
    }

    #[test]
    fn test_add_same_scale() {
        let a = Decimal::new(150, 1, Sign::Positive);
        let b = Decimal::new(250, 1, Sign::Positive);
        let result = a + b;
        assert_eq!(result, Decimal::new(40, 0, Sign::Positive));
    }

    #[test]
    fn test_add_different_scales() {
        let a = Decimal::new(15, 0, Sign::Positive);
        let b = Decimal::new(250, 2, Sign::Positive);
        let result = a + b;
        assert_eq!(result, Decimal::new(175, 1, Sign::Positive));
    }

    #[test]
    fn test_add_positive_negative_same_magnitude() {
        let a = Decimal::new(500, 2, Sign::Positive);
        let b = Decimal::new(500, 2, Sign::Negative);
        let result = a + b;
        assert_eq!(result, Decimal::ZERO);
    }

    #[test]
    fn test_add_positive_negative_different_magnitude() {
        let a = Decimal::new(300, 2, Sign::Positive);
        let b = Decimal::new(500, 2, Sign::Negative);
        let result = a + b;
        assert_eq!(result, Decimal::new(2, 0, Sign::Negative));
    }

    #[test]
    fn test_add_both_negative() {
        let a = Decimal::new(300, 2, Sign::Negative);
        let b = Decimal::new(200, 2, Sign::Negative);
        let result = a + b;
        assert_eq!(result, Decimal::new(5, 0, Sign::Negative));
    }

    #[test]
    fn test_add_with_zero() {
        let a = Decimal::new(500, 2, Sign::Positive);
        let result = a + Decimal::ZERO;
        assert_eq!(result, Decimal::new(5, 0, Sign::Positive));
    }

    #[test]
    fn test_add_normalization() {
        let a = Decimal::new(1000, 3, Sign::Positive);
        let b = Decimal::new(2000, 3, Sign::Positive);
        let result = a + b;
        assert_eq!(result, Decimal::new(3, 0, Sign::Positive));
    }

    #[test]
    fn test_add_overflow_handling() {
        let a = Decimal::new(u64::MAX - 1, 0, Sign::Positive);
        let b = Decimal::new(5, 0, Sign::Positive);
        let result = a + b;
        // This should handle overflow by scaling up
        println!("Overflow result: {:?}", result);
    }

    #[test]
    fn test_add_positive_negative_same_value() {
        let a = Decimal::new(500, 2, Sign::Positive);
        let b = Decimal::new(500, 2, Sign::Negative);
        let result = a + b;
        assert_eq!(result, Decimal::ZERO);
    }

    #[test]
    fn test_add_positive_negative_different_value() {
        let a = Decimal::new(300, 2, Sign::Positive);
        let b = Decimal::new(500, 2, Sign::Negative);
        let result = a + b;
        assert_eq!(result, Decimal::new(2, 0, Sign::Negative));
    }

    #[test]
    fn test_subtraction_negative_result() {
        let a = Decimal::new(300, 2, Sign::Positive);
        let b = Decimal::new(500, 2, Sign::Positive);
        let result = a - b;
        assert_eq!(result, Decimal::new(2, 0, Sign::Negative));
    }

    #[test]
    fn test_subtraction_with_negative() {
        let a = Decimal::new(300, 2, Sign::Positive);
        let b = Decimal::new(200, 2, Sign::Negative);
        let result = a - b;
        assert_eq!(result, Decimal::new(5, 0, Sign::Positive));
    }

    #[test]
    fn test_subtract_negative_from_negative() {
        let a = Decimal::new(500, 2, Sign::Negative);
        let b = Decimal::new(300, 2, Sign::Negative);
        let result = a - b;
        assert_eq!(result, Decimal::new(2, 0, Sign::Negative));
    }

    #[test]
    fn test_add_negative_positive_larger_positive() {
        let a = Decimal::new(200, 2, Sign::Negative);
        let b = Decimal::new(700, 2, Sign::Positive);
        let result = a + b;
        assert_eq!(result, Decimal::new(5, 0, Sign::Positive));
    }

    #[test]
    fn test_add_unsigned() {
        let a = Decimal::new(200, 2, Sign::Negative);
        let result = a + 200;
        assert_eq!(result, Decimal::new(198, 0, Sign::Positive));
    }

    #[test]
    fn test_mul_basic() {
        let a = Decimal::new(25, 1, Sign::Positive);
        let b = Decimal::new(40, 1, Sign::Positive);
        let result = a * b;
        assert_eq!(result, Decimal::new(10, 0, Sign::Positive));
    }

    #[test]
    fn test_mul_different_scales() {
        let a = Decimal::new(123, 2, Sign::Positive);
        let b = Decimal::new(45, 1, Sign::Positive);
        let result = a * b;
        assert_eq!(result, Decimal::new(5535, 3, Sign::Positive));
    }

    #[test]
    fn test_mul_with_zero() {
        let a = Decimal::new(123, 2, Sign::Positive);
        let result = a * Decimal::ZERO;
        assert_eq!(result, Decimal::ZERO);
    }

    #[test]
    fn test_mul_with_one() {
        let a = Decimal::new(123, 2, Sign::Positive);
        let result = a * Decimal::ONE;
        assert_eq!(result, a);
    }

    #[test]
    fn test_mul_signs() {
        let pos = Decimal::new(25, 1, Sign::Positive);
        let neg = Decimal::new(40, 1, Sign::Negative);

        let result1 = pos * neg;
        assert_eq!(result1, Decimal::new(10, 0, Sign::Negative));

        let result2 = neg * neg;
        assert_eq!(result2, Decimal::new(16, 0, Sign::Positive));

        let result3 = pos * pos;
        assert_eq!(result3, Decimal::new(625, 2, Sign::Positive));
    }

    #[test]
    fn test_mul_normalization() {
        let a = Decimal::new(1000, 3, Sign::Positive);
        let b = Decimal::new(2000, 3, Sign::Positive);
        let result = a * b;
        assert_eq!(result, Decimal::new(2, 0, Sign::Positive));
    }

    #[test]
    fn test_mul_scalar() {
        let a = Decimal::new(25, 1, Sign::Positive);
        let result = a * 4u64;
        assert_eq!(result, Decimal::new(10, 0, Sign::Positive));

        let result2 = a * (-2i64);
        assert_eq!(result2, Decimal::new(5, 0, Sign::Negative));
    }

    #[test]
    fn test_mul_scale_math() {
        let a = Decimal::new(1, 1, Sign::Positive);
        let b = Decimal::new(1, 2, Sign::Positive);
        let result = a * b;
        assert_eq!(result, Decimal::new(1, 3, Sign::Positive));

        let c = Decimal::new(1234, 2, Sign::Positive);
        let d = Decimal::new(56, 1, Sign::Positive);
        let result2 = c * d;
        assert_eq!(result2, Decimal::new(69104, 3, Sign::Positive));
    }

    #[test]
    fn test_mul_overflow_handling() {
        let a = Decimal::new(u32::MAX as u64, 0, Sign::Positive);
        let b = Decimal::new(u32::MAX as u64, 0, Sign::Positive);
        let result = a * b;
        println!("Overflow multiplication result: {:?}", result);
        assert_ne!(result.value, 0);
    }

    #[test]
    fn test_mul_assign() {
        let mut a = Decimal::new(25, 1, Sign::Positive);
        a *= Decimal::new(4, 0, Sign::Positive);
        assert_eq!(a, Decimal::new(10, 0, Sign::Positive));
    }

    #[test]
    fn test_div_basic() {
        let a = Decimal::new(100, 1, Sign::Positive);
        let b = Decimal::new(20, 1, Sign::Positive);
        let result = a / b;
        assert_eq!(result, Decimal::new(5, 0, Sign::Positive));
    }

    #[test]
    fn test_div_with_precision() {
        let a = Decimal::new(100, 1, Sign::Positive);
        let b = Decimal::new(3, 0, Sign::Positive);
        let result = a / b;
        println!("10.0 / 3.0 = {:?}", result);

        assert_eq!(result.sign, Sign::Positive);
        assert!(result.value > 3);
    }

    #[test]
    fn test_div_different_scales() {
        let a = Decimal::new(1234, 2, Sign::Positive);
        let b = Decimal::new(56, 1, Sign::Positive);
        let result = a / b;
        println!("12.34 / 5.6 = {:?}", result);

        assert_eq!(result.sign, Sign::Positive);
        assert!(result.value >= 2);
    }

    #[test]
    fn test_div_signs() {
        let pos = Decimal::new(100, 1, Sign::Positive);
        let neg = Decimal::new(20, 1, Sign::Negative);

        let result1 = pos / neg;
        assert_eq!(result1.sign, Sign::Negative);
        assert_eq!(result1.value, 5);

        let result2 = neg / neg;
        assert_eq!(result2.sign, Sign::Positive);
        assert_eq!(result2.value, 1);

        let result3 = pos / pos;
        assert_eq!(result3.sign, Sign::Positive);
        assert_eq!(result3.value, 1);
    }

    #[test]
    fn test_div_by_one() {
        let a = Decimal::new(1234, 2, Sign::Positive);
        let result = a / Decimal::ONE;
        assert_eq!(result, a);
    }

    #[test]
    fn test_div_zero_dividend() {
        let result = Decimal::ZERO / Decimal::new(5, 0, Sign::Positive);
        assert_eq!(result, Decimal::ZERO);
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_div_by_zero() {
        let a = Decimal::new(100, 1, Sign::Positive);
        let _result = a / Decimal::ZERO; // Should panic
    }

    #[test]
    fn test_div_scalar() {
        let a = Decimal::new(100, 1, Sign::Positive);
        let result = a / 2u64;
        assert_eq!(result, Decimal::new(5, 0, Sign::Positive));

        let result2 = a / (-2i64);
        assert_eq!(result2, Decimal::new(5, 0, Sign::Negative));
    }

    #[test]
    fn test_div_assign() {
        let mut a = Decimal::new(100, 1, Sign::Positive);
        a /= Decimal::new(2, 0, Sign::Positive);
        assert_eq!(a, Decimal::new(5, 0, Sign::Positive));
    }

    #[test]
    fn test_div_scale_math() {
        let a = Decimal::new(100, 0, Sign::Positive);
        let b = Decimal::new(10, 0, Sign::Positive);
        let result = a / b;
        assert_eq!(result, Decimal::new(10, 0, Sign::Positive));

        let c = Decimal::new(100, 1, Sign::Positive);
        let d = Decimal::new(10, 1, Sign::Positive);
        let result2 = c / d;
        println!("10.0 / 1.0 = {:?}", result2);
        assert_eq!(result2.value, 10);
    }

    #[test]
    fn test_div_precision_examples() {
        let one_third = Decimal::ONE / Decimal::new(3, 0, Sign::Positive);
        println!("1 / 3 = {:?}", one_third);
        assert_eq!(one_third.sign, Sign::Positive);
        assert!(one_third.scale > 0);

        let pi_approx = Decimal::new(22, 0, Sign::Positive) / Decimal::new(7, 0, Sign::Positive);
        println!("22 / 7 = {:?}", pi_approx);
        assert_eq!(pi_approx.sign, Sign::Positive);
        assert!(pi_approx.value > 3);

        let one_ninth = Decimal::ONE / Decimal::new(9, 0, Sign::Positive);
        println!("1 / 9 = {:?}", one_ninth);
        assert_eq!(one_ninth.sign, Sign::Positive);
    }

    #[test]
    fn test_div_exact_cases() {
        let a = Decimal::new(80, 1, Sign::Positive);
        let b = Decimal::new(20, 1, Sign::Positive);
        let result = a / b;
        assert_eq!(result, Decimal::new(4, 0, Sign::Positive));

        let c = Decimal::new(15, 1, Sign::Positive);
        let d = Decimal::new(5, 1, Sign::Positive);
        let result2 = c / d;
        assert_eq!(result2, Decimal::new(3, 0, Sign::Positive));

        let e = Decimal::new(25, 2, Sign::Positive);
        let f = Decimal::new(5, 2, Sign::Positive);
        let result3 = e / f;
        assert_eq!(result3, Decimal::new(5, 0, Sign::Positive));
    }

    #[test]
    fn test_mixed_operations() {
        let a = Decimal::new(20, 1, Sign::Positive);
        let b = Decimal::new(30, 1, Sign::Positive);
        let result = (a * b) / a;
        assert_eq!(result, Decimal::new(3, 0, Sign::Positive));

        let c = Decimal::new(100, 1, Sign::Positive);
        let d = Decimal::new(20, 1, Sign::Positive);
        let e = Decimal::new(10, 1, Sign::Positive);
        let result2 = (c / d) + e;
        assert_eq!(result2, Decimal::new(6, 0, Sign::Positive));
    }
}
