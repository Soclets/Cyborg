//! `std::math`'s functions, correctly rounded, computed by the oracle itself. (D256, D161)
//!
//! Correctly rounded means the exact result, rounded once to the nearest `f64`, ties to even.
//! The oracle never uses CORE-MATH or any other maths library, so its answers can check the
//! compiler's (rule 3), and it has no C in it (rule 1).
//!
//! Each function is worked out as an interval that's certain to hold the exact result, using
//! big integers in fixed point, every step rounded outward. When both ends of the interval
//! round to the same `f64`, that's the answer. When they don't, the work is done again with
//! twice the bits. The exact result of `sin`, `cos` or `tan` of any nonzero `f64` is
//! transcendental, so it's never exactly halfway between two `f64`s, and the loop always ends.

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, Zero};

/// A function `std::math` has, correctly rounded. (D245, D256)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Function {
    Sin,
    Cos,
    Tan,
}

impl Function {
    pub fn word(self) -> &'static str {
        match self {
            Function::Sin => "sin",
            Function::Cos => "cos",
            Function::Tan => "tan",
        }
    }
}

/// How many bits the first try uses, past what the argument itself needs.
const FIRST_BITS: u64 = 128;

/// Past this many bits the oracle says something's wrong, instead of running forever. No
/// `f64` input comes near it.
const MOST_BITS: u64 = 1 << 20;

/// `function` of `x`, correctly rounded, and the bits the interval that settled it used. An
/// error means the oracle got past `MOST_BITS`, which is a bug in it.
/// trace: D256, D161
pub fn correctly_rounded(function: Function, x: f64) -> Result<(f64, u64), String> {
    if x == 0.0 {
        // sin(±0) and tan(±0) are ±0 exactly, and cos(±0) is 1.
        return Ok((
            match function {
                Function::Sin | Function::Tan => x,
                Function::Cos => 1.0,
            },
            0,
        ));
    }
    if !x.is_finite() {
        return Err(format!(
            "`{}` of `{x}`, which isn't a number",
            function.word()
        ));
    }
    let mut bits = FIRST_BITS;
    while bits <= MOST_BITS {
        if let Some(found) = settle(function, x, bits) {
            return Ok((found, bits));
        }
        bits *= 2;
    }
    Err(format!(
        "`{}` of `{x}` didn't settle within {MOST_BITS} bits",
        function.word()
    ))
}

/// One try at `bits` bits: the answer, or nothing when the interval straddles two `f64`s.
fn settle(function: Function, x: f64, bits: u64) -> Option<f64> {
    let (low, high, scale) = enclose(function, x, bits)?;
    let low = round_to_f64(&low, scale);
    let high = round_to_f64(&high, scale);
    (low.to_bits() == high.to_bits()).then_some(low)
}

/// An interval holding `function(x)`, as two big integers over 2^scale, or nothing when this
/// many bits isn't enough to work it out.
pub(crate) fn enclose(function: Function, x: f64, bits: u64) -> Option<(BigInt, BigInt, u64)> {
    let (mantissa, exponent) = exact_parts(x);
    // Enough bits to hold `x` itself, and for range reduction to keep `bits` bits after
    // taking a multiple of π/2 as big as `x` away.
    let top = u64::try_from((exponent + 64).max(0)).ok()?;
    let below = u64::try_from((-exponent).max(0)).ok()?;
    let scale = (bits + top + 64).max(below);
    let fixed = Fixed { scale };
    let shift = u64::try_from(i64::from(exponent) + i64::try_from(scale).ok()?).ok()?;
    let argument: BigInt = BigInt::from(mantissa) << shift;
    let (quadrant, reduced) = fixed.reduce(&argument)?;
    let found = match (function, quadrant) {
        (Function::Sin, 0) | (Function::Cos, 3) => fixed.sin(&reduced)?,
        (Function::Sin, 1) | (Function::Cos, 0) => fixed.cos(&reduced)?,
        (Function::Sin, 2) | (Function::Cos, 1) => fixed.sin(&reduced)?.negate(),
        (Function::Sin | Function::Cos, _) => fixed.cos(&reduced)?.negate(),
        (Function::Tan, quadrant) => {
            let sin = fixed.sin(&reduced)?;
            let cos = fixed.cos(&reduced)?;
            if quadrant % 2 == 0 {
                fixed.divide(&sin, &cos)?
            } else {
                fixed.divide(&cos, &sin)?.negate()
            }
        }
    };
    Some((found.low, found.high, scale))
}

/// `x` as an odd integer times a power of two, with its sign on the integer.
fn exact_parts(x: f64) -> (i64, i32) {
    let bits = x.to_bits();
    let negative = bits >> 63 == 1;
    let biased = i32::try_from((bits >> 52) & 0x7ff).unwrap_or(0);
    let fraction = bits & ((1 << 52) - 1);
    let (mut mantissa, mut exponent) = if biased == 0 {
        (fraction, -1074)
    } else {
        (fraction | (1 << 52), biased - 1075)
    };
    while mantissa != 0 && mantissa % 2 == 0 {
        mantissa /= 2;
        exponent += 1;
    }
    let mantissa = i64::try_from(mantissa).unwrap_or(0);
    (if negative { -mantissa } else { mantissa }, exponent)
}

/// A closed interval of numbers, each a big integer over 2^scale.
#[derive(Clone, Debug)]
struct Interval {
    low: BigInt,
    high: BigInt,
}

impl Interval {
    fn exact(value: BigInt) -> Self {
        Self {
            low: value.clone(),
            high: value,
        }
    }

    fn negate(self) -> Self {
        Self {
            low: -self.high,
            high: -self.low,
        }
    }

    fn add(&self, other: &Interval) -> Self {
        Self {
            low: &self.low + &other.low,
            high: &self.high + &other.high,
        }
    }

    fn subtract(&self, other: &Interval) -> Self {
        Self {
            low: &self.low - &other.high,
            high: &self.high - &other.low,
        }
    }

    /// Wider by `amount` on each side.
    fn widen(&self, amount: &BigInt) -> Self {
        Self {
            low: &self.low - amount,
            high: &self.high + amount,
        }
    }

    /// The biggest size of anything in it.
    fn magnitude(&self) -> BigInt {
        self.low.abs().max(self.high.abs())
    }

    fn contains_zero(&self) -> bool {
        !self.low.is_positive() && !self.high.is_negative()
    }

    /// Divided by a positive whole number, rounded outward.
    fn over(&self, divisor: &BigInt) -> Self {
        Self {
            low: self.low.div_floor(divisor),
            high: ceil_divide(&self.high, divisor),
        }
    }
}

/// `value / divisor`, rounded up.
fn ceil_divide(value: &BigInt, divisor: &BigInt) -> BigInt {
    -((-value).div_floor(divisor))
}

/// Interval arithmetic at one scale: each number is a big integer over 2^scale.
struct Fixed {
    scale: u64,
}

impl Fixed {
    fn one(&self) -> BigInt {
        BigInt::one() << self.scale
    }

    /// The product of two intervals, rounded outward.
    fn multiply(&self, left: &Interval, right: &Interval) -> Interval {
        let products = [
            &left.low * &right.low,
            &left.low * &right.high,
            &left.high * &right.low,
            &left.high * &right.high,
        ];
        let smallest = products.iter().min().cloned().unwrap_or_default();
        let largest = products.iter().max().cloned().unwrap_or_default();
        Interval {
            low: smallest >> self.scale,
            high: -((-largest) >> self.scale),
        }
    }

    /// One interval over another that doesn't hold zero, rounded outward, or nothing when it
    /// might.
    fn divide(&self, top: &Interval, bottom: &Interval) -> Option<Interval> {
        if bottom.contains_zero() {
            return None;
        }
        let mut quotients = Vec::new();
        for numerator in [&top.low, &top.high] {
            let shifted: BigInt = numerator << self.scale;
            for denominator in [&bottom.low, &bottom.high] {
                quotients.push(shifted.div_floor(denominator));
                quotients.push(ceil_divide(&shifted, denominator));
            }
        }
        Some(Interval {
            low: quotients.iter().min().cloned()?,
            high: quotients.iter().max().cloned()?,
        })
    }

    /// arctan(1/m), by its alternating series, with the rest bounded by the last term.
    fn arctan_of_inverse(&self, m: u32) -> Interval {
        let m = BigInt::from(m);
        let m_squared = &m * &m;
        let mut power = Interval::exact(self.one()).over(&m);
        let mut sum = Interval::exact(BigInt::zero());
        let mut n: u64 = 0;
        loop {
            let term = power.over(&BigInt::from(2 * n + 1));
            sum = if n.is_multiple_of(2) {
                sum.add(&term)
            } else {
                sum.subtract(&term)
            };
            // The terms shrink, so what's left is smaller than this one.
            if term.magnitude() <= BigInt::one() {
                return sum.widen(&BigInt::one());
            }
            power = power.over(&m_squared);
            n += 1;
        }
    }

    /// π/2, from Machin's formula: π = 16 arctan(1/5) - 4 arctan(1/239).
    fn half_pi(&self) -> Interval {
        let fifth = self.arctan_of_inverse(5);
        let small = self.arctan_of_inverse(239);
        let scaled = |interval: &Interval, by: i32| Interval {
            low: &interval.low * by,
            high: &interval.high * by,
        };
        scaled(&fifth, 8).subtract(&scaled(&small, 2))
    }

    /// `x` less the nearest multiple of π/2, and which quarter turn that multiple is, or
    /// nothing when π isn't known well enough here to say.
    fn reduce(&self, x: &BigInt) -> Option<(u8, Interval)> {
        let half_pi = self.half_pi();
        let middle: BigInt = (&half_pi.low + &half_pi.high) >> 1;
        // The nearest whole number to x / (π/2).
        let doubled: BigInt = x * 2 + &middle;
        let turns = doubled.div_floor(&(&middle * 2));
        let taken = if turns.is_negative() {
            Interval {
                low: &turns * &half_pi.high,
                high: &turns * &half_pi.low,
            }
        } else {
            Interval {
                low: &turns * &half_pi.low,
                high: &turns * &half_pi.high,
            }
        };
        let reduced = Interval::exact(x.clone()).subtract(&taken);
        // The series below need the reduced argument at most 1.
        if reduced.magnitude() > self.one() {
            return None;
        }
        let quadrant = turns.mod_floor(&BigInt::from(4));
        let quadrant = u8::try_from(quadrant).ok()?;
        Some((quadrant, reduced))
    }

    /// sin(r) for |r| ≤ 1, by its alternating series, with the rest bounded by the last term.
    fn sin(&self, r: &Interval) -> Option<Interval> {
        self.series(r.clone(), r, 1)
    }

    /// cos(r) for |r| ≤ 1, the same way.
    fn cos(&self, r: &Interval) -> Option<Interval> {
        self.series(Interval::exact(self.one()), r, 0)
    }

    /// Σ (-1)^n r^(2n+start) / (2n+start)!, from its first term.
    fn series(&self, first: Interval, r: &Interval, start: u64) -> Option<Interval> {
        let squared = self.multiply(r, r);
        let mut term = first;
        let mut sum = term.clone();
        let mut n: u64 = 0;
        loop {
            if term.magnitude() <= BigInt::one() {
                // With |r| ≤ 1 the terms shrink, so what's left is smaller than this one.
                return Some(sum.widen(&BigInt::one()));
            }
            n += 1;
            let index = 2 * n + start;
            let divisor = BigInt::from(index.checked_mul(index - 1)?);
            term = self.multiply(&term, &squared).over(&divisor).negate();
            sum = sum.add(&term);
        }
    }
}

/// `value` over 2^scale, rounded to the nearest `f64`, ties to even. Subnormals round at
/// their own spacing, and anything past the largest `f64` is infinity.
pub(crate) fn round_to_f64(value: &BigInt, scale: u64) -> f64 {
    let negative = value.is_negative();
    let magnitude = value.abs();
    if magnitude.is_zero() {
        return if negative { -0.0 } else { 0.0 };
    }
    let length = i64::try_from(magnitude.bits()).unwrap_or(i64::MAX);
    let scale = i64::try_from(scale).unwrap_or(i64::MAX);
    // The value is in [2^top, 2^(top + 1)).
    let top = length - 1 - scale;
    // How many low bits go: down to 53 significant bits, or to 2^-1074 for a subnormal.
    let dropped = if top >= -1022 {
        length - 53
    } else {
        scale - 1074
    };
    let mut kept: BigInt = if dropped > 0 {
        let dropped_bits = u64::try_from(dropped).unwrap_or(0);
        let kept: BigInt = &magnitude >> dropped_bits;
        let rest: BigInt = &magnitude - (&kept << dropped_bits);
        let half: BigInt = BigInt::one() << (dropped_bits - 1);
        if rest > half || (rest == half && kept.is_odd()) {
            kept + 1
        } else {
            kept
        }
    } else {
        magnitude << u64::try_from(-dropped).unwrap_or(0)
    };
    let mut exponent = dropped - scale;
    if kept.bits() > 53 {
        kept >>= 1;
        exponent += 1;
    }
    let kept = u64::try_from(&kept).unwrap_or(0);
    let bits = if kept < (1 << 52) {
        // A subnormal, at 2^-1074 per step.
        kept
    } else {
        let biased = exponent + 52 + 1023;
        if biased >= 0x7ff {
            0x7ff_u64 << 52
        } else {
            (u64::try_from(biased).unwrap_or(0) << 52) | (kept - (1 << 52))
        }
    };
    let magnitude = f64::from_bits(bits);
    if negative {
        -magnitude
    } else {
        magnitude
    }
}

#[cfg(test)]
mod tests {
    use super::Function::{Cos, Sin, Tan};
    use super::{correctly_rounded, enclose, round_to_f64, Function};

    /// The answer, checked again by one interval at far more bits than it settled at, which
    /// has to round the same way.
    fn checked(function: Function, x: f64) -> f64 {
        let (found, bits) = correctly_rounded(function, x).unwrap_or((f64::NAN, 0));
        assert!(!found.is_nan(), "{} of {x} didn't settle", function.word());
        if x != 0.0 {
            let (low, high, scale) = enclose(function, x, bits * 8 + 512).unwrap_or_default();
            assert_eq!(round_to_f64(&low, scale).to_bits(), found.to_bits());
            assert_eq!(round_to_f64(&high, scale).to_bits(), found.to_bits());
        }
        found
    }

    /// trace: D256
    #[test]
    fn exact_cases() {
        assert_eq!(checked(Function::Sin, 0.0).to_bits(), 0.0_f64.to_bits());
        assert_eq!(checked(Function::Sin, -0.0).to_bits(), (-0.0_f64).to_bits());
        assert_eq!(checked(Function::Tan, -0.0).to_bits(), (-0.0_f64).to_bits());
        assert_eq!(checked(Function::Cos, -0.0).to_bits(), 1.0_f64.to_bits());
    }

    /// Every answer from -40 to 40, checked again at far more bits, and held to identities that
    /// only need IEEE `+`, `*` and `/`, which round the same on every machine. The platform's
    /// own `sin` isn't used, since it isn't correctly rounded and differs between machines.
    /// trace: D256
    #[test]
    fn sweep_keeps_the_identities() {
        let mut x: f64 = -40.0;
        while x < 40.0 {
            let sine = checked(Sin, x);
            let cosine = checked(Cos, x);
            let tangent = checked(Tan, x);
            let one = sine.mul_add(sine, cosine * cosine);
            assert!(
                (one - 1.0).abs() <= 4.0 * f64::EPSILON,
                "sin and cos of {x}: {sine} and {cosine}"
            );
            if cosine.abs() > 1e-3 {
                let ratio = sine / cosine;
                assert!(
                    ((tangent - ratio) / tangent).abs() <= 4.0 * f64::EPSILON,
                    "tan of {x}: {tangent}, where sin over cos is {ratio}"
                );
            }
            x += 0.371_9;
        }
    }

    /// Rounding at the edges: the smallest subnormal, and a tie going to even.
    /// trace: D256
    #[test]
    fn rounding_edges() {
        use num_bigint::BigInt;
        assert_eq!(round_to_f64(&BigInt::from(1), 1074).to_bits(), 1);
        // 1 + 2^-53 is halfway between 1 and the next `f64`, and goes to 1, which is even.
        let halfway = (BigInt::from(1) << 53) + 1;
        assert_eq!(round_to_f64(&halfway, 53).to_bits(), 1.0_f64.to_bits());
        let past = (BigInt::from(1) << 54) + 3;
        assert_eq!(round_to_f64(&past, 54).to_bits(), 1.0_f64.to_bits() + 1);
    }

    /// A binary number written as the tables write it, like `1.0101` or `0.10101`, times
    /// 2^exponent, with `up` adding one in its last place, as a round up does.
    fn binary(digits: &str, exponent: i64, up: bool) -> f64 {
        use num_bigint::BigInt;
        let point = digits.find('.').unwrap_or(digits.len());
        let below = i64::try_from(digits.len() - point).unwrap_or(1) - 1;
        let whole: String = digits.chars().filter(|digit| *digit != '.').collect();
        let mut value = BigInt::parse_bytes(whole.as_bytes(), 2).unwrap_or_default();
        assert_eq!(value.bits(), 53, "{digits} isn't 53 bits");
        if up {
            value += 1;
        }
        round_to_f64(&value, u64::try_from(below - exponent).unwrap_or(0))
    }

    /// Lefèvre and Muller's published worst cases for correct rounding, from "Worst Cases for
    /// Correct Rounding of the Elementary Functions in Double Precision", Tables 8, 10 and 12,
    /// and section 2.3's cos(2^-26). Each is the input, the result's first 53 bits, and the bit
    /// after them, which decides rounding to the nearest. Each is checked again at far more bits.
    /// trace: D256
    /// Each worst case: the function, its input and exponent, the result's first 53 bits and
    /// exponent, and whether the bit after them rounds it up.
    type WorstCase = (Function, &'static str, i64, &'static str, i64, bool);

    const WORST_CASES: &[WorstCase] = &[
        (
            Sin,
            "1.1110000000000000000000000000000000000000000111000010",
            -20,
            "1.1101111111111111111111111111111111111111000000101110",
            -20,
            false,
        ),
        (
            Sin,
            "1.0101100110001011101011101001111001100011001011110110",
            -7,
            "1.0101100110001010000010101110101001001000100110010110",
            -7,
            false,
        ),
        (
            Sin,
            "1.1111111001110110011101110011100111010000111101101101",
            -2,
            "1.1110100110010101000001110011000011000100011010010101",
            -2,
            true,
        ),
        (
            Sin,
            "1.1001001000011111101101010100010001000010110100011000",
            0,
            "0.11111111111111111111111111111111111111111111111111111",
            0,
            true,
        ),
        (
            Sin,
            "1.0110011101010110011101000101011101110000101001010001",
            1,
            "1.0100111111110011010100001110010000010010100000100001",
            -2,
            false,
        ),
        (
            Cos,
            "1.1000000000000000000000000000000000000000000000001001",
            -23,
            "0.11111111111111111111111111111111111111111111101110000",
            0,
            false,
        ),
        (
            Cos,
            "1.1000000000000000000000000000000000000000000000100100",
            -22,
            "0.11111111111111111111111111111111111111111110111000000",
            0,
            false,
        ),
        (
            Cos,
            "1.0010000000000000000000000000000000000000111100110000",
            -18,
            "0.11111111111111111111111111111111111101011110000000000",
            0,
            false,
        ),
        (
            Cos,
            "1.0000011010110101000001010101010100001110011010110010",
            -9,
            "0.11111111111111111101111001001101000111111101111111110",
            0,
            false,
        ),
        (
            Cos,
            "1.1001011111001100110100111101001011000100001110001111",
            -6,
            "0.11111111111010111011001101011101010000111000010101000",
            0,
            true,
        ),
        (
            Cos,
            "1.0110101110001010011000100111001111010111110000100001",
            0,
            "1.0011001101111111110001011011000001110010110001010010",
            -3,
            true,
        ),
        (
            Cos,
            "1.0000000000000000000000000000000000000000000000000000",
            -26,
            "0.11111111111111111111111111111111111111111111111111111",
            0,
            false,
        ),
        (
            Tan,
            "1.1101111111111111111111111111111111111111111100011111",
            -22,
            "1.1110000000000000000000000000000000000000000101010001",
            -22,
            false,
        ),
        (
            Tan,
            "1.0110011111111111111111111111111111111010000100010100",
            -18,
            "1.0110100000000000000000000000000000001000111001100001",
            -18,
            true,
        ),
        (
            Tan,
            "1.0101000001001000011010110010111110000111000000010100",
            -5,
            "1.0101000001111000110011101011111111111001110001110010",
            -5,
            true,
        ),
        (
            Tan,
            "0.10100011010101100001101110010001001000011010100110110",
            0,
            "0.10111101110100100100111110111001110011000001010011110",
            0,
            true,
        ),
    ];

    #[test]
    fn published_worst_cases() {
        for (function, input, input_exponent, result, result_exponent, up) in WORST_CASES {
            let x = binary(input, *input_exponent, false);
            let wanted = binary(result, *result_exponent, *up);
            let found = checked(*function, x);
            assert_eq!(
                found.to_bits(),
                wanted.to_bits(),
                "{} of {x:e}: {found:e}, where the table gives {wanted:e}",
                function.word()
            );
        }
    }

    /// sin, cos and tan from `bc -l` at 1200 digits, rounded once to the nearest `f64` with exact
    /// fractions: inputs in every quadrant, `tan` of 7.23129999999994, which macOS's libm
    /// gets two steps low, and big arguments, where taking away multiples of π/2 needs π to
    /// over a thousand bits. Those are the largest `f64`, 10^22, 10^300, and the `f64` with the
    /// smallest cosine, 6381956970095103 × 2^797, whose cosine is about 4.7e-19 (Lefèvre and
    /// Muller, section 2.3), and which Linux's libm gets several steps off. Each row is the
    /// input's bits, then sin's, cos's and tan's.
    const FROM_BC: &[(u64, u64, u64, u64)] = &[
        // 0.5: sin 0.479425538604203 cos 0.8775825618903728 tan 0.5463024898437905
        (
            0x3fe0_0000_0000_0000,
            0x3fde_aee8_744b_05f0,
            0x3fec_1528_065b_7d50,
            0x3fe1_7b4f_5bf3_474a,
        ),
        // 1.0: sin 0.8414709848078965 cos 0.5403023058681398 tan 1.5574077246549023
        (
            0x3ff0_0000_0000_0000,
            0x3fea_ed54_8f09_0cee,
            0x3fe1_4a28_0fb5_068c,
            0x3ff8_eb24_5cbe_e3a6,
        ),
        // 2.0: sin 0.9092974268256817 cos -0.4161468365471424 tan -2.185039863261519
        (
            0x4000_0000_0000_0000,
            0x3fed_18f6_ead1_b446,
            0xbfda_a226_5753_7205,
            0xc001_7af6_2e09_50f8,
        ),
        // 3.0: sin 0.1411200080598672 cos -0.9899924966004454 tan -0.1425465430742778
        (
            0x4008_0000_0000_0000,
            0x3fc2_1038_6db6_d55b,
            0xbfef_ae04_be85_e5d2,
            0xbfc2_3ef7_1254_b86f,
        ),
        // 4.0: sin -0.7568024953079282 cos -0.6536436208636119 tan 1.1578212823495775
        (
            0x4010_0000_0000_0000,
            0xbfe8_37b9_dddc_1eae,
            0xbfe4_eaa6_06db_24c1,
            0x3ff2_866f_9be4_de13,
        ),
        // 5.0: sin -0.9589242746631385 cos 0.28366218546322625 tan -3.380515006246586
        (
            0x4014_0000_0000_0000,
            0xbfee_af81_f5e0_9933,
            0x3fd2_2785_706b_4ad9,
            0xc00b_0b4b_739b_bb07,
        ),
        // 6.0: sin -0.27941549819892586 cos 0.960170286650366 tan -0.29100619138474915
        (
            0x4018_0000_0000_0000,
            0xbfd1_e1f1_8ab0_a2c0,
            0x3fee_b9b7_0978_22f5,
            0xbfd2_9fd8_6ebb_95be,
        ),
        // -2.5: sin -0.5984721441039565 cos -0.8011436155469337 tan 0.7470222972386603
        (
            0xc004_0000_0000_0000,
            0xbfe3_26af_0dcf_cab1,
            0xbfe9_a2f7_ef85_8b7d,
            0x3fe7_e79b_4e00_bb15,
        ),
        // 10.0: sin -0.5440211108893698 cos -0.8390715290764524 tan 0.6483608274590866
        (
            0x4024_0000_0000_0000,
            0xbfe1_689e_f5f3_4f52,
            0xbfea_d9ac_890c_6b1f,
            0x3fe4_bf5f_34be_3782,
        ),
        // 100.0: sin -0.5063656411097588 cos 0.8623188722876839 tan -0.5872139151569291
        (
            0x4059_0000_0000_0000,
            0xbfe0_3425_b78c_4db8,
            0x3feb_981d_bf66_5fdf,
            0xbfe2_ca74_d62b_5d38,
        ),
        // 7.23129999999994: sin 0.8123174085394228 cos 0.5832155928846523 tan 1.3928252578460842
        (
            0x401c_ecd9_e83e_4217,
            0x3fe9_fe81_13f4_becf,
            0x3fe2_a9b3_bf3e_9e5e,
            0x3ff6_4903_2337_de00,
        ),
        // 1e+22: sin -0.8522008497671888 cos 0.523214785395139 tan -1.6287782256068988
        (
            0x4480_f0cf_064d_d592,
            0xbfeb_453a_b76b_f397,
            0x3fe0_be2c_ef01_c8f4,
            0xbffa_0f79_c1b6_b257,
        ),
        // -1e+22: sin 0.8522008497671888 cos 0.523214785395139 tan 1.6287782256068988
        (
            0xc480_f0cf_064d_d592,
            0x3feb_453a_b76b_f397,
            0x3fe0_be2c_ef01_c8f4,
            0x3ffa_0f79_c1b6_b257,
        ),
        // 1.7976931348623157e+308: sin 0.004961954789184062 cos -0.9999876894265599 tan -0.004962015874444895
        (
            0x7fef_ffff_ffff_ffff,
            0x3f74_52fc_98b3_4e97,
            0xbfef_ffe6_2ecf_ab75,
            0xbf74_530c_fe72_9484,
        ),
        // 1e+300: sin -0.8178819121159085 cos -0.5753861119575491 tan 1.4214488238747245
        (
            0x7e37_e43c_8800_759c,
            0xbfea_2c16_b010_e385,
            0xbfe2_6990_22ad_c4c1,
            0x3ff6_be41_1f37_ac77,
        ),
        // 5.319372648326541e+255: sin 1.0 cos -4.687165924254628e-19 tan -2.133485385753704e+18
        (
            0x7506_ac5b_262c_a1ff,
            0x3ff0_0000_0000_0000,
            0xbc21_4ae7_2e6b_a22f,
            0xc3bd_9ba9_a797_5636,
        ),
    ];

    /// trace: D256
    #[test]
    fn against_bc() {
        for (input, sine, cosine, tangent) in FROM_BC {
            let x = f64::from_bits(*input);
            for (function, wanted) in [(Sin, sine), (Cos, cosine), (Tan, tangent)] {
                let found = checked(function, x);
                assert_eq!(
                    found.to_bits(),
                    *wanted,
                    "{} of {x:e}: {found:e}, where bc gives {:e}",
                    function.word(),
                    f64::from_bits(*wanted)
                );
            }
        }
    }

    /// Tiny arguments, past what Lefèvre and Muller's Table 2 says rounds to the argument
    /// itself or to 1, subnormals included.
    /// trace: D256
    #[test]
    fn tiny_arguments() {
        for x in [1e-10, -1e-10, 1e-200, 5e-324, -5e-324, 2.2e-308] {
            assert_eq!(checked(Function::Sin, x).to_bits(), x.to_bits());
            assert_eq!(checked(Function::Tan, x).to_bits(), x.to_bits());
            assert_eq!(checked(Function::Cos, x).to_bits(), 1.0_f64.to_bits());
        }
    }
}
