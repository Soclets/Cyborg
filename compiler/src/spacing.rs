//! Spacing is part of the syntax, so every program reads the same (D23, D213), and a line breaks only where it
//! visibly isn't finished (D219). This reads the gaps between tokens, using what the parser says each token does.

use crate::diagnostic::Diagnostic;
use crate::lexer::{Token, TokenKind};
use crate::parser::Role;
use crate::source::{FileId, Span};

const NO_SPACE_WHY: &str = "no space goes around `.`, `::`, `%`, `%%` or `@`, so every program reads the same (D213)";
const MARK_WHY: &str =
    "a mark or a bracket touches what it goes with, so `( $x )`, `- $x` and `read() ?` are refused (D23, D213)";
const BINARY_WHY: &str =
    "a binary operator, `=` and word operators included, has one space on each side, so `1+8` is refused (D23, D213)";
const COMMA_WHY: &str = "a comma or a label's `:` is followed by one space (D213)";
const BREAK_WHY: &str =
    "a line breaks only after a comma, an opening bracket or a binary operator, so it visibly isn't finished (D219)";

struct Spacing<'a> {
    file: FileId,
    text: &'a str,
    tokens: &'a [Token],
    roles: &'a [Role],
    errors: Vec<Diagnostic>,
    /// Gaps a rule already judged, by the index of the token before them.
    judged: Vec<bool>,
}

/// Checks every gap between tokens.
#[must_use]
pub fn check(file: FileId, text: &str, tokens: &[Token], roles: &[Role], line_breaks: bool) -> Vec<Diagnostic> {
    let mut spacing = Spacing { file, text, tokens, roles, errors: Vec::new(), judged: vec![false; tokens.len()] };
    spacing.marks();
    spacing.binary_operators();
    spacing.commas_and_labels();
    if line_breaks {
        spacing.line_breaks();
    } else {
        for index in 0..tokens.len() {
            if spacing.gap_after(index).contains('\n') {
                spacing.judged[index] = true;
            }
        }
    }
    spacing.other_gaps();
    spacing.errors.sort_by_key(|error| error.span.start);
    spacing.errors
}

impl Spacing<'_> {
    fn token_text(&self, index: usize) -> &str {
        self.tokens.get(index).and_then(|token| self.text.get(token.span.range())).unwrap_or("")
    }

    /// The text between a token and the next one.
    fn gap_after(&self, index: usize) -> &str {
        match (self.tokens.get(index), self.tokens.get(index + 1)) {
            (Some(this), Some(next)) => self.text.get(this.span.end as usize..next.span.start as usize).unwrap_or(""),
            _ => "",
        }
    }

    fn kind(&self, index: usize) -> Option<TokenKind> {
        self.tokens.get(index).map(|token| token.kind)
    }

    fn role(&self, index: usize) -> Role {
        self.roles.get(index).copied().unwrap_or(Role::Plain)
    }

    /// The first token of the atom ending at `index`: a `$name` starts at its `$`.
    fn atom_start(&self, index: usize) -> usize {
        if index > 0 && self.kind(index - 1) == Some(TokenKind::Dollar) && self.gap_after(index - 1).is_empty() {
            index - 1
        } else {
            index
        }
    }

    /// The last token of the atom starting at `index`.
    fn atom_end(&self, index: usize) -> usize {
        if self.kind(index) == Some(TokenKind::Dollar) && index + 1 < self.tokens.len() { index + 1 } else { index }
    }

    fn span(&self, first: usize, last: usize) -> Span {
        let start = self.tokens.get(first).map_or(0, |token| token.span.start as usize);
        let end = self.tokens.get(last).map_or(start, |token| token.span.end as usize);
        Span::new(self.file, start, end)
    }

    /// The source from one token to another, with every gap inside it closed up.
    fn joined(&self, first: usize, last: usize) -> String {
        (first..=last).map(|index| self.token_text(index)).collect()
    }

    fn refuse_fix(&mut self, first: usize, last: usize, fix: String, why: &str) {
        let span = self.span(first, last);
        self.errors.push(Diagnostic::refused_fix(span, fix, why));
    }

    /// No space goes around `.`, `::`, `%`, `%%` or `@`, and marks touch what they go with.
    fn marks(&mut self) {
        for index in 0..self.tokens.len() {
            let Some(kind) = self.kind(index) else { continue };
            let after = self.gap_after(index).to_string();
            match kind {
                TokenKind::DoubleColon | TokenKind::Dot if index > 0 => {
                    let before = self.gap_after(index - 1).to_string();
                    let spaced = |gap: &str| !gap.is_empty() && !gap.contains('\n');
                    if (spaced(&before) || spaced(&after)) && index + 1 < self.tokens.len() {
                        self.judged[index - 1] = true;
                        self.judged[index] = true;
                        let fix = self.joined(index - 1, index + 1);
                        self.refuse_fix(index - 1, index + 1, fix, NO_SPACE_WHY);
                    }
                }
                TokenKind::Percent | TokenKind::DoublePercent | TokenKind::At if is_space(&after) => {
                    self.judged[index] = true;
                    let fix = self.joined(index, index + 1);
                    self.refuse_fix(index, index + 1, fix, NO_SPACE_WHY);
                }
                TokenKind::Dollar if is_space(&after) => {
                    self.judged[index] = true;
                    let fix = self.joined(index, index + 1);
                    self.refuse_fix(index, index + 1, fix, MARK_WHY);
                }
                _ if self.role(index) == Role::Prefix && is_space(&after) => {
                    self.judged[index] = true;
                    let last = self.atom_end(index + 1);
                    let fix = self.joined(index, last);
                    self.refuse_fix(index, last, fix, MARK_WHY);
                }
                TokenKind::LeftParen | TokenKind::LeftSquare if is_space(&after) => {
                    self.judged[index] = true;
                    let last = self.atom_end(index + 1);
                    let fix = self.joined(index, last);
                    self.refuse_fix(index, last, fix, MARK_WHY);
                }
                TokenKind::LeftAngle if self.role(index) == Role::TypeBracket && is_space(&after) => {
                    self.judged[index] = true;
                    let last = self.atom_end(index + 1);
                    let fix = self.joined(index, last);
                    self.refuse_fix(index, last, fix, MARK_WHY);
                }
                TokenKind::RightAngle
                    if self.role(index) == Role::TypeBracket && index > 0 && is_space(self.gap_after(index - 1)) =>
                {
                    self.judged[index - 1] = true;
                    let first = self.atom_start(index - 1);
                    let fix = self.joined(first, index);
                    self.refuse_fix(first, index, fix, MARK_WHY);
                }
                TokenKind::RightParen | TokenKind::RightSquare if index > 0 && is_space(self.gap_after(index - 1)) => {
                    self.judged[index - 1] = true;
                    let first = self.atom_start(index - 1);
                    let fix = self.joined(first, index);
                    self.refuse_fix(first, index, fix, MARK_WHY);
                }
                TokenKind::Question
                    if index > 0 && is_space(self.gap_after(index - 1)) && self.role(index) != Role::Prefix =>
                {
                    self.judged[index - 1] = true;
                    let fix = self.joined(index - 1, index);
                    self.refuse_fix(index - 1, index, fix, MARK_WHY);
                }
                _ => {}
            }
        }
    }

    /// A binary operator has one space on each side.
    fn binary_operators(&mut self) {
        for index in 0..self.tokens.len() {
            let Role::BinaryStart { last } = self.role(index) else { continue };
            if index == 0 || last + 1 >= self.tokens.len() {
                continue;
            }
            let before = self.gap_after(index - 1).to_string();
            let after = self.gap_after(last).to_string();
            self.judged[index - 1] = true;
            self.judged[last] = true;
            if before.contains('\n') || after.contains('\n') {
                continue;
            }
            if before == " " && after == " " {
                continue;
            }
            let first = self.atom_start(index - 1);
            let end = self.atom_end(last + 1);
            let operator = self.text.get(self.span(index, last).range()).unwrap_or("").to_string();
            let fix = format!("{} {operator} {}", self.joined(first, index - 1), self.joined(last + 1, end));
            self.refuse_fix(first, end, fix, BINARY_WHY);
        }
    }

    /// A comma or a label's `:` is followed by one space.
    fn commas_and_labels(&mut self) {
        for index in 0..self.tokens.len() {
            let is_comma = self.kind(index) == Some(TokenKind::Comma);
            if !is_comma && self.role(index) != Role::LabelColon {
                continue;
            }
            if index + 1 >= self.tokens.len() {
                continue;
            }
            let after = self.gap_after(index).to_string();
            self.judged[index] = true;
            if after == " " || after.contains('\n') {
                continue;
            }
            let end = self.atom_end(index + 1);
            let fix = format!("{} {}", self.token_text(index), self.joined(index + 1, end));
            self.refuse_fix(index, end, fix, COMMA_WHY);
        }
    }

    /// A line breaks only after a comma, an opening bracket or a binary operator, and a bracket that breaks its line
    /// closes on its own line (D219).
    fn line_breaks(&mut self) {
        // Which opening brackets broke their line, by the index of their closer.
        let mut open: Vec<(usize, bool)> = Vec::new();
        let mut closer_of_broken = vec![false; self.tokens.len()];
        for index in 0..self.tokens.len() {
            match self.kind(index) {
                Some(TokenKind::LeftParen | TokenKind::LeftSquare) => {
                    open.push((index, self.gap_after(index).contains('\n')));
                }
                Some(TokenKind::RightParen | TokenKind::RightSquare) => {
                    if let Some((_, broke)) = open.pop() {
                        closer_of_broken[index] = broke;
                        let on_own_line = index > 0 && self.gap_after(index - 1).contains('\n');
                        if broke && !on_own_line {
                            let bracket = self.token_text(index).to_string();
                            self.errors.push(Diagnostic::refused(
                                self.span(index, index),
                                format!(
                                    "this `{bracket}` goes on its own line, since its opening bracket breaks the line"
                                ),
                                "a bracket that breaks its line has its closing one on its own line (D219)",
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        let binary_ends: Vec<bool> = {
            let mut ends = vec![false; self.tokens.len()];
            for index in 0..self.tokens.len() {
                if let Role::BinaryStart { last } = self.role(index)
                    && let Some(end) = ends.get_mut(last)
                {
                    *end = true;
                }
            }
            ends
        };
        for index in 0..self.tokens.len().saturating_sub(1) {
            if !self.gap_after(index).contains('\n') {
                continue;
            }
            self.judged[index] = true;
            let allowed = matches!(
                self.kind(index),
                Some(TokenKind::Semicolon | TokenKind::Comma | TokenKind::LeftParen | TokenKind::LeftSquare)
            ) || matches!(self.role(index), Role::BlockColon | Role::LineEnd)
                || binary_ends[index]
                || closer_of_broken[index + 1];
            if allowed {
                continue;
            }
            let first = self.atom_start(index);
            let next = index + 1;
            let joiner = match self.kind(next) {
                Some(
                    TokenKind::Dot
                    | TokenKind::DoubleColon
                    | TokenKind::RightParen
                    | TokenKind::RightSquare
                    | TokenKind::Semicolon
                    | TokenKind::Comma,
                ) => "",
                _ => " ",
            };
            let next_end = self.atom_end(next);
            let fix = format!("{}{joiner}{}", self.joined(first, index), self.joined(next, next_end));
            let span = self.span(first, index);
            self.errors.push(Diagnostic::refused_fix(span, fix, BREAK_WHY));
        }
    }

    /// Several spaces, or a tab, anywhere else on a line isn't decided.
    fn other_gaps(&mut self) {
        for index in 0..self.tokens.len().saturating_sub(1) {
            if self.judged[index] {
                continue;
            }
            let gap = self.gap_after(index);
            if gap.contains('\n') || gap.is_empty() || gap == " " {
                continue;
            }
            let start = self.tokens[index].span.end as usize;
            let span = Span::new(self.file, start, start + gap.len());
            self.errors.push(Diagnostic::not_designed(
                span,
                "several spaces, or a tab, between two things on a line",
                "not decided yet, and the standard doesn't say",
            ));
        }
    }
}

/// Whether a gap holds spaces or tabs, but no line break.
fn is_space(gap: &str) -> bool {
    !gap.is_empty() && !gap.contains('\n')
}
