use core::{
    fmt::Display,
    hash::Hash,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
    str::FromStr,
};

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

use crate::Sign;

#[derive(Debug, Clone, Copy)]
pub struct Decimal {
    value: u64,
    scale: i32,
    pub sign: Sign,
}

impl Decimal {
    pub const MAX_PRECISION: i32 = 19;

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

    pub fn negate(self) -> Decimal {
        Decimal {
            value: self.value,
            scale: self.scale,
            sign: self.sign.negate(),
        }
    }

    /// Normalizes the Decimal value the best it can, rewriting to so that
    /// the scale is as close to 0 as possible.
    pub fn normalize(mut self) -> Decimal {
        match self.scale.cmp(&0) {
            core::cmp::Ordering::Less => {
                while self.value < (u64::MAX / 10) && self.scale < 0 {
                    self.value *= 10;
                    self.scale += 1;
                }
            }
            core::cmp::Ordering::Equal => {}
            core::cmp::Ordering::Greater => {
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

    fn rescale_with_precision_loss(self, target_scale: i32) -> Option<Decimal> {
        let mut working_scale = self.scale;
        let mut working_value = self.value;

        match working_scale.cmp(&target_scale) {
            core::cmp::Ordering::Less => {
                while working_scale < target_scale {
                    if let Some(new_value) = working_value.checked_mul(10) {
                        working_value = new_value;
                        working_scale += 1;
                    } else {
                        return None;
                    }
                }
            }
            core::cmp::Ordering::Equal => return Some(self),
            core::cmp::Ordering::Greater => {
                while working_scale > target_scale {
                    if let Some(new_value) = working_value.checked_div(10) {
                        working_value = new_value;
                        working_scale -= 1;
                    } else {
                        return None;
                    }
                }
            }
        }

        Some(Decimal {
            value: working_value,
            scale: working_scale,
            sign: self.sign,
        })
    }

    pub fn rescale(self, scale: i32) -> Option<Decimal> {
        if self.scale == scale {
            return Some(self);
        }

        let rescaled = match self.scale.cmp(&scale) {
            core::cmp::Ordering::Less => {
                // we need to scale up by the diff.
                let diff = scale - self.scale;

                if let Some(multiplier) = 10u64.checked_pow(diff as u32)
                    && let Some(result) = self.value.checked_mul(multiplier)
                {
                    result
                } else {
                    return self.rescale_with_precision_loss(scale);
                }
            }
            core::cmp::Ordering::Equal => self.value,
            core::cmp::Ordering::Greater => {
                let diff = self.scale - scale;

                if let Some(multiplier) = 10u64.checked_pow(diff as u32)
                    && let Some(result) = self.value.checked_div(multiplier)
                {
                    result
                } else {
                    return self.rescale_with_precision_loss(scale);
                }
            }
        };

        Some(Decimal {
            value: rescaled,
            scale,
            sign: self.sign,
        })
    }

    fn align_scales(a: Decimal, b: Decimal) -> (Decimal, Decimal) {
        let target_scale = a.scale.max(b.scale);
        let target_min = a.scale.min(b.scale);

        for scale in (target_min..=target_scale).rev() {
            if let (Some(a_scaled), Some(b_scaled)) = (a.rescale(scale), b.rescale(scale)) {
                return (a_scaled, b_scaled);
            }
        }

        panic!("Can't align at all");
    }

    pub fn pow(self, power: impl Into<Decimal>) -> Decimal {
        let power: Decimal = power.into();
        if !power.is_integer() || power.sign == Sign::Negative {
            panic!("Only non-negative integer exponents supported");
        }

        let mut current = Decimal::ONE;
        let mut remaining = power.value;

        while remaining > 0 {
            current *= self;
            remaining -= 1;
        }

        current.normalize()
    }

    pub fn is_integer(self) -> bool {
        self.scale == 0 || self.value % 10_u64.pow(self.scale as u32) == 0
    }

    pub fn to_u128(self) -> u128 {
        assert!(self.is_integer());
        (self.value as u128) * 10_u128.pow(self.scale as u32)
    }

    fn perfect_square(self) -> Option<Decimal> {
        // Using binary search to find a potential perfect square.
        // O(log n)

        if !self.is_integer() {
            return None;
        }

        let int_value: u128 = if self.scale <= 0 {
            (self.value as u128) * 10_u128.pow(self.scale.unsigned_abs())
        } else {
            (self.value as u128) / 10_u128.pow(self.scale.unsigned_abs())
        };

        let mut low = 0;
        let mut high = int_value;

        while low <= high {
            let mid = (low + high) / 2;
            let squared = mid * mid;

            match squared.cmp(&int_value) {
                core::cmp::Ordering::Equal => return Some(Decimal::from_unsigned(mid as u64)),
                core::cmp::Ordering::Less => low = mid + 1,
                core::cmp::Ordering::Greater => high = mid - 1,
            }
        }

        None
    }

    pub fn sqrt(self) -> Decimal {
        // Using Newton-Raphson estimation.
        // f(x) -> x^2 - k
        // f'(x) -> 2x

        if self.sign == Sign::Negative {
            panic!("Cant take sqrt of negative value");
        }

        // Check if it as a perfect square, if it is
        // we shouldn't estimate it
        if let Some(root) = self.perfect_square() {
            return root;
        }

        // If we are larger than roughly sqrt 2, better to 1/2 it.
        // Otherwise, self is a fine guess...
        let mut current = Decimal::ONE;

        for _ in 0..10 {
            let curr_squared = current.pow(2);
            let numerator = curr_squared - self;
            let denominator = 2 * current;
            current -= numerator / denominator;
        }

        current.normalize()
    }

    pub fn to_scientific_notation(self) -> String {
        let num_str = self.value.to_string();
        let total_digits = num_str.len();
        let decimal_pos = total_digits as i32 - self.scale;

        let exponent = decimal_pos - 1;

        let mantissa_digits: String = num_str
            .chars()
            .take((Decimal::MAX_PRECISION as usize).min(total_digits))
            .collect();

        let mantissa = if mantissa_digits.len() == 1 {
            mantissa_digits
        } else {
            let decimal_part = &mantissa_digits[1..];
            let trimmed_decimal = decimal_part.trim_end_matches('0');

            if trimmed_decimal.is_empty() {
                format!("{}.0", mantissa_digits.chars().next().unwrap())
            } else {
                format!(
                    "{}.{}",
                    mantissa_digits.chars().next().unwrap(),
                    trimmed_decimal
                )
            }
        };

        if exponent == 0 {
            format!("{}{mantissa}", self.sign)
        } else {
            format!("{}{mantissa}E{exponent}", self.sign)
        }
    }

    pub fn to_scientific_notation_alternate(self) -> String {
        let num_str = self.value.to_string();
        let total_digits = num_str.len();
        let decimal_pos = total_digits as i32 - self.scale;

        let exponent = decimal_pos - 1;

        let mantissa_digits: String = num_str
            .chars()
            .take((Decimal::MAX_PRECISION as usize).min(total_digits))
            .collect();

        let mantissa = if mantissa_digits.len() == 1 {
            mantissa_digits
        } else {
            let decimal_part = &mantissa_digits[1..];
            let trimmed_decimal = decimal_part.trim_end_matches('0');

            if trimmed_decimal.is_empty() {
                format!("{}.0", mantissa_digits.chars().next().unwrap())
            } else {
                format!(
                    "{}.{}",
                    mantissa_digits.chars().next().unwrap(),
                    trimmed_decimal
                )
            }
        };

        if exponent == 0 {
            format!("{}{mantissa}", self.sign)
        } else {
            format!("{}{mantissa} * 10^{exponent}", self.sign)
        }
    }

    pub fn factorial(self) -> Decimal {
        debug_assert!(self.is_integer());

        let int = self.to_u128() as u64;
        let mut result = Decimal::ONE;
        for i in 1..=int {
            result *= Self::from(i);
        }

        result
    }
}

impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        let (self_normal, other_normal) = (self.normalize(), other.normalize());
        self_normal.value == other_normal.value
            && self_normal.scale == other_normal.scale
            && self_normal.sign == other_normal.sign
    }
}

impl Eq for Decimal {}

impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Decimal {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let (self_normal, other_normal) = (self.normalize(), other.normalize());

        match (self_normal.sign, other_normal.sign) {
            (Sign::Positive, Sign::Negative) => core::cmp::Ordering::Greater,
            (Sign::Negative, Sign::Positive) => core::cmp::Ordering::Less,
            (Sign::Positive, Sign::Positive) => {
                let (self_aligned, other_aligned) =
                    Decimal::align_scales(self_normal, other_normal);
                self_aligned.value.cmp(&other_aligned.value)
            }
            (Sign::Negative, Sign::Negative) => {
                let (self_aligned, other_aligned) =
                    Decimal::align_scales(self_normal, other_normal);
                other_aligned.value.cmp(&self_aligned.value)
            }
        }
    }
}

impl Hash for Decimal {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        let normal = self.normalize();
        normal.value.hash(state);
        normal.scale.hash(state);
        normal.sign.hash(state);
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let num_str = self.value.to_string();

        match self.scale.cmp(&0) {
            core::cmp::Ordering::Less => {
                write!(
                    f,
                    "{}{num_str}{}",
                    self.sign,
                    "0".repeat(self.scale.unsigned_abs() as usize)
                )
            }
            core::cmp::Ordering::Equal => write!(f, "{}{num_str}", self.sign),
            core::cmp::Ordering::Greater => {
                let (pre, post) =
                    num_str.split_at(num_str.len().saturating_sub(self.scale as usize));
                write!(f, "{}{pre}.{post}", self.sign)
            }
        }
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
            (Sign::Negative, Sign::Negative) => {
                add_values(first.value, second.value, first.scale, Sign::Negative)
            }
            (Sign::Positive, Sign::Negative) => sub_values(first.value, second.value, first.scale),
            (Sign::Negative, Sign::Positive) => sub_values(second.value, first.value, first.scale),
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

        let (mut first, mut second) = (self, rhs);

        // This gets our target scale, properly scaling down our elements as well.
        // This process is slightly lossy...
        //
        //
        // TODO: This needs to work even if your scale is negative...
        let target_scale = match first.scale.checked_add(second.scale) {
            Some(s) => s,
            None => match first.scale.cmp(&second.scale) {
                core::cmp::Ordering::Less | core::cmp::Ordering::Equal => {
                    let new_scale = second.scale - (i32::MAX - first.scale);
                    second = second.rescale_with_precision_loss(new_scale).unwrap();
                    first.scale + new_scale
                }
                core::cmp::Ordering::Greater => {
                    let new_scale = first.scale - (i32::MAX - second.scale);
                    first = first.rescale_with_precision_loss(new_scale).unwrap();
                    new_scale + second.scale
                }
            },
        };

        let product = (first.value as u128) * (second.value as u128);

        if product <= u64::MAX as u128 {
            Decimal {
                value: product as u64,
                scale: target_scale,
                sign,
            }
            .normalize()
        } else {
            let mut result = product;
            let mut scale_reduction = 0;
            while result > (u64::MAX as u128) {
                result /= 10;
                scale_reduction += 1;
            }

            Decimal {
                value: result as u64,
                scale: target_scale - scale_reduction,
                sign,
            }
            .normalize()
        }
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

        let target_scale = self.scale - rhs.scale;
        let dividend = self.value;

        // Perfect divison
        if dividend % rhs.value == 0 {
            return Decimal {
                value: dividend / rhs.value,
                scale: target_scale,
                sign,
            }
            .normalize();
        }

        let mut result: u64 = dividend / rhs.value;

        let divisor = rhs.value as u128;
        let mut remainder: u128 = dividend as u128 % divisor;
        let mut additional_scale = 0;

        while remainder > 0 {
            // We are out of precision.
            if result > (u64::MAX / 10) {
                break;
            }

            remainder *= 10;
            let next_digit = remainder / divisor;
            remainder %= divisor;
            result = result * 10 + (next_digit as u64);
            additional_scale += 1;
        }

        Decimal {
            value: result,
            scale: target_scale + additional_scale,
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
                $imp::$method(self, core::convert::Into::<$promo>::into(other))
            }
        }

        impl $imp<&$scalar> for $res {
            type Output = $res;
            #[inline]
            fn $method(self, other: &$scalar) -> $res {
                $imp::$method(self, core::convert::Into::<$promo>::into(*other))
            }
        }

        impl $imp<$scalar> for &$res {
            type Output = $res;
            #[inline]
            fn $method(self, other: $scalar) -> $res {
                $imp::$method(*self, core::convert::Into::<$promo>::into(other))
            }
        }

        impl $imp<&$scalar> for &$res {
            type Output = $res;
            #[inline]
            fn $method(self, other: &$scalar) -> $res {
                $imp::$method(*self, core::convert::Into::<$promo>::into(*other))
            }
        }

        // Commutative: scalar + Decimal
        impl $imp<$res> for $scalar {
            type Output = $res;
            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(other, core::convert::Into::<$promo>::into(self))
            }
        }

        impl $imp<&$res> for $scalar {
            type Output = $res;
            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(*other, core::convert::Into::<$promo>::into(self))
            }
        }

        impl $imp<$res> for &$scalar {
            type Output = $res;
            #[inline]
            fn $method(self, other: $res) -> $res {
                $imp::$method(other, core::convert::Into::<$promo>::into(*self))
            }
        }

        impl $imp<&$res> for &$scalar {
            type Output = $res;
            #[inline]
            fn $method(self, other: &$res) -> $res {
                $imp::$method(*other, core::convert::Into::<$promo>::into(*self))
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

    // #[test]
    // fn test_add_overflow_handling() {
    //     let a = Decimal::new(u64::MAX - 1, 0, Sign::Positive);
    //     let b = Decimal::new(5, 0, Sign::Positive);
    //     let result = a + b;
    //     // This should handle overflow by scaling up
    // }

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

        assert_eq!(result.sign, Sign::Positive);
        assert!(result.value > 3);
    }

    #[test]
    fn test_div_different_scales() {
        let a = Decimal::new(1234, 2, Sign::Positive);
        let b = Decimal::new(56, 1, Sign::Positive);
        let result = a / b;

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
        assert_eq!(result2.value, 10);
    }

    #[test]
    fn test_div_precision_examples() {
        let one_third = Decimal::ONE / Decimal::new(3, 0, Sign::Positive);
        assert_eq!(one_third.sign, Sign::Positive);
        assert!(one_third.scale > 0);

        let pi_approx = Decimal::new(22, 0, Sign::Positive) / Decimal::new(7, 0, Sign::Positive);
        assert_eq!(pi_approx.sign, Sign::Positive);
        assert!(pi_approx.value > 3);

        let one_ninth = Decimal::ONE / Decimal::new(9, 0, Sign::Positive);
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

    #[test]
    fn test_proper_rescaling_add() {
        let mut res = Decimal::new(15, 1, Sign::Positive).pow(2);
        assert_eq!(res, Decimal::new(225, 2, Sign::Positive));
        res -= 2.into();
        assert_eq!(res, Decimal::new(25, 2, Sign::Positive));
    }

    #[test]
    fn test_sqrt_precision() {
        // Generally, we lose precision during certain ops but it should be within 8 digits.
        assert_eq!(
            Decimal::PI.pow(2).sqrt().rescale(15).unwrap(),
            Decimal::PI.rescale(15).unwrap()
        );

        assert_eq!(
            Decimal::E.pow(2).sqrt().rescale(15).unwrap(),
            Decimal::E.rescale(15).unwrap()
        );
    }
}
