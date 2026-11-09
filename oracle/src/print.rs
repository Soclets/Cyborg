//! How a format print shapes each value into text. (D194)

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::ast::{Align, Base, Shapes, WidthUnit};
use crate::report::{Outcome, Report, ReportExtra, Span};
use crate::types::{show_float, stored_float, FloatKind, Value};

/// How NaN and infinity print, since they have no digits. (D166)
/// trace: D166
pub fn non_finite_text(value: f64) -> Option<&'static str> {
    if value.is_nan() {
        Some("NaN")
    } else if value == f64::INFINITY {
        Some("inf")
    } else if value == f64::NEG_INFINITY {
        Some("-inf")
    } else {
        None
    }
}

/// One value, shaped as its hole says. `.stored` prints floats exactly as stored. (D194, D162)
/// trace: D194
pub fn render(value: &Value, shapes: &Shapes, stored: bool, span: Span) -> Outcome<String> {
    let (negative, body) = match value {
        Value::Str(text) => (false, text.clone()),
        Value::Integer(number) => (*number < 0, integer_digits(*number, shapes.base, span)?),
        Value::F32(number) => float_body(f64::from(*number), FloatKind::F32, shapes, stored, span)?,
        Value::F64(number) => float_body(*number, FloatKind::F64, shapes, stored, span)?,
        _ => {
            return Err(Report::check_failed(
                span,
                "the oracle got lost: a value the checker said can't be printed",
                "this is a bug in the oracle, not in your program (rule 6)",
            ));
        }
    };
    let sign = if negative {
        "-"
    } else if shapes.sign {
        "+"
    } else {
        ""
    };
    let whole = format!("{sign}{body}");
    let Some((width, unit)) = shapes.width else {
        return Ok(whole);
    };
    let used = measure(&whole, unit);
    // A value wider than its width prints whole.
    if used >= width {
        return Ok(whole);
    }
    let gap = width - used;
    // A `0` and a space each take one unit of every kind of width.
    if shapes.zeros {
        return Ok(format!("{sign}{}{body}", "0".repeat(gap)));
    }
    match shapes.align {
        Some(Align::Left) => Ok(format!("{whole}{}", " ".repeat(gap))),
        Some(Align::Right) => Ok(format!("{}{whole}", " ".repeat(gap))),
        Some(Align::Center) if gap.is_multiple_of(2) => {
            let half = " ".repeat(gap / 2);
            Ok(format!("{half}{whole}{half}"))
        }
        Some(Align::Center) => Err(Report::not_designed(
            span,
            "which side of a centred value gets the odd space",
            "",
        )),
        None => Err(Report::not_designed(
            span,
            "which way a value lines up when only its width is given",
            "",
        )),
    }
}

fn measure(text: &str, unit: WidthUnit) -> usize {
    match unit {
        WidthUnit::Cols => text.width(),
        WidthUnit::ColsEastAsian => text.width_cjk(),
        WidthUnit::Graphemes => text.graphemes(true).count(),
        WidthUnit::Bytes => text.len(),
    }
}

fn integer_digits(number: i128, base: Option<Base>, span: Span) -> Outcome<String> {
    let magnitude = number.unsigned_abs();
    if number < 0 && base.is_some() {
        return Err(Report::not_designed(
            span,
            "how a negative number prints with `base:`",
            "",
        ));
    }
    Ok(match base {
        None => magnitude.to_string(),
        Some(Base::Bin) => format!("{magnitude:b}"),
        Some(Base::Oct) => format!("{magnitude:o}"),
        Some(Base::Hex) => format!("{magnitude:x}"),
        Some(Base::HexUpper) => format!("{magnitude:X}"),
    })
}

/// A float's sign and digits, with `places:` and `form: science` worked out exactly from what's
/// stored. A value exactly halfway has no stated way to round, so it isn't guessed.
fn float_body(
    number: f64,
    kind: FloatKind,
    shapes: &Shapes,
    stored: bool,
    span: Span,
) -> Outcome<(bool, String)> {
    if let Some(word) = non_finite_text(number) {
        if shapes.sign || shapes.zeros || shapes.places.is_some() || shapes.science {
            return Err(Report::not_designed(
                span,
                format!("how `{word}` prints with shapes on it"),
                "",
            ));
        }
        return Ok((false, word.to_string()));
    }
    let negative = number.is_sign_negative();
    let magnitude = number.abs();
    let body = if stored {
        stored_float(magnitude)
    } else if shapes.science {
        science(magnitude, kind, shapes.places, span)?
    } else if let Some(places) = shapes.places {
        round_to_places(&stored_float(magnitude), places)
            .ok_or_else(|| halfway(span, magnitude, kind))?
    } else {
        show_float(kind, magnitude)
    };
    Ok((negative, body))
}

fn halfway(span: Span, value: f64, kind: FloatKind) -> Box<Report> {
    Report::not_designed(span, "which way a value exactly halfway rounds", "")
        .when(format!("the value is {}", show_float(kind, value)))
}

/// `form: science`, as in `1.25e1`. The mantissa keeps its `.`, as a float literal does. (D165)
fn science(magnitude: f64, kind: FloatKind, places: Option<usize>, span: Span) -> Outcome<String> {
    if magnitude < 1.0 {
        return Err(Report::not_designed(
            span,
            "a negative exponent, or zero, in `form: science`",
            "12.2",
        ));
    }
    let exact = stored_float(magnitude);
    let (whole, fraction) = exact.split_once('.').unwrap_or((&exact, ""));
    let mut exponent = whole.len() - 1;
    let mantissa = match places {
        None => {
            #[allow(clippy::cast_possible_truncation)] // An `f32` was widened, so this is exact.
            let shortest = match kind {
                FloatKind::F32 => format!("{:e}", magnitude as f32),
                FloatKind::F64 => format!("{magnitude:e}"),
            };
            let digits = shortest.split('e').next().unwrap_or("").to_string();
            if digits.contains('.') {
                digits
            } else {
                format!("{digits}.0")
            }
        }
        Some(places) => {
            let all = format!("{whole}{fraction}");
            let (first, rest) = all.split_at(1);
            let rounded = round_to_places(&format!("{first}.{rest}"), places)
                .ok_or_else(|| halfway(span, magnitude, kind))?;
            // Rounding 9.99 up gives 10.00, which is one more power of ten.
            if rounded.starts_with("10") {
                exponent += 1;
                if places == 0 {
                    "1".to_string()
                } else {
                    format!("1.{}", "0".repeat(places))
                }
            } else {
                rounded
            }
        }
    };
    Ok(format!("{mantissa}e{exponent}"))
}

/// Rounds an exact decimal to `places` places, to the nearest, and gives nothing at a tie.
fn round_to_places(exact: &str, places: usize) -> Option<String> {
    let (whole, fraction) = exact.split_once('.').unwrap_or((exact, ""));
    let mut kept: Vec<u8> = whole.bytes().chain(fraction.bytes().take(places)).collect();
    kept.extend(std::iter::repeat_n(
        b'0',
        places.saturating_sub(fraction.len()),
    ));
    let next = fraction.as_bytes().get(places).copied().unwrap_or(b'0');
    let rest_is_zero = fraction
        .as_bytes()
        .get(places + 1..)
        .is_none_or(|rest| rest.iter().all(|digit| *digit == b'0'));
    let up = match next {
        b'6'..=b'9' => true,
        b'5' if !rest_is_zero => true,
        b'5' => return None,
        _ => false,
    };
    if up {
        let mut index = kept.len();
        loop {
            if index == 0 {
                kept.insert(0, b'1');
                break;
            }
            index -= 1;
            if let Some(digit) = kept.get_mut(index) {
                if *digit == b'9' {
                    *digit = b'0';
                } else {
                    *digit += 1;
                    break;
                }
            }
        }
    }
    let text = String::from_utf8(kept).ok()?;
    let split = text.len() - places;
    let (kept_whole, kept_fraction) = text.split_at(split);
    Some(if places == 0 {
        kept_whole.to_string()
    } else {
        format!("{kept_whole}.{kept_fraction}")
    })
}
