//! Currency requires precision for up to N places past the decimal. The most
//! efficient way to approach this is by using integers which represent the
//! currency (multiplied by 10^N) accurately.
//!
//! There is no way to know what the maximum value may be. The currency is
//! unknown as well, so we can't assume anything about its range. It is given
//! u64 as its representation, whose maximum value is exactly
//! ±9,223,372,036,854,775,807. If four of these are decimal values, it is safe
//! to assume that having roughly four times more digits for the rest (15
//! digits) is enough. Making the switch to u128 would allow a range large
//! enough that overflows are out of the question, but that would greatly impact
//! the performance, since its operations are much slower.
//!
//! Additionally, we will need to make sure no overflows occur. This could be
//! done with the `Saturating` wrapper [1], but it's not stable yet,
//! unfortunately. We will stick to using the `saturating` methods in `u64` to
//! avoid using a new library.
//!
//! [1] <https://doc.rust-lang.org/std/num/struct.Saturating.html>

use std::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use serde::{Deserialize, Serialize};

/// A number precise up to `N` digits.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct PreciseCurrency<const N: u32>(i64);

/// Custom serialization that takes floating points
impl<const N: u32> Serialize for PreciseCurrency<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let symbol = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.abs();
        let precision = 10_i64.pow(N);
        let integer = abs / precision;
        let decimals = abs % precision;
        let digits = format!("{symbol}{integer}.{decimals:0>4}");
        serializer.serialize_str(&digits)
    }
}

/// Custom deserialization that outputs floating points
impl<'de, const N: u32> Deserialize<'de> for PreciseCurrency<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        /// Vistor to help deserialize currency
        pub struct CurrencyVisitor<const N: u32>;
        impl<'de, const N: u32> serde::de::Visitor<'de> for CurrencyVisitor<N> {
            type Value = PreciseCurrency<N>;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a currency with four digits of precision")
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(PreciseCurrency((v * 10_u64.pow(N) as f64) as i64))
            }
        }

        deserializer.deserialize_f64(CurrencyVisitor::<N>)
    }
}

/// Converting from the original type
impl<const N: u32> From<i64> for PreciseCurrency<N> {
    fn from(v: i64) -> Self {
        Self(v)
    }
}

/// Custom `Saturating` wrapper
impl<const N: u32> Add for PreciseCurrency<N> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}
impl<const N: u32> AddAssign for PreciseCurrency<N> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_add(rhs.0);
    }
}

/// Custom `Saturating` wrapper
impl<const N: u32> Sub for PreciseCurrency<N> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}
impl<const N: u32> SubAssign for PreciseCurrency<N> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_sub(rhs.0);
    }
}

#[cfg(test)]
mod tests {
    use super::PreciseCurrency;

    use anyhow::Result;

    fn try_serialize(test_cur: PreciseCurrency<4>, expected: &str) -> Result<()> {
        let mut writer = csv::Writer::from_writer(vec![]);
        writer.serialize(test_cur)?;

        let data = String::from_utf8(writer.into_inner()?)?;
        assert_eq!(&data[..(data.len() - 1)], expected, "serialization");

        Ok(())
    }

    fn try_deserialize(test_str: &str, expected: PreciseCurrency<4>) -> Result<()> {
        let test_str = format!("x\n{test_str}\n");
        let mut reader = csv::Reader::from_reader(test_str.as_bytes());
        let data = reader.deserialize::<PreciseCurrency<4>>().next().unwrap()?;

        assert_eq!(data, expected, "deserialization");

        Ok(())
    }

    fn try_both(test_cur: PreciseCurrency<4>, test_str: &str) -> Result<()> {
        try_serialize(test_cur, test_str)?;
        try_deserialize(test_str, test_cur)
    }

    #[test]
    fn test_big() -> Result<()> {
        try_both(PreciseCurrency(9876543210_i64), "987654.3210")?;
        try_both(PreciseCurrency(-9876543210_i64), "-987654.3210")
    }
    #[test]
    fn test_full() -> Result<()> {
        try_both(PreciseCurrency(123444_i64), "12.3444")?;
        try_both(PreciseCurrency(-123444_i64), "-12.3444")
    }
    #[test]
    fn test_integer() -> Result<()> {
        try_both(PreciseCurrency(140000_i64), "14.0000")?;
        try_both(PreciseCurrency(-140000_i64), "-14.0000")
    }
    #[test]
    fn test_decimals() -> Result<()> {
        try_both(PreciseCurrency(1234_i64), "0.1234")?;
        try_both(PreciseCurrency(-1234_i64), "-0.1234")
    }
    #[test]
    fn test_partial1() -> Result<()> {
        try_both(PreciseCurrency(123_i64), "0.0123")?;
        try_both(PreciseCurrency(-123_i64), "-0.0123")
    }
    #[test]
    fn test_partial2() -> Result<()> {
        try_both(PreciseCurrency(10_i64), "0.0010")?;
        try_both(PreciseCurrency(-10_i64), "-0.0010")
    }
    #[test]
    fn test_zero() -> Result<()> {
        try_both(PreciseCurrency(0_i64), "0.0000")
    }
}
