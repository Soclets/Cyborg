//! Float literals: which values a type holds exactly (D59), the `exponent` word (D165), and printing a float the way
//! errors show it (D161).

use num_bigint::BigUint;

/// A float literal, read from its source.
pub struct FloatLiteral {
    /// The value at the width it lands in, as an `f64` that holds it exactly.
    pub value: f64,
    /// Whether that width holds the literal exactly.
    pub exact: bool,
    /// Whether it's too large, even rounded.
    pub too_large: bool,
}

/// Reads a literal's digits, like `6.02214076e23`, at a width of 32 or 64 bits.
#[must_use]
pub fn read(text: &str, bits: u32, negative: bool) -> Option<FloatLiteral> {
    let clean = text.replace('_', "");
    let (value, too_large) = if bits == 32 {
        let single: f32 = clean.parse().ok()?;
        (f64::from(single), single.is_infinite())
    } else {
        let double: f64 = clean.parse().ok()?;
        (double, double.is_infinite())
    };
    let exact = !too_large && is_exact(&clean, value);
    let value = if negative { -value } else { value };
    Some(FloatLiteral { value, exact, too_large })
}

/// Whether a finite float is exactly the decimal written: digits × 10^power against m × 2^e, cross-multiplied.
fn is_exact(text: &str, value: f64) -> bool {
    let (mantissa_text, exponent) = match text.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (mantissa, exponent.parse::<i64>().unwrap_or(0)),
        None => (text, 0),
    };
    let (whole, fraction) = mantissa_text.split_once('.').unwrap_or((mantissa_text, ""));
    let digits: String = format!("{whole}{fraction}");
    let Some(digits) = BigUint::parse_bytes(digits.as_bytes(), 10) else { return false };
    let Ok(fraction_length) = i64::try_from(fraction.len()) else { return false };
    let decimal_power = exponent - fraction_length;
    let (mantissa, binary_power) = split(value);
    let ten = BigUint::from(10u32);
    let two = BigUint::from(2u32);
    let power = |base: &BigUint, by: i64| -> BigUint { num_traits_pow(base, u64::try_from(by.max(0)).unwrap_or(0)) };
    let left = digits * power(&ten, decimal_power) * power(&two, -binary_power);
    let right = BigUint::from(mantissa) * power(&two, binary_power) * power(&ten, -decimal_power);
    left == right
}

fn num_traits_pow(base: &BigUint, exponent: u64) -> BigUint {
    let mut result = BigUint::from(1u32);
    let mut square = base.clone();
    let mut left = exponent;
    while left > 0 {
        if left & 1 == 1 {
            result *= &square;
        }
        square = &square * &square;
        left >>= 1;
    }
    result
}

/// A finite, non-negative `f64` as m × 2^e.
fn split(value: f64) -> (u64, i64) {
    let bits = value.abs().to_bits();
    let exponent_field = i64::try_from((bits >> 52) & 0x7ff).unwrap_or(0);
    let fraction = bits & ((1u64 << 52) - 1);
    if exponent_field == 0 { (fraction, -1074) } else { (fraction | (1u64 << 52), exponent_field - 1075) }
}

/// A float printed as the shortest text that reads back as it, at its width, always with a point (D161).
#[must_use]
pub fn shortest(value: f64, bits: u32) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value < 0.0 { "-inf".to_string() } else { "inf".to_string() };
    }
    #[allow(clippy::cast_possible_truncation)]
    let text = if bits == 32 { format!("{}", value as f32) } else { format!("{value}") };
    if text.contains('.') { text } else { format!("{text}.0") }
}
