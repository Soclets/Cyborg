//! Format texts, as `std::io::fmt_print` reads them (D194, D197): text, escapes, `{newln}` and `{tab}`, and holes
//! with their shapes. A literal format is checked when building, so this works on the literal's source text and
//! points each error at the exact bytes.

use crate::diagnostic::{Diagnostic, Edit};
use crate::source::{FileId, Span};

const ESCAPES_WHY: &str =
    "the escapes are `\\\\ \\\" \\' \\r \\0 \\xHH \\uHHHH \\UHHHHHHHH`, and `\\{` and `\\}` for a brace (D194, D197)";

/// Which value a hole takes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Takes {
    /// `{}`: the next value.
    Next,
    /// `{2}`, counting from 1.
    Numbered(usize),
    /// `{$name}`, with any fields after it.
    Named { name: String, fields: Vec<String>, span: Span },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidthUnit {
    Columns,
    ColumnsEastAsian,
    Graphemes,
    Bytes,
}

/// A shape, as written in a hole.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShapeWord {
    Align(crate::checked::Align),
    Width(u64, WidthUnit),
    Places(u32),
    Base(crate::checked::Base),
    Science,
    Zeros,
    Sign,
    Unit,
}

impl ShapeWord {
    /// The shape's name as an error writes it.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            ShapeWord::Align(_) => "align:",
            ShapeWord::Width(..) => "width:",
            ShapeWord::Places(_) => "places:",
            ShapeWord::Base(_) => "base:",
            ShapeWord::Science => "form:",
            ShapeWord::Zeros => "zeros",
            ShapeWord::Sign => "sign",
            ShapeWord::Unit => "unit",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Hole {
    pub takes: Takes,
    pub shapes: Vec<ShapeWord>,
    /// The whole hole, braces included.
    pub span: Span,
    /// The shapes' part, from the first shape to the closing brace, which a shape error points at.
    pub shapes_span: Span,
}

#[derive(Clone, Debug)]
pub enum Part {
    Text(String),
    Hole(Hole),
}

/// Reads a format literal, whose source text, quotes included, starts at `start`.
#[must_use]
pub fn read(file: FileId, raw: &str, start: usize, errors: &mut Vec<Diagnostic>) -> Option<Vec<Part>> {
    let span = |from: usize, to: usize| Span::new(file, start + from, start + to);
    let mut parts = Vec::new();
    let mut text = String::new();
    let inner_end = raw.len().saturating_sub(1).max(1);
    let bytes = raw.as_bytes();
    let mut at = 1;
    let mut failed = false;
    while at < inner_end {
        let rest = &raw[at..inner_end];
        let Some(character) = rest.chars().next() else {
            break;
        };
        match character {
            '\\' => match escape(rest) {
                Ok((decoded, length)) => {
                    text.push_str(&decoded);
                    at += length;
                }
                Err((diagnostic_kind, length)) => {
                    let place = span(at, at + length);
                    errors.push(escape_error(diagnostic_kind, place, &rest[..length.min(rest.len())]));
                    failed = true;
                    at += length.max(1);
                }
            },
            '{' => {
                let Some(close) = rest.find('}') else {
                    errors.push(Diagnostic::refused_fix(
                        span(at, at + 1),
                        "\\{",
                        "`fmt_` says `{` starts a hole, so a brace in the text is `\\{` (D197)",
                    ));
                    failed = true;
                    at += 1;
                    continue;
                };
                let inside = &rest[1..close];
                if inside == "newln" {
                    text.push('\n');
                } else if inside == "tab" {
                    text.push('\t');
                } else {
                    if !text.is_empty() {
                        parts.push(Part::Text(std::mem::take(&mut text)));
                    }
                    match hole(file, inside, start + at, errors) {
                        Some(hole) => parts.push(Part::Hole(hole)),
                        None => failed = true,
                    }
                }
                at += close + 1;
            }
            '}' => {
                errors.push(Diagnostic::not_designed(
                    span(at, at + 1),
                    "a bare `}` in a format's text",
                    "not decided yet (standard 12.2)",
                ));
                failed = true;
                at += 1;
            }
            _ => {
                text.push(character);
                at += character.len_utf8();
            }
        }
        let _ = bytes;
    }
    if !text.is_empty() {
        parts.push(Part::Text(text));
    }
    if failed { None } else { Some(parts) }
}

/// What's wrong with an escape.
#[derive(Clone, Copy, Debug)]
pub enum EscapeError {
    Newline,
    Tab,
    Unknown,
    HighByte,
    NotCharacter,
}

fn escape_error(kind: EscapeError, place: Span, written: &str) -> Diagnostic {
    let words = "a newline and a tab are words inside the text, `{newln}` and `{tab}` (D194)";
    match kind {
        EscapeError::Newline => Diagnostic::refused_fix(place, "{newln}", words),
        EscapeError::Tab => Diagnostic::refused_fix(place, "{tab}", words),
        EscapeError::Unknown => Diagnostic::refused(place, format!("`{written}` isn't an escape"), ESCAPES_WHY),
        EscapeError::HighByte => Diagnostic::not_designed(
            place,
            "`\\x` above `7F`, which isn't one character in UTF-8",
            "not decided yet (standard 12.2)",
        ),
        EscapeError::NotCharacter => Diagnostic::refused(
            place,
            format!("`{}` isn't a character", written.get(2..).unwrap_or("")),
            "`\\uHHHH` and `\\UHHHHHHHH` give a character by its number (D194)",
        ),
    }
}

/// Reads one escape at the start of `rest`, giving what it means and how many bytes it takes.
///
/// # Errors
/// What's wrong with it, and how many bytes the error covers.
pub fn escape(rest: &str) -> Result<(String, usize), (EscapeError, usize)> {
    let mut characters = rest.chars();
    characters.next();
    let Some(code) = characters.next() else {
        return Err((EscapeError::Unknown, 1));
    };
    let simple = |text: &str| Ok((text.to_string(), 2));
    match code {
        '\\' => simple("\\"),
        '"' => simple("\""),
        '\'' => simple("'"),
        'r' => simple("\r"),
        '0' => simple("\0"),
        '{' => simple("{"),
        '}' => simple("}"),
        'n' => Err((EscapeError::Newline, 2)),
        't' => Err((EscapeError::Tab, 2)),
        'x' | 'u' | 'U' => {
            let digits = match code {
                'x' => 2,
                'u' => 4,
                _ => 8,
            };
            let hex = rest.get(2..2 + digits).filter(|hex| hex.bytes().all(|byte| byte.is_ascii_hexdigit()));
            let Some(hex) = hex else {
                return Err((EscapeError::Unknown, 2));
            };
            let length = 2 + digits;
            let value = u32::from_str_radix(hex, 16).map_err(|_| (EscapeError::Unknown, length))?;
            if code == 'x' && value > 0x7F {
                return Err((EscapeError::HighByte, length));
            }
            match char::from_u32(value) {
                Some(character) => Ok((character.to_string(), length)),
                None => Err((EscapeError::NotCharacter, length)),
            }
        }
        other => Err((EscapeError::Unknown, 1 + other.len_utf8())),
    }
}

const SHAPES_WHY: &str =
    "the shapes are `align:`, `width:`, `places:`, `base:`, `form:`, `zeros`, `sign` and `unit` (D194, D246)";

/// Reads a hole's insides, which start one byte after `open`, the offset of its `{`.
fn hole(file: FileId, inside: &str, open: usize, errors: &mut Vec<Diagnostic>) -> Option<Hole> {
    let whole = Span::new(file, open, open + inside.len() + 2);
    let mut pieces: Vec<(usize, &str)> = Vec::new();
    let mut offset = 0;
    for piece in inside.split(',') {
        let lead = piece.len() - piece.trim_start().len();
        pieces.push((offset + lead, piece.trim()));
        offset += piece.len() + 1;
    }
    let mut takes = Takes::Next;
    let mut shape_start = 0;
    if let Some(&(first_offset, first)) = pieces.first() {
        if let Some(name) = first.strip_prefix('$') {
            let mut words = name.split('.');
            let name = words.next().unwrap_or("").to_string();
            let fields = words.map(str::to_string).collect();
            let span = Span::new(file, open + 1 + first_offset, open + 1 + first_offset + first.len());
            takes = Takes::Named { name, fields, span };
            shape_start = 1;
        } else if !first.is_empty() && first.bytes().all(|byte| byte.is_ascii_digit()) {
            let number: usize = first.parse().unwrap_or(0);
            if number == 0 {
                errors.push(Diagnostic::refused_fix(
                    whole,
                    "{1}",
                    "`{1}`, `{2}` are numbered holes, counting from 1 (D194)",
                ));
                return None;
            }
            takes = Takes::Numbered(number);
            shape_start = 1;
            if pieces.len() > 1 {
                errors.push(Diagnostic::not_designed(
                    whole,
                    "shapes on a numbered hole, and how they're written",
                    "not decided yet (standard 12.2)",
                ));
                return None;
            }
        } else if first.is_empty() && pieces.len() == 1 {
            return Some(Hole { takes, shapes: Vec::new(), span: whole, shapes_span: whole });
        }
    }
    let shapes_from = pieces.get(shape_start).map_or(inside.len(), |(offset, _)| *offset);
    let shapes_span =
        if shape_start == 0 { whole } else { Span::new(file, open + 1 + shapes_from, open + inside.len() + 2) };
    let mut shapes = Vec::new();
    let mut seen: Vec<&'static str> = Vec::new();
    for &(_, piece) in pieces.iter().skip(shape_start) {
        let shape = match shape_word(piece) {
            Ok(shape) => shape,
            Err(ShapeError::Unknown) => {
                let name = piece.split(':').next().unwrap_or(piece).trim();
                errors.push(Diagnostic::refused(whole, format!("`{name}` isn't a shape"), SHAPES_WHY));
                return None;
            }
            Err(ShapeError::WidthWithoutUnit) => {
                errors.push(Diagnostic::refused(
                    whole,
                    "a width says its unit",
                    "a width is written with its unit, `cols`, `cols.east_asian`, `graphemes`, `points` or `bytes` (D194)",
                ));
                return None;
            }
            Err(ShapeError::Points) => {
                errors.push(Diagnostic::not_designed(
                    whole,
                    "what a point is, for a width in a print",
                    "not decided yet (standard 12.2)",
                ));
                return None;
            }
        };
        if seen.contains(&shape.label()) {
            errors.push(Diagnostic::refused(
                whole,
                format!("`{}` is written twice in this hole", shape.label()),
                "each shape is written once (D194)",
            ));
            return None;
        }
        seen.push(shape.label());
        shapes.push(shape);
    }
    Some(Hole { takes, shapes, span: whole, shapes_span })
}

enum ShapeError {
    Unknown,
    WidthWithoutUnit,
    Points,
}

fn shape_word(piece: &str) -> Result<ShapeWord, ShapeError> {
    use crate::checked::{Align, Base};
    match piece {
        "zeros" => return Ok(ShapeWord::Zeros),
        "sign" => return Ok(ShapeWord::Sign),
        "unit" => return Ok(ShapeWord::Unit),
        _ => {}
    }
    let Some((label, value)) = piece.split_once(':') else {
        return Err(ShapeError::Unknown);
    };
    let value = value.trim();
    match label.trim() {
        "align" => match value {
            "left" => Ok(ShapeWord::Align(Align::Left)),
            "right" => Ok(ShapeWord::Align(Align::Right)),
            "center" => Ok(ShapeWord::Align(Align::Center)),
            _ => Err(ShapeError::Unknown),
        },
        "width" => {
            let mut words = value.split_whitespace();
            let number: u64 = words.next().and_then(|number| number.parse().ok()).ok_or(ShapeError::Unknown)?;
            let unit = match words.next() {
                None => return Err(ShapeError::WidthWithoutUnit),
                Some("cols") => WidthUnit::Columns,
                Some("cols.east_asian") => WidthUnit::ColumnsEastAsian,
                Some("graphemes") => WidthUnit::Graphemes,
                Some("bytes") => WidthUnit::Bytes,
                Some("points") => return Err(ShapeError::Points),
                Some(_) => return Err(ShapeError::Unknown),
            };
            Ok(ShapeWord::Width(number, unit))
        }
        "places" => value.parse().map(ShapeWord::Places).map_err(|_| ShapeError::Unknown),
        "base" => match value {
            "bin" => Ok(ShapeWord::Base(Base::Binary)),
            "oct" => Ok(ShapeWord::Base(Base::Octal)),
            "hex" => Ok(ShapeWord::Base(Base::Hex)),
            "hex.upper" => Ok(ShapeWord::Base(Base::HexUpper)),
            _ => Err(ShapeError::Unknown),
        },
        "form" if value == "science" => Ok(ShapeWord::Science),
        _ => Err(ShapeError::Unknown),
    }
}

/// Decodes a plain string literal's escapes, for a `str` value. Only a format print's text says what `\` means
/// (D194), so any `\` in a plain string isn't decided.
#[must_use]
pub fn plain_text(file: FileId, raw: &str, start: usize, errors: &mut Vec<Diagnostic>) -> Option<String> {
    let inner = raw.get(1..raw.len().saturating_sub(1)).unwrap_or("");
    if let Some(at) = inner.find('\\') {
        let place = Span::new(file, start + 1 + at, start + 2 + at);
        errors.push(Diagnostic::not_designed(place, "`\\` in a string", "not decided yet (standard 12.2)"));
        return None;
    }
    Some(inner.to_string())
}

/// An edit that does nothing, for errors whose fix isn't an edit of the format.
#[must_use]
pub fn no_edit(span: Span) -> Edit {
    Edit { span, replacement: String::new() }
}
