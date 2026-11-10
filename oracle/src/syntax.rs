//! How Cyborg is written: the one place that knows spelling, so a spelling change stays small.

use std::collections::HashSet;

use crate::ast::{Align, Assembly, Base, FormatPiece, Hole, Shapes, WidthUnit};
use crate::ast::{
    Argument, Arm, BinaryOperator, BorrowSources, CompareOperator, Contract, Describe, Expression,
    ExpressionKind, FailureEntry, FailureSet, FieldDeclaration, FieldValue, Fill, ForSource,
    Function, FunctionKind, HandWord, Handler, HandlerSet, Handling, IfLink, IfTest, Import,
    InterfaceDeclaration, InterfaceOperator, Item, LabelLine, LastsAtLeast, Name, Narrowing,
    Parameter, ParameterWord, Path, Program, RangeEnd, RangeExpression, Satisfies, ShieldContent,
    ShieldItem, SourceDeclaration, SourceFill, SourceValue, Statement, StatementKind,
    StructDeclaration, Together, TogetherOperator, TypeExpression, TypeKind, TypeName,
    TypeParameter, TypeSetDeclaration, UnionCase, UnionDeclaration, UnitMeaning, UnitMember,
    UnitSet, Use,
};
use crate::report::{Kind, Outcome, Report, ReportExtra, Span};

/// Words that clash with a name of yours, which is then written `my::name`. (D75)
const CLASHING_WORDS: &[&str] = &[
    // `fn.main` starts a program, so a function named `main` says `my::`. (D214)
    "main",
    "fn",
    "end",
    "let",
    "mut",
    "change",
    "const",
    "if",
    "for",
    "in",
    "match",
    "case",
    "return",
    "struct",
    "union",
    "import",
    "use",
    "as",
    "otherwise",
    "fail",
    "fails",
    "requires",
    "ensures",
    "labels",
    "defaults",
    "shield",
    "reason",
    "Give",
    "Copy",
    "Lend",
    "Given",
    "Lent",
    "empty",
    "all",
    "inf",
    "true",
    "false",
    "else",
    "finish",
    "instead",
    "while",
    "forever",
    "break",
    "range",
    "bake",
    "fam",
    "type",
    "together",
    "comment",
    "x",
    "div",
    "mod",
    "at_least",
    "at_most",
    "and",
    "or",
    "not",
    "bit_and",
    "bit_or",
    "bit_xor",
    "bit_not",
    "shl",
    "shr",
    "void",
    "bool",
    "str",
    "i32",
    "i64",
    "u8",
    "u32",
    "u64",
    "f32",
    "f64",
];

/// trace: D75
pub fn is_clashing(word: &str) -> bool {
    CLASHING_WORDS.contains(&word)
}

/// Reads a format print's text, whose first character is at `start` in the file. Its escapes,
/// holes and shapes are all spelled here. (D194, D197)
/// trace: D194, D197
#[allow(clippy::too_many_lines)] // The whole little language of a format, read in one pass.
pub fn parse_format(raw: &str, start: usize) -> Outcome<Vec<FormatPiece>> {
    let mut pieces: Vec<FormatPiece> = Vec::new();
    let mut text = String::new();
    let mut characters = raw.char_indices().peekable();
    let span_of = |from: usize, to: usize| Span::new(start + from, start + to);
    while let Some((at, character)) = characters.next() {
        match character {
            '\\' => {
                let Some((_, escape)) = characters.next() else {
                    return Err(Report::refused(
                        span_of(at, at + 1),
                        "this `\\` escapes nothing",
                        "a `\\` starts an escape, as in `\\\\` or `\\{` (D194, D197)",
                    ));
                };
                let mut hex = |count: usize| -> Option<u32> {
                    let mut digits = String::new();
                    for _ in 0..count {
                        let (_, digit) = characters.next()?;
                        digits.push(digit);
                    }
                    u32::from_str_radix(&digits, 16).ok()
                };
                let read = match escape {
                    '\\' => Some('\\'),
                    '"' => Some('"'),
                    '\'' => Some('\''),
                    'r' => Some('\r'),
                    '0' => Some('\0'),
                    '{' => Some('{'),
                    '}' => Some('}'),
                    'x' => match hex(2) {
                        Some(value) if value < 0x80 => char::from_u32(value),
                        Some(_) => {
                            return Err(Report::not_designed(
                                span_of(at, at + 4),
                                "`\\x` above `7F`, which isn't one character in UTF-8",
                                "12.2",
                            ));
                        }
                        None => None,
                    },
                    'u' | 'U' => {
                        let digits = if escape == 'u' { 4 } else { 8 };
                        match hex(digits) {
                            Some(value) => match char::from_u32(value) {
                                Some(found) => Some(found),
                                None => {
                                    return Err(Report::refused(
                                        span_of(at, at + 2 + digits),
                                        format!("`{value:X}` isn't a character"),
                                        "`\\uHHHH` and `\\UHHHHHHHH` give a character by its number (D194)",
                                    ));
                                }
                            },
                            None => None,
                        }
                    }
                    'n' => {
                        return Err(Report::refused(
                            span_of(at, at + 2),
                            "a newline in a format is `{newln}`",
                            "a newline and a tab are words inside the text, `{newln}` and `{tab}` (D194)",
                        )
                        .fix("{newln}"));
                    }
                    _ => {
                        return Err(Report::refused(
                            span_of(at, at + 1 + escape.len_utf8()),
                            format!("`\\{escape}` isn't an escape"),
                            "the escapes are `\\\\ \\\" \\' \\r \\0 \\xHH \\uHHHH \\UHHHHHHHH`, and `\\{` and `\\}` for a brace (D194, D197)",
                        ));
                    }
                };
                let Some(read) = read else {
                    return Err(Report::refused(
                        span_of(at, at + 2),
                        format!("`\\{escape}` needs hex digits for a character"),
                        "`\\xHH`, `\\uHHHH` and `\\UHHHHHHHH` give a character by its number (D194)",
                    ));
                };
                text.push(read);
            }
            '{' => {
                let mut content = String::new();
                let mut close = None;
                for (inside_at, inside) in characters.by_ref() {
                    if inside == '}' {
                        close = Some(inside_at);
                        break;
                    }
                    content.push(inside);
                }
                let Some(close) = close else {
                    return Err(Report::refused(
                        span_of(at, at + 1),
                        "a bare `{` in a format",
                        "`fmt_` says `{` starts a hole, so a brace in the text is `\\{` (D197)",
                    )
                    .fix("\\{"));
                };
                let span = span_of(at, close + 1);
                match content.trim() {
                    "newln" => text.push('\n'),
                    "tab" => text.push('\t'),
                    inner => {
                        if !text.is_empty() {
                            pieces.push(FormatPiece::Text(std::mem::take(&mut text)));
                        }
                        pieces.push(FormatPiece::Hole(parse_hole(inner, span)?));
                    }
                }
            }
            '}' => {
                return Err(Report::not_designed(
                    span_of(at, at + 1),
                    "a bare `}` in a format's text",
                    "12.2",
                ));
            }
            other => text.push(other),
        }
    }
    if !text.is_empty() {
        pieces.push(FormatPiece::Text(text));
    }
    Ok(pieces)
}

/// `{$t}`, a hole naming its variable. (D210)
/// trace: D210
fn parse_named_hole(name: &str, span: Span) -> Outcome<Hole> {
    let is_name = name.starts_with(is_word_start) && name.chars().all(is_word_part);
    if is_name {
        return Ok(Hole {
            number: None,
            name: Some(name.to_string()),
            shapes: Shapes::default(),
            span,
        });
    }
    Err(Report::not_designed(
        span,
        if name.contains(',') {
            "shapes on a named hole"
        } else {
            "a named hole that's more than a variable, like a field"
        },
        "12.2",
    ))
}

/// What's inside a hole's `{ }`: nothing, a number, or its shapes. (D194)
fn parse_hole(inner: &str, span: Span) -> Outcome<Hole> {
    let mut shapes = Shapes::default();
    if inner.is_empty() {
        return Ok(Hole {
            number: None,
            name: None,
            shapes,
            span,
        });
    }
    if let Some(name) = inner.strip_prefix('$') {
        return parse_named_hole(name, span);
    }
    if inner.chars().all(|c| c.is_ascii_digit()) {
        let number: usize = inner.parse().unwrap_or(usize::MAX);
        if number == 0 {
            return Err(Report::refused(
                span,
                "numbered holes count from 1",
                "`{1}`, `{2}` are numbered holes, counting from 1 (D194)",
            )
            .fix("{1}"));
        }
        return Ok(Hole {
            number: Some(number),
            name: None,
            shapes,
            span,
        });
    }
    let mut parts = inner.split(',').map(str::trim);
    if inner
        .split(',')
        .next()
        .is_some_and(|first| first.trim().chars().all(|c| c.is_ascii_digit()))
    {
        let _ = parts.next();
        return Err(Report::not_designed(
            span,
            "shapes on a numbered hole, and how they're written",
            "12.2",
        ));
    }
    for part in parts {
        let (word, value) = match part.split_once(':') {
            Some((word, value)) => (word.trim(), Some(value.trim())),
            None => (part, None),
        };
        let name: &'static str = match word {
            "align" => "align:",
            "width" => "width:",
            "places" => "places:",
            "base" => "base:",
            "form" => "form:",
            "zeros" => "zeros",
            "sign" => "sign",
            _ => {
                return Err(Report::refused(
                    span,
                    format!("`{part}` isn't a shape"),
                    "the shapes are `align:`, `width:`, `places:`, `base:`, `form:`, `zeros` and `sign` (D194)",
                ));
            }
        };
        if shapes.written.iter().any(|(found, _)| *found == name) {
            return Err(Report::refused(
                span,
                format!("`{name}` is written twice in this hole"),
                "each shape is written once (D194)",
            ));
        }
        shapes.written.push((name, span));
        parse_shape(&mut shapes, name, value, part, span)?;
    }
    if shapes.zeros && shapes.width.is_none() {
        return Err(Report::refused(
            span,
            "`zeros` needs a width to fill",
            "`zeros` pads a value to its width (D194)",
        ));
    }
    Ok(Hole {
        number: None,
        name: None,
        shapes,
        span,
    })
}

/// One shape inside a hole, such as `width: 8 cols`. (D194)
fn parse_shape(
    shapes: &mut Shapes,
    name: &str,
    value: Option<&str>,
    part: &str,
    span: Span,
) -> Outcome<()> {
    let bad = |what: String| {
        Report::refused(span, what, "shapes are words, as in `width: 8 cols` (D194)")
    };
    match (name, value) {
        ("zeros", None) => shapes.zeros = true,
        ("sign", None) => shapes.sign = true,
        ("align:", Some(value)) => {
            shapes.align = Some(match value {
                "left" => Align::Left,
                "right" => Align::Right,
                "center" => Align::Center,
                _ => return Err(bad(format!("`{value}` isn't an alignment"))),
            });
        }
        ("width:", Some(value)) => {
            let mut words = value.split_whitespace();
            let count = words.next().and_then(|count| count.parse::<usize>().ok());
            let Some(count) = count else {
                return Err(bad(format!("`{value}` isn't a width")));
            };
            let unit = match words.next() {
                None => {
                    return Err(Report::refused(
                        span,
                        "a width says its unit",
                        "a width is written with its unit, `cols`, `cols.east_asian`, `graphemes`, `points` or `bytes` (D194)",
                    ));
                }
                Some("cols") => WidthUnit::Cols,
                Some("cols.east_asian") => WidthUnit::ColsEastAsian,
                Some("graphemes") => WidthUnit::Graphemes,
                Some("bytes") => WidthUnit::Bytes,
                Some("points") => {
                    return Err(Report::not_designed(
                        span,
                        "what a point is, for a width in a print",
                        "12.2",
                    ));
                }
                Some(other) => return Err(bad(format!("`{other}` isn't a width's unit"))),
            };
            if words.next().is_some() {
                return Err(bad(format!("`{value}` isn't a width")));
            }
            shapes.width = Some((count, unit));
        }
        ("places:", Some(value)) => {
            let Ok(count) = value.parse::<usize>() else {
                return Err(bad(format!("`{value}` isn't a number of places")));
            };
            shapes.places = Some(count);
        }
        ("base:", Some(value)) => {
            shapes.base = Some(match value {
                "bin" => Base::Bin,
                "oct" => Base::Oct,
                "hex" => Base::Hex,
                "hex.upper" => Base::HexUpper,
                _ => return Err(bad(format!("`{value}` isn't a base"))),
            });
        }
        ("form:", Some("science")) => shapes.science = true,
        ("form:", Some(value)) => return Err(bad(format!("`{value}` isn't a form"))),
        (_, _) => return Err(bad(format!("`{part}` isn't written like that"))),
    }
    Ok(())
}

/// The builtins that set a file's rules, and the line that asks for the compiler's defaults.
/// (D201)
const RULES_WORDS: &[&str] = &["config", "match_config", "tighten_config", "loosen_config"];
const RULES_DEFAULT: &str = "@config(default)";

/// The word in front of a declaration that others can see. (D200)
const PUB_WORD: &str = "pub";

/// The word in front of an import that brings in both halves of a package. (D188)
const PKG_WORD: &str = "pkg";

/// The word between two sources, one lasting at least as long as the other. (D171)
const LASTS_AT_LEAST: &str = "lasts_at_least";

/// The placeholder narrowing on a stored borrow that lasts to the end of its block. (D169)
const HELD_WORD: &str = "held";

/// The placeholder word in front of an exponent literal. (D165)
const EXPONENT_WORD: &str = "exponent";

/// `6.02214076e23` in plain digits, as `602214076000000000000000.0`. Past what any float holds, the
/// digits stop growing, since the value is too large either way and a giant exponent mustn't
/// build a giant string.
/// trace: D165
fn expand_exponent(mantissa: &str, power: &str) -> String {
    const PAST_ANY_FLOAT: usize = 400;
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let all: String = format!("{whole}{fraction}");
    let Some(first) = all.find(|c: char| c != '0') else {
        return "0.0".to_string();
    };
    let power: usize = power.parse().unwrap_or(usize::MAX);
    let point = whole.len().saturating_add(power);
    if point.saturating_sub(first) > PAST_ANY_FLOAT {
        return format!("1{}.0", "0".repeat(PAST_ANY_FLOAT));
    }
    let digits = if point >= all.len() {
        format!("{all}{}.0", "0".repeat(point - all.len()))
    } else {
        format!("{}.{}", &all[..point], &all[point..])
    };
    let trimmed = digits.trim_start_matches('0');
    if trimmed.starts_with('.') {
        format!("0{trimmed}")
    } else {
        trimmed.to_string()
    }
}

/// Whether a program could write this as a plain name, which is what a quoted `use` has to give.
/// trace: D151
pub fn could_be_called(text: &str) -> bool {
    let mut characters = text.chars();
    characters.next().is_some_and(is_word_start)
        && characters.all(is_word_part)
        && !is_clashing(text)
}

/// A word that can follow a value as an operator, so it can't be that value's unit.
const OPERATOR_WORDS: &[&str] = &[
    "x",
    "div",
    "mod",
    "plus",
    "minus",
    "times",
    "at_least",
    "at_most",
    "and",
    "or",
    "bit_and",
    "bit_or",
    "bit_xor",
    "shl",
    "shr",
    "as",
    "otherwise",
];

const BIT_WORDS: &[(&str, BinaryOperator)] = &[
    ("bit_and", BinaryOperator::BitAnd),
    ("bit_or", BinaryOperator::BitOr),
    ("bit_xor", BinaryOperator::BitXor),
    ("shl", BinaryOperator::ShiftLeft),
    ("shr", BinaryOperator::ShiftRight),
];

#[derive(Clone, Debug, PartialEq)]
enum Lexeme {
    Word(String),
    Variable(String),
    Integer(String),
    Float(String),
    /// `` `text` ``, a source a borrow comes from. (D171)
    Source(String),
    /// `6.02214076e23`, which only `exponent` in front may write. (D165)
    Exponent {
        written: String,
        mantissa: String,
        power: String,
    },
    Text(String),
    Symbol(&'static str),
    /// One line of raw assembly inside an `inline.asm:` block, which LLVM reads. (D233)
    Raw(String),
    End,
}

/// The whitespace between two tokens, which the spacing rules judge. A gap with a line break is
/// judged by where a line can break instead, since indentation isn't decided. (D213, D219)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Gap {
    Touching,
    One,
    Several,
    Line,
}

impl Gap {
    fn between(text: &str) -> Gap {
        if text.contains('\n') {
            Gap::Line
        } else if text.is_empty() {
            Gap::Touching
        } else if text == " " {
            Gap::One
        } else {
            Gap::Several
        }
    }

    fn written(self) -> &'static str {
        if self == Gap::One {
            " "
        } else {
            ""
        }
    }
}

#[derive(Clone, Debug)]
struct Token {
    lexeme: Lexeme,
    span: Span,
    space_before: bool,
    space_after: bool,
    /// What sits between this token and the one before it, and the one after it. (D213)
    gap_before: Gap,
    gap_after: Gap,
    /// The token as written, for a `fix:` that shows the spaced version.
    text: String,
    /// Whether a newline comes between this token and the one before it.
    starts_a_line: bool,
}

/// Longest first, so `??` isn't read as two `?`.
const SYMBOLS: &[&str] = &[
    ".|.", "::", "->", "==", "!=", "??", "<=", ">=", ":=", "=>", "//", ".", ":", ";", ",", "(",
    ")", "[", "]", "<", ">", "=", "+", "-", "~", "?", "%", "@", "*", "/",
];

fn is_word_start(character: char) -> bool {
    character.is_alphabetic() || character == '_'
}

fn is_word_part(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

/// A line's first word `comment`, when the line is exactly a comment block's opener.
fn comment_opener(line: &str) -> Option<&str> {
    let inner = line.strip_prefix("comment")?.strip_suffix(':')?;
    if inner.is_empty() {
        return Some("");
    }
    let name = inner.strip_prefix(char::is_whitespace)?.trim();
    (!name.is_empty() && name.chars().all(is_word_part)).then_some(name)
}

fn comment_closer(line: &str) -> Option<&str> {
    let inner = line.strip_prefix("end")?;
    let inner = inner.strip_prefix(char::is_whitespace)?.trim_start();
    let inner = inner.strip_prefix("comment")?;
    if inner.is_empty() {
        return Some("");
    }
    let name = inner.strip_prefix(char::is_whitespace)?.trim();
    (!name.is_empty() && name.chars().all(is_word_part)).then_some(name)
}

/// Where the line starting at `start` ends, not counting its newline.
fn line_end(source: &str, start: usize) -> usize {
    source
        .get(start..)
        .and_then(|rest| rest.find('\n'))
        .map_or(source.len(), |at| start + at)
}

/// A span around a line's text, leaving out its indent and any trailing space.
fn trimmed_line_span(source: &str, start: usize, end: usize) -> Span {
    let line = source.get(start..end).unwrap_or("");
    let lead = line.len() - line.trim_start().len();
    Span::new(start + lead, start + lead + line.trim().len())
}

/// Reads a comment block starting on the line at `start`, and returns where it ends.
/// A block is read line by line, since what's in it isn't code. (D119)
fn skip_comment_block(source: &str, start: usize) -> Outcome<usize> {
    const WHY: &str =
        "a comment block has a required name, and a closer names the block it closes (D119)";
    let mut stack: Vec<(String, Span)> = Vec::new();
    let mut position = start;
    loop {
        let end = line_end(source, position);
        let line = source.get(position..end).unwrap_or("").trim();
        let span = trimmed_line_span(source, position, end);
        if let Some(name) = comment_opener(line) {
            if name.is_empty() {
                return Err(Report::refused(span, "this comment block has no name", WHY));
            }
            stack.push((name.to_string(), span));
        } else if let Some(name) = comment_closer(line) {
            match stack.pop() {
                Some((open, open_span)) if open != name => {
                    return Err(Report::refused(
                        span,
                        if name.is_empty() {
                            format!("this closer doesn't name the block `{open}`")
                        } else {
                            format!("this closes `{name}`, but the open block is `{open}`")
                        },
                        WHY,
                    )
                    .fix(format!("end comment {open}"))
                    .because(open_span, format!("`{open}` was opened here")));
                }
                _ => {}
            }
            if stack.is_empty() {
                return Ok(end);
            }
        }
        if end >= source.len() {
            let (open, open_span) = stack.pop().unwrap_or((String::new(), span));
            return Err(Report::refused(
                open_span,
                format!("the comment block `{open}` never closes"),
                WHY,
            ));
        }
        position = end + 1;
    }
}

/// Reads one `//` or `///` comment, and returns where it ends. (D118, D119)
fn skip_line_comment(source: &str, start: usize) -> Outcome<usize> {
    let end = line_end(source, start);
    let text = source.get(start..end).unwrap_or("");
    if text.starts_with("///") {
        return Ok(end);
    }
    let after = text.get(2..).unwrap_or("");
    // trace: D119
    if !after.is_empty() && !after.starts_with(char::is_whitespace) {
        return Err(Report::refused(
            Span::new(start, end),
            "`//` needs a space after it",
            "`//` needs a space after it, and `///` is its own form (D119)",
        )
        .fix(format!("// {after}")));
    }
    Ok(end)
}

/// trace: D16
#[allow(clippy::too_many_lines)] // One pass over the characters reads better than split-up pieces.
fn lex(source: &str) -> Outcome<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut position = 0;
    let mut at_line_start = true;
    // Where an `inline.asm:` line opened a block whose raw lines haven't started yet. (D233)
    let mut assembly: Option<Span> = None;
    while position < source.len() {
        let rest = source.get(position..).unwrap_or("");
        let Some(character) = rest.chars().next() else {
            break;
        };
        if character == '\n' {
            position += 1;
            at_line_start = true;
            continue;
        }
        if character.is_whitespace() {
            position += character.len_utf8();
            continue;
        }
        if at_line_start {
            at_line_start = false;
            let end = line_end(source, position);
            let line = source.get(position..end).unwrap_or("").trim_end();
            // trace: D119
            if comment_opener(line).is_some() {
                position = skip_comment_block(source, position)?;
                continue;
            }
            if let Some(name) = comment_closer(line) {
                return Err(Report::refused(
                    trimmed_line_span(source, position, end),
                    format!("there's no open comment block for `end comment {name}` to close"),
                    "a closer names the block it closes (D119)",
                ));
            }
            // An assembly block's lines after `reads:`, `writes:` and `clobbers:` are raw, up to
            // `end inline.asm`, so they're read whole, as a comment block's are. (D233)
            // trace: D233
            let code = line.split(" //").next().unwrap_or(line).trim_end();
            if let Some(opened) = assembly {
                let header = ["reads:", "writes:", "clobbers:"]
                    .iter()
                    .any(|word| code.starts_with(word));
                if !header {
                    assembly = None;
                    if code != "end inline.asm" {
                        position = read_raw_lines(source, position, opened, &mut tokens)?;
                        at_line_start = true;
                        continue;
                    }
                }
            }
            if code == "inline.asm:" {
                assembly = Some(trimmed_line_span(source, position, end));
            }
        }
        // A `//` comment can trail code on its line, while a `///` attaches to what's below it.
        // trace: D205
        if rest.starts_with("//") {
            let line_start = source
                .get(..position)
                .and_then(|before| before.rfind('\n'))
                .map_or(0, |at| at + 1);
            let after_code = source
                .get(line_start..position)
                .is_some_and(|before| !before.trim().is_empty());
            // trace: D206
            if after_code && rest.starts_with("///") {
                let end = line_end(source, position);
                let doc = source.get(position..end).unwrap_or("").trim_end();
                return Err(Report::refused(
                    Span::new(position, position + doc.len()),
                    "a doc comment goes on its own line, above what it describes",
                    "a `///` sits above what it describes, so it can't trail code, though a `//` can (D206, D205)",
                )
                .fix_elsewhere(doc.to_string()));
            }
            position = skip_line_comment(source, position)?;
            continue;
        }
        let start = position;
        let lexeme = if character == '`' {
            // trace: D171
            let name_start = position + 1;
            let name: String = source
                .get(name_start..)
                .unwrap_or("")
                .chars()
                .take_while(|c| is_word_part(*c))
                .collect();
            let close = name_start + name.len();
            if name.is_empty() || source.get(close..close + 1) != Some("`") {
                return Err(Report::refused(
                    Span::new(position, close),
                    "a source is a name in backticks, as in `` `text` ``",
                    "a struct names the sources of its borrows in backticks (D171)",
                ));
            }
            position = close + 1;
            Lexeme::Source(name)
        } else if character == '"' {
            lex_text(source, &mut position)?
        } else if character == '$' {
            let name_start = position + 1;
            let name: String = source
                .get(name_start..)
                .unwrap_or("")
                .chars()
                .take_while(|c| is_word_part(*c))
                .collect();
            if name.is_empty() {
                // `$ total` is refused as spacing, with the name joined on as its fix. (D213)
                let after = source.get(name_start..).unwrap_or("");
                let spaced = after.trim_start_matches([' ', '\t']);
                let word: String = spaced.chars().take_while(|c| is_word_part(*c)).collect();
                if spaced.len() < after.len() && !word.is_empty() {
                    let end = name_start + (after.len() - spaced.len()) + word.len();
                    return Err(Report::refused(
                        Span::new(position, end),
                        "this is spaced the wrong way",
                        SPACING_MARKS,
                    )
                    .fix(format!("${word}")));
                }
                return Err(Report::refused(
                    Span::new(position, name_start),
                    "`$` needs a name after it",
                    "variables, parameters and constants start with `$` (D57, D58)",
                ));
            }
            position = name_start + name.len();
            Lexeme::Variable(name)
        } else if character.is_ascii_digit() {
            // `int.signed.32_bit` starts a piece of a path with a digit, and it's no number there.
            let after_dot = matches!(
                tokens.last().map(|token| &token.lexeme),
                Some(Lexeme::Symbol("."))
            );
            if after_dot {
                let word: String = rest.chars().take_while(|c| is_word_part(*c)).collect();
                position += word.len();
                Lexeme::Word(word)
            } else {
                lex_number(source, &mut position)?
            }
        } else if is_word_start(character) {
            let word: String = rest.chars().take_while(|c| is_word_part(*c)).collect();
            position += word.len();
            Lexeme::Word(word)
        } else if let Some(symbol) = SYMBOLS.iter().find(|symbol| rest.starts_with(**symbol)) {
            position += symbol.len();
            Lexeme::Symbol(symbol)
        } else if character == '＠' {
            // trace: D36
            return Err(Report::refused(
                Span::new(position, position + character.len_utf8()),
                "the limb mark is a plain `@`",
                "the limb mark is a plain `@`, not the fullwidth one Radium uses (D36)",
            )
            .fix("@"));
        } else {
            return Err(Report::refused(
                Span::new(position, position + character.len_utf8()),
                format!("`{character}` isn't part of Cyborg"),
                "only the operators in the standard exist (D43, D44, D49)",
            ));
        };
        let space_before = source
            .get(..start)
            .and_then(|before| before.chars().next_back())
            .is_none_or(char::is_whitespace);
        let space_after = source
            .get(position..)
            .and_then(|after| after.chars().next())
            .is_none_or(char::is_whitespace);
        let starts_a_line = tokens.last().is_none_or(|last: &Token| {
            source
                .get(last.span.end..start)
                .is_some_and(|between| between.contains('\n'))
        });
        let gap_before = tokens.last().map_or(Gap::Line, |last: &Token| {
            Gap::between(source.get(last.span.end..start).unwrap_or("\n"))
        });
        if let Some(last) = tokens.last_mut() {
            last.gap_after = gap_before;
        }
        tokens.push(Token {
            lexeme,
            span: Span::new(start, position),
            space_before,
            space_after,
            gap_before,
            gap_after: Gap::Line,
            text: source.get(start..position).unwrap_or("").to_string(),
            starts_a_line,
        });
    }
    tokens.push(Token {
        lexeme: Lexeme::End,
        span: Span::new(source.len(), source.len()),
        space_before: true,
        space_after: true,
        gap_before: Gap::Line,
        gap_after: Gap::Line,
        text: String::new(),
        starts_a_line: true,
    });
    Ok(tokens)
}

fn lex_text(source: &str, position: &mut usize) -> Outcome<Lexeme> {
    let start = *position;
    let rest = source.get(start + 1..).unwrap_or("");
    let mut text = String::new();
    let mut escaped = false;
    for character in rest.chars() {
        if escaped {
            escaped = false;
            if character != '\n' {
                text.push(character);
                continue;
            }
        }
        match character {
            '"' => {
                *position = start + 1 + text.len() + 1;
                return Ok(Lexeme::Text(text));
            }
            // The text keeps its `\`, since only a format print's text says what one means,
            // and an escaped `"` doesn't end it. (D194)
            '\\' => {
                text.push('\\');
                escaped = true;
            }
            '\n' => {
                return Err(Report::not_designed(
                    Span::new(start, start + 1 + text.len()),
                    "a string spanning lines",
                    "12.2",
                ));
            }
            other => text.push(other),
        }
    }
    Err(Report::refused(
        Span::new(start, start + 1 + text.len()),
        "this string never closes",
        "a string is written between two `\"` (D50)",
    ))
}

fn lex_number(source: &str, position: &mut usize) -> Outcome<Lexeme> {
    let start = *position;
    let rest = source.get(start..).unwrap_or("");
    let is_digit_part = |c: &char| c.is_ascii_digit() || *c == '_';
    let whole: String = rest.chars().take_while(is_digit_part).collect();
    let mut length = whole.len();
    let after_whole = rest.get(length..).unwrap_or("");
    let mut is_float = false;
    let mut number = whole;
    if let Some(after_dot) = after_whole.strip_prefix('.') {
        if after_dot.starts_with(|c: char| c.is_ascii_digit()) {
            let fraction: String = after_dot.chars().take_while(is_digit_part).collect();
            length += 1 + fraction.len();
            number.push('.');
            number.push_str(&fraction);
            is_float = true;
        }
    }
    // trace: D165
    let after_number = source.get(start + length..).unwrap_or("");
    if let Some(power) = after_number.strip_prefix('e') {
        let digits_after: String = power.chars().take_while(char::is_ascii_digit).collect();
        let sign = power.chars().next().filter(|c| *c == '-' || *c == '+');
        let signed_digits = sign.map_or(0, |_| {
            power
                .get(1..)
                .unwrap_or("")
                .chars()
                .take_while(char::is_ascii_digit)
                .count()
        });
        if sign.is_some() && signed_digits > 0 {
            let span = Span::new(start, start + length + 2 + signed_digits);
            return Err(Report::not_designed(
                span,
                if sign == Some('-') {
                    "a negative exponent"
                } else {
                    "an exponent written with `+`"
                },
                "12.2",
            ));
        }
        if !digits_after.is_empty() {
            let span = Span::new(start, start + length + 1 + digits_after.len());
            if !is_float {
                return Err(Report::not_designed(
                    span,
                    "an exponent on an integer",
                    "12.2",
                ));
            }
            *position = span.end;
            let mantissa: String = number.chars().filter(|c| *c != '_').collect();
            return Ok(Lexeme::Exponent {
                written: format!("{mantissa}e{digits_after}"),
                mantissa,
                power: digits_after,
            });
        }
    }
    let end = start + length;
    if let Some(next) = source.get(end..).and_then(|after| after.chars().next()) {
        if is_word_part(next) {
            let extra: String = source
                .get(end..)
                .unwrap_or("")
                .chars()
                .take_while(|c| is_word_part(*c))
                .collect();
            return Err(Report::not_designed(
                Span::new(start, end + extra.len()),
                format!("the number form `{number}{extra}`"),
                "",
            ));
        }
    }
    *position = end;
    let digits: String = number.chars().filter(|c| *c != '_').collect();
    Ok(if is_float {
        Lexeme::Float(digits)
    } else {
        Lexeme::Integer(digits)
    })
}

/// trace: D33
/// The program, when it parses, and every error found reading it. A spacing error doesn't stop
/// the parse, so a program can come back with errors. Any other error stops it, so it's the last.
/// trace: D207, D213
pub fn parse(source: &str) -> (Option<Program>, Vec<Report>) {
    if source.starts_with('\u{feff}') {
        let report = Report::not_designed(
            Span::new(0, 3),
            "a byte order mark at the start of a source file",
            "",
        );
        return (None, vec![*report]);
    }
    let tokens = match lex(source) {
        Ok(tokens) => tokens,
        Err(report) => return (None, vec![*report]),
    };
    let mut parser = Parser {
        tokens,
        position: 0,
        depth: 0,
        limb_short_names: Vec::new(),
        source_fills: Vec::new(),
        spacing: Vec::new(),
        line_starts: HashSet::new(),
        break_after: HashSet::new(),
        type_brackets: Vec::new(),
    };
    let result = parser.parse_program();
    // A parse that stopped judges only the breaks before what stopped it, so a break the stop
    // explains isn't refused twice.
    let limit = result
        .as_ref()
        .err()
        .map_or(source.len() + 1, |report| report.span.start);
    parser.check_line_breaks(limit);
    // Several spaces that a rule judges are that rule's refusal, so the `NotDesignedYet` saying
    // no rule covers them goes. (D213)
    let refusals: Vec<Span> = parser
        .spacing
        .iter()
        .filter(|report| report.kind == Kind::Refused)
        .map(|report| report.span)
        .collect();
    let mut errors: Vec<Report> = parser
        .spacing
        .into_iter()
        .filter(|report| {
            report.kind == Kind::Refused
                || !refusals
                    .iter()
                    .any(|span| span.start <= report.span.start && report.span.end <= span.end)
        })
        .collect();
    match result {
        Ok(program) => (Some(program), errors),
        Err(report) => {
            errors.push(*report);
            (None, errors)
        }
    }
}

/// trace: D213
const SPACING_JOINS: &str =
    "no space goes around `.`, `::`, `%`, `%%` or `@`, so every program reads the same (D213)";
const SPACING_MARKS: &str = "a mark or a bracket touches what it goes with, so `( $x )`, `- $x` and `read() ?` are refused (D23, D213)";
const SPACING_OPERATORS: &str = "a binary operator, `=` and word operators included, has one space on each side, so `1<8` is refused (D23, D213)";
const SPACING_LISTS: &str = "a comma or a label's `:` is followed by one space (D213)";

struct Parser {
    tokens: Vec<Token>,
    position: usize,
    depth: usize,
    /// Short names from `use` on a limb, with the path each stands for. (D155)
    limb_short_names: Vec<(String, String)>,
    /// Source fills read inside a type's `< >`, collected for that type. (D171)
    source_fills: Vec<SourceFill>,
    /// Spacing errors, which don't stop the parse. (D213)
    spacing: Vec<Report>,
    /// Tokens that start a line of their own: a statement, an item, a field, a closer. (D219)
    line_starts: HashSet<usize>,
    /// Tokens a line can break after, past the ones that always allow it: the last token of a
    /// binary operator, and a type's opening `<`. (D219)
    break_after: HashSet<usize>,
    /// A type's `<` and `>`, by token, since a comparison's `<` is no bracket. (D219)
    type_brackets: Vec<(usize, usize)>,
}

/// How deep brackets and blocks can nest here. Past this the oracle says so, instead of running
/// out of stack and crashing, which rule 6 forbids.
const DEPTH_LIMIT: usize = 2000;

const BLOCK_WHY: &str = "`;` ends a statement, `:` opens a block, and `end` closes it (D33)";

impl Parser {
    fn peek(&self) -> &Token {
        self.peek_at(0)
    }

    fn peek_at(&self, offset: usize) -> &Token {
        let last = self.tokens.len().saturating_sub(1);
        // `lex` always ends with `End`, so `last` is a real token, and the fallback never runs.
        self.tokens
            .get((self.position + offset).min(last))
            .unwrap_or(&END_TOKEN)
    }

    fn span(&self) -> Span {
        self.peek().span
    }

    fn previous_span(&self) -> Span {
        self.position
            .checked_sub(1)
            .and_then(|index| self.tokens.get(index))
            .map_or(self.span(), |token| token.span)
    }

    fn advance(&mut self) -> Token {
        let token = self.peek().clone();
        self.check_punctuation(self.position, &token);
        if self.position + 1 < self.tokens.len() {
            self.position += 1;
        }
        token
    }

    /// Spacing that a symbol always has, wherever it is. Operators whose role depends on where
    /// they are, like `-` and `<`, are checked where the parser knows that role. (D213)
    fn check_punctuation(&mut self, index: usize, token: &Token) {
        let Lexeme::Symbol(symbol) = token.lexeme else {
            if !matches!(token.lexeme, Lexeme::End) {
                self.check_uncovered(index, token);
            }
            return;
        };
        let touching = Some(Gap::Touching);
        let one = Some(Gap::One);
        match symbol {
            "." | "::" => self.check_gaps(index, index, touching, touching, SPACING_JOINS),
            "@" | "%" => self.check_gaps(index, index, None, touching, SPACING_JOINS),
            "(" | "[" | "~" => self.check_gaps(index, index, None, touching, SPACING_MARKS),
            // A postfix `?` is checked where it's read, since `?item` starts with one. (D234)
            ")" | "]" => self.check_gaps(index, index, touching, None, SPACING_MARKS),
            "," | ":" => self.check_gaps(index, index, None, one, SPACING_LISTS),
            "=" | ":=" => {
                self.break_after.insert(index);
                self.check_gaps(index, index, one, one, SPACING_OPERATORS);
            }
            _ => {}
        }
        self.check_uncovered(index, token);
    }

    /// Several spaces or a tab where no rule says what goes. Any rule that judges this gap
    /// refuses it first, so this only speaks where none does. (D213)
    fn check_uncovered(&mut self, index: usize, token: &Token) {
        if token.gap_before != Gap::Several {
            return;
        }
        let Some(previous) = index.checked_sub(1).and_then(|at| self.tokens.get(at)) else {
            return;
        };
        self.spacing.push(*Report::not_designed(
            Span::new(previous.span.end, token.span.start),
            "several spaces, or a tab, between two things on a line",
            "",
        ));
    }

    /// Checks the gap before the token at `first` and after the one at `last`, which together
    /// are one operator or mark, against what goes there. A refusal shows the spaced version,
    /// with the neighbours on each side it judges. (D213)
    fn check_gaps(
        &mut self,
        first: usize,
        last: usize,
        before: Option<Gap>,
        after: Option<Gap>,
        why: &'static str,
    ) {
        let (Some(opening), Some(closing)) = (self.tokens.get(first), self.tokens.get(last)) else {
            return;
        };
        let judged = |gap: Gap, want: Option<Gap>| want.filter(|_| gap != Gap::Line);
        let before = judged(opening.gap_before, before);
        let after = judged(closing.gap_after, after);
        let wrong_before = before.is_some_and(|want| want != opening.gap_before);
        let wrong_after = after.is_some_and(|want| want != closing.gap_after);
        if !wrong_before && !wrong_after {
            return;
        }
        let mut fix: String = self
            .tokens
            .get(first..=last)
            .unwrap_or_default()
            .iter()
            .map(|token| token.text.as_str())
            .collect();
        let mut span = opening.span.to(closing.span);
        if let Some(want) = before {
            if let Some(previous) = first.checked_sub(1).and_then(|at| self.tokens.get(at)) {
                fix = format!("{}{}{fix}", previous.text, want.written());
                span = previous.span.to(span);
            }
        }
        if let Some(want) = after {
            if let Some(next) = self
                .tokens
                .get(last + 1)
                .filter(|next| !matches!(next.lexeme, Lexeme::End))
            {
                fix = format!("{fix}{}{}", want.written(), next.text);
                span = span.to(next.span);
            }
        }
        self.spacing
            .push(*Report::refused(span, "this is spaced the wrong way", why).fix(fix));
    }

    /// A binary operator from the token at `first` to the last one read, with one space on
    /// each side, and a line can break after it. (D23, D213, D219)
    fn check_operator(&mut self, first: usize) {
        let last = self.position.saturating_sub(1).max(first);
        self.break_after.insert(last);
        self.check_gaps(
            first,
            last,
            Some(Gap::One),
            Some(Gap::One),
            SPACING_OPERATORS,
        );
    }

    /// The token about to be read starts a line of its own. (D219)
    fn mark_line_start(&mut self) {
        self.line_starts.insert(self.position);
    }

    /// A line breaks inside a statement only after a comma, an opening bracket or a binary
    /// operator, and a bracket that breaks its line has its closing one on its own line.
    /// Anywhere else the break is refused, with a fix that joins the line. Only breaks before
    /// tokens starting before `limit` are judged.
    /// trace: D219
    fn check_line_breaks(&mut self, limit: usize) {
        const WHY: &str = "a line breaks only after a comma, an opening bracket or a binary operator, so it visibly isn't finished (D219)";
        const WHY_CLOSING: &str =
            "a bracket that breaks its line has its closing one on its own line (D219)";
        let read = self
            .tokens
            .iter()
            .position(|token| token.span.start >= limit || matches!(token.lexeme, Lexeme::End))
            .map_or(self.tokens.len(), |at| at.saturating_sub(1));
        // Each `(` and `[` with its closing one, and each type's `< >`.
        let mut pairs: Vec<(usize, usize)> = self.type_brackets.clone();
        let mut open: Vec<usize> = Vec::new();
        for (index, token) in self.tokens.iter().enumerate().take(read + 1) {
            match token.lexeme {
                Lexeme::Symbol("(" | "[") => open.push(index),
                Lexeme::Symbol(")" | "]") => {
                    if let Some(opening) = open.pop() {
                        pairs.push((opening, index));
                    }
                }
                _ => {}
            }
        }
        let broken_closers: HashSet<usize> = pairs
            .iter()
            .filter(|(opening, _)| {
                self.tokens
                    .get(*opening)
                    .is_some_and(|token| token.gap_after == Gap::Line)
            })
            .map(|(_, closing)| *closing)
            .collect();
        for index in 1..=read {
            let (Some(previous), Some(token)) =
                (self.tokens.get(index - 1), self.tokens.get(index))
            else {
                continue;
            };
            if matches!(token.lexeme, Lexeme::End) {
                continue;
            }
            if broken_closers.contains(&index) {
                if token.gap_before != Gap::Line {
                    self.spacing.push(*Report::refused(
                        token.span,
                        format!(
                            "this `{}` goes on its own line, since its opening bracket breaks the line",
                            token.text
                        ),
                        WHY_CLOSING,
                    ));
                }
                continue;
            }
            if token.gap_before != Gap::Line {
                continue;
            }
            let allowed = self.line_starts.contains(&index)
                || matches!(&token.lexeme, Lexeme::Word(word) if matches!(word.as_str(), "end" | "case" | "else"))
                || matches!(previous.lexeme, Lexeme::Symbol(";" | "," | "(" | "["))
                || self.break_after.contains(&(index - 1));
            if allowed {
                continue;
            }
            let joined = format!(
                "{}{}{}",
                previous.text,
                join_gap(previous, token),
                token.text
            );
            self.spacing.push(
                *Report::refused(
                    previous.span.to(token.span),
                    "this line breaks where it can't",
                    WHY,
                )
                .fix(joined),
            );
        }
    }

    fn at_word(&self, word: &str) -> bool {
        matches!(&self.peek().lexeme, Lexeme::Word(found) if found == word)
    }

    fn word_at(&self, offset: usize, word: &str) -> bool {
        matches!(&self.peek_at(offset).lexeme, Lexeme::Word(found) if found == word)
    }

    fn at_symbol(&self, symbol: &str) -> bool {
        self.symbol_at(0, symbol)
    }

    fn symbol_at(&self, offset: usize, symbol: &str) -> bool {
        matches!(&self.peek_at(offset).lexeme, Lexeme::Symbol(found) if *found == symbol)
    }

    fn eat_word(&mut self, word: &str) -> bool {
        let found = self.at_word(word);
        if found {
            self.advance();
        }
        found
    }

    fn eat_symbol(&mut self, symbol: &str) -> bool {
        let found = self.at_symbol(symbol);
        if found {
            self.advance();
        }
        found
    }

    fn describe_here(&self) -> String {
        match &self.peek().lexeme {
            Lexeme::Word(word) => format!("`{word}`"),
            Lexeme::Variable(name) => format!("`${name}`"),
            Lexeme::Integer(digits) | Lexeme::Float(digits) => format!("`{digits}`"),
            Lexeme::Exponent { written, .. } => format!("`{written}`"),
            Lexeme::Source(name) => format!("the source `` `{name}` ``"),
            Lexeme::Text(_) => "a string".to_string(),
            Lexeme::Symbol(symbol) => format!("`{symbol}`"),
            Lexeme::Raw(_) => "a line of assembly".to_string(),
            Lexeme::End => "the end of the file".to_string(),
        }
    }

    fn expected(&self, what: &str) -> Box<Report> {
        Report::refused(
            self.span(),
            format!("expected {what}, but found {}", self.describe_here()),
            BLOCK_WHY,
        )
    }

    fn expect_symbol(&mut self, symbol: &'static str) -> Outcome<Span> {
        if self.at_symbol(symbol) {
            Ok(self.advance().span)
        } else {
            Err(self.expected(&format!("`{symbol}`")))
        }
    }

    fn expect_semicolon(&mut self) -> Outcome<Span> {
        if self.at_symbol(";") {
            return Ok(self.advance().span);
        }
        // The `;` belongs right after what came before, so that's the part in question.
        Err(Report::refused(
            self.previous_span(),
            format!(
                "expected `;` after this, but found {}",
                self.describe_here()
            ),
            BLOCK_WHY,
        ))
    }

    fn expect_name(&mut self, what: &str) -> Outcome<Name> {
        if let Lexeme::Word(word) = &self.peek().lexeme {
            let text = word.clone();
            let span = self.advance().span;
            Ok(Name { text, span })
        } else {
            Err(self.expected(what))
        }
    }

    fn expect_variable(&mut self, what: &str) -> Outcome<Name> {
        if let Lexeme::Variable(word) = &self.peek().lexeme {
            let text = word.clone();
            let span = self.advance().span;
            Ok(Name { text, span })
        } else {
            Err(self.expected(what))
        }
    }

    fn enter(&mut self) -> Outcome<()> {
        self.depth += 1;
        if self.depth > DEPTH_LIMIT {
            return Err(Report::not_designed(
                self.span(),
                format!("nesting deeper than the oracle's limit of {DEPTH_LIMIT}"),
                "",
            ));
        }
        Ok(())
    }

    fn leave(&mut self) {
        self.depth -= 1;
    }

    /// Reads `end` and the words that must follow it, such as `fn` and `total`. Each word can be
    /// a dotted run, as in `fn.main`. The closer repeats what it closes. (D33)
    fn expect_closer(&mut self, words: &[&str]) -> Outcome<Span> {
        let wanted = format!("end {}", words.join(" "));
        let start = self.span();
        // trace: D126
        if self.at_word("else") {
            return Err(Report::refused(
                start,
                "`else` continues an `if`, and there's none open here",
                "`else if` and `else:` continue an `if`, and one `end if` closes the chain (D126)",
            ));
        }
        if !self.eat_word("end") {
            return Err(self.expected(&format!("`{wanted}`")));
        }
        // trace: D200
        if self.at_word(PUB_WORD) {
            return Err(Report::not_designed(
                self.span(),
                "whether a closer repeats `pub`",
                "12.2",
            ));
        }
        let pieces_start = self.position;
        let mut last = start;
        for word in words {
            let found = self.parse_closer_piece();
            if found.as_deref() != Some(*word) {
                self.position = pieces_start;
                let span = self.closer_span(start, words.len());
                // trace: D121
                return Err(Report::refused(
                    span,
                    format!("this block closes with `{wanted}`"),
                    "the closer repeats all of what it closes, including its narrowings and name (D33, D121)",
                )
                .fix(wanted));
            }
            last = self.previous_span();
        }
        Ok(start.to(last))
    }

    /// `end @csup::union.untagged number`, which has a mark and a `::` in it.
    fn expect_limb_closer(&mut self, path: &str, name: &str) -> Outcome<()> {
        let wanted = format!("end {path} {name}");
        let start = self.span();
        if !self.eat_word("end") {
            return Err(self.expected(&format!("`{wanted}`")));
        }
        let closer = self.parse_path()?;
        let closer_name = self.expect_name("the name it closes")?;
        if closer.text != path || closer_name.text != name {
            return Err(Report::refused(
                start.to(closer_name.span),
                format!("this block closes with `{wanted}`"),
                "the closer repeats all of what it closes, including its narrowings and name (D33, D121)",
            )
            .fix(wanted));
        }
        Ok(())
    }

    /// The span of what was written where a closer's words belong.
    fn closer_span(&mut self, start: Span, pieces: usize) -> Span {
        let mut end = start;
        for _ in 0..pieces {
            if self.parse_closer_piece().is_none() {
                break;
            }
            end = self.previous_span();
        }
        start.to(end)
    }

    fn parse_use(&mut self) -> Outcome<Use> {
        let start = self.advance().span; // `use`
        let mut path = self.parse_path()?;
        // `std::list.%append`, a kit method named with its type. (D178)
        let mut method = false;
        let mut path_marks = 0;
        if self.at_symbol(".") && self.symbol_at(1, "%") {
            // `.%%append` keeps its second `%`, which says it changes its receiver. (D211)
            let marks = if self.symbol_at(2, "%") { 2 } else { 1 };
            if let Lexeme::Word(name) = self.peek_at(1 + marks).lexeme.clone() {
                for _ in 0..=marks {
                    self.advance();
                }
                let end = self.advance().span;
                path.text.push('.');
                path.text.push_str(&"%".repeat(marks));
                path.text.push_str(&name);
                path.span = path.span.to(end);
                method = true;
                path_marks = marks;
            }
        }
        if !self.eat_word("as") {
            // trace: D93
            return Err(Report::refused(
                path.span,
                "`use` always writes `as`",
                "`use` gives a path a shorter name, and `as` is always written (D93)",
            ));
        }
        // trace: D155
        if matches!(self.peek().lexeme, Lexeme::Symbol("@")) {
            let mark = self.peek().span;
            let Lexeme::Word(word) = self.peek_at(1).lexeme.clone() else {
                return Err(Report::refused(
                    mark,
                    "the short name has no `@`",
                    "`@` with no `::` is a compiler builtin (D155, D56)",
                ));
            };
            return Err(Report::refused(
                mark.to(self.peek_at(1).span),
                "the short name has no `@`",
                format!("`@` with no `::` is a compiler builtin, so `@{word}` looks like one (D155, D56)"),
            )
            .fix(word));
        }
        let percent_span = self.span();
        let mut short_marks = 0;
        while short_marks < 2 && self.eat_symbol("%") {
            short_marks += 1;
        }
        // trace: D150
        let quoted = matches!(self.peek().lexeme, Lexeme::Text(_));
        let short = if let Lexeme::Text(text) = self.peek().lexeme.clone() {
            let span = self.advance().span;
            Name { text, span }
        } else {
            self.expect_name("the shorter name")?
        };
        Self::refuse_a_lost_percent(path_marks, short_marks, quoted, &short, percent_span)?;
        self.expect_semicolon()?;
        if path.text.starts_with('@') {
            self.limb_short_names
                .push((short.text.clone(), path.text.clone()));
        }
        Ok(Use {
            span: start.to(self.previous_span()),
            path,
            short,
            quoted,
            method,
        })
    }

    /// A method's short name keeps its `%`, outside any quotes, and nothing else has one.
    /// trace: D178, D179
    fn refuse_a_lost_percent(
        path_marks: usize,
        short_marks: usize,
        quoted: bool,
        short: &Name,
        percent_span: Span,
    ) -> Outcome<()> {
        let bare = short.text.trim_start_matches('%');
        let mark = "%".repeat(path_marks);
        let fix = if quoted {
            format!("{mark}\"{bare}\"")
        } else {
            format!("{mark}{bare}")
        };
        let span = percent_span.to(short.span);
        let method = path_marks > 0;
        if method && (short_marks != path_marks || (quoted && short.text.starts_with('%'))) {
            return Err(Report::refused(
                span,
                format!("a method's short name keeps its `{mark}`, outside the quotes"),
                "`%` lends a receiver read-only and `%%` lends it to change, so a short name keeps which (D178, D179, D211)",
            )
            .fix(fix));
        }
        let percent = short_marks > 0;
        if !method && percent {
            return Err(Report::refused(
                span,
                "only a method's short name has `%`",
                "`%` marks a method call, where ownership isn't written (D178, D4)",
            ));
        }
        Ok(())
    }

    fn parse_closer_piece(&mut self) -> Option<String> {
        let Lexeme::Word(first) = self.peek().lexeme.clone() else {
            return None;
        };
        self.advance();
        let mut text = first;
        // `end fn my::x`. (D214)
        if text == "my" && self.at_symbol("::") {
            if let Lexeme::Word(next) = self.peek_at(1).lexeme.clone() {
                self.advance();
                self.advance();
                text = format!("my::{next}");
            }
        }
        while self.at_symbol(".") {
            if self.symbol_at(1, "%") {
                // trace: D211
                let marks = if self.symbol_at(2, "%") { 2 } else { 1 };
                let Lexeme::Word(next) = self.peek_at(1 + marks).lexeme.clone() else {
                    break;
                };
                for _ in 0..=(marks + 1) {
                    self.advance();
                }
                text.push('.');
                text.push_str(&"%".repeat(marks));
                text.push_str(&next);
                continue;
            }
            let Lexeme::Word(next) = self.peek_at(1).lexeme.clone() else {
                break;
            };
            self.advance();
            self.advance();
            text.push('.');
            text.push_str(&next);
        }
        Some(text)
    }

    /// A path with `.` and `::` in it. It stops before `.%`, `::%` and `.*`, which aren't paths.
    fn parse_path(&mut self) -> Outcome<Path> {
        let start = self.span();
        let mut text = String::new();
        if self.eat_symbol("@") {
            text.push('@');
        }
        let Lexeme::Word(first) = self.peek().lexeme.clone() else {
            return Err(self.expected("a name"));
        };
        let mut end = self.advance().span;
        text.push_str(&first);
        loop {
            let separator = if self.at_symbol(".") {
                "."
            } else if self.at_symbol("::") {
                "::"
            } else {
                break;
            };
            // `inline.@llvm::ir`: a limb's inline language, the one place a path sits inside a
            // narrowing. (D227)
            // trace: D227
            if text == "inline" && separator == "." && self.symbol_at(1, "@") {
                let Lexeme::Word(limb) = self.peek_at(2).lexeme.clone() else {
                    break;
                };
                self.advance();
                self.advance();
                end = self.advance().span;
                text.push_str(".@");
                text.push_str(&limb);
                continue;
            }
            let Lexeme::Word(next) = self.peek_at(1).lexeme.clone() else {
                break;
            };
            self.advance();
            end = self.advance().span;
            text.push_str(separator);
            text.push_str(&next);
        }
        Ok(Path {
            text,
            span: start.to(end),
        })
    }

    // ---------------------------------------------------------------------------------------------
    // The top of the file.
    // ---------------------------------------------------------------------------------------------

    fn parse_program(&mut self) -> Outcome<Program> {
        let mut program = Program {
            imports: Vec::new(),
            uses: Vec::new(),
            items: Vec::new(),
        };
        // A file's rules come first, then any tightening or loosening, then everything else.
        // trace: D201, D204
        let rules = self.parse_rules_block()?;
        while !matches!(self.peek().lexeme, Lexeme::End) {
            self.mark_line_start();
            if self.at_symbol("@") && self.rules_word_ahead().is_some() {
                return Err(self.rules_line_too_late(rules));
            }
            if self.at_word("import") {
                let span = self.span();
                // trace: D92
                if !program.items.is_empty() || !program.uses.is_empty() {
                    return Err(Report::refused(
                        span,
                        "imports go at the top of the file",
                        "imports go at the top, so a file's dependencies can be read without running anything (D92)",
                    ));
                }
                self.advance();
                // `import pkg std;` brings in both halves of a package. (D188)
                // trace: D188
                let whole = self.at_word(PKG_WORD)
                    && !matches!(self.peek_at(1).lexeme, Lexeme::Symbol(";"));
                if whole {
                    self.advance();
                }
                let limb = self.eat_symbol("@");
                if whole && limb {
                    return Err(Report::refused(
                        self.previous_span(),
                        "`pkg` names the whole package, so it takes no `@`",
                        "`import pkg std;` brings in both halves, and `@` picks only the limb half (D188, D187)",
                    ));
                }
                let mut name = self.expect_name("what to import")?;
                // `import std::io::fmt_print;` names one thing. (D198)
                // trace: D198
                while self.at_symbol("::") {
                    self.advance();
                    let next = self.expect_name("what to import")?;
                    name.text.push_str("::");
                    name.text.push_str(&next.text);
                    name.span = name.span.to(next.span);
                }
                if self.at_symbol(".") {
                    return Err(Report::not_designed(
                        self.span(),
                        "importing one narrowing, since importing a thing brings all of them",
                        "12.2",
                    ));
                }
                self.expect_semicolon()?;
                program.imports.push(Import { limb, whole, name });
            } else if self.at_word("package") && matches!(self.peek_at(1).lexeme, Lexeme::Word(_)) {
                // trace: D189
                return Err(Report::refused(
                    self.span(),
                    "a file doesn't say which package it's in",
                    "the manifest names your package, and `fn.main` already marks where a program starts (D189)",
                ));
            } else if self.at_word("use") {
                let declared = self.parse_use()?;
                program.uses.push(declared);
            } else {
                self.eat_pub()?;
                let item = self.parse_item()?;
                program.items.push(item);
            }
        }
        Ok(program)
    }

    /// `pub` in front of a declaration. A single file can only check where it goes. (D200)
    /// trace: D200
    fn eat_pub(&mut self) -> Outcome<()> {
        if !self.at_word(PUB_WORD) {
            return Ok(());
        }
        let start = self.advance().span;
        if self.at_word("fn") && self.symbol_at(1, ".") && self.word_at(2, "main") {
            return Err(Report::not_designed(
                start,
                "`pub` on `fn.main`, which needs none",
                "",
            ));
        }
        if self.at_word(PUB_WORD) {
            return Err(Report::not_designed(start, "`pub pub`", ""));
        }
        Ok(())
    }

    /// Every file sets its rules first, and there's no unsaid default. (D201)
    fn no_rules_line(span: Span) -> Box<Report> {
        Report::refused(
            span,
            "this file doesn't say its rules",
            "every file's first line is `@config(…);` or `@match_config(…);`, and the compiler's defaults are `@config(default);` (D201, D204)",
        )
        .fix_elsewhere(format!("{RULES_DEFAULT};"))
    }

    /// `@config`, `@match_config`, `@tighten_config` or `@loosen_config`, when one is next.
    fn rules_word_ahead(&self) -> Option<&'static str> {
        let Lexeme::Word(word) = &self.peek_at(1).lexeme else {
            return None;
        };
        RULES_WORDS.iter().copied().find(|found| found == word)
    }

    /// The file's first line, `@config(default);`, and any tightening or loosening straight after.
    /// Only `@config(default);` has settled contents. (D201, D204)
    /// trace: D201, D204
    fn parse_rules_block(&mut self) -> Outcome<(Span, &'static str)> {
        let first = self.rules_word_ahead().filter(|_| self.at_symbol("@"));
        let Some(word @ ("config" | "match_config")) = first else {
            return Err(Self::no_rules_line(self.span()));
        };
        let start = self.span();
        self.advance(); // `@`
        self.advance(); // the word
        self.expect_symbol("(")?;
        let inside = self.span();
        if word == "match_config" {
            return Err(Report::not_designed(
                inside,
                "how `@match_config` names the file it copies",
                "12.2",
            ));
        }
        if !(self.at_word("default") && self.symbol_at(1, ")")) {
            return Err(Report::not_designed(
                inside,
                "how a rule is spelled, and the list of rules",
                "12.2",
            ));
        }
        self.advance(); // `default`
        self.expect_symbol(")")?;
        self.expect_semicolon()?;
        let rules = (start.to(self.previous_span()), word);
        if self.at_symbol("@")
            && matches!(
                self.rules_word_ahead(),
                Some("tighten_config" | "loosen_config")
            )
        {
            self.advance(); // `@`
            self.advance(); // the word
            self.expect_symbol("(")?;
            return Err(Report::not_designed(
                self.span(),
                "how a rule is spelled, and the list of rules",
                "12.2",
            ));
        }
        Ok(rules)
    }

    /// A rules line anywhere but the top. (D204, D201)
    fn rules_line_too_late(&self, rules: (Span, &'static str)) -> Box<Report> {
        let word = self.rules_word_ahead().unwrap_or("config");
        let (earlier, earlier_word) = rules;
        if matches!(word, "config" | "match_config") && word != earlier_word {
            return Report::refused(
                self.span(),
                "a file sets its first rules with `@config` or `@match_config`, not both",
                "every file starts with one of the two (D201)",
            )
            .because(earlier, "they're set here");
        }
        Report::refused(
            self.span(),
            format!("`@{word}` goes at the top, with the file's other rules"),
            "a file's rules come first, and tightening or loosening follows straight after (D204)",
        )
        .because(earlier, "the file's rules are here")
    }

    fn parse_item(&mut self) -> Outcome<Item> {
        self.mark_line_start();
        // trace: D236
        if self.at_word("op") {
            return Ok(Item::Function(self.parse_operator()?));
        }
        if self.at_word("fn") {
            return Ok(Item::Function(self.parse_function()?));
        }
        if self.at_word("struct") {
            return Ok(Item::Struct(self.parse_struct()?));
        }
        if self.at_word("union") {
            return Ok(Item::Union(self.parse_union()?));
        }
        if self.at_word("fam") {
            return self.parse_family();
        }
        if self.at_word("type") {
            return self.parse_type_name();
        }
        // trace: D135
        if self.at_word("narrowings") && self.symbol_at(1, ".") {
            let start = self.span();
            let sort = match &self.peek_at(2).lexeme {
                Lexeme::Word(word) => word.clone(),
                _ => String::new(),
            };
            return Err(Report::refused(
                start.to(self.peek_at(2).span),
                "a declared group of related names is a family",
                "`fam.` replaces `narrowings.`, and narrowing now only means the dot (D135)",
            )
            .fix(format!("fam.{sort}")));
        }
        if self.at_word("together") {
            return Ok(Item::Together(self.parse_together()?));
        }
        if self.at_word("const") {
            return Ok(Item::Constant(self.parse_constant()?));
        }
        if self.at_word("shield") {
            return Ok(Item::Shield(self.parse_shield_item()?));
        }
        // trace: D235
        if self.at_word("interface") {
            return Ok(Item::Interface(self.parse_interface()?));
        }
        if self.at_word("type_set") {
            return Ok(Item::TypeSet(self.parse_type_set()?));
        }
        if self.word_at(1, "satisfies") || self.symbol_at(1, ".|.") {
            return Ok(Item::Satisfies(self.parse_satisfies()?));
        }
        if self.word_at(1, "is") {
            return Err(Report::refused(
                self.peek_at(1).span,
                "a type fits an interface with `satisfies`",
                "a type fits an interface only through a `satisfies` line (D235, D87)",
            )
            .fix("satisfies"));
        }
        // trace: D56
        if self.at_symbol("@") && self.word_at(1, "import") {
            return Err(Report::refused(
                self.span().to(self.peek_at(1).span),
                "there is no `@import`",
                "importing is a plain keyword (D56, D92)",
            )
            .fix("import"));
        }
        // trace: D38
        if self.at_symbol("@") {
            let path = self.parse_path()?;
            if path.text == "@csup::union.untagged" {
                return Err(Report::refused(
                    path.span,
                    "`@csup::union.untagged` must be inside a shield",
                    "a shield is the only place dangerous code may live (D34, D38)",
                ));
            }
            return Err(Report::not_designed(
                path.span,
                format!("`{}` at the top of a file", path.text),
                "12.9",
            ));
        }
        Err(self.expected("`fn`, `struct`, `union.tagged`, `type`, `fam`, `const` or `shield`"))
    }

    fn parse_function(&mut self) -> Outcome<Function> {
        let header_first = self.position;
        let start = self.advance().span; // `fn`
        let mut narrowing = String::new();
        while self.at_symbol(".") {
            self.advance();
            let piece = self.expect_name("a narrowing of `fn`")?;
            narrowing.push('.');
            narrowing.push_str(&piece.text);
        }
        // `pub` says who can see a thing, not what kind it is, so it's no narrowing. (D200)
        // trace: D200
        if let Some(rest) = narrowing.strip_prefix(".pub") {
            return Err(Report::refused(
                start.to(self.previous_span()),
                "`pub` goes in front, as a word of its own",
                "`pub` says who can see a thing, not what kind of thing it is, so it isn't a narrowing (D200)",
            )
            .fix(format!("pub fn{rest}")));
        }
        let kind = match narrowing.as_str() {
            "" => FunctionKind::Plain,
            ".main" => FunctionKind::Main,
            ".pure" => FunctionKind::Pure,
            ".pure.baked" => FunctionKind::PureBaked,
            // trace: D223
            ".baked.pure" => {
                return Err(Report::refused(
                    start.to(self.previous_span()),
                    "narrowings go in a fixed order, and this one is `fn.pure.baked`",
                    "narrowings go strongest first in a fixed order, and `fn.pure.baked` is how this one is spelled (D223, D91)",
                )
                .fix("fn.pure.baked"));
            }
            other => {
                return Err(Report::not_designed(
                    start.to(self.previous_span()),
                    format!("`fn{other}`"),
                    "",
                ));
            }
        };
        let name = if kind == FunctionKind::Main {
            Name {
                text: "fn.main".to_string(),
                span: start.to(self.previous_span()),
            }
        } else {
            self.parse_function_name()?
        };
        let type_parameters = self.parse_type_parameters()?;
        let sources = self.parse_source_declarations()?;
        let parameters = self.parse_parameters()?;
        self.expect_symbol("->")?;
        let return_type = self.parse_type()?;
        let header_end = self.previous_span();
        // trace: D52
        if self.at_word("or") && self.word_at(1, "fail") {
            return Err(Report::refused(
                self.span(),
                "`or fail[…]` is the older spelling",
                "a function lists its failures in its header, after its return type, with `fails` (D52, D209)",
            ));
        }
        // A function's failures are listed after its return type. (D209)
        // trace: D209
        let header_fails = if self.at_word("fails") {
            Some(self.parse_failure_list()?)
        } else {
            None
        };
        let header_last = self.position.saturating_sub(1);
        self.expect_symbol(":")?;
        let mut function = Function {
            kind,
            operator: None,
            type_parameters,
            span: start.to(header_end),
            name,
            parameters,
            return_type,
            labels: None,
            defaults: Vec::new(),
            requires: Vec::new(),
            lasts: Vec::new(),
            sources,
            ensures: Vec::new(),
            fails: None,
            body: Vec::new(),
            end_span: start,
        };
        function.fails = header_fails;
        self.parse_top_of_body(&mut function, (header_first, header_last))?;
        function.body = self.parse_block()?;
        function.end_span = self.parse_function_closer(kind, &function.name.text)?;
        Ok(function)
    }

    /// A plain name, or a method's type and name, as in `reading.%warm_up`. (D130)
    /// A declared name, written `my::x` when it clashes, there as everywhere. (D214, D75)
    /// trace: D214
    fn expect_declared_name(&mut self, keyword: &str, what: &str) -> Outcome<(Name, String)> {
        if self.at_word("my") && self.symbol_at(1, "::") {
            let start = self.advance().span;
            self.advance();
            let name = self.expect_name(what)?;
            if !is_clashing(&name.text) {
                return Err(Report::not_designed(
                    start.to(name.span),
                    format!("`my::` on `{}`, which doesn't clash", name.text),
                    "",
                ));
            }
            let written = format!("my::{}", name.text);
            return Ok((
                Name {
                    text: name.text,
                    span: start.to(name.span),
                },
                written,
            ));
        }
        let keyword_span = self.previous_span();
        let name = self.expect_name(what)?;
        if keyword == "fn" && name.text == "main" {
            return Err(Report::refused(
                keyword_span.to(name.span),
                "a program starts at `fn.main`, and a function named `main` is `fn my::main`",
                "`main` clashes, so a plain `fn main` is refused (D214, D11)",
            )
            .fix("fn.main"));
        }
        if is_clashing(&name.text) {
            return Err(Report::refused(
                name.span,
                format!("`{}` clashes, so its declaration says `my::` too", name.text),
                "a clashing name is written `my::x` everywhere, its declaration and closer included (D214, D75)",
            )
            .fix(format!("my::{}", name.text)));
        }
        let written = name.text.clone();
        Ok((name, written))
    }

    fn parse_function_name(&mut self) -> Outcome<Name> {
        if !self.symbol_at(1, ".") {
            let (name, _) = self.expect_declared_name("fn", "the function's name")?;
            return Ok(name);
        }
        let name = self.expect_name("the function's name")?;
        if !self.at_symbol(".") {
            return Ok(name);
        }
        // trace: D130
        if self.symbol_at(1, "%") {
            self.advance();
            self.advance();
            // `reading.%%warm_up` says its receiver is lent to change. (D211)
            // trace: D211
            let mark = if self.eat_symbol("%") { "%%" } else { "%" };
            let method = self.expect_name("the method's name")?;
            return Ok(Name {
                text: format!("{}.{mark}{}", name.text, method.text),
                span: name.span.to(method.span),
            });
        }
        self.advance();
        let narrowing = self.expect_name("a method's `%`")?;
        if self.at_symbol(".") {
            return Err(Report::not_designed(
                name.span.to(narrowing.span),
                "a method on a type from a family",
                "",
            ));
        }
        Err(Report::refused(
            name.span.to(narrowing.span),
            "a method's name has its `%`",
            "a method is declared with its type in the name, as in `reading.%warm_up` (D130)",
        )
        .fix(format!("{}.%{}", name.text, narrowing.text)))
    }

    fn parse_function_closer(&mut self, kind: FunctionKind, name: &str) -> Outcome<Span> {
        // trace: D214
        let written = if is_clashing(name) {
            format!("my::{name}")
        } else {
            name.to_string()
        };
        let name = written.as_str();
        match kind {
            FunctionKind::Main => self.expect_closer(&["fn.main"]),
            FunctionKind::Plain => self.expect_closer(&["fn", name]),
            // trace: D121
            FunctionKind::Pure => self.expect_closer(&["fn.pure", name]),
            FunctionKind::PureBaked => self.expect_closer(&["fn.pure.baked", name]),
        }
    }

    fn parse_parameters(&mut self) -> Outcome<Vec<Parameter>> {
        self.expect_symbol("[")?;
        let mut parameters = Vec::new();
        while !self.at_symbol("]") {
            let name = self.expect_variable("a parameter, starting with `$`")?;
            Self::refuse_infer_here(self.at_symbol(":="), self.span(), "a parameter")?;
            self.expect_symbol(":")?;
            let word_span = self.span();
            let word = if self.at_word("Given") {
                self.advance();
                ParameterWord::Given
            } else if self.at_word("Lent") && self.symbol_at(1, ".") && self.word_at(2, "mut") {
                self.advance();
                self.advance();
                self.advance();
                ParameterWord::LentMut
            } else if self.at_word("Lent") {
                // trace: D2
                return Err(Report::refused(
                    word_span,
                    "`Lent` alone is written bare",
                    "a bare parameter is lent, so writing `Lent` alone is refused (D2)",
                ));
            } else if self.at_word("Copy") {
                // trace: D2
                return Err(Report::refused(
                    word_span,
                    "there's no parameter word for `Copy`",
                    "`Copy` hands over a separate copy, to a `Given` parameter (D2)",
                )
                .fix("Given"));
            } else if self.at_word("Give") {
                // trace: D2
                return Err(Report::refused(
                    word_span,
                    "`Give` is the call's word",
                    "`Give` goes at the call, and `Given` on the parameter (D2)",
                )
                .fix("Given"));
            } else {
                ParameterWord::Bare
            };
            let written_type = self.parse_type()?;
            parameters.push(Parameter {
                name,
                word,
                word_span,
                written_type,
            });
            if !self.eat_symbol(",") {
                break;
            }
        }
        self.expect_symbol("]")?;
        Ok(parameters)
    }

    /// `labels`, `defaults`, `requires`, `ensures` and `fails` go first, in that order. (D114)
    fn parse_top_of_body(
        &mut self,
        function: &mut Function,
        header: (usize, usize),
    ) -> Outcome<()> {
        let mut phase = 0;
        loop {
            self.mark_line_start();
            let (word_phase, word) = if self.at_word("labels") {
                (1, "labels")
            } else if self.at_word("defaults") {
                (2, "defaults")
            } else if self.at_word("requires") {
                (3, "requires")
            } else if self.at_word("ensures") {
                (4, "ensures")
            } else if self.at_word("fails") {
                (5, "fails")
            } else {
                return Ok(());
            };
            // trace: D114
            if word_phase < phase {
                return Err(Report::refused(
                    self.span(),
                    format!("`{word}` is out of order"),
                    "the top of a body goes `labels`, `defaults`, `requires`, `ensures`, then the code (D114, D209)",
                ));
            }
            if word_phase == phase && word_phase != 3 && word_phase != 4 {
                return Err(Report::refused(
                    self.span(),
                    format!("`{word}` is written once"),
                    "the top of a body goes `labels`, `defaults`, `requires`, `ensures`, then the code (D114, D209)",
                ));
            }
            phase = word_phase;
            match word_phase {
                1 => function.labels = Some(self.parse_labels()?),
                2 => function.defaults = self.parse_defaults()?,
                // trace: D171
                3 if matches!(self.peek_at(1).lexeme, Lexeme::Variable(_))
                    && self.word_at(2, LASTS_AT_LEAST) =>
                {
                    let start = self.advance().span; // `requires`
                    let longer = self.expect_variable("a parameter")?;
                    self.advance(); // `lasts_at_least`
                    let shorter = self.expect_variable("a parameter")?;
                    self.expect_semicolon()?;
                    function.lasts.push(LastsAtLeast {
                        longer,
                        shorter,
                        span: start.to(self.previous_span()),
                    });
                }
                3 => {
                    let contract = self.parse_contract()?;
                    function.requires.push(contract);
                }
                4 => {
                    let contract = self.parse_contract()?;
                    function.ensures.push(contract);
                }
                // trace: D209
                _ => {
                    let at = self.span();
                    let first = self.position;
                    self.parse_fails()?;
                    let listed = self.tokens_text(first, self.position.saturating_sub(2));
                    let header_text = self.tokens_text(header.0, header.1);
                    return Err(Report::refused(
                        at,
                        "`fails` goes in the header, after the return type",
                        "the header says a function can fail, so its failures are listed there (D209)",
                    )
                    .fix_elsewhere(format!("{header_text} {listed}:")));
                }
            }
        }
    }

    fn parse_labels(&mut self) -> Outcome<LabelLine> {
        let label_start = self.advance().span; // `labels`
        let mut any_order = false;
        if self.at_symbol(".") {
            self.advance();
            let narrowing = self.expect_name("a narrowing of `labels`")?;
            match narrowing.text.as_str() {
                "any_order" => any_order = true,
                // trace: D114
                "in_order" => {
                    return Err(Report::refused(
                        label_start.to(narrowing.span),
                        "keeping labels in order is the default",
                        "a default the language fixes can't be written out (D51, D114)",
                    )
                    .fix("labels"));
                }
                other => {
                    return Err(Report::not_designed(
                        narrowing.span,
                        format!("`labels.{other}`"),
                        "",
                    ));
                }
            }
        }
        self.expect_symbol("[")?;
        let mut entries = Vec::new();
        while !self.at_symbol("]") {
            let parameter = self.expect_variable("a parameter")?;
            if !self.eat_word("as") {
                return Err(self.expected("`as`"));
            }
            // trace: D144
            let label = self.parse_path()?;
            let label = Name {
                text: label.text,
                span: label.span,
            };
            entries.push((parameter, label));
            if !self.eat_symbol(",") {
                break;
            }
        }
        self.expect_symbol("]")?;
        self.expect_semicolon()?;
        Ok(LabelLine { any_order, entries })
    }

    fn parse_defaults(&mut self) -> Outcome<Vec<(Name, Expression)>> {
        self.advance(); // `defaults`
        self.expect_symbol("[")?;
        let mut entries = Vec::new();
        while !self.at_symbol("]") {
            let parameter = self.expect_variable("a parameter")?;
            self.expect_symbol("=")?;
            let value = self.parse_expression()?;
            entries.push((parameter, value));
            if !self.eat_symbol(",") {
                break;
            }
        }
        self.expect_symbol("]")?;
        self.expect_semicolon()?;
        Ok(entries)
    }

    fn parse_contract(&mut self) -> Outcome<Contract> {
        self.advance(); // `requires` or `ensures`
        if self.at_symbol(".") {
            self.advance();
            let narrowing = self.expect_name("a narrowing of the contract")?;
            // The oracle treats every check as on, as in `Working`, so this changes nothing here.
            // trace: D106
            if narrowing.text != "only_in_working" {
                return Err(Report::not_designed(
                    narrowing.span,
                    format!("the contract narrowing `{}`", narrowing.text),
                    "",
                ));
            }
        }
        let condition = self.parse_expression()?;
        self.expect_semicolon()?;
        Ok(Contract { condition })
    }

    fn parse_fails(&mut self) -> Outcome<Vec<FailureEntry>> {
        let entries = self.parse_failure_list()?;
        self.expect_semicolon()?;
        Ok(entries)
    }

    /// `fails [odd, negative[val: i32]]`. (D52, D209)
    fn parse_failure_list(&mut self) -> Outcome<Vec<FailureEntry>> {
        self.advance(); // `fails`
        self.expect_symbol("[")?;
        let mut entries = Vec::new();
        while !self.at_symbol("]") {
            entries.push(self.parse_failure_entry()?);
            if !self.eat_symbol(",") {
                break;
            }
        }
        self.expect_symbol("]")?;
        Ok(entries)
    }

    fn parse_failure_entry(&mut self) -> Outcome<FailureEntry> {
        let path = self.parse_path()?;
        let data = if self.at_symbol("[") {
            self.parse_field_declarations_in_brackets()?
        } else {
            Vec::new()
        };
        Ok(FailureEntry { path, data })
    }

    fn parse_field_declarations_in_brackets(&mut self) -> Outcome<Vec<FieldDeclaration>> {
        self.expect_symbol("[")?;
        let mut fields = Vec::new();
        while !self.at_symbol("]") {
            let name = self.expect_name("a field's name")?;
            self.expect_symbol(":")?;
            let written_type = self.parse_type()?;
            fields.push(FieldDeclaration { name, written_type });
            if !self.eat_symbol(",") {
                break;
            }
        }
        self.expect_symbol("]")?;
        Ok(fields)
    }

    fn parse_struct(&mut self) -> Outcome<StructDeclaration> {
        self.advance(); // `struct`
        let mut must_give = false;
        if self.at_symbol(".") {
            self.advance();
            let narrowing = self.expect_name("a narrowing of `struct`")?;
            if narrowing.text == "must_Give" {
                must_give = true;
            } else {
                return Err(Report::not_designed(
                    narrowing.span,
                    format!("`struct.{}`", narrowing.text),
                    "",
                ));
            }
        }
        let (name, written) = self.expect_declared_name("struct", "the struct's name")?;
        let type_parameters = self.parse_type_parameters()?;
        let sources = self.parse_source_declarations()?;
        self.expect_symbol(":")?;
        let mut fields = Vec::new();
        while !self.at_word("end") {
            self.mark_line_start();
            let field_name = self.expect_name("a field, with no `$`")?;
            Self::refuse_infer_here(self.at_symbol(":="), self.span(), "a struct field")?;
            self.expect_symbol(":")?;
            let written_type = self.parse_type()?;
            self.expect_semicolon()?;
            fields.push(FieldDeclaration {
                name: field_name,
                written_type,
            });
        }
        if must_give {
            self.expect_closer(&["struct.must_Give", &written])?;
        } else {
            self.expect_closer(&["struct", &written])?;
        }
        Ok(StructDeclaration {
            must_give,
            name,
            type_parameters,
            sources,
            fields,
        })
    }

    fn parse_union(&mut self) -> Outcome<UnionDeclaration> {
        let start = self.advance().span; // `union`
                                         // trace: D33
        if !self.at_symbol(".") {
            return Err(Report::refused(
                start,
                "there's no bare `union`",
                "\"union\" means opposite things to a C programmer and a Rust one, so it says which (D33)",
            )
            .fix("union.tagged"));
        }
        self.advance();
        let narrowing = self.expect_name("a narrowing of `union`")?;
        match narrowing.text.as_str() {
            "tagged" => {}
            // trace: D35
            "untagged" => {
                return Err(Report::refused(
                    start.to(narrowing.span),
                    "C's untagged union is `@csup::union.untagged`, inside a shield",
                    "C support is the limb `@csup`, and that union must be inside a shield (D35, D38)",
                ));
            }
            other => {
                return Err(Report::not_designed(
                    narrowing.span,
                    format!("`union.{other}`"),
                    "",
                ));
            }
        }
        let (name, written) = self.expect_declared_name("union.tagged", "the union's name")?;
        let sources = self.parse_source_declarations()?;
        self.expect_symbol(":")?;
        let mut cases = Vec::new();
        while !self.at_word("end") {
            self.mark_line_start();
            let case_name = self.expect_name("a case")?;
            let fields = if self.at_symbol("[") {
                self.parse_field_declarations_in_brackets()?
            } else {
                Vec::new()
            };
            // A member line ends with `;`, like a struct's fields. (D208)
            // trace: D208
            self.expect_semicolon()?;
            cases.push(UnionCase {
                name: case_name,
                fields,
            });
        }
        self.expect_closer(&["union.tagged", &written])?;
        Ok(UnionDeclaration {
            name,
            sources,
            cases,
        })
    }

    /// A family: a declared group of related names. (D135)
    /// trace: D135
    fn parse_family(&mut self) -> Outcome<Item> {
        let start = self.advance().span; // `fam`
        if !self.at_symbol(".") {
            return Err(self.expected("`.failure`, `.handler`, `.unit` or `.type`"));
        }
        self.advance();
        let sort = self.expect_name("`failure`, `handler`, `unit` or `type`")?;
        match sort.text.as_str() {
            "failure" => self.parse_failure_set(),
            "handler" => self.parse_handler_set(),
            "unit" => self.parse_unit_set(),
            "type" => {
                let new = self.parse_type_kind(start)?;
                self.parse_type_family(new)
            }
            other => Err(Report::not_designed(
                start.to(sort.span),
                format!("`fam.{other}`"),
                "",
            )),
        }
    }

    /// `.alias` or `.new` after `type`, saying whether this makes a different type. (D134)
    fn parse_type_kind(&mut self, start: Span) -> Outcome<bool> {
        if !self.at_symbol(".") {
            return Err(Report::refused(
                start.to(self.previous_span()),
                "there's no bare `type`, so pick `type.alias` or `type.new`",
                "`type.alias` is another name and `type.new` a different type, and no default is obvious (D134)",
            ));
        }
        self.advance();
        let kind = self.expect_name("`alias` or `new`")?;
        match kind.text.as_str() {
            "alias" => Ok(false),
            "new" => Ok(true),
            other => Err(Report::not_designed(
                start.to(kind.span),
                format!("`type.{other}`"),
                "",
            )),
        }
    }

    /// trace: D134
    fn parse_type_name(&mut self) -> Outcome<Item> {
        let start = self.advance().span; // `type`
        let new = self.parse_type_kind(start)?;
        let name = self.parse_path()?;
        self.expect_symbol("=")?;
        let written_type = self.parse_type()?;
        self.expect_semicolon()?;
        Ok(Item::TypeName(TypeName {
            new,
            family: None,
            name,
            written_type,
        }))
    }

    /// `fam.type.new ids:` with lines like `ids.user = i64;`, always used with the family.
    /// trace: D134
    fn parse_type_family(&mut self, new: bool) -> Outcome<Item> {
        let family = self.expect_name("the family's name")?;
        self.expect_symbol(":")?;
        let kind = if new {
            "fam.type.new"
        } else {
            "fam.type.alias"
        };
        let mut members = Vec::new();
        while !self.at_word("end") {
            self.mark_line_start();
            if self.at_word("type") {
                return Err(Report::refused(
                    self.span(),
                    "a family of types states its kind once",
                    "a family doesn't mix `type.alias` and `type.new` (D134)",
                ));
            }
            let name = self.parse_path()?;
            let prefix = format!("{}.", family.text);
            if !name.text.starts_with(&prefix) {
                return Err(Report::refused(
                    name.span,
                    format!("this type is in the family `{}`", family.text),
                    "a family's types are always used with the family (D134)",
                )
                .fix(format!("{prefix}{}", name.text)));
            }
            Self::refuse_a_deep_family(&family, &name)?;
            self.expect_symbol("=")?;
            let written_type = self.parse_type()?;
            self.expect_semicolon()?;
            members.push(TypeName {
                new,
                family: Some(family.text.clone()),
                name,
                written_type,
            });
        }
        self.expect_closer(&[kind, &family.text])?;
        Ok(Item::TypeFamily(members))
    }

    fn parse_failure_set(&mut self) -> Outcome<Item> {
        let name = self.expect_name("the set's name")?;
        self.expect_symbol(":")?;
        let mut members = Vec::new();
        while !self.at_word("end") {
            self.mark_line_start();
            let entry = self.parse_failure_entry()?;
            Self::refuse_a_deep_family(&name, &entry.path)?;
            members.push(entry);
            self.expect_semicolon()?;
        }
        self.expect_closer(&["fam.failure", &name.text])?;
        Ok(Item::FailureSet(FailureSet { name, members }))
    }

    fn parse_handler_set(&mut self) -> Outcome<Item> {
        let name = self.expect_name("the set's name")?;
        self.expect_symbol(":")?;
        let mut handlers = Vec::new();
        while !self.at_word("end") {
            self.mark_line_start();
            let family = self.expect_name("the family")?;
            let member = if self.eat_symbol(".") {
                self.expect_name("the handler's name")?
            } else {
                return Err(self.expected("`.` and the handler's name"));
            };
            self.expect_symbol(":")?;
            let arms = self.parse_arms()?;
            let closer = format!("{}.{}", family.text, member.text);
            self.expect_closer(&[&closer])?;
            if family.text != name.text {
                return Err(Report::refused(
                    family.span,
                    format!("this handler is in the set `{}`", name.text),
                    "a handler always says its family (D46, D112)",
                )
                .fix(format!("{}.{}", name.text, member.text)));
            }
            handlers.push(Handler { name: member, arms });
        }
        self.expect_closer(&["fam.handler", &name.text])?;
        Ok(Item::HandlerSet(HandlerSet { name, handlers }))
    }

    /// A family is one level deep, so a family can't hold a family.
    /// trace: D161
    fn refuse_a_deep_family(family: &Name, member: &Path) -> Outcome<()> {
        let prefix = format!("{}.", family.text);
        match member.text.strip_prefix(&prefix) {
            Some(rest) if rest.contains('.') => Err(Report::refused(
                member.span,
                format!(
                    "`{}` goes more than one level into `{}`",
                    member.text, family.text
                ),
                "a family is one level deep, so a family can't hold a family (D161)",
            )),
            _ => Ok(()),
        }
    }

    fn parse_unit_set(&mut self) -> Outcome<Item> {
        let family = self.expect_name("the family's name")?;
        self.expect_symbol(":")?;
        let mut members = Vec::new();
        while !self.at_word("end") {
            self.mark_line_start();
            let path = self.parse_path()?;
            Self::refuse_a_deep_family(&family, &path)?;
            let meaning = if self.eat_symbol(":") {
                if self.eat_word("base") {
                    UnitMeaning::Base
                } else {
                    return Err(self.expected("`base`"));
                }
            } else if self.eat_symbol("=") {
                let start = self.span();
                self.parse_primary_number()?;
                let Some(unit) = self.parse_unit_after_number()? else {
                    return Err(Report::not_designed(
                        start.to(self.previous_span()),
                        "a conversion that isn't a number and a unit",
                        "12.2",
                    ));
                };
                if !self.at_symbol(";") {
                    return Err(Report::not_designed(
                        start.to(self.span()),
                        "how a conversion other than a multiple is written",
                        "12.2",
                    ));
                }
                UnitMeaning::Conversion { unit }
            } else {
                UnitMeaning::Unsaid
            };
            self.expect_semicolon()?;
            members.push(UnitMember { path, meaning });
        }
        self.expect_closer(&["fam.unit", &family.text])?;
        Ok(Item::UnitSet(UnitSet { family, members }))
    }

    fn parse_together(&mut self) -> Outcome<Together> {
        self.advance(); // `together`
        let left = self.parse_path()?;
        let operator = if self.eat_symbol("/") {
            TogetherOperator::Divide
        } else if self.eat_word("x") {
            TogetherOperator::Multiply
        } else {
            return Err(self.expected("`/` or `x`"));
        };
        let right = self.parse_path()?;
        self.expect_symbol("=")?;
        let result = self.parse_path()?;
        self.expect_semicolon()?;
        Ok(Together {
            left,
            operator,
            right,
            result,
        })
    }

    fn parse_constant(&mut self) -> Outcome<Statement> {
        let start = self.advance().span; // `const`
        let name = self.expect_variable("the constant's name, starting with `$`")?;
        // trace: D216
        if self.eat_symbol(":=") {
            return self.parse_inferred(start, true, false, name);
        }
        self.expect_symbol(":")?;
        let written_type = self.parse_type()?;
        Self::refuse_typed_infer(self.at_symbol(":="), self.span())?;
        self.expect_symbol("=")?;
        let value = self.parse_expression()?;
        let end = self.expect_semicolon()?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::Constant {
                name,
                written_type,
                value,
            },
        })
    }

    fn parse_shield_header(&mut self) -> Outcome<(Name, Option<String>)> {
        self.advance(); // `shield`
        let name = self.expect_name("the shield's name")?;
        self.expect_symbol(":")?;
        let mut reason = None;
        self.mark_line_start();
        if self.eat_word("reason") {
            if let Lexeme::Text(text) = self.peek().lexeme.clone() {
                self.advance();
                reason = Some(text);
            } else {
                return Err(self.expected("the reason, as a string"));
            }
        }
        Ok((name, reason))
    }

    fn parse_shield_item(&mut self) -> Outcome<ShieldItem> {
        let (name, reason) = self.parse_shield_header()?;
        let mut items = Vec::new();
        while !self.at_word("end") {
            self.mark_line_start();
            // A short name from `use` stands in for the limb's whole path, here as anywhere.
            // trace: D155
            let short_limb = match &self.peek().lexeme {
                Lexeme::Word(word) => self
                    .limb_short_names
                    .iter()
                    .find(|(short, _)| short == word)
                    .cloned(),
                _ => None,
            };
            if self.at_symbol("@") || short_limb.is_some() {
                let written = self.parse_path()?;
                let path = match &short_limb {
                    Some((_, whole)) => Path {
                        text: whole.clone(),
                        span: written.span,
                    },
                    None => written.clone(),
                };
                let union_name = self.expect_name("the union's name")?;
                self.expect_symbol(":")?;
                // The fields are only read past: nothing can use them until C's layouts exist.
                while !self.at_word("end") {
                    self.mark_line_start();
                    self.expect_name("a field")?;
                    self.expect_symbol(":")?;
                    self.parse_type()?;
                    // trace: D208
                    self.expect_semicolon()?;
                }
                self.expect_limb_closer(&written.text, &union_name.text)?;
                items.push(ShieldContent::UntaggedUnion { path });
            } else {
                items.push(ShieldContent::Item(Box::new(self.parse_item()?)));
            }
        }
        self.expect_closer(&["shield", &name.text])?;
        Ok(ShieldItem {
            name,
            reason,
            items,
        })
    }

    // ---------------------------------------------------------------------------------------------
    // Types.
    // ---------------------------------------------------------------------------------------------

    fn parse_type(&mut self) -> Outcome<TypeExpression> {
        self.enter()?;
        let result = self.parse_type_inner();
        self.leave();
        result
    }

    fn parse_type_inner(&mut self) -> Outcome<TypeExpression> {
        let start = self.span();
        // trace: D234
        if self.at_symbol("?") {
            let name = self.parse_parameter_name()?;
            return Ok(TypeExpression {
                span: name.span,
                kind: TypeKind::Parameter(name),
            });
        }
        if self.at_word("Lend") {
            return self.parse_stored_borrow();
        }
        let mut path = self.parse_path()?;
        // `f64.?u`: a unit parameter, where a unit goes. (D243)
        // trace: D243
        if self.at_symbol(".") && self.symbol_at(1, "?") {
            self.advance();
            let name = self.parse_parameter_name()?;
            path.text.push_str(".?");
            path.text.push_str(&name.text);
            path.span = path.span.to(name.span);
        }
        let mut fill = Vec::new();
        let outer_fills = std::mem::take(&mut self.source_fills);
        if self.at_symbol("<") {
            fill = self.parse_fill()?;
        }
        let sources = std::mem::replace(&mut self.source_fills, outer_fills);
        let mut describe = Vec::new();
        if self.at_symbol("[") {
            self.advance();
            while !self.at_symbol("]") {
                let label = self.parse_path()?;
                if self.at_symbol("=") {
                    return Err(Report::refused(
                        self.span(),
                        "a type's `[ ]` names its values with `:`",
                        "a label names a value inside brackets, and `=` sets a field (D13, D57)",
                    ));
                }
                self.expect_symbol(":")?;
                let value = self.parse_compare()?;
                describe.push(Describe { label, value });
                if !self.eat_symbol(",") {
                    break;
                }
            }
            self.expect_symbol("]")?;
        }
        Ok(TypeExpression {
            span: start.to(self.previous_span()),
            kind: TypeKind::Named {
                path,
                fill,
                describe,
                sources,
            },
        })
    }

    /// ``<`text`, `table`: lasts_at_least `text`>`` after a struct's, union's or function's name.
    /// trace: D171
    fn parse_source_declarations(&mut self) -> Outcome<Vec<SourceDeclaration>> {
        if !self.at_symbol("<") {
            return Ok(Vec::new());
        }
        if !matches!(self.peek_at(1).lexeme, Lexeme::Source(_)) {
            return Err(Report::not_designed(
                self.span(),
                "the spelling of generics",
                "12.5",
            ));
        }
        self.advance(); // `<`
        let mut sources = Vec::new();
        while !self.at_symbol(">") {
            let name = self.expect_source()?;
            let mut lasts_at_least = Vec::new();
            if self.eat_symbol(":") {
                if !self.eat_word(LASTS_AT_LEAST) {
                    return Err(self.expected("`lasts_at_least`"));
                }
                lasts_at_least.push(self.expect_source()?);
            }
            sources.push(SourceDeclaration {
                name,
                lasts_at_least,
            });
            if !self.eat_symbol(",") {
                break;
            }
        }
        self.expect_symbol(">")?;
        Ok(sources)
    }

    fn expect_source(&mut self) -> Outcome<Name> {
        if let Lexeme::Source(text) = self.peek().lexeme.clone() {
            let span = self.advance().span;
            Ok(Name { text, span })
        } else {
            Err(self.expected("a source in backticks, as in `` `text` ``"))
        }
    }

    fn parse_fill(&mut self) -> Outcome<Vec<Fill>> {
        // trace: D23, D213, D219
        let open = self.position;
        self.advance(); // `<`
        self.break_after.insert(open);
        self.check_gaps(open, open, None, Some(Gap::Touching), SPACING_MARKS);
        let mut fill = Vec::new();
        while !self.at_symbol(">") {
            // ``index<`text`: $article>`` fills a source, not a type. (D171)
            if matches!(self.peek().lexeme, Lexeme::Source(_)) {
                let source = self.expect_source()?;
                self.expect_symbol(":")?;
                let value = if let Lexeme::Source(_) = self.peek().lexeme {
                    SourceValue::Source(self.expect_source()?)
                } else if self.at_word("program") {
                    {
                        self.advance();
                        SourceValue::Program
                    }
                } else {
                    SourceValue::Variable(
                        self.expect_variable("`$` and a name, `program` or a source")?,
                    )
                };
                self.source_fills.push(SourceFill { source, value });
                if !self.eat_symbol(",") {
                    break;
                }
                continue;
            }
            let label = if matches!(self.peek().lexeme, Lexeme::Word(_)) && self.symbol_at(1, ":") {
                let name = self.expect_name("a label")?;
                self.advance();
                Some(name)
            } else {
                None
            };
            let written_type = self.parse_type()?;
            fill.push(Fill {
                label,
                written_type,
            });
            if !self.eat_symbol(",") {
                break;
            }
        }
        if !self.at_symbol(">") {
            return Err(self.expected("`>`"));
        }
        let close = self.position;
        self.advance();
        self.type_brackets.push((open, close));
        self.check_gaps(close, close, Some(Gap::Touching), None, SPACING_MARKS);
        Ok(fill)
    }

    fn parse_stored_borrow(&mut self) -> Outcome<TypeExpression> {
        let start = self.advance().span; // `Lend`
        let mut mutable = false;
        let mut held = false;
        // `Lend.mut`, `Lend.held` and `Lend.mut.held`, in that order. (D169)
        while self.at_symbol(".") {
            self.advance();
            let narrowing = self.expect_name("`mut` or `held`")?;
            match narrowing.text.as_str() {
                "mut" if !mutable && !held => mutable = true,
                // How it can be used goes before how long, strongest first. (D223)
                // trace: D223
                "mut" if held && !mutable => {
                    return Err(Report::refused(
                        start.to(narrowing.span),
                        "narrowings go strongest first, so `mut` comes before `held`",
                        "the narrowing that changes the action most sits next to the thing, and for a borrow how it can be used comes before how long (D223, D169)",
                    )
                    .fix("Lend.mut.held"));
                }
                HELD_WORD if !held => held = true,
                _ => return Err(self.expected("`mut` or `held`")),
            }
        }
        if !self.eat_word("from") {
            return Err(self.expected("`from`"));
        }
        let sources = if let Lexeme::Source(_) = self.peek().lexeme {
            // trace: D171
            BorrowSources::Source(self.expect_source()?)
        } else if self.eat_word("program") {
            BorrowSources::Program
        } else if self.at_symbol("<") {
            self.advance();
            let mut variables = Vec::new();
            while !self.at_symbol(">") {
                variables.push(self.expect_variable("a variable")?);
                if !self.eat_symbol(",") {
                    break;
                }
            }
            self.expect_symbol(">")?;
            BorrowSources::Variables(variables)
        } else {
            BorrowSources::Variables(vec![self.expect_variable("a variable")?])
        };
        let inner = self.parse_type()?;
        Ok(TypeExpression {
            span: start.to(self.previous_span()),
            kind: TypeKind::StoredBorrow {
                mutable,
                held,
                sources,
                inner: Box::new(inner),
            },
        })
    }

    // ---------------------------------------------------------------------------------------------
    // Statements.
    // ---------------------------------------------------------------------------------------------

    fn parse_block(&mut self) -> Outcome<Vec<Statement>> {
        self.enter()?;
        let mut statements = Vec::new();
        while !self.at_word("end") && !self.at_word("case") && !self.at_word("else") {
            if matches!(self.peek().lexeme, Lexeme::End) {
                return Err(Report::refused(
                    self.previous_span(),
                    "the file ended inside a block",
                    BLOCK_WHY,
                ));
            }
            statements.push(self.parse_statement()?);
        }
        self.leave();
        Ok(statements)
    }

    /// Nothing is written after an `otherwise:` block's closer. (8.6)
    fn ends_in_block(expression: &Expression) -> bool {
        matches!(
            &expression.kind,
            ExpressionKind::Otherwise {
                handling: Handling::Block(_),
                ..
            }
        )
    }

    fn end_statement(&mut self, value: &Expression) -> Outcome<Span> {
        if Self::ends_in_block(value) {
            Ok(self.previous_span())
        } else {
            self.expect_semicolon()
        }
    }

    #[allow(clippy::too_many_lines)] // One arm per statement, read top to bottom.
    fn parse_statement(&mut self) -> Outcome<Statement> {
        self.mark_line_start();
        let start = self.span();
        if let Lexeme::Word(word) = &self.peek().lexeme {
            match word.as_str() {
                "let" => return self.parse_let(),
                // trace: D203
                "decl"
                    if matches!(self.peek_at(1).lexeme, Lexeme::Variable(_))
                        || self.word_at(1, "mut") =>
                {
                    return self.parse_decl();
                }
                "assign" if matches!(self.peek_at(1).lexeme, Lexeme::Variable(_)) => {
                    let start = self.advance().span;
                    let name = self.expect_variable("the variable's name, starting with `$`")?;
                    self.expect_symbol("=")?;
                    let value = self.parse_expression()?;
                    let end = self.end_statement(&value)?;
                    return Ok(Statement {
                        span: start.to(end),
                        kind: StatementKind::Assign { name, value },
                    });
                }
                "const" => return self.parse_constant(),
                "change" => return self.parse_change(),
                "if" => return self.parse_if(),
                "match" => return self.parse_match(),
                "for" => return self.parse_for(),
                "while" => return self.parse_while(),
                "forever" => return self.parse_forever(),
                // trace: D233
                "on" if self.symbol_at(1, ".") && self.word_at(2, "target") => {
                    return self.parse_on_target();
                }
                "inline"
                    if self.symbol_at(1, ".")
                        && self.word_at(2, "asm")
                        && self.symbol_at(3, ":") =>
                {
                    return self.parse_assembly();
                }
                "break" => return self.parse_break(),
                "type" | "fam" if self.symbol_at(1, ".") => {
                    return Err(Report::not_designed(
                        start,
                        format!("`{word}` inside a function"),
                        "",
                    ));
                }
                // trace: D139
                "instead" if !self.symbol_at(1, "(") => {
                    self.advance();
                    let value = self.parse_expression()?;
                    let end = self.end_statement(&value)?;
                    return Ok(Statement {
                        span: start.to(end),
                        kind: StatementKind::Instead(value),
                    });
                }
                // trace: D129
                "finish" if !self.symbol_at(1, "(") => {
                    self.advance();
                    let target = self.parse_postfix()?;
                    let end = self.expect_semicolon()?;
                    return Ok(Statement {
                        span: start.to(end),
                        kind: StatementKind::Finish { target },
                    });
                }
                // trace: D128
                "continue" if self.symbol_at(1, ";") => {
                    return Err(Report::refused(
                        start,
                        "there is no `continue`, so wrap the work in an `if`",
                        "each loop says how it ends, and there is no `continue` (D128)",
                    ));
                }
                // trace: D128
                "loop" if self.symbol_at(1, ":") => {
                    return Err(Report::refused(
                        start,
                        "a loop with no end says so",
                        "`forever` has no end unless you `break`, so one word finds every endless loop (D128)",
                    )
                    .fix("forever"));
                }
                "shield" => return self.parse_shield_statement(),
                "return" => {
                    self.advance();
                    let value = self.parse_expression()?;
                    let end = self.end_statement(&value)?;
                    return Ok(Statement {
                        span: start.to(end),
                        kind: StatementKind::Return(value),
                    });
                }
                "fail" => {
                    self.advance();
                    let path = self.parse_path()?;
                    let data = if self.at_symbol("[") {
                        self.parse_field_values()?
                    } else {
                        Vec::new()
                    };
                    let end = self.expect_semicolon()?;
                    return Ok(Statement {
                        span: start.to(end),
                        kind: StatementKind::Fail { path, data },
                    });
                }
                // trace: D84
                "precond" | "postcond" if self.symbol_at(1, "=>") => {
                    let wanted = if word == "precond" {
                        "requires"
                    } else {
                        "ensures"
                    };
                    return Err(Report::refused(
                        start.to(self.peek_at(1).span),
                        format!("`{word} =>` went back to `{wanted}`"),
                        "contracts are `requires` and `ensures`, with no `=>` (D84)",
                    )
                    .fix(wanted));
                }
                "labels" | "defaults" | "requires" | "ensures" | "fails" => {
                    // trace: D114
                    return Err(Report::refused(
                        start,
                        format!("`{word}` goes first in the body"),
                        "the top of a body goes `labels`, `defaults`, `requires`, `ensures`, then the code (D114, D209)",
                    ));
                }
                _ => {}
            }
        }
        let value = self.parse_expression()?;
        let end = self.end_statement(&value)?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::Expression(value),
        })
    }

    /// `decl $x: i32;` declares a variable with no value yet. (D203)
    /// trace: D203
    fn parse_decl(&mut self) -> Outcome<Statement> {
        let start = self.advance().span; // `decl`
        let mutable = self.eat_word("mut");
        let name = self.expect_variable("the variable's name, starting with `$`")?;
        self.expect_symbol(":")?;
        let written_type = self.parse_type()?;
        if self.at_symbol("=") {
            return Err(Report::refused(
                self.span(),
                "`decl` gives no value, and `let` does",
                "`decl` declares a variable with no value yet, and `assign` gives it one (D203)",
            ));
        }
        let end = self.expect_semicolon()?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::Decl {
                mutable,
                name,
                written_type,
            },
        })
    }

    fn parse_let(&mut self) -> Outcome<Statement> {
        let start = self.advance().span; // `let`
        let mutable = self.eat_word("mut");
        let name = self.expect_variable("the variable's name, starting with `$`")?;
        // trace: D216
        if self.eat_symbol(":=") {
            return self.parse_inferred(start, false, mutable, name);
        }
        self.expect_symbol(":")?;
        let written_type = self.parse_type()?;
        Self::refuse_typed_infer(self.at_symbol(":="), self.span())?;
        self.expect_symbol("=")?;
        let value = self.parse_expression()?;
        let end = self.end_statement(&value)?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::Let {
                mutable,
                name,
                written_type,
                value,
            },
        })
    }

    /// The rest of `let $y := $x + 1;`, after the `:=`. (D216)
    fn parse_inferred(
        &mut self,
        start: Span,
        constant: bool,
        mutable: bool,
        name: Name,
    ) -> Outcome<Statement> {
        let value = self.parse_expression()?;
        let end = self.end_statement(&value)?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::Inferred {
                constant,
                mutable,
                name,
                value,
            },
        })
    }

    /// `:=` is for `let` and `const`, and a parameter or field always writes its type. (D216)
    fn refuse_infer_here(found: bool, span: Span, what: &str) -> Outcome<()> {
        if found {
            return Err(Report::refused(
                span,
                format!("{what} always writes its type"),
                "`:=` leaves the type out only on a `let` or a `const`, not a parameter, a return type or a field (D216)",
            ));
        }
        Ok(())
    }

    /// `:=` leaves the type out, so a written type uses `=`. (D216)
    /// trace: D216
    fn refuse_typed_infer(found: bool, span: Span) -> Outcome<()> {
        if found {
            return Err(Report::refused(
                span,
                "this writes its type, so it uses `=`",
                "`:=` leaves the type out, and a written type uses `=` (D216)",
            )
            .fix("="));
        }
        Ok(())
    }

    fn parse_change(&mut self) -> Outcome<Statement> {
        let start = self.advance().span; // `change`
        let target = self.parse_postfix_only()?;
        self.expect_symbol("=")?;
        let value = self.parse_expression()?;
        let end = self.end_statement(&value)?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::Change { target, value },
        })
    }

    /// trace: D126
    fn parse_if(&mut self) -> Outcome<Statement> {
        let start = self.advance().span; // `if`
        let mut links = Vec::new();
        let mut last = None;
        loop {
            let test = self.parse_if_test()?;
            let body = self.parse_block()?;
            links.push(IfLink { test, body });
            if !self.at_word("else") {
                break;
            }
            let else_span = self.advance().span;
            if self.eat_word("if") {
                continue;
            }
            if !self.at_symbol(":") {
                return Err(self.expected("`if` or `:` after `else`"));
            }
            self.advance();
            last = Some(self.parse_block()?);
            if self.at_word("else") {
                return Err(Report::refused(
                    self.span(),
                    "nothing follows `else:` but `end if`",
                    "`else if` and `else:` continue an `if`, and one `end if` closes the chain (D126)",
                )
                .because(else_span, "the `else:` is here"));
            }
            break;
        }
        let end = self.expect_closer(&["if"])?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::If { links, last },
        })
    }

    /// What follows `if` or `else if`, up to and including the `:` that opens its block.
    fn parse_if_test(&mut self) -> Outcome<IfTest> {
        let condition = self.parse_expression()?;
        let colon = self.position;
        self.expect_symbol(":")?;
        // `if $s: status.failed:` reads a type then a second `:`. Anything else is a plain block.
        let could_name_a_case = !self.peek().starts_a_line
            && !matches!(&self.peek().lexeme, Lexeme::Word(word) if is_clashing(word));
        if let Ok(written_type) = self.parse_type() {
            if could_name_a_case
                && matches!(self.peek().lexeme, Lexeme::Variable(_))
                && self.symbol_at(1, ":")
            {
                return Err(Report::not_designed(
                    self.span(),
                    "naming the case in a test, which D139 only shows on `case`",
                    "",
                ));
            }
            if self.eat_symbol(":") {
                return Ok(IfTest::Type {
                    subject: condition,
                    written_type,
                });
            }
        }
        self.position = colon;
        self.expect_symbol(":")?;
        Ok(IfTest::Condition(condition))
    }

    fn parse_arms(&mut self) -> Outcome<Vec<Arm>> {
        let mut arms = Vec::new();
        while self.at_word("case") {
            self.advance();
            let pattern = self.parse_path()?;
            // trace: D139
            let binding = if matches!(self.peek().lexeme, Lexeme::Variable(_)) {
                Some(self.expect_variable("the name for this case")?)
            } else {
                None
            };
            self.expect_symbol(":")?;
            let body = self.parse_block()?;
            arms.push(Arm {
                pattern,
                binding,
                body,
            });
        }
        Ok(arms)
    }

    fn parse_match(&mut self) -> Outcome<Statement> {
        let start = self.advance().span; // `match`
        let subject = self.parse_expression()?;
        self.expect_symbol(":")?;
        let arms = self.parse_arms()?;
        let end = self.expect_closer(&["match"])?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::Match { subject, arms },
        })
    }

    /// `for rows $row in $grid:`. The name is only for `break`. (D128)
    /// trace: D128
    fn parse_for(&mut self) -> Outcome<Statement> {
        let start = self.advance().span; // `for`
        let loop_name = if matches!(self.peek().lexeme, Lexeme::Word(_)) {
            Some(self.expect_name("the loop's name")?)
        } else {
            None
        };
        let name = self.expect_variable("the item's name, starting with `$`")?;
        if !self.eat_word("in") {
            return Err(self.expected("`in`"));
        }
        let source = if self.at_word("range") && self.symbol_at(1, "[") {
            ForSource::Range(Box::new(self.parse_range()?))
        } else {
            ForSource::Collection(self.parse_expression()?)
        };
        self.expect_symbol(":")?;
        let body = self.parse_block()?;
        let end = self.expect_loop_closer("for", loop_name.as_ref())?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::For {
                loop_name,
                name,
                source,
                body,
            },
        })
    }

    /// `range[from: 1, to: 5, step_by: 2]`, with its labels in that order. (D128)
    /// trace: D128
    fn parse_range(&mut self) -> Outcome<RangeExpression> {
        const WHY: &str = "`from:` is required, `to:` or `before:` says where it ends, and `step_by:` comes after them, in that order (D128)";
        let start = self.advance().span; // `range`
        self.advance(); // `[`
        let mut from = None;
        let mut end = None;
        let mut step = None;
        let mut reached = 0;
        while !self.at_symbol("]") {
            let label = self.expect_name("`from:`, `to:`, `before:` or `step_by:`")?;
            let order = match label.text.as_str() {
                "from" => 1,
                "to" | "before" => 2,
                "step_by" => 3,
                other => {
                    return Err(Report::refused(
                        label.span,
                        format!("a range has no `{other}:`"),
                        WHY,
                    ));
                }
            };
            if order <= reached {
                return Err(Report::refused(
                    label.span,
                    format!("`{}:` is out of order here", label.text),
                    WHY,
                ));
            }
            reached = order;
            if self.at_symbol("=") {
                return Err(Report::refused(
                    self.span(),
                    "a range's values are named with `:`",
                    "a label names a value inside brackets (D13)",
                )
                .fix(":"));
            }
            self.expect_symbol(":")?;
            let value = self.parse_expression()?;
            match label.text.as_str() {
                "from" => from = Some(value),
                "step_by" => step = Some(value),
                _ => {
                    end = Some(RangeEnd {
                        value,
                        included: label.text == "to",
                    });
                }
            }
            if !self.eat_symbol(",") {
                break;
            }
        }
        let close = self.expect_symbol("]")?;
        let span = start.to(close);
        let Some(from) = from else {
            return Err(Report::refused(span, "a range needs `from:`", WHY));
        };
        let Some(end) = end else {
            return Err(Report::refused(
                span,
                "this range has no end, so it can never reach one",
                "a range that can never reach its end is refused, and an endless loop is `forever` (D128)",
            ));
        };
        Ok(RangeExpression {
            span,
            from,
            end,
            step,
        })
    }

    /// A word that names a `while` or `forever`, which 4.8 implies but doesn't show.
    fn refuse_named_loop_for_now(&mut self, keyword: &str) -> Outcome<()> {
        let Lexeme::Word(word) = &self.peek().lexeme else {
            return Ok(());
        };
        if is_clashing(word) {
            return Ok(());
        }
        let is_named = match keyword {
            "forever" => self.symbol_at(1, ":"),
            _ => matches!(
                self.peek_at(1).lexeme,
                Lexeme::Variable(_)
                    | Lexeme::Integer(_)
                    | Lexeme::Float(_)
                    | Lexeme::Text(_)
                    | Lexeme::Word(_)
            ),
        };
        if is_named {
            return Err(Report::not_designed(
                self.span(),
                format!("how `{keyword}` is named, which 4.8 only shows on `for`"),
                "",
            ));
        }
        Ok(())
    }

    /// trace: D128
    fn parse_while(&mut self) -> Outcome<Statement> {
        let start = self.advance().span; // `while`
        self.refuse_named_loop_for_now("while")?;
        let condition = match self.parse_if_test()? {
            IfTest::Condition(condition) => condition,
            IfTest::Type { written_type, .. } => {
                return Err(Report::not_designed(
                    written_type.span,
                    "the test in a `while`",
                    "12.4",
                ));
            }
        };
        let body = self.parse_block()?;
        let end = self.expect_loop_closer("while", None)?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::While {
                loop_name: None,
                condition,
                body,
            },
        })
    }

    /// trace: D128
    fn parse_forever(&mut self) -> Outcome<Statement> {
        let start = self.advance().span; // `forever`
        self.refuse_named_loop_for_now("forever")?;
        self.expect_symbol(":")?;
        let body = self.parse_block()?;
        let end = self.expect_loop_closer("forever", None)?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::Forever {
                loop_name: None,
                body,
            },
        })
    }

    /// `end for rows` on a named loop and `end for` on one without a name. (D33, D128)
    fn expect_loop_closer(&mut self, keyword: &str, name: Option<&Name>) -> Outcome<Span> {
        if let Some(name) = name {
            return self.expect_closer(&[keyword, &name.text]);
        }
        let end = self.expect_closer(&[keyword])?;
        if matches!(self.peek().lexeme, Lexeme::Word(_)) && !self.peek().starts_a_line {
            return Err(Report::refused(
                end.to(self.span()),
                format!("this `{keyword}` has no name for its closer to repeat"),
                "the closer repeats what it closes, including the name (D33, D128)",
            )
            .fix(format!("end {keyword}")));
        }
        Ok(end)
    }

    /// `break;` and `break rows;`.
    /// trace: D128
    fn parse_break(&mut self) -> Outcome<Statement> {
        let start = self.advance().span; // `break`
        let name = if self.at_symbol(";") {
            None
        } else {
            Some(self.parse_path()?)
        };
        let end = self.expect_semicolon()?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::Break { name },
        })
    }

    /// `on.target aarch64:`, its branch, an optional `else:`, and `end on.target`. (D233)
    /// trace: D233
    fn parse_on_target(&mut self) -> Outcome<Statement> {
        let start = self.advance().span; // `on`
        self.advance(); // `.`
        self.advance(); // `target`
        if self.at_symbol(".") {
            let narrowing = self.peek_at(1).text.clone();
            return Err(Report::not_designed(
                start.to(self.peek_at(1).span),
                format!("`on.target.{narrowing}`, a narrowing of `on.target`"),
                "12.5",
            ));
        }
        let chip = self.expect_name("the chip's name")?;
        self.expect_symbol(":")?;
        let body = self.parse_block()?;
        let other = if self.eat_word("else") {
            if !self.at_symbol(":") {
                return Err(Report::not_designed(
                    self.span(),
                    "a second chip after `else` in an `on.target` choice",
                    "",
                ));
            }
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        let end = self.expect_closer(&["on.target"])?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::OnTarget { chip, body, other },
        })
    }

    /// `inline.asm:`, its `reads:`, `writes:` and `clobbers:` lines in that order, its raw
    /// lines, and `end inline.asm`. (D233)
    /// trace: D233
    fn parse_assembly(&mut self) -> Outcome<Statement> {
        let start = self.span();
        self.advance(); // `inline`
        self.advance(); // `.`
        let asm = self.advance().span; // `asm`
        let opener = start.to(asm);
        self.expect_symbol(":")?;
        self.mark_line_start();
        let reads = self.parse_registers("reads", |this| {
            this.expect_symbol("=")?;
            this.parse_expression()
        })?;
        self.mark_line_start();
        let writes = self.parse_registers("writes", |this| {
            this.expect_symbol("=>")?;
            this.expect_variable("the variable it goes into, starting with `$`")
        })?;
        self.mark_line_start();
        let clobbers = self.parse_registers("clobbers", |_| Ok(()))?;
        let mut lines = Vec::new();
        while let Lexeme::Raw(text) = self.peek().lexeme.clone() {
            self.mark_line_start();
            lines.push((text, self.advance().span));
        }
        let end = self.expect_closer(&["inline.asm"])?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::Assembly(Box::new(Assembly {
                opener,
                reads,
                writes,
                clobbers: clobbers.into_iter().map(|(name, ())| name).collect(),
                lines,
            })),
        })
    }

    /// `word: [register …, register …];`, with what follows each register read by `rest`.
    fn parse_registers<T>(
        &mut self,
        word: &str,
        mut rest: impl FnMut(&mut Self) -> Outcome<T>,
    ) -> Outcome<Vec<(Name, T)>> {
        if !self.at_word(word) {
            return Err(self.expected(&format!("`{word}:`")));
        }
        self.advance();
        self.expect_symbol(":")?;
        self.expect_symbol("[")?;
        let mut entries = Vec::new();
        while !self.at_symbol("]") {
            let register = self.expect_name("a register")?;
            entries.push((register, rest(self)?));
            if !self.eat_symbol(",") {
                break;
            }
        }
        self.expect_symbol("]")?;
        self.expect_semicolon()?;
        Ok(entries)
    }

    fn parse_shield_statement(&mut self) -> Outcome<Statement> {
        let start = self.span();
        let (name, reason) = self.parse_shield_header()?;
        if self.at_word("requires") || self.at_word("ensures") {
            return Err(Report::not_designed(
                self.span(),
                "how a shield's own entry and exit are written",
                "12.9",
            ));
        }
        let body = self.parse_block()?;
        let end = self.expect_closer(&["shield", &name.text])?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::Shield { name, reason, body },
        })
    }

    // ---------------------------------------------------------------------------------------------
    // Expressions, loosest first. (Standard 3.9.)
    // ---------------------------------------------------------------------------------------------

    /// trace: D116
    fn parse_expression(&mut self) -> Outcome<Expression> {
        self.enter()?;
        let result = self.parse_otherwise_level();
        self.leave();
        result
    }

    fn parse_otherwise_level(&mut self) -> Outcome<Expression> {
        let value = self.parse_or()?;
        if self.at_word("otherwise") {
            let handling = self.parse_otherwise()?;
            let end = self.previous_span();
            return Ok(Expression {
                span: value.span.to(end),
                kind: ExpressionKind::Otherwise {
                    value: Box::new(value),
                    handling,
                },
            });
        }
        Ok(value)
    }

    fn parse_otherwise(&mut self) -> Outcome<Handling> {
        self.advance(); // `otherwise`
        if self.eat_symbol(":") {
            let arms = self.parse_arms()?;
            self.expect_closer(&["otherwise"])?;
            return Ok(Handling::Block(arms));
        }
        let family = self.expect_name("`:` or a handler")?;
        if !self.at_symbol(".") {
            return Err(self.expected("`.` and the handler's name"));
        }
        self.advance();
        let jumps = self.eat_symbol("*");
        let name = self.expect_name("the handler's name")?;
        Ok(Handling::Named {
            span: family.span.to(name.span),
            family: family.text,
            name: name.text,
            jumps,
        })
    }

    /// Bit operators and shifts mixed with anything need grouping. (D116)
    fn refuse_bit_mix(expression: &Expression) -> Outcome<()> {
        let is_bit = match &expression.kind {
            ExpressionKind::BitNot(_) => true,
            ExpressionKind::Binary { operator, .. } => matches!(
                operator,
                BinaryOperator::BitAnd
                    | BinaryOperator::BitOr
                    | BinaryOperator::BitXor
                    | BinaryOperator::ShiftLeft
                    | BinaryOperator::ShiftRight
            ),
            _ => false,
        };
        if is_bit {
            // trace: D116
            return Err(Report::refused(
                expression.span,
                "this needs `( )` around it, since maths gives bit operators no order",
                "where maths gives no order, you group with `( )` (D116)",
            ));
        }
        Ok(())
    }

    fn binary(
        operator: BinaryOperator,
        left: Expression,
        right: Expression,
    ) -> Outcome<Expression> {
        Self::refuse_bit_mix(&left)?;
        Self::refuse_bit_mix(&right)?;
        Ok(Expression {
            span: left.span.to(right.span),
            kind: ExpressionKind::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            },
        })
    }

    fn parse_or(&mut self) -> Outcome<Expression> {
        let mut left = self.parse_and()?;
        let depth_before = self.depth;
        while self.at_word("or") {
            self.enter()?;
            let operator = if self.logic_narrowing("or")? {
                BinaryOperator::OrBoth
            } else {
                BinaryOperator::Or
            };
            let right = self.parse_and()?;
            Self::refuse_as_on_the_right(&right)?;
            left = Self::binary(operator, left, right)?;
        }
        self.depth = depth_before;
        Ok(left)
    }

    /// `not` sits below comparisons, so `not $a < 3` means `not ($a < 3)`. (D131)
    /// trace: D131
    fn parse_not(&mut self) -> Outcome<Expression> {
        let start = self.span();
        if !self.eat_word("not") {
            return self.parse_compare();
        }
        self.enter()?;
        let result = self.parse_not();
        self.leave();
        let inner = result?;
        Self::refuse_bit_mix(&inner)?;
        Self::refuse_as_on_the_right(&inner)?;
        Ok(Expression {
            span: start.to(inner.span),
            kind: ExpressionKind::Not(Box::new(inner)),
        })
    }

    fn parse_and(&mut self) -> Outcome<Expression> {
        let mut left = self.parse_not()?;
        let depth_before = self.depth;
        while self.at_word("and") {
            self.enter()?;
            let operator = if self.logic_narrowing("and")? {
                BinaryOperator::AndBoth
            } else {
                BinaryOperator::And
            };
            let right = self.parse_not()?;
            Self::refuse_as_on_the_right(&right)?;
            left = Self::binary(operator, left, right)?;
        }
        self.depth = depth_before;
        Ok(left)
    }

    /// Reads `and` or `or`, and says whether it was narrowed to run both sides. (D124)
    fn logic_narrowing(&mut self, word: &str) -> Outcome<bool> {
        let first = self.position;
        let start = self.advance().span;
        if !self.at_symbol(".") {
            self.check_operator(first);
            return Ok(false);
        }
        self.advance();
        let narrowing = self.expect_name("a narrowing")?;
        self.check_operator(first);
        if narrowing.text == "both" {
            // trace: D124
            return Ok(true);
        }
        Err(Report::not_designed(
            start.to(narrowing.span),
            format!("`{word}.{}`", narrowing.text),
            "",
        ))
    }

    fn compare_operator_here(&mut self) -> Outcome<Option<CompareOperator>> {
        if self.at_symbol("<=") || self.at_symbol(">=") {
            // trace: D49
            let fix = if self.at_symbol("<=") {
                "at_most"
            } else {
                "at_least"
            };
            return Err(Report::refused(
                self.span(),
                "there's no such operator",
                "`at_least` and `at_most` are words, and there is no `<=` or `>=` (D49)",
            )
            .fix(fix));
        }
        let operator = if self.at_symbol("==") {
            CompareOperator::Equal
        } else if self.at_symbol("!=") {
            CompareOperator::NotEqual
        } else if self.at_symbol("<") {
            CompareOperator::Less
        } else if self.at_symbol(">") {
            CompareOperator::Greater
        } else if self.at_word("at_least") {
            CompareOperator::AtLeast
        } else if self.at_word("at_most") {
            CompareOperator::AtMost
        } else {
            return Ok(None);
        };
        // trace: D23, D213
        let first = self.position;
        self.advance();
        self.check_operator(first);
        Ok(Some(operator))
    }

    fn parse_compare(&mut self) -> Outcome<Expression> {
        let first = self.parse_additive()?;
        let mut rest = Vec::new();
        let mut end = first.span;
        // trace: D117
        while let Some(operator) = self.compare_operator_here()? {
            let value = self.parse_additive()?;
            Self::refuse_as_on_the_right(&value)?;
            Self::refuse_bit_mix(&value)?;
            end = value.span;
            rest.push((operator, value));
        }
        if rest.is_empty() {
            return Ok(first);
        }
        Self::refuse_bit_mix(&first)?;
        Ok(Expression {
            span: first.span.to(end),
            kind: ExpressionKind::Compare {
                first: Box::new(first),
                rest,
            },
        })
    }

    fn parse_additive(&mut self) -> Outcome<Expression> {
        let mut left = self.parse_multiplicative()?;
        let depth_before = self.depth;
        loop {
            self.enter()?;
            let first = self.position;
            let operator = if self.at_symbol("+") {
                self.advance();
                BinaryOperator::Add
            } else if self.at_symbol("-") {
                self.advance();
                BinaryOperator::Subtract
            } else if self.at_word("plus") && self.symbol_at(1, ".") {
                self.advance();
                self.advance();
                let narrowing = self.expect_name("a narrowing of `plus`")?;
                if narrowing.text != "wrap" {
                    return Err(Report::not_designed(
                        narrowing.span,
                        format!("`plus.{}`", narrowing.text),
                        "12.3",
                    ));
                }
                // trace: D42
                BinaryOperator::PlusWrap
            } else if self.at_word("minus") || self.at_word("times") {
                return Err(Report::not_designed(
                    self.span(),
                    "the wrapping forms of `-` and `x`",
                    "12.3",
                ));
            } else {
                break;
            };
            self.check_operator(first);
            let right = self.parse_multiplicative()?;
            Self::refuse_as_on_the_right(&right)?;
            left = Self::binary(operator, left, right)?;
        }
        self.depth = depth_before;
        Ok(left)
    }

    /// `$a + $b as str` is refused, because `as` needs a single value on its left. (D116)
    fn refuse_as_on_the_right(right: &Expression) -> Outcome<()> {
        if let ExpressionKind::As { .. } = right.kind {
            // trace: D116
            return Err(Report::refused(
                right.span,
                "this `as` is mixed with other operators, so it needs `( )`",
                "`as` works on a single value, and where maths gives no order you group with `( )` (D116)",
            ));
        }
        Ok(())
    }

    fn parse_narrowed_operator(&mut self, word: &str) -> Outcome<Narrowing> {
        let start = self.advance().span; // `div` or `mod`
                                         // The types say whether a bare `div` is allowed, and those aren't known here.
                                         // trace: D164
        if word == "div" && !self.at_symbol(".") {
            return Ok(Narrowing::Bare(start));
        }
        if !self.at_symbol(".") {
            // trace: D97
            return Err(Report::refused(
                start,
                format!("there's no bare `{word}`, so pick a narrowing"),
                format!("`{word}` has no single right answer, so you pick one (D97)"),
            ));
        }
        self.advance();
        let narrowing = self.expect_name("a narrowing")?;
        match (word, narrowing.text.as_str()) {
            (_, "trunc") => Ok(Narrowing::Trunc),
            (_, "floor") => Ok(Narrowing::Floor),
            ("div", "exact") => Ok(Narrowing::Exact),
            (_, other) => Err(Report::not_designed(
                start.to(narrowing.span),
                format!("`{word}.{other}`"),
                "12.3",
            )),
        }
    }

    fn parse_multiplicative(&mut self) -> Outcome<Expression> {
        let mut left = self.parse_unary()?;
        let depth_before = self.depth;
        loop {
            self.enter()?;
            let first = self.position;
            let operator = if self.eat_word("x") {
                BinaryOperator::Multiply
            } else if self.at_word("div") {
                BinaryOperator::Divide(self.parse_narrowed_operator("div")?)
            } else if self.at_word("mod") {
                BinaryOperator::Remainder(self.parse_narrowed_operator("mod")?)
            } else if self.at_symbol("*") {
                return Err(Report::refused(
                    self.span(),
                    "multiply is `x`",
                    "multiply is `x`, as in `$a x $b` (D43)",
                )
                .fix("x"));
            } else if self.at_symbol("/") {
                return Err(Report::refused(
                    self.span(),
                    "divide has a narrowing, and no symbol",
                    "division has no single right answer, so you pick one (D48)",
                ));
            } else if self.at_symbol("%") {
                return Err(Report::refused(
                    self.span(),
                    "a remainder has a narrowing, and no symbol",
                    "a remainder has no single right answer, so you pick one (D97)",
                ));
            } else {
                break;
            };
            self.check_operator(first);
            let right = self.parse_unary()?;
            Self::refuse_as_on_the_right(&right)?;
            left = Self::binary(operator, left, right)?;
        }
        self.depth = depth_before;
        Ok(left)
    }

    fn parse_unary(&mut self) -> Outcome<Expression> {
        self.enter()?;
        let result = self.parse_unary_inner();
        self.leave();
        result
    }

    fn parse_unary_inner(&mut self) -> Outcome<Expression> {
        let start = self.span();
        // trace: D131
        if self.at_word("not") {
            return Err(Report::refused(
                start,
                "`not` sits below comparisons and maths, so here it needs `( )`",
                "`not` sits just above `and`, and `( )` groups (D116, D131)",
            ));
        }
        let value = if self.eat_word("bit_not") {
            let inner = self.parse_unary()?;
            Self::refuse_bit_mix(&inner)?;
            Self::refuse_as_on_the_right(&inner)?;
            Expression {
                span: start.to(inner.span),
                kind: ExpressionKind::BitNot(Box::new(inner)),
            }
        } else if self.at_symbol("-")
            && (self.peek().space_after
                || !matches!(
                    self.peek_at(1).lexeme,
                    Lexeme::Integer(_) | Lexeme::Float(_)
                ))
        {
            // trace: D116, D213
            let minus = self.position;
            self.advance();
            self.check_gaps(minus, minus, None, Some(Gap::Touching), SPACING_MARKS);
            let inner = self.parse_unary()?;
            Self::refuse_bit_mix(&inner)?;
            Self::refuse_as_on_the_right(&inner)?;
            Expression {
                span: start.to(inner.span),
                kind: ExpressionKind::Negate(Box::new(inner)),
            }
        } else {
            self.parse_as_level()?
        };
        // trace: D116
        if let Some((_, operator)) = self.bit_word_here() {
            let first = self.position;
            self.advance();
            self.check_operator(first);
            let right = self.parse_unary_operand()?;
            Self::refuse_bit_mix(&value)?;
            Self::refuse_bit_mix(&right)?;
            let span = value.span.to(right.span);
            let expression = Expression {
                span,
                kind: ExpressionKind::Binary {
                    operator,
                    left: Box::new(value),
                    right: Box::new(right),
                },
            };
            if self.bit_word_here().is_some() {
                return Self::refuse_bit_mix(&expression).map(|()| expression);
            }
            return Ok(expression);
        }
        Ok(value)
    }

    fn parse_unary_operand(&mut self) -> Outcome<Expression> {
        if self.at_word("not") || self.at_word("bit_not") {
            let start = self.span();
            return Err(Report::refused(
                start,
                "this needs `( )` around it, since maths gives bit operators no order",
                "where maths gives no order, you group with `( )` (D116)",
            ));
        }
        self.parse_as_level()
    }

    fn bit_word_here(&self) -> Option<(&'static str, BinaryOperator)> {
        BIT_WORDS
            .iter()
            .find(|(word, _)| self.at_word(word))
            .copied()
    }

    fn parse_as_level(&mut self) -> Outcome<Expression> {
        let mut value = self.parse_postfix()?;
        while self.at_word("as") {
            let first = self.position;
            self.advance();
            self.check_operator(first);
            let target = self.parse_type()?;
            value = Expression {
                span: value.span.to(target.span),
                kind: ExpressionKind::As {
                    value: Box::new(value),
                    target,
                },
            };
            value = self.parse_question_marks(value);
            while self.at_symbol("??") {
                value = self.parse_fallback_on(value)?;
                value = self.parse_question_marks(value);
            }
        }
        Ok(value)
    }

    /// A place, for `change`.
    fn parse_postfix_only(&mut self) -> Outcome<Expression> {
        self.parse_postfix()
    }

    /// A value and everything that sticks to it, including `??` and what it falls back on.
    fn parse_postfix(&mut self) -> Outcome<Expression> {
        let mut value = self.parse_single_value()?;
        while self.at_symbol("??") {
            value = self.parse_fallback_on(value)?;
            value = self.parse_question_marks(value);
        }
        Ok(value)
    }

    /// `??` sits level with `?`, and its right side is the next single value. (D132)
    /// trace: D132
    fn parse_fallback_on(&mut self, value: Expression) -> Outcome<Expression> {
        let first = self.position;
        self.advance(); // `??`
        self.check_operator(first);
        if self.at_symbol("-")
            && !matches!(
                self.peek_at(1).lexeme,
                Lexeme::Integer(_) | Lexeme::Float(_)
            )
        {
            return Err(Report::refused(
                self.span(),
                "`??`'s right side is one value, and unary `-` sits below it, so this needs `( )`",
                "`??` sits level with `?`, and its right side is the next single value (D132)",
            ));
        }
        let fallback = self.parse_single_value()?;
        Ok(Expression {
            span: value.span.to(fallback.span),
            kind: ExpressionKind::Fallback {
                value: Box::new(value),
                fallback: Box::new(fallback),
            },
        })
    }

    fn parse_question_marks(&mut self, mut value: Expression) -> Expression {
        while self.at_symbol("?") {
            let mark = self.position;
            let end = self.advance().span;
            // trace: D213
            self.check_gaps(mark, mark, Some(Gap::Touching), None, SPACING_MARKS);
            value = Expression {
                span: value.span.to(end),
                kind: ExpressionKind::Try(Box::new(value)),
            };
        }
        value
    }

    fn parse_single_value(&mut self) -> Outcome<Expression> {
        let mut value = self.parse_primary()?;
        loop {
            if self.at_symbol(".") {
                value = self.parse_after_dot(value)?;
            } else if self.at_symbol("?") {
                value = self.parse_question_marks(value);
            } else if self.at_symbol("[") {
                value = self.parse_position(value)?;
            } else {
                return Ok(value);
            }
        }
    }

    /// `$xs[pos: 1]` and `$xs[offset: 0]`. The word says which you mean. (D133)
    /// trace: D133
    fn parse_position(&mut self, base: Expression) -> Outcome<Expression> {
        const WHY: &str = "`pos:` counts from 1, and `offset.items:` counts from 0, so the word says which you mean (D133, D144)";
        let open = self.advance().span; // `[`
        let word = if let Lexeme::Word(word) = &self.peek().lexeme {
            word.clone()
        } else {
            String::new()
        };
        if word != "pos" && word != "offset" {
            return Err(Report::refused(
                open,
                "getting an item says `pos:` or `offset.items:`",
                WHY,
            ));
        }
        let word_span = self.advance().span;
        if self.at_symbol(".") {
            let unit = self.parse_label_unit()?;
            if word == "offset" && unit.text == "items" {
                // trace: D144
                return self.parse_position_value(base, true);
            }
            if word == "offset" {
                return Err(Report::refused(
                    word_span.to(unit.span),
                    format!("an offset counts `items`, not `{}`", unit.text),
                    "a label's unit takes a plain number or that unit, and a value in another is refused (D144)",
                )
                .fix("offset.items"));
            }
            return Err(Report::not_designed(
                word_span.to(unit.span),
                format!("`pos.{}:`", unit.text),
                "",
            ));
        }
        // trace: D144
        if word == "offset" {
            return Err(Report::refused(
                word_span,
                "an offset says its unit in the label",
                "an offset could read as bytes, so there's no bare `offset:` (D143, D144)",
            )
            .fix("offset.items"));
        }
        if self.at_symbol("=") {
            return Err(Report::refused(
                self.span(),
                "`pos:` and `offset.items:` are labels",
                "a label names a value inside brackets (D13)",
            )
            .fix(":"));
        }
        self.parse_position_value(base, false)
    }

    /// What follows `.` in a label, as a unit's path.
    fn parse_label_unit(&mut self) -> Outcome<Path> {
        self.advance(); // `.`
        self.parse_path()
    }

    fn parse_position_value(
        &mut self,
        base: Expression,
        counts_from_zero: bool,
    ) -> Outcome<Expression> {
        self.expect_symbol(":")?;
        let position = self.parse_expression()?;
        let close = self.expect_symbol("]")?;
        Ok(Expression {
            span: base.span.to(close),
            kind: ExpressionKind::Item {
                base: Box::new(base),
                position: Box::new(position),
                counts_from_zero,
            },
        })
    }

    fn parse_after_dot(&mut self, receiver: Expression) -> Outcome<Expression> {
        self.advance(); // `.`
                        // `.%area()`, `.std::%append()`, `.@csup::%name()`, `.field` and the refused `.append()`.
        let mut kit = None;
        if self.at_symbol("@")
            || (matches!(self.peek().lexeme, Lexeme::Word(_)) && self.symbol_at(1, "::"))
        {
            let mut text = String::new();
            if self.eat_symbol("@") {
                text.push('@');
            }
            let name = self.expect_name("a kit")?;
            text.push_str(&name.text);
            self.expect_symbol("::")?;
            kit = Some(text);
        }
        let percent = self.eat_symbol("%");
        // trace: D211
        let changes = percent && self.eat_symbol("%");
        let name = self.expect_name("a field or a method")?;
        if self.at_symbol("(") {
            let (arguments, end) = self.parse_arguments()?;
            return Ok(Expression {
                span: receiver.span.to(end),
                kind: ExpressionKind::Method {
                    receiver: Box::new(receiver),
                    kit,
                    name,
                    percent,
                    changes,
                    arguments,
                },
            });
        }
        if percent || kit.is_some() {
            return Err(self.expected("`(`, since a `.%` always runs code"));
        }
        Ok(Expression {
            span: receiver.span.to(name.span),
            kind: ExpressionKind::Field {
                base: Box::new(receiver),
                name,
            },
        })
    }

    /// Whether a label comes next, which can have a unit in it, as in `at.screen.pixels: 5`.
    fn label_is_ahead(&self) -> bool {
        let mut offset = 0;
        if !matches!(self.peek_at(offset).lexeme, Lexeme::Word(_)) {
            return false;
        }
        offset += 1;
        while self.symbol_at(offset, ".") || self.symbol_at(offset, "::") {
            if !matches!(self.peek_at(offset + 1).lexeme, Lexeme::Word(_)) {
                return false;
            }
            offset += 2;
        }
        self.symbol_at(offset, ":")
    }

    fn parse_arguments(&mut self) -> Outcome<(Vec<Argument>, Span)> {
        self.expect_symbol("(")?;
        let mut arguments = Vec::new();
        while !self.at_symbol(")") {
            let label = if self.label_is_ahead() {
                let label = self.parse_path()?;
                self.advance(); // `:`
                Some(Name {
                    text: label.text,
                    span: label.span,
                })
            } else {
                None
            };
            let value = self.parse_expression()?;
            arguments.push(Argument { label, value });
            if !self.eat_symbol(",") {
                break;
            }
        }
        let end = self.expect_symbol(")")?;
        Ok((arguments, end))
    }

    fn parse_field_values(&mut self) -> Outcome<Vec<FieldValue>> {
        self.expect_symbol("[")?;
        let mut fields = Vec::new();
        while !self.at_symbol("]") {
            let name = self.expect_name("a field's name")?;
            if self.at_symbol(":") {
                // trace: D57
                return Err(Report::refused(
                    name.span.to(self.span()),
                    "`=` sets a field",
                    "building one uses `[ ]`, with `=` setting a field (D57)",
                )
                .fix(format!("{} =", name.text)));
            }
            self.expect_symbol("=")?;
            let value = self.parse_expression()?;
            fields.push(FieldValue { name, value });
            if !self.eat_symbol(",") {
                break;
            }
        }
        self.expect_symbol("]")?;
        Ok(fields)
    }

    fn parse_hand(&mut self, word: HandWord, start: Span) -> Outcome<Expression> {
        let value = self.parse_postfix()?;
        Ok(Expression {
            span: start.to(value.span),
            kind: ExpressionKind::Hand {
                word,
                value: Box::new(value),
            },
        })
    }

    fn parse_primary_number(&mut self) -> Outcome<Expression> {
        let start = self.span();
        // trace: D165
        let said_exponent = self.eat_word(EXPONENT_WORD);
        let inexact = self.eat_symbol("~");
        // trace: D59
        let negative = self.eat_symbol("-");
        let lexeme = self.peek().lexeme.clone();
        let kind = match lexeme {
            Lexeme::Exponent {
                written,
                mantissa,
                power,
            } => {
                if !said_exponent {
                    let shown = format!(
                        "{EXPONENT_WORD} {}{}{written}",
                        if inexact { "~" } else { "" },
                        if negative { "-" } else { "" }
                    );
                    return Err(Report::refused(
                        start.to(self.span()),
                        format!("an exponent says so with `{EXPONENT_WORD}` in front"),
                        "a stray `e` can't quietly change a number, so the word goes in front (D165)",
                    )
                    .fix(shown));
                }
                ExpressionKind::Float {
                    digits: expand_exponent(&mantissa, &power),
                    negative,
                    inexact,
                    exponent: Some(written),
                }
            }
            _ if said_exponent => {
                return Err(Report::refused(
                    start.to(self.span()),
                    format!("`{EXPONENT_WORD}` goes only on a literal with an exponent"),
                    "the word says the literal has an `e` in it (D165)",
                ));
            }
            Lexeme::Integer(digits) if !inexact => ExpressionKind::Integer { digits, negative },
            Lexeme::Integer(digits) | Lexeme::Float(digits) => ExpressionKind::Float {
                digits,
                negative,
                inexact,
                exponent: None,
            },
            _ => return Err(self.expected("a number")),
        };
        let end = self.advance().span;
        Ok(Expression {
            span: start.to(end),
            kind,
        })
    }

    /// A unit behind a number, unless what follows is an operator. (D99)
    fn parse_unit_after_number(&mut self) -> Outcome<Option<Path>> {
        if self.peek().starts_a_line {
            return Ok(None);
        }
        let is_operator = match &self.peek().lexeme {
            Lexeme::Word(word) => OPERATOR_WORDS.contains(&word.as_str()) || is_clashing(word),
            _ => true,
        };
        if is_operator {
            return Ok(None);
        }
        Ok(Some(self.parse_path()?))
    }

    fn with_unit(&mut self, number: Expression) -> Outcome<Expression> {
        if let Some(unit) = self.parse_unit_after_number()? {
            return Ok(Expression {
                span: number.span.to(unit.span),
                kind: ExpressionKind::WithUnit {
                    number: Box::new(number),
                    unit,
                },
            });
        }
        Ok(number)
    }

    #[allow(clippy::too_many_lines)] // One arm per form of value, read top to bottom.
    fn parse_primary(&mut self) -> Outcome<Expression> {
        self.enter()?;
        let result = self.parse_primary_inner();
        self.leave();
        result
    }

    /// A number starts here, perhaps with `exponent` in front of it. (D165)
    fn at_number_start(&self) -> bool {
        let starts = |lexeme: &Lexeme| {
            matches!(
                lexeme,
                Lexeme::Integer(_)
                    | Lexeme::Float(_)
                    | Lexeme::Exponent { .. }
                    | Lexeme::Symbol("-" | "~")
            )
        };
        let first = &self.peek().lexeme;
        starts(first)
            || matches!(first, Lexeme::Word(word) if word == EXPONENT_WORD)
                && starts(&self.peek_at(1).lexeme)
    }

    fn parse_primary_inner(&mut self) -> Outcome<Expression> {
        let start = self.span();
        match self.peek().lexeme.clone() {
            _ if self.at_number_start() => {
                let number = self.parse_primary_number()?;
                self.with_unit(number)
            }
            Lexeme::Text(text) => {
                self.advance();
                // trace: D103
                if text.is_empty() {
                    return Err(Report::refused(
                        start,
                        "an empty string is `empty`",
                        "`\"\"` could look like unfinished work, so `empty` says it (D103)",
                    )
                    .fix("empty"));
                }
                Ok(Expression {
                    span: start,
                    kind: ExpressionKind::Text(text),
                })
            }
            Lexeme::Variable(name) => {
                self.advance();
                Ok(Expression {
                    span: start,
                    kind: ExpressionKind::Variable(name),
                })
            }
            Lexeme::Symbol("(") => {
                self.advance();
                let inner = self.parse_expression()?;
                let end = self.expect_symbol(")")?;
                Ok(Expression {
                    span: start.to(end),
                    kind: ExpressionKind::Group(Box::new(inner)),
                })
            }
            Lexeme::Symbol("[") => {
                self.advance();
                // trace: D103
                if self.at_symbol("]") {
                    let end = self.advance().span;
                    return Err(Report::refused(
                        start.to(end),
                        "an empty list is `empty`",
                        "`[]` is refused, because `empty` says it (D103)",
                    )
                    .fix("empty"));
                }
                let mut items = Vec::new();
                while !self.at_symbol("]") {
                    items.push(self.parse_expression()?);
                    if !self.eat_symbol(",") {
                        break;
                    }
                }
                let end = self.expect_symbol("]")?;
                Ok(Expression {
                    span: start.to(end),
                    kind: ExpressionKind::Items(items),
                })
            }
            Lexeme::Symbol("%") => {
                // trace: D5
                Err(Report::refused(
                    start,
                    "`%` goes only on a method call",
                    "`%` goes only on a method call, never on an argument (D5)",
                ))
            }
            Lexeme::Symbol("@") => {
                let path = self.parse_path()?;
                if self.at_symbol("<") {
                    let _ = self.parse_fill()?;
                }
                if self.at_symbol("(") {
                    let (arguments, end) = self.parse_arguments()?;
                    let span = start.to(end);
                    return Ok(Expression {
                        span,
                        kind: if path.text.contains("::") {
                            ExpressionKind::Call { path, arguments }
                        } else {
                            ExpressionKind::Builtin(path)
                        },
                    });
                }
                Ok(Expression {
                    span: path.span,
                    kind: if path.text.contains("::") {
                        ExpressionKind::Path(path)
                    } else {
                        ExpressionKind::Builtin(path)
                    },
                })
            }
            Lexeme::Word(word) => self.parse_word_primary(&word, start),
            _ => Err(self.expected("a value")),
        }
    }

    #[allow(clippy::too_many_lines)] // One arm per word, read top to bottom.
    fn parse_word_primary(&mut self, word: &str, start: Span) -> Outcome<Expression> {
        match word {
            "empty" if !self.symbol_at(1, "(") => {
                self.advance();
                return Ok(Expression {
                    span: start,
                    kind: ExpressionKind::Empty,
                });
            }
            "void" if !self.symbol_at(1, "(") => {
                self.advance();
                return Ok(Expression {
                    span: start,
                    kind: ExpressionKind::Void,
                });
            }
            "inf" if !self.symbol_at(1, "(") => {
                self.advance();
                let number = Expression {
                    span: start,
                    kind: ExpressionKind::Inf,
                };
                return self.with_unit(number);
            }
            "all" => {
                self.advance();
                let value = self.parse_unary()?;
                return Ok(Expression {
                    span: start.to(value.span),
                    kind: ExpressionKind::All(Box::new(value)),
                });
            }
            "bake" => {
                self.advance();
                let value = self.parse_postfix()?;
                return Ok(Expression {
                    span: start.to(value.span),
                    kind: ExpressionKind::Bake(Box::new(value)),
                });
            }
            // trace: D2
            "Given" | "Lent" => {
                let wanted = if word == "Given" { "Give" } else { "Lend" };
                let mut end = self.advance().span;
                let mut wanted = wanted.to_string();
                if word == "Lent" && self.at_symbol(".") && self.word_at(1, "mut") {
                    self.advance();
                    end = self.advance().span;
                    wanted.push_str(".mut");
                }
                return Err(Report::refused(
                    start.to(end),
                    format!("`{word}` is the parameter's word"),
                    "the call and the parameter each have their own word (D2)",
                )
                .fix(wanted));
            }
            "Give" => {
                self.advance();
                return self.parse_hand(HandWord::Give, start);
            }
            "Copy" => {
                self.advance();
                return self.parse_hand(HandWord::Copy, start);
            }
            "Lend" => {
                self.advance();
                if self.at_symbol(".") {
                    self.advance();
                    let narrowing = self.expect_name("`mut`")?;
                    if narrowing.text != "mut" {
                        return Err(Report::refused(
                            narrowing.span,
                            format!("`Lend.{}` doesn't exist", narrowing.text),
                            "the words are `Give`, `Copy`, `Lend` and `Lend.mut` (D2)",
                        ));
                    }
                    return self.parse_hand(HandWord::LendMut, start);
                }
                return self.parse_hand(HandWord::Lend, start);
            }
            _ => {}
        }
        if word == "range" && self.symbol_at(1, "[") {
            return Err(Report::not_designed(
                start,
                "a range anywhere but straight after `in`",
                "",
            ));
        }
        // trace: D75
        if self.symbol_at(1, "(") && word != "my" && is_clashing(word) {
            let close = self.matching_paren_span();
            let shown = self.tokens_text(self.position, close);
            return Err(Report::refused(
                start.to(self.tokens_span(close)),
                format!("`{word}` is built in"),
                "a name of yours that clashes is written with `my::` (D75)",
            )
            .fix(format!("my::{shown}")));
        }
        let path = self.parse_path()?;
        if self.at_symbol("(") {
            let (arguments, end) = self.parse_arguments()?;
            return Ok(Expression {
                span: start.to(end),
                kind: ExpressionKind::Call { path, arguments },
            });
        }
        if self.at_symbol("[") {
            let fields = self.parse_field_values()?;
            let end = self.previous_span();
            return Ok(Expression {
                span: start.to(end),
                kind: ExpressionKind::Build { path, fields },
            });
        }
        if self.at_symbol("<") && !self.peek().space_before {
            return Err(Report::not_designed(
                self.span(),
                "filling in a type here, as a value",
                "",
            ));
        }
        Ok(Expression {
            span: path.span,
            kind: ExpressionKind::Path(path),
        })
    }

    /// The index of the `)` matching the `(` right after this token, or the last token.
    fn matching_paren_span(&self) -> usize {
        let mut depth = 0usize;
        let mut index = self.position + 1;
        while let Some(token) = self.tokens.get(index) {
            match token.lexeme {
                Lexeme::Symbol("(") => depth += 1,
                Lexeme::Symbol(")") => {
                    depth -= 1;
                    if depth == 0 {
                        return index;
                    }
                }
                Lexeme::End => return index,
                _ => {}
            }
            index += 1;
        }
        index.saturating_sub(1)
    }

    fn tokens_span(&self, index: usize) -> Span {
        self.tokens
            .get(index)
            .map_or(self.previous_span(), |token| token.span)
    }

    /// Writes tokens back out, for a `fix:` that shows code the way it was typed.
    fn tokens_text(&self, from: usize, to: usize) -> String {
        let mut text = String::new();
        for index in from..=to {
            let Some(token) = self.tokens.get(index) else {
                break;
            };
            if token.space_before && index != from {
                text.push(' ');
            }
            match &token.lexeme {
                Lexeme::Word(word)
                | Lexeme::Integer(word)
                | Lexeme::Float(word)
                | Lexeme::Raw(word)
                | Lexeme::Exponent { written: word, .. } => {
                    text.push_str(word);
                }
                Lexeme::Variable(name) => {
                    text.push('$');
                    text.push_str(name);
                }
                Lexeme::Source(name) => {
                    text.push('`');
                    text.push_str(name);
                    text.push('`');
                }
                Lexeme::Text(content) => {
                    text.push('"');
                    text.push_str(content);
                    text.push('"');
                }
                Lexeme::Symbol(symbol) => text.push_str(symbol),
                Lexeme::End => {}
            }
        }
        text
    }
}

/// Only here so `peek_at` has something to return without a panic path.
static END_TOKEN: Token = Token {
    lexeme: Lexeme::End,
    span: Span { start: 0, end: 0 },
    space_before: true,
    space_after: true,
    gap_before: Gap::Line,
    gap_after: Gap::Line,
    text: String::new(),
    starts_a_line: true,
};

/// What goes between two tokens when a refused line break joins them: nothing where D213 says
/// they touch, and one space everywhere else. (D213, D219)
fn join_gap(previous: &Token, next: &Token) -> &'static str {
    let touches_next = matches!(
        next.lexeme,
        Lexeme::Symbol("." | "::" | ")" | "]" | ">" | "?" | "," | ";" | ":" | "(")
    );
    let touches_previous = matches!(
        previous.lexeme,
        Lexeme::Symbol("(" | "[" | "<" | "." | "::" | "@" | "%" | "~" | "-")
    );
    if touches_next || touches_previous {
        ""
    } else {
        " "
    }
}

/// Reads an assembly block's raw lines, one token each, from `position` up to the line that
/// closes it, and gives where that line starts. Blank lines are left out. (D233)
/// trace: D233
fn read_raw_lines(
    source: &str,
    mut position: usize,
    opened: Span,
    tokens: &mut Vec<Token>,
) -> Outcome<usize> {
    loop {
        let line_start = source
            .get(..position)
            .and_then(|before| before.rfind('\n'))
            .map_or(0, |at| at + 1);
        let end = line_end(source, line_start);
        let line = source.get(line_start..end).unwrap_or("");
        if line.trim() == "end inline.asm" {
            return Ok(line_start);
        }
        if !line.trim().is_empty() {
            let span = trimmed_line_span(source, line_start, end);
            if let Some(last) = tokens.last_mut() {
                last.gap_after = Gap::Line;
            }
            let text = line.trim().to_string();
            tokens.push(Token {
                lexeme: Lexeme::Raw(text.clone()),
                span,
                space_before: true,
                space_after: true,
                gap_before: Gap::Line,
                gap_after: Gap::Line,
                text,
                starts_a_line: true,
            });
        }
        if end >= source.len() {
            return Err(Report::refused(
                opened,
                "this assembly block never closes",
                "an assembly block holds raw assembly, one instruction per line, and closes with `end inline.asm` (D233)",
            ));
        }
        position = end + 1;
    }
}

impl Parser {
    /// `op < [$a: reading, $b: reading] -> bool:`, its body, and `end op <`: a type's own
    /// operator, written as a function named by it. (D236, D237)
    /// trace: D236, D237
    fn parse_operator(&mut self) -> Outcome<Function> {
        let start = self.advance().span; // `op`
        let (operator, operator_span) = self.parse_operator_word()?;
        let parameters = self.parse_parameters()?;
        // trace: D237
        if operator == "-" && parameters.len() == 1 {
            return Err(Report::not_designed(
                start.to(operator_span),
                "a type's own unary `-`",
                "12.3",
            ));
        }
        self.expect_symbol("->")?;
        let return_type = self.parse_type()?;
        let header_end = self.previous_span();
        self.expect_symbol(":")?;
        let types: Vec<String> = parameters
            .iter()
            .map(|parameter| self.text_of_span(parameter.written_type.span))
            .collect();
        let body = self.parse_block()?;
        let end_span = self.parse_operator_closer(&operator)?;
        Ok(Function {
            kind: FunctionKind::Plain,
            operator: Some(operator.clone()),
            type_parameters: Vec::new(),
            name: Name {
                text: format!("op {operator} [{}]", types.join(", ")),
                span: start.to(operator_span),
            },
            span: start.to(header_end),
            parameters,
            return_type,
            labels: None,
            defaults: Vec::new(),
            requires: Vec::new(),
            lasts: Vec::new(),
            sources: Vec::new(),
            ensures: Vec::new(),
            fails: None,
            body,
            end_span,
        })
    }

    /// The operator after `op`: arithmetic, the `div` narrowings, `==` or `<`. The comparisons
    /// that follow from those two, and what isn't maths, are refused. (D237)
    /// trace: D237
    fn parse_operator_word(&mut self) -> Outcome<(String, Span)> {
        const WHY_FOLLOWS: &str = "`!=`, `>`, `at_least` and `at_most` follow from `==` and `<`, so they can never disagree, and aren't written (D237)";
        const WHY_LANGUAGE: &str = "`and`, `or` and `not` stop early, which an operator couldn't promise, and `=`, `?`, `??` and `as` are part of the language, not maths (D237, D124)";
        let token = self.peek().clone();
        let text = token.text.clone();
        let refused = |what: String, why: &str| Err(Report::refused(token.span, what, why));
        match text.as_str() {
            "+" | "-" | "<" | "==" | "x" => {
                self.advance();
                Ok((text, token.span))
            }
            "div" => {
                self.advance();
                if !self.eat_symbol(".") {
                    return Err(Report::not_designed(
                        token.span,
                        "a type's own bare `div`",
                        "12.3",
                    ));
                }
                let narrowing = self.expect_name("a narrowing of `div`")?;
                if !matches!(narrowing.text.as_str(), "trunc" | "floor" | "exact") {
                    return Err(Report::not_designed(
                        token.span.to(narrowing.span),
                        format!("`div.{}`", narrowing.text),
                        "12.3",
                    ));
                }
                Ok((
                    format!("div.{}", narrowing.text),
                    token.span.to(narrowing.span),
                ))
            }
            "!=" | ">" | "at_least" | "at_most" => refused(
                format!("`{text}` follows from `==` and `<`, so a type doesn't write it"),
                WHY_FOLLOWS,
            ),
            "and" | "or" | "not" | "=" | "?" | "??" | "as" => refused(
                format!("`{text}` can't be defined for a type"),
                WHY_LANGUAGE,
            ),
            "<=" | ">=" => refused(
                format!("there's no `{text}`"),
                "`at_least` and `at_most` are words, and there is no `<=` or `>=` (D49)",
            ),
            "bit_and" | "bit_or" | "bit_xor" | "bit_not" | "shl" | "shr" | "mod" | "plus"
            | "minus" | "times" => Err(Report::not_designed(
                token.span,
                format!("a type's own `{text}`"),
                "12.3",
            )),
            _ => refused(
                format!("`{text}` isn't an operator a type can define"),
                "a type can define `+`, `-`, `x`, the `div` narrowings, `==` and `<` (D237)",
            ),
        }
    }

    /// `end op <`, repeating the operator.
    fn parse_operator_closer(&mut self, operator: &str) -> Outcome<Span> {
        let start = self.span();
        if !self.eat_word("end") || !self.eat_word("op") {
            return Err(Report::refused(
                start,
                format!("expected `end op {operator}`"),
                BLOCK_WHY,
            ));
        }
        let mut written = String::new();
        let mut end = self.previous_span();
        while !self.peek().starts_a_line && !matches!(self.peek().lexeme, Lexeme::End) {
            written.push_str(&self.peek().text);
            end = self.advance().span;
        }
        if written != operator {
            return Err(Report::refused(
                start.to(end),
                format!("this closes `op {written}`, but the open one is `op {operator}`"),
                "the closer repeats all of what it closes (D33, D121)",
            )
            .fix(format!("end op {operator}")));
        }
        Ok(start.to(end))
    }

    /// The source text a span covers, as tokens wrote it.
    fn text_of_span(&self, span: Span) -> String {
        let pieces: Vec<&Token> = self
            .tokens
            .iter()
            .filter(|token| token.span.start >= span.start && token.span.end <= span.end)
            .collect();
        let mut text = String::new();
        for (index, token) in pieces.iter().enumerate() {
            if index > 0 && token.gap_before != Gap::Touching {
                text.push(' ');
            }
            text.push_str(&token.text);
        }
        text
    }
}

const WHY_PARAMETER: &str =
    "a type parameter is `?` and a lowercase name, declared in `< >` straight after the name (D234)";
const WHY_REQUIREMENT: &str =
    "every type parameter writes what it needs, `any.type` when it needs nothing (D239, D243)";
const WHY_NEVER_MIXED: &str = "an `interface` lists what a type can do, and a `type_set` lists types, and the two are never mixed (D235)";

impl Parser {
    /// `?item`: a `?` and a lowercase name, touching. (D234)
    /// trace: D234
    fn parse_parameter_name(&mut self) -> Outcome<Name> {
        let mark = self.position;
        let start = self.advance().span; // `?`
        self.check_gaps(mark, mark, None, Some(Gap::Touching), SPACING_MARKS);
        let name = self.expect_name("a type parameter's name")?;
        let span = start.to(name.span);
        if name.text.chars().any(char::is_uppercase) {
            return Err(Report::refused(
                span,
                format!("`?{}` is written in lowercase", name.text),
                WHY_PARAMETER,
            )
            .fix(format!("?{}", name.text.to_lowercase())));
        }
        Ok(Name {
            text: name.text,
            span,
        })
    }

    /// `<?item: whole and ordered, ?other: any>` straight after a function's or struct's name,
    /// or nothing when there's none. (D234, D235, D239)
    /// trace: D234, D235, D239
    fn parse_type_parameters(&mut self) -> Outcome<Vec<TypeParameter>> {
        if !(self.at_symbol("<") && self.symbol_at(1, "?")) {
            return Ok(Vec::new());
        }
        let open = self.position;
        if self.peek().gap_before != Gap::Touching {
            let previous = open
                .checked_sub(1)
                .and_then(|at| self.tokens.get(at))
                .cloned();
            if let Some(previous) = previous {
                return Err(Report::refused(
                    previous.span.to(self.span()),
                    "type parameters go straight after the name",
                    WHY_PARAMETER,
                )
                .fix(format!("{}<", previous.text)));
            }
        }
        self.advance(); // `<`
        self.break_after.insert(open);
        self.check_gaps(open, open, None, Some(Gap::Touching), SPACING_MARKS);
        let mut parameters = Vec::new();
        loop {
            if !self.at_symbol("?") {
                return Err(self.expected("`?` and a type parameter's name"));
            }
            let name = self.parse_parameter_name()?;
            if !self.at_symbol(":") {
                return Err(Report::refused(
                    name.span,
                    format!(
                        "`{}` doesn't say what it needs",
                        self.text_of_span(name.span)
                    ),
                    WHY_REQUIREMENT,
                )
                .fix(format!("?{}: any.type", name.text)));
            }
            self.advance(); // `:`
            let mut requirement = vec![self.parse_path()?];
            while self.eat_word("and") {
                requirement.push(self.parse_path()?);
            }
            parameters.push(TypeParameter { name, requirement });
            if !self.eat_symbol(",") {
                break;
            }
        }
        let close = self.position;
        self.expect_symbol(">")?;
        self.type_brackets.push((open, close));
        self.check_gaps(close, close, Some(Gap::Touching), None, SPACING_MARKS);
        Ok(parameters)
    }

    /// `interface ordered<?self>:` with `includes`, `op` and `law` lines, and its closer. How a
    /// type slot is written isn't decided, so a second `?` in its `< >` isn't either. How laws
    /// are proven isn't decided, so a law is read past. (D235, D238)
    /// trace: D235, D238
    fn parse_interface(&mut self) -> Outcome<InterfaceDeclaration> {
        let start = self.advance().span; // `interface`
        let name = self.expect_name("the interface's name")?;
        if !(self.at_symbol("<") && self.symbol_at(1, "?")) {
            return Err(Report::not_designed(
                start.to(name.span),
                "an interface with no `<?self>`",
                "12.5",
            ));
        }
        let open = self.position;
        self.advance(); // `<`
        let own = self.parse_parameter_name()?;
        if self.at_symbol(",") || self.at_symbol(":") {
            return Err(Report::not_designed(
                self.span(),
                "how a type slot is written",
                "12.5",
            ));
        }
        let close = self.position;
        self.expect_symbol(">")?;
        self.type_brackets.push((open, close));
        self.expect_symbol(":")?;
        let mut includes = Vec::new();
        let mut operators = Vec::new();
        while !self.at_word("end") {
            self.mark_line_start();
            if self.eat_word("includes") {
                includes.push(self.expect_name("the interface it includes")?);
                self.expect_semicolon()?;
            } else if self.at_word("op") {
                let op_start = self.advance().span;
                let (operator, operator_span) = self.parse_operator_word()?;
                let parameters = self.parse_parameters()?;
                self.expect_symbol("->")?;
                let return_type = self.parse_type()?;
                self.expect_semicolon()?;
                operators.push(InterfaceOperator {
                    operator,
                    span: op_start.to(operator_span),
                    parameters,
                    return_type,
                });
            } else if self.eat_word("law") {
                while !self.at_symbol(";") && !matches!(self.peek().lexeme, Lexeme::End) {
                    self.advance();
                }
                self.expect_semicolon()?;
            } else {
                let line = self.span();
                let looks_like_a_type = self.symbol_at(1, ";");
                return Err(Report::refused(
                    line,
                    if looks_like_a_type {
                        "an interface lists operations, not types".to_string()
                    } else {
                        format!(
                            "an interface holds `includes`, `op` and `law` lines, and found {}",
                            self.describe_here()
                        )
                    },
                    WHY_NEVER_MIXED,
                ));
            }
        }
        self.expect_closer(&["interface", &name.text])?;
        Ok(InterfaceDeclaration {
            name,
            own,
            includes,
            operators,
        })
    }

    /// `type_set whole:`, a type per line, and its closer. (D235)
    /// trace: D235
    fn parse_type_set(&mut self) -> Outcome<TypeSetDeclaration> {
        self.advance(); // `type_set`
        let name = self.expect_name("the type set's name")?;
        self.expect_symbol(":")?;
        let mut members = Vec::new();
        while !self.at_word("end") {
            self.mark_line_start();
            if self.at_word("op") || self.at_word("includes") || self.at_word("law") {
                return Err(Report::refused(
                    self.span(),
                    "a type set lists types, not operations",
                    WHY_NEVER_MIXED,
                ));
            }
            members.push(self.parse_type()?);
            self.expect_semicolon()?;
        }
        self.expect_closer(&["type_set", &name.text])?;
        Ok(TypeSetDeclaration { name, members })
    }

    /// `reading satisfies ordered;`, or `reading .|. ordered;`. (D235)
    /// trace: D235
    fn parse_satisfies(&mut self) -> Outcome<Satisfies> {
        let start = self.span();
        let written_type = self.parse_type()?;
        if !self.eat_word("satisfies") && !self.eat_symbol(".|.") {
            return Err(self.expected("`satisfies`"));
        }
        let interface = self.expect_name("the interface")?;
        let end = self.expect_semicolon()?;
        Ok(Satisfies {
            span: start.to(end),
            written_type,
            interface,
        })
    }
}
