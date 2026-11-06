//! How Cyborg is written: the one place that knows spelling, so a spelling change stays small.

use crate::ast::{
    Argument, Arm, BinaryOperator, BorrowSources, CompareOperator, Contract, Describe, Expression,
    ExpressionKind, FailureEntry, FailureSet, FieldDeclaration, FieldValue, Fill, Function,
    FunctionKind, HandWord, Handler, HandlerSet, Handling, Import, Item, LabelLine, Name,
    Narrowing, Parameter, ParameterWord, Path, Program, ShieldContent, ShieldItem, Statement,
    StatementKind, StructDeclaration, Together, TogetherOperator, TypeExpression, TypeKind,
    UnionCase, UnionDeclaration, UnitMeaning, UnitMember, UnitSet, Use,
};
use crate::report::{Outcome, Report, ReportExtra, Span};

/// Words that clash with a name of yours, which is then written `my::name`. (D75)
const CLASHING_WORDS: &[&str] = &[
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
    "bake",
    "narrowings",
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
    "f32",
    "f64",
];

/// trace: D75
pub fn is_clashing(word: &str) -> bool {
    CLASHING_WORDS.contains(&word)
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
    Text(String),
    Symbol(&'static str),
    End,
}

#[derive(Clone, Debug)]
struct Token {
    lexeme: Lexeme,
    span: Span,
    space_before: bool,
    space_after: bool,
    /// Whether a newline comes between this token and the one before it.
    starts_a_line: bool,
}

/// Longest first, so `??` isn't read as two `?`.
const SYMBOLS: &[&str] = &[
    "::", "->", "==", "!=", "??", "<=", ">=", ":=", "=>", "//", ".", ":", ";", ",", "(", ")", "[",
    "]", "<", ">", "=", "+", "-", "~", "?", "%", "@", "*", "/",
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
        }
        if rest.starts_with("//") {
            position = skip_line_comment(source, position)?;
            continue;
        }
        let start = position;
        let lexeme = if character == '"' {
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
        tokens.push(Token {
            lexeme,
            span: Span::new(start, position),
            space_before,
            space_after,
            starts_a_line,
        });
    }
    tokens.push(Token {
        lexeme: Lexeme::End,
        span: Span::new(source.len(), source.len()),
        space_before: true,
        space_after: true,
        starts_a_line: true,
    });
    Ok(tokens)
}

fn lex_text(source: &str, position: &mut usize) -> Outcome<Lexeme> {
    let start = *position;
    let rest = source.get(start + 1..).unwrap_or("");
    let mut text = String::new();
    for character in rest.chars() {
        match character {
            '"' => {
                *position = start + 1 + text.len() + 1;
                return Ok(Lexeme::Text(text));
            }
            '\\' => {
                let at = start + 1 + text.len();
                return Err(Report::not_designed(
                    Span::new(at, at + 1),
                    "`\\` in a string",
                    "12.2",
                ));
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
pub fn parse(source: &str) -> Outcome<Program> {
    if source.starts_with('\u{feff}') {
        return Err(Report::not_designed(
            Span::new(0, 3),
            "a byte order mark at the start of a source file",
            "",
        ));
    }
    let tokens = lex(source)?;
    let mut parser = Parser {
        tokens,
        position: 0,
        depth: 0,
    };
    parser.parse_program()
}

struct Parser {
    tokens: Vec<Token>,
    position: usize,
    depth: usize,
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
        if self.position + 1 < self.tokens.len() {
            self.position += 1;
        }
        token
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
            Lexeme::Text(_) => "a string".to_string(),
            Lexeme::Symbol(symbol) => format!("`{symbol}`"),
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
        if !self.eat_word("end") {
            return Err(self.expected(&format!("`{wanted}`")));
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

    fn parse_closer_piece(&mut self) -> Option<String> {
        let Lexeme::Word(first) = self.peek().lexeme.clone() else {
            return None;
        };
        self.advance();
        let mut text = first;
        while self.at_symbol(".") {
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
        while !matches!(self.peek().lexeme, Lexeme::End) {
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
                let limb = self.eat_symbol("@");
                let name = self.expect_name("what to import")?;
                self.expect_semicolon()?;
                program.imports.push(Import { limb, name });
            } else if self.at_word("use") {
                self.advance();
                let path = self.parse_path()?;
                if !self.eat_word("as") {
                    // trace: D93
                    return Err(Report::refused(
                        path.span,
                        "`use` always writes `as`",
                        "`use` gives a path a shorter name, and `as` is always written (D93)",
                    ));
                }
                let short = self.expect_name("the shorter name")?;
                self.expect_semicolon()?;
                program.uses.push(Use { path, short });
            } else {
                let item = self.parse_item()?;
                program.items.push(item);
            }
        }
        Ok(program)
    }

    fn parse_item(&mut self) -> Outcome<Item> {
        if self.at_word("fn") {
            return Ok(Item::Function(self.parse_function()?));
        }
        if self.at_word("struct") {
            return Ok(Item::Struct(self.parse_struct()?));
        }
        if self.at_word("union") {
            return Ok(Item::Union(self.parse_union()?));
        }
        if self.at_word("narrowings") {
            return self.parse_narrowings();
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
        if self.word_at(1, "is") {
            return Err(Report::not_designed(
                self.span(),
                "declaring that a type fits a bound",
                "12.5",
            ));
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
        Err(self.expected("`fn`, `struct`, `union.tagged`, `narrowings`, `const` or `shield`"))
    }

    fn parse_function(&mut self) -> Outcome<Function> {
        let start = self.advance().span; // `fn`
        let mut narrowing = String::new();
        while self.at_symbol(".") {
            self.advance();
            let piece = self.expect_name("a narrowing of `fn`")?;
            narrowing.push('.');
            narrowing.push_str(&piece.text);
        }
        let kind = match narrowing.as_str() {
            "" => FunctionKind::Plain,
            ".main" => FunctionKind::Main,
            ".pure" => FunctionKind::Pure,
            ".pure.baked" => FunctionKind::PureBaked,
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
                text: "main".to_string(),
                span: start.to(self.previous_span()),
            }
        } else {
            self.expect_name("the function's name")?
        };
        if self.at_symbol("<") {
            return Err(Report::not_designed(
                self.span(),
                "the spelling of generics",
                "12.5",
            ));
        }
        let parameters = self.parse_parameters()?;
        self.expect_symbol("->")?;
        let return_type = self.parse_type()?;
        let header_end = self.previous_span();
        // trace: D52
        if self.at_word("or") && self.word_at(1, "fail") {
            return Err(Report::refused(
                self.span(),
                "`or fail[…]` is the older spelling",
                "a function lists its failures in its body, with `fails` (D52)",
            ));
        }
        self.expect_symbol(":")?;
        let mut function = Function {
            kind,
            span: start.to(header_end),
            name,
            parameters,
            return_type,
            labels: None,
            defaults: Vec::new(),
            requires: Vec::new(),
            ensures: Vec::new(),
            fails: None,
            body: Vec::new(),
            end_span: start,
        };
        self.parse_top_of_body(&mut function)?;
        function.body = self.parse_block()?;
        function.end_span = self.parse_function_closer(kind, &function.name.text)?;
        Ok(function)
    }

    fn parse_function_closer(&mut self, kind: FunctionKind, name: &str) -> Outcome<Span> {
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
    fn parse_top_of_body(&mut self, function: &mut Function) -> Outcome<()> {
        let mut phase = 0;
        loop {
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
                    "the top of a body goes `labels`, `defaults`, `requires`, `ensures`, `fails`, then the code (D114)",
                ));
            }
            if word_phase == phase && word_phase != 3 && word_phase != 4 {
                return Err(Report::refused(
                    self.span(),
                    format!("`{word}` is written once"),
                    "the top of a body goes `labels`, `defaults`, `requires`, `ensures`, `fails`, then the code (D114)",
                ));
            }
            phase = word_phase;
            match word_phase {
                1 => function.labels = Some(self.parse_labels()?),
                2 => function.defaults = self.parse_defaults()?,
                3 => {
                    let contract = self.parse_contract()?;
                    function.requires.push(contract);
                }
                4 => {
                    let contract = self.parse_contract()?;
                    function.ensures.push(contract);
                }
                _ => function.fails = Some(self.parse_fails()?),
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
            let label = self.expect_name("the label")?;
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
        self.expect_semicolon()?;
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
        let name = self.expect_name("the struct's name")?;
        if self.at_symbol("<") {
            return Err(Report::not_designed(
                self.span(),
                "the spelling of generics",
                "12.5",
            ));
        }
        self.expect_symbol(":")?;
        let mut fields = Vec::new();
        while !self.at_word("end") {
            let field_name = self.expect_name("a field, with no `$`")?;
            self.expect_symbol(":")?;
            let written_type = self.parse_type()?;
            self.expect_semicolon()?;
            fields.push(FieldDeclaration {
                name: field_name,
                written_type,
            });
        }
        if must_give {
            self.expect_closer(&["struct.must_Give", &name.text])?;
        } else {
            self.expect_closer(&["struct", &name.text])?;
        }
        Ok(StructDeclaration {
            must_give,
            name,
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
        let name = self.expect_name("the union's name")?;
        if self.at_symbol("<") {
            return Err(Report::not_designed(
                self.span(),
                "the spelling of generics",
                "12.5",
            ));
        }
        self.expect_symbol(":")?;
        let mut cases = Vec::new();
        while !self.at_word("end") {
            let case_name = self.expect_name("a case")?;
            let fields = if self.at_symbol("[") {
                self.parse_field_declarations_in_brackets()?
            } else {
                Vec::new()
            };
            cases.push(UnionCase {
                name: case_name,
                fields,
            });
        }
        self.expect_closer(&["union.tagged", &name.text])?;
        Ok(UnionDeclaration { name, cases })
    }

    fn parse_narrowings(&mut self) -> Outcome<Item> {
        let start = self.advance().span; // `narrowings`
        if !self.at_symbol(".") {
            return Err(self.expected("`.failure`, `.handler` or `.unit`"));
        }
        self.advance();
        let sort = self.expect_name("`failure`, `handler` or `unit`")?;
        match sort.text.as_str() {
            "failure" => self.parse_failure_set(),
            "handler" => self.parse_handler_set(),
            "unit" => self.parse_unit_set(),
            other => Err(Report::not_designed(
                start.to(sort.span),
                format!("`narrowings.{other}`"),
                "",
            )),
        }
    }

    fn parse_failure_set(&mut self) -> Outcome<Item> {
        let name = self.expect_name("the set's name")?;
        self.expect_symbol(":")?;
        let mut members = Vec::new();
        while !self.at_word("end") {
            members.push(self.parse_failure_entry()?);
            self.expect_semicolon()?;
        }
        self.expect_closer(&["narrowings.failure", &name.text])?;
        Ok(Item::FailureSet(FailureSet { name, members }))
    }

    fn parse_handler_set(&mut self) -> Outcome<Item> {
        let name = self.expect_name("the set's name")?;
        self.expect_symbol(":")?;
        let mut handlers = Vec::new();
        while !self.at_word("end") {
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
        self.expect_closer(&["narrowings.handler", &name.text])?;
        Ok(Item::HandlerSet(HandlerSet { name, handlers }))
    }

    fn parse_unit_set(&mut self) -> Outcome<Item> {
        let family = self.expect_name("the family's name")?;
        self.expect_symbol(":")?;
        let mut members = Vec::new();
        while !self.at_word("end") {
            let path = self.parse_path()?;
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
        self.expect_closer(&["narrowings.unit", &family.text])?;
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
        self.expect_symbol(":")?;
        let written_type = self.parse_type()?;
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
            if self.at_symbol("@") {
                let path = self.parse_path()?;
                let union_name = self.expect_name("the union's name")?;
                self.expect_symbol(":")?;
                // The fields are only read past: nothing can use them until C's layouts exist.
                while !self.at_word("end") {
                    self.expect_name("a field")?;
                    self.expect_symbol(":")?;
                    self.parse_type()?;
                }
                self.expect_limb_closer(&path.text, &union_name.text)?;
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
        if self.at_symbol("?") {
            return Err(Report::not_designed(
                start,
                "the spelling of generics",
                "12.5",
            ));
        }
        if self.at_word("Lend") {
            return self.parse_stored_borrow();
        }
        let path = self.parse_path()?;
        let mut fill = Vec::new();
        if self.at_symbol("<") {
            fill = self.parse_fill()?;
        }
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
            },
        })
    }

    fn parse_fill(&mut self) -> Outcome<Vec<Fill>> {
        let open = self.advance(); // `<`
                                   // trace: D23
        if open.space_after {
            return Err(Report::refused(
                open.span,
                "an opening `<` has no space after it",
                "an opening `<` has no space after it, and a closing `>` has none before it (D23)",
            ));
        }
        let mut fill = Vec::new();
        while !self.at_symbol(">") {
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
        let close = self.advance();
        if close.space_before {
            return Err(Report::refused(
                close.span,
                "a closing `>` has no space before it",
                "an opening `<` has no space after it, and a closing `>` has none before it (D23)",
            ));
        }
        Ok(fill)
    }

    fn parse_stored_borrow(&mut self) -> Outcome<TypeExpression> {
        let start = self.advance().span; // `Lend`
        let mut mutable = false;
        if self.at_symbol(".") {
            self.advance();
            let narrowing = self.expect_name("`mut`")?;
            if narrowing.text != "mut" {
                return Err(self.expected("`mut`"));
            }
            mutable = true;
        }
        if !self.eat_word("from") {
            return Err(self.expected("`from`"));
        }
        let sources = if self.eat_word("program") {
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
        while !self.at_word("end") && !self.at_word("case") {
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
        let start = self.span();
        if let Lexeme::Word(word) = &self.peek().lexeme {
            match word.as_str() {
                "let" => return self.parse_let(),
                "const" => return self.parse_constant(),
                "change" => return self.parse_change(),
                "if" => return self.parse_if(),
                "match" => return self.parse_match(),
                "for" => return self.parse_for(),
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
                "else" | "while" | "loop" | "break" | "continue" => {
                    return Err(Report::not_designed(start, format!("`{word}`"), "12.4"));
                }
                "labels" | "defaults" | "requires" | "ensures" | "fails" => {
                    // trace: D114
                    return Err(Report::refused(
                        start,
                        format!("`{word}` goes first in the body"),
                        "the top of a body goes `labels`, `defaults`, `requires`, `ensures`, `fails`, then the code (D114)",
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

    fn parse_let(&mut self) -> Outcome<Statement> {
        let start = self.advance().span; // `let`
        let mutable = self.eat_word("mut");
        let name = self.expect_variable("the variable's name, starting with `$`")?;
        self.expect_symbol(":")?;
        let written_type = self.parse_type()?;
        if self.at_symbol(":=") {
            return Err(Report::refused(
                self.span(),
                "`:=` is gone",
                "a constant is `const`, with no `let` (D83)",
            ));
        }
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

    fn parse_if(&mut self) -> Outcome<Statement> {
        let start = self.advance().span; // `if`
        let condition = self.parse_expression()?;
        let colon = self.position;
        self.expect_symbol(":")?;
        // `if $s: status.failed:` reads a type then a second `:`. Anything else is a plain block.
        if let Ok(written_type) = self.parse_type() {
            if self.eat_symbol(":") {
                let body = self.parse_block()?;
                let end = self.expect_closer(&["if"])?;
                return Ok(Statement {
                    span: start.to(end),
                    kind: StatementKind::Test {
                        subject: condition,
                        written_type,
                        body,
                    },
                });
            }
        }
        self.position = colon;
        self.expect_symbol(":")?;
        let body = self.parse_block()?;
        let end = self.expect_closer(&["if"])?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::If { condition, body },
        })
    }

    fn parse_arms(&mut self) -> Outcome<Vec<Arm>> {
        let mut arms = Vec::new();
        while self.at_word("case") {
            self.advance();
            let pattern = self.parse_path()?;
            self.expect_symbol(":")?;
            let body = self.parse_block()?;
            arms.push(Arm { pattern, body });
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

    fn parse_for(&mut self) -> Outcome<Statement> {
        let start = self.advance().span; // `for`
        let name = self.expect_variable("the item's name, starting with `$`")?;
        if !self.eat_word("in") {
            return Err(self.expected("`in`"));
        }
        if self.at_word("range") {
            return Err(Report::not_designed(self.span(), "ranges", "12.4"));
        }
        let collection = self.parse_expression()?;
        self.expect_symbol(":")?;
        let body = self.parse_block()?;
        let end = self.expect_closer(&["for"])?;
        Ok(Statement {
            span: start.to(end),
            kind: StatementKind::For {
                name,
                collection,
                body,
            },
        })
    }

    fn parse_shield_statement(&mut self) -> Outcome<Statement> {
        let start = self.span();
        let (name, reason) = self.parse_shield_header()?;
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
        let result = self.parse_fallback();
        self.leave();
        result
    }

    fn parse_fallback(&mut self) -> Outcome<Expression> {
        let value = self.parse_or()?;
        if self.at_symbol("??") {
            self.advance();
            Self::refuse_bit_mix(&value)?;
            let fallback = self.parse_or()?;
            Self::refuse_as_on_the_right(&fallback)?;
            Self::refuse_bit_mix(&fallback)?;
            return Ok(Expression {
                span: value.span.to(fallback.span),
                kind: ExpressionKind::Fallback {
                    value: Box::new(value),
                    fallback: Box::new(fallback),
                },
            });
        }
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

    fn parse_and(&mut self) -> Outcome<Expression> {
        let mut left = self.parse_compare()?;
        let depth_before = self.depth;
        while self.at_word("and") {
            self.enter()?;
            let operator = if self.logic_narrowing("and")? {
                BinaryOperator::AndBoth
            } else {
                BinaryOperator::And
            };
            let right = self.parse_compare()?;
            Self::refuse_as_on_the_right(&right)?;
            left = Self::binary(operator, left, right)?;
        }
        self.depth = depth_before;
        Ok(left)
    }

    /// Reads `and` or `or`, and says whether it was narrowed to run both sides. (D124)
    fn logic_narrowing(&mut self, word: &str) -> Outcome<bool> {
        let start = self.advance().span;
        if !self.at_symbol(".") {
            return Ok(false);
        }
        self.advance();
        let narrowing = self.expect_name("a narrowing")?;
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
        let token = self.peek().clone();
        // trace: D23
        if matches!(operator, CompareOperator::Less | CompareOperator::Greater)
            && !(token.space_before && token.space_after)
        {
            let word = operator.word();
            return Err(Report::refused(
                token.span,
                format!("a comparison has a space on each side of `{word}`"),
                "a comparison has a space on each side, so `$a<$b` is refused (D23)",
            )
            .fix(format!(" {word} ")));
        }
        self.advance();
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
        if !self.at_symbol(".") {
            // trace: D48, D97
            return Err(Report::refused(
                start,
                format!("there's no bare `{word}`, so pick a narrowing"),
                format!(
                    "`{word}` has no single right answer, so you pick one ({})",
                    if word == "div" { "D48" } else { "D97" }
                ),
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
        let value = if self.eat_word("not") {
            let inner = self.parse_unary()?;
            Self::refuse_bit_mix(&inner)?;
            Self::refuse_as_on_the_right(&inner)?;
            Expression {
                span: start.to(inner.span),
                kind: ExpressionKind::Not(Box::new(inner)),
            }
        } else if self.eat_word("bit_not") {
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
            // trace: D116
            self.advance();
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
            self.advance();
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
            self.advance();
            let target = self.parse_type()?;
            value = Expression {
                span: value.span.to(target.span),
                kind: ExpressionKind::As {
                    value: Box::new(value),
                    target,
                },
            };
            while self.at_symbol("?") {
                let end = self.advance().span;
                value = Expression {
                    span: value.span.to(end),
                    kind: ExpressionKind::Try(Box::new(value)),
                };
            }
        }
        Ok(value)
    }

    /// A place, for `change`.
    fn parse_postfix_only(&mut self) -> Outcome<Expression> {
        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Outcome<Expression> {
        let mut value = self.parse_primary()?;
        loop {
            if self.at_symbol(".") {
                value = self.parse_after_dot(value)?;
            } else if self.at_symbol("?") {
                let end = self.advance().span;
                value = Expression {
                    span: value.span.to(end),
                    kind: ExpressionKind::Try(Box::new(value)),
                };
            } else if self.at_symbol("[") {
                return Err(Report::not_designed(
                    self.span(),
                    "indexing, and what a bad index does",
                    "12.2",
                ));
            } else {
                return Ok(value);
            }
        }
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

    fn parse_arguments(&mut self) -> Outcome<(Vec<Argument>, Span)> {
        self.expect_symbol("(")?;
        let mut arguments = Vec::new();
        while !self.at_symbol(")") {
            let label = if matches!(self.peek().lexeme, Lexeme::Word(_)) && self.symbol_at(1, ":") {
                let name = self.expect_name("a label")?;
                self.advance();
                Some(name)
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
                    self.span(),
                    "`=` sets a field",
                    "building one uses `[ ]`, with `=` setting a field (D57)",
                )
                .fix("="));
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
        let inexact = self.eat_symbol("~");
        // trace: D59
        let negative = self.eat_symbol("-");
        let lexeme = self.peek().lexeme.clone();
        let kind = match lexeme {
            Lexeme::Integer(digits) if !inexact => ExpressionKind::Integer { digits, negative },
            Lexeme::Integer(digits) | Lexeme::Float(digits) => ExpressionKind::Float {
                digits,
                negative,
                inexact,
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

    fn parse_primary_inner(&mut self) -> Outcome<Expression> {
        let start = self.span();
        match self.peek().lexeme.clone() {
            Lexeme::Integer(_) | Lexeme::Float(_) | Lexeme::Symbol("-" | "~") => {
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
            "empty" => {
                self.advance();
                return Ok(Expression {
                    span: start,
                    kind: ExpressionKind::Empty,
                });
            }
            "void" => {
                self.advance();
                return Ok(Expression {
                    span: start,
                    kind: ExpressionKind::Void,
                });
            }
            "inf" => {
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
                Lexeme::Word(word) | Lexeme::Integer(word) | Lexeme::Float(word) => {
                    text.push_str(word);
                }
                Lexeme::Variable(name) => {
                    text.push('$');
                    text.push_str(name);
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
    starts_a_line: true,
};
