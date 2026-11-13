//! Turns source text into tokens.
//!
//! Spacing is part of the syntax (D213, D219), so tokens keep exact spans and the gaps between them are read back
//! from the text when spacing is checked. Comments are kept apart from the tokens, since they can go anywhere (D118).

use crate::diagnostic::Diagnostic;
use crate::source::{FileId, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// A name or a keyword: letters, digits and `_`, not starting with a digit.
    Word,
    /// Digits, maybe with `_` between them.
    Integer,
    /// Digits, a `.`, and digits, maybe with an exponent.
    Float,
    /// A string literal, quotes included, with its escapes still written out.
    Text,
    LeftParen,
    RightParen,
    LeftSquare,
    RightSquare,
    /// `<`, which only ever opens a bracket (D278).
    LeftAngle,
    RightAngle,
    Comma,
    Semicolon,
    Colon,
    DoubleColon,
    Dot,
    Plus,
    Minus,
    EqualEqual,
    NotEqual,
    Question,
    DoubleQuestion,
    Tilde,
    Star,
    Dollar,
    At,
    /// The fullwidth `＠` Radium uses, which Cyborg refuses (D36).
    FullwidthAt,
    Percent,
    DoublePercent,
    Backtick,
    /// `=`, which assigns (D283).
    Equal,
    /// `:=`, refused (D277).
    ColonEqual,
    /// `->`, refused in favour of `returns` (D275).
    Arrow,
    /// `=>`, which an assembly block's `writes:` uses.
    FatArrow,
    /// `.|.`, an easter egg for `satisfies` (D240).
    Satisfies,
    /// `<=`, `>=`, refused in favour of words (D278).
    LessEqual,
    GreaterEqual,
    Slash,
    Backslash,
    Quote,
    /// A line of raw assembly in an `inline.asm:` block, which only LLVM's assembler reads (D233).
    AssemblyLine,
    /// Any other character.
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

/// A comment: `//`, `///`, or a named block (D118, D119).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment {
    pub span: Span,
    pub doc: bool,
    /// Whether code comes before it on its line.
    pub trailing: bool,
}

#[derive(Debug, Default)]
pub struct Lexed {
    pub tokens: Vec<Token>,
    pub comments: Vec<Comment>,
}

const COMMENT_WHY: &str = "a comment block has a required name, and a closer names the block it closes (D119)";

/// Reads a file's tokens and comments, adding comment errors to `errors`.
#[must_use]
pub fn lex(file: FileId, text: &str, errors: &mut Vec<Diagnostic>) -> Lexed {
    let mut lexed = Lexed::default();
    let bytes = text.as_bytes();
    let span = |start: usize, end: usize| Span::new(file, start, end);
    // Open comment blocks, with their names and where they opened.
    let mut blocks: Vec<(String, Span)> = Vec::new();
    let mut at = 0;
    let mut line_has_code = false;
    // Inside an `inline.asm:` block, past its opener: lines are raw, except its `reads:`, `writes:` and `clobbers:`,
    // until a line starting with `end` (D233).
    let mut assembly = false;
    while at < bytes.len() {
        // At the start of a line, a comment block's opener or closer counts only when it's alone on the line.
        let line_start = at == 0 || bytes.get(at - 1) == Some(&b'\n');
        if line_start {
            line_has_code = false;
            let line_end = text[at..].find('\n').map_or(text.len(), |offset| at + offset);
            let line = &text[at..line_end];
            let trimmed = line.trim();
            let indent = line.len() - line.trim_start().len();
            let line_span = span(at + indent, at + indent + trimmed.len());
            if let Some(name) = block_opener(trimmed) {
                if name.is_empty() {
                    errors.push(Diagnostic::refused(line_span, "this comment block has no name", COMMENT_WHY));
                }
                blocks.push((name.to_string(), line_span));
                at = (line_end + 1).min(bytes.len());
                continue;
            }
            if let Some(name) =
                trimmed.strip_prefix("end comment").filter(|rest| rest.is_empty() || rest.starts_with(' '))
            {
                let name = name.trim();
                match blocks.pop() {
                    Some((open, opened)) => {
                        if open != name && !open.is_empty() {
                            errors.push(
                                Diagnostic::refused_fix(line_span, format!("end comment {open}"), COMMENT_WHY)
                                    .with_because(opened, format!("`{open}` was opened here")),
                            );
                        }
                    }
                    None => errors.push(Diagnostic::refused(
                        line_span,
                        format!("there's no open comment block for `{trimmed}` to close"),
                        "a closer names the block it closes (D119)",
                    )),
                }
                at = (line_end + 1).min(bytes.len());
                continue;
            }
            if !blocks.is_empty() {
                at = (line_end + 1).min(bytes.len());
                continue;
            }
            let first_word = trimmed.split(|character: char| !character.is_alphanumeric() && character != '_').next();
            if assembly {
                if first_word == Some("end") {
                    assembly = false;
                } else if !trimmed.is_empty() && !matches!(first_word, Some("reads" | "writes" | "clobbers")) {
                    lexed.tokens.push(Token { kind: TokenKind::AssemblyLine, span: line_span });
                    at = (line_end + 1).min(bytes.len());
                    continue;
                }
            }
            if trimmed
                .strip_prefix("inline.asm:")
                .is_some_and(|rest| rest.trim().is_empty() || rest.trim().starts_with("//"))
            {
                assembly = true;
            }
        }
        let byte = bytes[at];
        if byte == b'\n' {
            at += 1;
            continue;
        }
        if byte == b' ' || byte == b'\t' || byte == b'\r' {
            at += 1;
            continue;
        }
        if text[at..].starts_with("//") {
            let line_end = text[at..].find('\n').map_or(text.len(), |offset| at + offset);
            let comment_text = text[at..line_end].trim_end_matches('\r');
            let comment_span = span(at, at + comment_text.len());
            let doc = comment_text.starts_with("///");
            if doc && line_has_code {
                errors.push(Diagnostic::refused_fix(
                    comment_span,
                    comment_text,
                    "a `///` sits above what it describes, so it can't trail code, though a `//` can (D206, D205)",
                ));
            } else {
                let marker = if doc { 3 } else { 2 };
                let after = &comment_text[marker..];
                if !after.is_empty() && !after.starts_with(' ') && !(doc && after.starts_with('/')) {
                    errors.push(Diagnostic::refused_fix(
                        comment_span,
                        format!("{} {after}", &comment_text[..marker]),
                        "`//` needs a space after it, and `///` is its own form (D119)",
                    ));
                }
            }
            lexed.comments.push(Comment { span: comment_span, doc, trailing: line_has_code });
            at = line_end;
            continue;
        }
        line_has_code = true;
        let start = at;
        let (kind, end) = token_at(text, at);
        lexed.tokens.push(Token { kind, span: span(start, end) });
        at = end;
    }
    for (name, opened) in blocks {
        errors.push(Diagnostic::refused(opened, format!("the comment block `{name}` never closes"), COMMENT_WHY));
    }
    lexed
}

/// `comment name:` alone on a line gives the name, and `comment:` gives an empty one.
fn block_opener(trimmed: &str) -> Option<&str> {
    let rest = trimmed.strip_prefix("comment")?;
    let name = rest.strip_suffix(':')?;
    if name.is_empty() {
        return Some("");
    }
    let name = name.strip_prefix(' ')?;
    if !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
        Some(name)
    } else {
        None
    }
}

fn is_word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// The token starting at `at`, and where it ends.
fn token_at(text: &str, at: usize) -> (TokenKind, usize) {
    let bytes = text.as_bytes();
    let byte = bytes[at];
    let next = bytes.get(at + 1).copied();
    let third = bytes.get(at + 2).copied();
    if byte.is_ascii_alphabetic() || byte == b'_' {
        let mut end = at;
        while end < bytes.len() && is_word_byte(bytes[end]) {
            end += 1;
        }
        return (TokenKind::Word, end);
    }
    if byte.is_ascii_digit() {
        return number_at(bytes, at);
    }
    if byte == b'"' {
        let mut end = at + 1;
        while end < bytes.len() {
            match bytes[end] {
                b'\\' => end += 2,
                b'"' => return (TokenKind::Text, end + 1),
                b'\n' => return (TokenKind::Text, end),
                _ => end += 1,
            }
        }
        return (TokenKind::Text, bytes.len());
    }
    let two = |kind| (kind, at + 2);
    let one = |kind| (kind, at + 1);
    match (byte, next) {
        (b'(', _) => one(TokenKind::LeftParen),
        (b')', _) => one(TokenKind::RightParen),
        (b'[', _) => one(TokenKind::LeftSquare),
        (b']', _) => one(TokenKind::RightSquare),
        (b'<', Some(b'=')) => two(TokenKind::LessEqual),
        (b'<', _) => one(TokenKind::LeftAngle),
        (b'>', Some(b'=')) => two(TokenKind::GreaterEqual),
        (b'>', _) => one(TokenKind::RightAngle),
        (b',', _) => one(TokenKind::Comma),
        (b';', _) => one(TokenKind::Semicolon),
        (b':', Some(b':')) => two(TokenKind::DoubleColon),
        (b':', Some(b'=')) => two(TokenKind::ColonEqual),
        (b':', _) => one(TokenKind::Colon),
        (b'.', Some(b'|')) if third == Some(b'.') => (TokenKind::Satisfies, at + 3),
        (b'.', _) => one(TokenKind::Dot),
        (b'+', _) => one(TokenKind::Plus),
        (b'-', Some(b'>')) => two(TokenKind::Arrow),
        (b'-', _) => one(TokenKind::Minus),
        (b'=', Some(b'=')) => two(TokenKind::EqualEqual),
        (b'=', Some(b'>')) => two(TokenKind::FatArrow),
        (b'=', _) => one(TokenKind::Equal),
        (b'!', Some(b'=')) => two(TokenKind::NotEqual),
        (b'?', Some(b'?')) => two(TokenKind::DoubleQuestion),
        (b'?', _) => one(TokenKind::Question),
        (b'~', _) => one(TokenKind::Tilde),
        (b'*', _) => one(TokenKind::Star),
        (b'$', _) => one(TokenKind::Dollar),
        (b'@', _) => one(TokenKind::At),
        (b'%', Some(b'%')) => two(TokenKind::DoublePercent),
        (b'%', _) => one(TokenKind::Percent),
        (b'`', _) => one(TokenKind::Backtick),
        (b'/', _) => one(TokenKind::Slash),
        (b'\\', _) => one(TokenKind::Backslash),
        (b'\'', _) => one(TokenKind::Quote),
        _ => {
            let width = text[at..].chars().next().map_or(1, char::len_utf8);
            if text[at..].starts_with('＠') {
                (TokenKind::FullwidthAt, at + width)
            } else {
                (TokenKind::Unknown, at + width)
            }
        }
    }
}

/// An integer, `86_400`, or a float, `2.5` or `6.02214076e23`.
fn number_at(bytes: &[u8], at: usize) -> (TokenKind, usize) {
    let digits = |mut end: usize| {
        while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b'_') {
            end += 1;
        }
        end
    };
    let mut end = digits(at);
    let mut kind = TokenKind::Integer;
    if bytes.get(end) == Some(&b'.') && bytes.get(end + 1).is_some_and(u8::is_ascii_digit) {
        kind = TokenKind::Float;
        end = digits(end + 1);
    }
    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let mut exponent = end + 1;
        if matches!(bytes.get(exponent), Some(b'+' | b'-')) {
            exponent += 1;
        }
        if bytes.get(exponent).is_some_and(u8::is_ascii_digit) {
            kind = TokenKind::Float;
            end = digits(exponent);
        }
    }
    // A word stuck to a number, like `2nd`, stays one token, so it can be refused as a whole.
    while end < bytes.len() && (bytes[end].is_ascii_alphabetic()) {
        end += 1;
    }
    (kind, end)
}
