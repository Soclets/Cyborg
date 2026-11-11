//! Turns tokens into the syntax tree.
//!
//! The parser also records what each token is doing, such as being a binary operator or a label's `:`, since spacing
//! depends on that (D213), and the spacing checks read it afterwards.

use crate::diagnostic::Diagnostic;
use crate::lexer::{Token, TokenKind};
use crate::source::{FileId, Span};
use crate::syntax::{
    Argument, Arm, BinaryOperator, Branch, Case, Closer, Condition, Expression, ExpressionKind, FailEntry, FailList,
    Family, FamilyEntry, FamilyEntryKind, Field, File, For, Function, Generic, GenericKind, Handling, If, Import, Item,
    ItemKind, Joint, Label, Let, Mark, Match, Name, Ownership, Parameter, ParameterOwnership, Parameters, Path,
    Requirement, RequirementEntry, Requirements, Segment, Shield, ShortName, Statement, StatementKind, Struct, Type,
    TypeArgument, TypeArgumentValue, TypeKind, UnaryOperator, Union, UnitDefinition, Use,
};

/// What a token does, for the spacing checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Plain,
    /// The first token of a binary operator, with the token index of its last.
    BinaryStart {
        last: usize,
    },
    /// A prefix mark that touches what follows: unary `-`, `~`.
    Prefix,
    /// A `:` after a label or a name being typed, followed by one space.
    LabelColon,
    /// A `:` that opens a block.
    BlockColon,
    /// The last token of a line that ends with no `;` or `:`, like a closer, `end if`.
    LineEnd,
}

pub struct Parsed {
    pub file: File,
    pub roles: Vec<Role>,
}

const STATEMENT_WHY: &str = "`;` ends a statement, `:` opens a block, and `end` closes it (D33)";

/// Words that are operators, so they never start a unit after a number.
const OPERATOR_WORDS: &[&str] = &[
    "x",
    "div",
    "mod",
    "below",
    "above",
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
    "plus",
    "as",
    "set_to",
    "otherwise",
    "in",
    "per",
    "offset",
    "then",
    "satisfies",
    "with",
    "lasts_at_least",
];

pub struct Parser<'source> {
    file: FileId,
    text: &'source str,
    tokens: &'source [Token],
    at: usize,
    pub roles: Vec<Role>,
    pub errors: Vec<Diagnostic>,
}

/// A parse failure that's already been reported.
#[derive(Debug)]
pub struct Reported;

type Parse<T> = Result<T, Reported>;

impl<'source> Parser<'source> {
    #[must_use]
    pub fn new(file: FileId, text: &'source str, tokens: &'source [Token]) -> Parser<'source> {
        Parser { file, text, tokens, at: 0, roles: vec![Role::Plain; tokens.len()], errors: Vec::new() }
    }

    // ----- Looking at tokens -----

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.at).copied()
    }

    fn peek_at(&self, ahead: usize) -> Option<Token> {
        self.tokens.get(self.at + ahead).copied()
    }

    fn kind(&self) -> Option<TokenKind> {
        self.peek().map(|token| token.kind)
    }

    fn kind_at(&self, ahead: usize) -> Option<TokenKind> {
        self.peek_at(ahead).map(|token| token.kind)
    }

    fn text_of(&self, token: Token) -> &'source str {
        self.text.get(token.span.range()).unwrap_or("")
    }

    /// The word at the current token, if it's a word.
    fn word(&self) -> Option<&'source str> {
        self.peek().filter(|token| token.kind == TokenKind::Word).map(|token| self.text_of(token))
    }

    fn word_at(&self, ahead: usize) -> Option<&'source str> {
        self.peek_at(ahead).filter(|token| token.kind == TokenKind::Word).map(|token| self.text_of(token))
    }

    fn is_word(&self, word: &str) -> bool {
        self.word() == Some(word)
    }

    fn is(&self, kind: TokenKind) -> bool {
        self.kind() == Some(kind)
    }

    fn advance(&mut self) -> Option<Token> {
        let token = self.peek();
        if token.is_some() {
            self.at += 1;
        }
        token
    }

    /// The span of the token just consumed.
    fn previous_span(&self) -> Span {
        self.at.checked_sub(1).and_then(|index| self.tokens.get(index)).map_or(self.end_span(), |token| token.span)
    }

    /// An empty span at the end of the file.
    fn end_span(&self) -> Span {
        let end = self.text.len();
        Span::new(self.file, end, end)
    }

    fn current_span(&self) -> Span {
        self.peek().map_or(self.end_span(), |token| token.span)
    }

    /// The span of the token `ahead` places on.
    fn span_at(&self, ahead: usize) -> Span {
        self.tokens.get(self.at + ahead).map_or(self.end_span(), |token| token.span)
    }

    /// Whether the current token starts a new line.
    fn starts_line(&self) -> bool {
        self.starts_line_at(self.at)
    }

    fn starts_line_at(&self, index: usize) -> bool {
        let Some(token) = self.tokens.get(index) else {
            return true;
        };
        let Some(previous) = index.checked_sub(1).and_then(|before| self.tokens.get(before)) else {
            return true;
        };
        self.text.get(previous.span.end as usize..token.span.start as usize).is_some_and(|gap| gap.contains('\n'))
    }

    fn found(&self) -> String {
        match self.peek() {
            Some(token) => format!("`{}`", self.text_of(token)),
            None => "the end of the file".to_string(),
        }
    }

    // ----- Errors -----

    fn error_expected_after(&mut self, expected: &str) -> Reported {
        let span = self.previous_span();
        let found = self.found();
        self.errors.push(Diagnostic::refused(
            span,
            format!("expected `{expected}` after this, but found {found}"),
            STATEMENT_WHY,
        ));
        Reported
    }

    fn error_expected(&mut self, expected: &str) -> Reported {
        let span = self.current_span();
        let found = self.found();
        self.errors.push(Diagnostic::refused(span, format!("expected {expected}, but found {found}"), STATEMENT_WHY));
        Reported
    }

    fn expect(&mut self, kind: TokenKind, shown: &str) -> Parse<Span> {
        if self.is(kind) {
            Ok(self.advance().map_or(self.end_span(), |token| token.span))
        } else {
            Err(self.error_expected_after(shown))
        }
    }

    fn expect_word(&mut self, word: &str) -> Parse<Span> {
        if self.is_word(word) {
            Ok(self.advance().map_or(self.end_span(), |token| token.span))
        } else {
            Err(self.error_expected(&format!("`{word}`")))
        }
    }

    fn expect_block_colon(&mut self) -> Parse<Span> {
        if self.is(TokenKind::Colon) {
            self.roles[self.at] = Role::BlockColon;
        }
        self.expect(TokenKind::Colon, ":")
    }

    /// Skips to the end of the statement after an error: past the next `;` at this depth, or to the start of the
    /// next line, whichever comes first.
    fn recover(&mut self) {
        let start = self.at;
        self.recover_from(start);
    }

    /// Skips to the end of a broken statement that began at token `begin`: past the next `;` at its depth, or to
    /// the start of the next line, whichever comes first.
    fn recover_from(&mut self, begin: usize) {
        let mut depth = 0usize;
        while let Some(token) = self.peek() {
            if self.at > begin && depth == 0 && self.starts_line() {
                return;
            }
            match token.kind {
                TokenKind::LeftParen | TokenKind::LeftSquare | TokenKind::LeftAngle => depth += 1,
                TokenKind::RightParen | TokenKind::RightSquare | TokenKind::RightAngle => {
                    depth = depth.saturating_sub(1);
                }
                TokenKind::Semicolon if depth == 0 => {
                    self.advance();
                    return;
                }
                _ => {}
            }
            self.advance();
        }
    }

    // ----- Names and paths -----

    fn name(&mut self) -> Parse<Name> {
        let after_dot = self
            .at
            .checked_sub(1)
            .and_then(|index| self.tokens.get(index))
            .is_some_and(|token| token.kind == TokenKind::Dot);
        match self.peek() {
            Some(token) if token.kind == TokenKind::Word || (after_dot && token.kind == TokenKind::Integer) => {
                self.advance();
                Ok(Name { text: self.text_of(token).to_string(), span: token.span })
            }
            _ => Err(self.error_expected("a name")),
        }
    }

    /// `$name`, as one name whose span covers the `$`.
    fn variable_name(&mut self) -> Parse<Name> {
        let start = self.current_span();
        if !self.is(TokenKind::Dollar) {
            return Err(self.error_expected("a `$` name"));
        }
        self.advance();
        let name = self.name()?;
        Ok(Name { text: format!("${}", name.text), span: start.to(name.span) })
    }

    /// A path: segments joined by `::` and `.`, a limb's `@` on any segment, and `%` or `%%` marks on methods.
    fn path(&mut self) -> Parse<Path> {
        let start = self.current_span();
        let mut segments = Vec::new();
        let mut joint = Joint::Start;
        loop {
            let mut limb = false;
            if self.is(TokenKind::At) {
                limb = true;
                self.advance();
            }
            let mut mark = Mark::None;
            match self.kind() {
                Some(TokenKind::Percent) => {
                    mark = Mark::Percent;
                    self.advance();
                }
                Some(TokenKind::DoublePercent) => {
                    mark = Mark::DoublePercent;
                    self.advance();
                }
                Some(TokenKind::Star) if joint == Joint::Narrowing => {
                    mark = Mark::Star;
                    self.advance();
                }
                _ => {}
            }
            let name = self.name()?;
            segments.push(Segment { joint, mark, limb, name });
            match (self.kind(), self.kind_at(1)) {
                (
                    Some(TokenKind::DoubleColon),
                    Some(TokenKind::Word | TokenKind::At | TokenKind::Percent | TokenKind::DoublePercent),
                ) => {
                    joint = Joint::Path;
                    self.advance();
                }
                (Some(TokenKind::Dot), Some(TokenKind::Word | TokenKind::At | TokenKind::Integer)) => {
                    joint = Joint::Narrowing;
                    self.advance();
                }
                (Some(TokenKind::Dot), Some(TokenKind::Percent | TokenKind::DoublePercent | TokenKind::Star))
                    if self.path_continues_after_mark() =>
                {
                    joint = Joint::Narrowing;
                    self.advance();
                }
                _ => break,
            }
        }
        Ok(Path { segments, span: start.to(self.previous_span()) })
    }

    /// In a declaration's name, `reading.%%warm_up`, and in a handler, `word_failed.*bail`, a mark follows a `.`.
    fn path_continues_after_mark(&self) -> bool {
        self.kind_at(2) == Some(TokenKind::Word)
    }

    // ----- Files and items -----

    /// Parses a whole file.
    pub fn file(&mut self) -> File {
        let mut items = Vec::new();
        while self.peek().is_some() {
            let before = self.at;
            if let Ok(item) = self.item() {
                items.push(item)
            } else {
                if self.at == before {
                    self.advance();
                }
                self.recover_item();
            }
        }
        File { items }
    }

    /// After a broken item, skips to the next line that starts at the left edge.
    fn recover_item(&mut self) {
        while self.peek().is_some() {
            if self.starts_line() && self.column_of(self.at) == 0 && !self.is_word("end") {
                return;
            }
            self.advance();
        }
    }

    /// The byte column a token starts at, from 0.
    fn column_of(&self, index: usize) -> usize {
        let Some(token) = self.tokens.get(index) else {
            return 0;
        };
        let start = token.span.start as usize;
        let line_start = self.text.get(..start).and_then(|before| before.rfind('\n')).map_or(0, |at| at + 1);
        start - line_start
    }

    fn item(&mut self) -> Parse<Item> {
        let start = self.current_span();
        let mut public = None;
        if self.is_word("pub") {
            public = Some(self.current_span());
            self.advance();
        }
        if self.is(TokenKind::At) && self.word_at(1) == Some("import") {
            // trace: D56
            let span = self.current_span().to(self.span_at(1));
            self.errors.push(Diagnostic::refused_fix(span, "import", "importing is a plain keyword (D56, D92)"));
            self.advance();
        }
        let Some(word) = self.word() else {
            let statement = self.statement()?;
            return Ok(Item { span: statement.span, kind: ItemKind::Stray(statement), public });
        };
        let kind = match word {
            "import" => self.import()?,
            "use" => self.use_item()?,
            "fn" | "op" => ItemKind::Function(self.function()?),
            "const" => ItemKind::Const(self.let_statement()?),
            "struct" => ItemKind::Struct(self.struct_item()?),
            "union" => ItemKind::Union(self.union_item()?),
            "fam" => ItemKind::Family(self.family()?),
            "type" if self.kind_at(1) == Some(TokenKind::Dot) => self.type_name()?,
            "type" if self.word_at(1).is_some() => {
                // trace: D134
                self.errors.push(Diagnostic::refused(
                    self.current_span(),
                    "there's no bare `type`, so pick `type.alias` or `type.new`",
                    "`type.alias` is another name and `type.new` a different type, and no default is obvious (D134)",
                ));
                self.recover();
                return Err(Reported);
            }
            "interface" | "type_set" => self.requirements(word)?,
            _ if self.is_satisfies() => self.satisfies()?,
            _ => {
                let statement = self.statement()?;
                return Ok(Item { span: statement.span, kind: ItemKind::Stray(statement), public });
            }
        };
        Ok(Item { kind, span: start.to(self.previous_span()), public })
    }

    fn import(&mut self) -> Parse<ItemKind> {
        self.advance();
        let mut whole_package = None;
        if self.is_word("pkg") && self.kind_at(1) == Some(TokenKind::Word) {
            whole_package = Some(self.current_span());
            self.advance();
        }
        let path = self.path()?;
        self.expect(TokenKind::Semicolon, ";")?;
        Ok(ItemKind::Import(Import { whole_package, path }))
    }

    fn use_item(&mut self) -> Parse<ItemKind> {
        self.advance();
        let path = self.path()?;
        self.expect_word("as")?;
        let start = self.current_span();
        let mut limb = false;
        if self.is(TokenKind::At) {
            limb = true;
            self.advance();
        }
        let mut mark = Mark::None;
        match self.kind() {
            Some(TokenKind::Percent) => {
                mark = Mark::Percent;
                self.advance();
            }
            Some(TokenKind::DoublePercent) => {
                mark = Mark::DoublePercent;
                self.advance();
            }
            _ => {}
        }
        let (text, quoted) = match self.peek() {
            Some(token) if token.kind == TokenKind::Text => {
                self.advance();
                let raw = self.text_of(token);
                (raw.trim_start_matches('"').trim_end_matches('"').to_string(), true)
            }
            Some(token) if token.kind == TokenKind::Word => {
                self.advance();
                (self.text_of(token).to_string(), false)
            }
            _ => return Err(self.error_expected("a short name")),
        };
        let span = start.to(self.previous_span());
        self.expect(TokenKind::Semicolon, ";")?;
        Ok(ItemKind::Use(Use { path, short: ShortName { mark, limb, text, quoted, span } }))
    }

    fn is_satisfies(&self) -> bool {
        let mut ahead = 0;
        while let Some(kind) = self.kind_at(ahead) {
            match kind {
                TokenKind::Satisfies => return true,
                TokenKind::Word if self.word_at(ahead) == Some("satisfies") => return true,
                TokenKind::Semicolon | TokenKind::Colon => return false,
                _ => ahead += 1,
            }
            if self.starts_line_at(self.at + ahead) {
                return false;
            }
        }
        false
    }

    fn satisfies(&mut self) -> Parse<ItemKind> {
        let subject = self.path()?;
        if self.is(TokenKind::Satisfies) || self.is_word("satisfies") {
            self.advance();
        } else {
            return Err(self.error_expected("`satisfies`"));
        }
        let interface = self.path()?;
        self.expect(TokenKind::Semicolon, ";")?;
        Ok(ItemKind::Satisfies { subject, interface })
    }

    fn type_name(&mut self) -> Parse<ItemKind> {
        let keyword = self.path()?;
        if self.is(TokenKind::Colon) {
            // A family of them is `fam.type.new`; a block here isn't designed.
            return Err(self.error_expected("`=`"));
        }
        let name = self.path()?;
        self.setter()?;
        let target = self.parse_type()?;
        self.expect(TokenKind::Semicolon, ";")?;
        Ok(ItemKind::TypeName { keyword, name, target })
    }

    /// An `interface` or a `type_set`, with its lines (D235, D238).
    fn requirements(&mut self, word: &str) -> Parse<ItemKind> {
        let start = self.current_span();
        let keyword = self.path()?;
        let name = self.path()?;
        let generics = self.generics()?;
        self.expect_block_colon()?;
        let mut entries = Vec::new();
        while self.peek().is_some() && !self.is_word("end") {
            let entry_start = self.current_span();
            let result = (|| {
                if self.is_word("includes") {
                    self.advance();
                    let path = self.path()?;
                    self.expect(TokenKind::Semicolon, ";")?;
                    return Ok(RequirementEntry::Includes(path));
                }
                if self.is_word("op") {
                    self.advance();
                    let token = self.advance().ok_or(Reported)?;
                    let operator = Name { text: self.text_of(token).to_string(), span: token.span };
                    let parameters = self.parameters()?;
                    if self.is(TokenKind::Arrow) {
                        self.advance();
                        self.parse_type()?;
                    }
                    self.expect(TokenKind::Semicolon, ";")?;
                    let values = parameters.list.len();
                    return Ok(RequirementEntry::Op { operator, values, span: entry_start.to(self.previous_span()) });
                }
                if self.is_word("law") {
                    while self.peek().is_some() && !self.is(TokenKind::Semicolon) {
                        self.advance();
                    }
                    self.expect(TokenKind::Semicolon, ";")?;
                    return Ok(RequirementEntry::Law(entry_start.to(self.previous_span())));
                }
                let written = self.parse_type()?;
                self.expect(TokenKind::Semicolon, ";")?;
                Ok(RequirementEntry::Type(written))
            })();
            match result {
                Ok(entry) => entries.push(entry),
                Err(Reported) => self.recover(),
            }
        }
        let end = self.closer()?;
        let requirements = Requirements { keyword, name, generics, entries, end, span: start.to(self.previous_span()) };
        Ok(if word == "interface" { ItemKind::Interface(requirements) } else { ItemKind::TypeSet(requirements) })
    }

    /// `end` and the rest of its line.
    fn closer(&mut self) -> Parse<Closer> {
        let start = self.current_span();
        if !self.is_word("end") {
            return Err(self.error_expected("`end`"));
        }
        self.advance();
        let mut words = Vec::new();
        let mut keyword = None;
        let mut name = None;
        // `end` alone on its line is never a whole closer, so the next line finishes it, and the spacing check
        // refuses the break (D219).
        let broken = self.starts_line() && self.peek().is_some();
        if (!self.starts_line() || broken) && self.peek().is_some() {
            let first = self.at;
            if let Ok(path) = self.path() {
                keyword = Some(path);
            }
            if !self.starts_line()
                && self.peek().is_some()
                && matches!(self.kind(), Some(TokenKind::Word | TokenKind::At))
                && let Ok(path) = self.path()
            {
                name = Some(path);
            }
            for index in first..self.at {
                if let Some(token) = self.tokens.get(index)
                    && token.kind == TokenKind::Word
                {
                    words.push(Name { text: self.text_of(*token).to_string(), span: token.span });
                }
            }
        }
        // Anything else on the line belongs to the closer, and is refused by the check on closers.
        while self.peek().is_some() && !self.starts_line() {
            self.advance();
        }
        if let Some(role) = self.at.checked_sub(1).and_then(|last| self.roles.get_mut(last)) {
            *role = Role::LineEnd;
        }
        Ok(Closer { span: start.to(self.previous_span()), words, name, keyword })
    }

    fn generics(&mut self) -> Parse<Vec<Generic>> {
        let mut generics = Vec::new();
        if !self.is(TokenKind::LeftAngle) {
            return Ok(generics);
        }
        self.advance();
        loop {
            let start = self.current_span();
            let (name, kind) = match self.kind() {
                Some(TokenKind::Question) => {
                    self.roles[self.at] = Role::Prefix;
                    self.advance();
                    let name = self.name()?;
                    (Name { text: name.text, span: start.to(name.span) }, GenericKind::Type)
                }
                Some(TokenKind::Backtick) => (self.source_name()?, GenericKind::Source),
                _ => return Err(self.error_expected("a type parameter")),
            };
            let mut requirement = None;
            if self.is(TokenKind::Colon) {
                self.roles[self.at] = Role::LabelColon;
                self.advance();
                requirement = Some(self.requirement()?);
            }
            generics.push(Generic { name, kind, requirement, span: start.to(self.previous_span()) });
            if self.is(TokenKind::Comma) {
                self.advance();
                continue;
            }
            break;
        }
        self.expect(TokenKind::RightAngle, ">")?;
        Ok(generics)
    }

    /// `` `text` ``, as one name whose span covers the backticks.
    fn source_name(&mut self) -> Parse<Name> {
        let start = self.current_span();
        self.expect(TokenKind::Backtick, "`")?;
        let name = self.name()?;
        if !self.is(TokenKind::Backtick) {
            // trace: D171
            self.errors.push(Diagnostic::refused(
                start.to(name.span),
                "a source is a name in backticks, as in `` `text` ``",
                "a struct names the sources of its borrows in backticks (D171)",
            ));
            return Err(Reported);
        }
        let end = self.expect(TokenKind::Backtick, "`")?;
        Ok(Name { text: name.text, span: start.to(end) })
    }

    fn requirement(&mut self) -> Parse<Requirement> {
        let first = if self.is_word("lasts_at_least") {
            self.advance();
            Requirement::LastsAtLeast(self.source_name()?)
        } else {
            Requirement::Path(self.path()?)
        };
        if self.is_word("and") {
            self.advance();
            let rest = self.requirement()?;
            return Ok(Requirement::And(Box::new(first), Box::new(rest)));
        }
        Ok(first)
    }

    fn function(&mut self) -> Parse<Function> {
        let start = self.current_span();
        let keyword = self.path()?;
        let is_operator = keyword.is_single("op");
        let mut name = None;
        let mut operator = None;
        if is_operator {
            let token = self.peek();
            if let Some(token) = token {
                self.advance();
                let mut text = self.text_of(token).to_string();
                let mut span = token.span;
                // `op div.floor`, with its narrowing.
                while self.is(TokenKind::Dot) && self.kind_at(1) == Some(TokenKind::Word) {
                    self.advance();
                    if let Some(word) = self.advance() {
                        text = format!("{text}.{}", self.text_of(word));
                        span = span.to(word.span);
                    }
                }
                operator = Some(Name { text, span });
            }
        } else if self.is(TokenKind::Word) || self.is(TokenKind::At) {
            name = Some(self.path()?);
        }
        let generics = self.generics()?;
        let parameters = self.parameters()?;
        let returns_word;
        let mut returns = None;
        if self.is(TokenKind::Arrow) {
            returns_word = self.current_span();
            self.advance();
            returns = Some(self.parse_type()?);
        } else if self.is_word("returns") {
            // trace: D287
            returns_word = self.current_span();
            self.errors.push(Diagnostic::refused_fix(
                returns_word,
                "->",
                "`->` is an anchor the eye skips straight to, marking where the result starts, so `returns` is never written (D287)",
            ));
            self.advance();
            returns = Some(self.parse_type()?);
        } else {
            returns_word = self.current_span();
        }
        let mut fails = None;
        if self.is_word("or") {
            // trace: D52
            self.errors.push(Diagnostic::refused(
                self.current_span(),
                "`or fail[…]` is the older spelling",
                "a function lists its failures in its header, after its return type, with `fails` (D52, D209)",
            ));
            while self.peek().is_some() && !self.is(TokenKind::Colon) {
                self.advance();
            }
        }
        if self.is_word("fails") {
            fails = Some(self.fail_list()?);
        }
        let header_span = start.to(self.previous_span());
        self.expect_block_colon()?;
        let body = self.block(&["end"])?;
        let end = self.closer()?;
        Ok(Function {
            keyword,
            name,
            operator,
            generics,
            parameters,
            returns,
            returns_word,
            fails,
            body,
            end,
            header_span,
        })
    }

    fn parameters(&mut self) -> Parse<Parameters> {
        let start = self.current_span();
        self.expect(TokenKind::LeftSquare, "[")?;
        let mut parameters = Parameters { none: None, list: Vec::new(), span: start };
        if self.is_word("none") && self.kind_at(1) == Some(TokenKind::RightSquare) {
            parameters.none = Some(self.current_span());
            self.advance();
        }
        while !self.is(TokenKind::RightSquare) && self.peek().is_some() && parameters.none.is_none() {
            let parameter_start = self.current_span();
            let name = self.variable_name()?;
            if self.is(TokenKind::ColonEqual) {
                self.errors.push(Diagnostic::refused(
                    self.current_span(),
                    "`:=` is gone, and a parameter always writes its type",
                    "there's no type inference in the base language, so everything writes its type (D277)",
                ));
                return Err(Reported);
            }
            if self.is(TokenKind::Colon) {
                self.roles[self.at] = Role::LabelColon;
            }
            self.expect(TokenKind::Colon, ":")?;
            let ownership_start = self.current_span();
            let (ownership, ownership_span) = match self.word() {
                Some("Given") => {
                    self.advance();
                    (ParameterOwnership::Given, Some(ownership_start))
                }
                // The call's words on a parameter: the parameter says `Given` (D2).
                Some(word @ ("Give" | "Copy")) => {
                    let why = if word == "Copy" {
                        "`Copy` hands over a separate copy, to a `Given` parameter (D2)"
                    } else {
                        "`Give` goes at the call, and `Given` on the parameter (D2)"
                    };
                    self.errors.push(Diagnostic::refused_fix(ownership_start, "Given", why));
                    self.advance();
                    (ParameterOwnership::Given, Some(ownership_start))
                }
                Some("Lent") => {
                    let path = self.path()?;
                    if path.text() == "Lent.mut" {
                        (ParameterOwnership::LentMut, Some(path.span))
                    } else {
                        (ParameterOwnership::LentWritten, Some(path.span))
                    }
                }
                _ => (ParameterOwnership::Lent, None),
            };
            let written_type = self.parse_type()?;
            parameters.list.push(Parameter {
                name,
                ownership,
                ownership_span,
                span: parameter_start.to(self.previous_span()),
                written_type,
            });
            if self.is(TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        let end = self.expect(TokenKind::RightSquare, "]")?;
        parameters.span = start.to(end);
        Ok(parameters)
    }

    fn fail_list(&mut self) -> Parse<FailList> {
        let start = self.current_span();
        self.advance();
        self.expect(TokenKind::LeftSquare, "[")?;
        let mut entries = Vec::new();
        while !self.is(TokenKind::RightSquare) && self.peek().is_some() {
            let entry_start = self.current_span();
            let name = self.path()?;
            let mut fields = Vec::new();
            if self.is(TokenKind::LeftSquare) {
                fields = self.field_list()?;
            }
            entries.push(FailEntry { name, fields, span: entry_start.to(self.previous_span()) });
            if self.is(TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        let end = self.expect(TokenKind::RightSquare, "]")?;
        Ok(FailList { span: start.to(end), entries })
    }

    /// `[name: type, …]`, the fields of a case or failure.
    fn field_list(&mut self) -> Parse<Vec<Field>> {
        self.expect(TokenKind::LeftSquare, "[")?;
        let mut fields = Vec::new();
        while !self.is(TokenKind::RightSquare) && self.peek().is_some() {
            let start = self.current_span();
            let name = self.name()?;
            if self.is(TokenKind::Colon) {
                self.roles[self.at] = Role::LabelColon;
            }
            self.expect(TokenKind::Colon, ":")?;
            let written_type = self.parse_type()?;
            fields.push(Field { name, written_type, span: start.to(self.previous_span()) });
            if self.is(TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RightSquare, "]")?;
        Ok(fields)
    }

    fn struct_item(&mut self) -> Parse<Struct> {
        let keyword = self.path()?;
        let name = self.path()?;
        let generics = self.generics()?;
        self.expect_block_colon()?;
        let mut fields = Vec::new();
        while self.peek().is_some() && !self.is_word("end") {
            let start = self.current_span();
            let result = (|| {
                let name = self.name()?;
                if self.is(TokenKind::ColonEqual) {
                    self.errors.push(Diagnostic::refused(
                        self.current_span(),
                        "`:=` is gone, and a struct field always writes its type",
                        "there's no type inference in the base language, so everything writes its type (D277)",
                    ));
                    return Err(Reported);
                }
                if self.is(TokenKind::Colon) {
                    self.roles[self.at] = Role::LabelColon;
                }
                self.expect(TokenKind::Colon, ":")?;
                let written_type = self.parse_type()?;
                if self.is(TokenKind::Equal) {
                    // `decl` gives no value (D203).
                    self.advance();
                    let value = self.expression()?;
                    self.errors.push(Diagnostic::refused(
                        value.span,
                        "`decl` gives no value, and `let` does",
                        "`decl` declares a variable with no value yet, and `assign` gives it one (D203)",
                    ));
                    self.expect(TokenKind::Semicolon, ";")?;
                    return Err(Reported);
                }
                self.expect(TokenKind::Semicolon, ";")?;
                Ok(Field { name, written_type, span: start.to(self.previous_span()) })
            })();
            match result {
                Ok(field) => fields.push(field),
                Err(Reported) => self.recover(),
            }
        }
        let end = self.closer()?;
        Ok(Struct { keyword, name, generics, fields, end })
    }

    fn union_item(&mut self) -> Parse<Union> {
        let keyword = self.path()?;
        let name = self.path()?;
        let generics = self.generics()?;
        self.expect_block_colon()?;
        let mut cases = Vec::new();
        while self.peek().is_some() && !self.is_word("end") {
            let start = self.current_span();
            let result = (|| {
                let name = self.name()?;
                let mut fields = Vec::new();
                if self.is(TokenKind::LeftSquare) {
                    fields = self.field_list()?;
                }
                self.expect(TokenKind::Semicolon, ";")?;
                Ok(Case { name, fields, span: start.to(self.previous_span()) })
            })();
            match result {
                Ok(case) => cases.push(case),
                Err(Reported) => self.recover(),
            }
        }
        let end = self.closer()?;
        Ok(Union { keyword, name, generics, cases, end })
    }

    fn family(&mut self) -> Parse<Family> {
        let start = self.current_span();
        let keyword = self.path()?;
        let name = self.path()?;
        self.expect_block_colon()?;
        let mut entries = Vec::new();
        // In a family of types, what follows `=` is always a type, even a plain name like `point` (D134), and in a
        // family of units, a unit's definition (D109).
        let types = keyword.text().starts_with("fam.type");
        let units = keyword.text() == "fam.unit";
        while self.peek().is_some() && !self.is_word("end") {
            if units && self.unit_entry_ahead() {
                match self.unit_entry() {
                    Ok(entry) => entries.push(entry),
                    Err(Reported) => self.recover(),
                }
                continue;
            }
            match self.family_entry(types) {
                Ok(entry) => entries.push(entry),
                Err(Reported) => self.recover(),
            }
        }
        let end = self.closer()?;
        Ok(Family { keyword, name, entries, span: start.to(self.previous_span()), end })
    }

    /// Whether a `fam.unit` line ahead is `name = …;`.
    fn unit_entry_ahead(&self) -> bool {
        let mut at = self.at;
        while let Some(token) = self.tokens.get(at) {
            match token.kind {
                TokenKind::Word | TokenKind::Dot | TokenKind::DoubleColon => at += 1,
                _ => return self.text_of(*token) == "=",
            }
        }
        false
    }

    /// `money.dollars = 100 money.cents;`, `motion.speed = length.meters per time.seconds;`, and the forms kept for
    /// their errors (D109, D242, D249).
    fn unit_entry(&mut self) -> Parse<FamilyEntry> {
        let start = self.current_span();
        let name = self.path()?;
        self.take_setter();
        let value_start = self.current_span();
        let definition = if self.is_word("decibels") {
            self.skip_to_semicolon();
            UnitDefinition::Decibels(value_start.to(self.previous_span()))
        } else if matches!(self.kind(), Some(TokenKind::Integer | TokenKind::Float)) {
            let amount = self.primary_number()?;
            let unit = self.path()?;
            if self.is(TokenKind::Semicolon) {
                UnitDefinition::Multiple { amount, unit }
            } else {
                self.skip_to_semicolon();
                UnitDefinition::Other(value_start.to(self.previous_span()))
            }
        } else {
            let first = self.path()?;
            let mut factors = vec![(first.clone(), 1)];
            let mut offset = None;
            loop {
                if self.is_word("per") {
                    self.advance();
                    factors.push((self.path()?, -1));
                } else if self.is_word("x") {
                    self.advance();
                    factors.push((self.path()?, 1));
                } else if self.is_word("offset") && factors.len() == 1 {
                    self.advance();
                    offset = Some(self.expression()?);
                    break;
                } else {
                    break;
                }
            }
            if let Some(amount) = offset {
                UnitDefinition::Offset { unit: first, amount, span: value_start.to(self.previous_span()) }
            } else if self.is(TokenKind::Semicolon) {
                UnitDefinition::Made { factors }
            } else {
                self.skip_to_semicolon();
                UnitDefinition::Other(value_start.to(self.previous_span()))
            }
        };
        self.expect(TokenKind::Semicolon, ";")?;
        Ok(FamilyEntry { name, kind: FamilyEntryKind::UnitSetTo(definition), span: start.to(self.previous_span()) })
    }

    /// Moves to the `;` that ends this line, without passing it.
    fn skip_to_semicolon(&mut self) {
        while self.peek().is_some() && !self.is(TokenKind::Semicolon) && !self.is_word("end") {
            self.advance();
        }
    }

    /// A number literal, as a unit's amount.
    fn primary_number(&mut self) -> Parse<Expression> {
        let span = self.current_span();
        let text = self.text_of(self.peek().ok_or(Reported)?).to_string();
        let kind =
            if self.is(TokenKind::Integer) { ExpressionKind::Integer(text) } else { ExpressionKind::Float(text) };
        self.advance();
        Ok(Expression { kind, span })
    }

    fn family_entry(&mut self, types: bool) -> Parse<FamilyEntry> {
        let start = self.current_span();
        if self.is_word("type") && self.kind_at(1) == Some(TokenKind::Dot) {
            self.errors.push(Diagnostic::refused(
                start,
                "a family of types states its kind once",
                "a family doesn't mix `type.alias` and `type.new` (D134)",
            ));
            return Err(Reported);
        }
        let name = self.path()?;
        let kind = if self.is(TokenKind::Colon) && self.word_at(1) == Some("base") {
            self.roles[self.at] = Role::LabelColon;
            self.advance();
            self.advance();
            let mut with = None;
            if self.is_word("with") {
                self.advance();
                with = Some(self.path()?);
            }
            self.expect(TokenKind::Semicolon, ";")?;
            FamilyEntryKind::Base { with }
        } else if self.is(TokenKind::Colon) {
            // A handler: its arms follow, until its own closer.
            self.roles[self.at] = Role::BlockColon;
            self.advance();
            let arms = self.arms()?;
            let end = self.closer()?;
            FamilyEntryKind::Handler { arms, end }
        } else if self.at_setter() {
            self.take_setter();
            let value = if types || self.looks_like_type() {
                let written_type = self.parse_type()?;
                self.expect(TokenKind::Semicolon, ";")?;
                return Ok(FamilyEntry {
                    name,
                    kind: FamilyEntryKind::TypeSetTo(written_type),
                    span: start.to(self.previous_span()),
                });
            } else {
                self.expression()?
            };
            self.expect(TokenKind::Semicolon, ";")?;
            FamilyEntryKind::SetTo(value)
        } else if self.is(TokenKind::LeftSquare) {
            let fields = self.field_list()?;
            self.expect(TokenKind::Semicolon, ";")?;
            FamilyEntryKind::Failure { fields }
        } else {
            self.expect(TokenKind::Semicolon, ";")?;
            FamilyEntryKind::Bare
        };
        Ok(FamilyEntry { name, kind, span: start.to(self.previous_span()) })
    }

    /// Whether what follows `=` in a family is a type, like `i64`, rather than a value.
    fn looks_like_type(&self) -> bool {
        matches!(self.word(), Some(word) if is_type_word(word))
            && matches!(self.kind_at(1), Some(TokenKind::Semicolon | TokenKind::LeftSquare | TokenKind::Dot))
            && !matches!(self.kind_at(1), Some(TokenKind::Dot) if self.word_at(2).is_some_and(|w| !matches!(w, "stable" | "fast" | "ieee")))
    }

    // ----- Statements -----

    /// Statements until one of `stops` starts a line, such as `end`, `else` or `case`.
    fn block(&mut self, stops: &[&str]) -> Parse<Vec<Statement>> {
        let mut statements = Vec::new();
        while let Some(word) = self.peek().map(|token| self.text_of(token)) {
            if stops.contains(&word) && self.kind() == Some(TokenKind::Word) {
                // `end` that isn't a closer here, like `end` alone mid-statement, still ends the block.
                break;
            }
            let before = self.at;
            if let Ok(statement) = self.statement() {
                statements.push(statement)
            } else {
                if self.at == before {
                    self.advance();
                }
                self.recover_from(before);
                statements.push(Statement { kind: StatementKind::Broken, span: self.previous_span() });
            }
        }
        Ok(statements)
    }

    pub fn statement(&mut self) -> Parse<Statement> {
        let start = self.current_span();
        if self.is_word("type") && self.kind_at(1) == Some(TokenKind::Dot) {
            self.errors.push(Diagnostic::not_designed(
                start,
                "`type` inside a function",
                "not decided yet, and the standard doesn't say",
            ));
            self.recover();
            return Err(Reported);
        }
        if let Some(old @ ("precond" | "postcond")) = self.word()
            && self.kind_at(1) == Some(TokenKind::FatArrow)
        {
            // trace: D84
            let fixed = if old == "precond" { "requires" } else { "ensures" };
            let span = start.to(self.span_at(1));
            self.errors.push(Diagnostic::refused_fix(
                span,
                fixed,
                "contracts are `requires` and `ensures`, with no `=>` (D84)",
            ));
            self.recover();
            return Err(Reported);
        }
        let kind = match self.word() {
            Some("let" | "const") => StatementKind::Let(self.let_statement()?),
            Some("decl") => {
                self.advance();
                let mutable = if self.is_word("mut") {
                    self.advance();
                    true
                } else {
                    false
                };
                let name = self.variable_name()?;
                if self.is(TokenKind::Colon) {
                    self.roles[self.at] = Role::LabelColon;
                }
                self.expect(TokenKind::Colon, ":")?;
                let written_type = self.parse_type()?;
                self.expect(TokenKind::Semicolon, ";")?;
                StatementKind::Decl { mutable, name, written_type }
            }
            Some("assign" | "change") => {
                let assign = self.is_word("assign");
                self.advance();
                let target = self.postfix_expression()?;
                self.setter()?;
                let value = self.expression()?;
                self.statement_end(&value)?;
                if assign { StatementKind::Assign { target, value } } else { StatementKind::Change { target, value } }
            }
            Some("return") => {
                self.advance();
                if self.is(TokenKind::Semicolon) {
                    self.advance();
                    StatementKind::Return(None)
                } else {
                    let value = self.expression()?;
                    self.statement_end(&value)?;
                    StatementKind::Return(Some(value))
                }
            }
            Some("if") => StatementKind::If(self.if_statement()?),
            Some("match") => StatementKind::Match(self.match_statement()?),
            Some("for") => StatementKind::For(self.for_statement()?),
            Some("while") => {
                self.advance();
                let label = self.loop_label();
                let condition = self.condition()?;
                self.expect_block_colon()?;
                let body = self.block(&["end"])?;
                let end = self.closer()?;
                StatementKind::While { label, condition, body, end }
            }
            Some("forever") => {
                self.advance();
                let label = if self.is(TokenKind::Word) { Some(self.name()?) } else { None };
                self.expect_block_colon()?;
                let body = self.block(&["end"])?;
                let end = self.closer()?;
                StatementKind::Forever { label, body, end }
            }
            Some("break") => {
                self.advance();
                let label = if self.is(TokenKind::Word) {
                    let path = self.path()?;
                    Some(Name { text: path.text(), span: path.span })
                } else {
                    None
                };
                self.expect(TokenKind::Semicolon, ";")?;
                StatementKind::Break(label)
            }
            Some("fail") if self.kind_at(1) != Some(TokenKind::LeftParen) => {
                self.advance();
                let value = self.expression()?;
                self.expect(TokenKind::Semicolon, ";")?;
                StatementKind::Fail(value)
            }
            Some("instead") if self.kind_at(1) != Some(TokenKind::LeftParen) => {
                self.advance();
                let value = self.expression()?;
                self.expect(TokenKind::Semicolon, ";")?;
                StatementKind::Instead(value)
            }
            Some("finish") if self.kind_at(1) != Some(TokenKind::LeftParen) => {
                self.advance();
                let value = self.expression()?;
                self.expect(TokenKind::Semicolon, ";")?;
                StatementKind::Finish(value)
            }
            Some("labels" | "defaults")
                if self.kind_at(1) == Some(TokenKind::LeftSquare) || self.kind_at(1) == Some(TokenKind::Dot) =>
            {
                let defaults = self.is_word("defaults");
                let _keyword = self.path()?;
                let labels = self.label_list()?;
                self.expect(TokenKind::Semicolon, ";")?;
                if defaults { StatementKind::Defaults(labels) } else { StatementKind::Labels(labels) }
            }
            Some("requires" | "ensures") => {
                let ensures = self.is_word("ensures");
                let keyword = self.path()?;
                let condition = self.expression()?;
                self.expect(TokenKind::Semicolon, ";")?;
                if ensures {
                    StatementKind::Ensures { keyword, condition }
                } else {
                    StatementKind::Requires { keyword, condition }
                }
            }
            Some("shield") if self.kind_at(1) == Some(TokenKind::Word) => StatementKind::Shield(self.shield()?),
            Some("loop") if self.kind_at(1) == Some(TokenKind::Colon) => {
                // There's no `loop`: an endless loop is `forever` (D128). Read on as one.
                let span = self.current_span();
                self.errors.push(Diagnostic::refused_fix(
                    span,
                    "forever",
                    "`forever` has no end unless you `break`, so one word finds every endless loop (D128)",
                ));
                self.advance();
                self.advance();
                let _ = self.block(&["end"])?;
                let _ = self.closer()?;
                return Err(Reported);
            }
            Some("else") => {
                let span = self.current_span();
                self.errors.push(Diagnostic::refused(
                    span,
                    "`else` continues an `if`, and there's none open here",
                    "`else if` and `else:` continue an `if`, and one `end if` closes the chain (D126)",
                ));
                // Read on as if the `if` were there, up to its `end if`.
                self.advance();
                if self.is_word("if") {
                    self.advance();
                    let _ = self.condition();
                }
                if self.is(TokenKind::Colon) {
                    self.advance();
                }
                let _ = self.block(&["end"])?;
                let _ = self.closer()?;
                return Err(Reported);
            }
            Some("continue") if self.kind_at(1) == Some(TokenKind::Semicolon) => {
                let span = self.current_span();
                self.errors.push(Diagnostic::refused(
                    span,
                    "there is no `continue`, so wrap the work in an `if`",
                    "each loop says how it ends, and there is no `continue` (D128)",
                ));
                self.advance();
                self.advance();
                return Err(Reported);
            }
            _ => {
                let value = self.expression()?;
                self.statement_end(&value)?;
                StatementKind::Expression(value)
            }
        };
        Ok(Statement { kind, span: start.to(self.previous_span()) })
    }

    /// A `;` after a statement, which a value ending in an `otherwise:` block doesn't need, since its `end` closes
    /// it (D45).
    fn statement_end(&mut self, value: &Expression) -> Parse<()> {
        if ends_in_block(value) && !self.is(TokenKind::Semicolon) {
            return Ok(());
        }
        if !self.is(TokenKind::Semicolon) {
            return Err(self.error_expected_after(";"));
        }
        self.advance();
        Ok(())
    }

    /// A loop's name, as in `while rows …`: a word that doesn't start the condition.
    fn loop_label(&mut self) -> Option<Name> {
        let word = self.word()?;
        if matches!(word, "true" | "false" | "not" | "empty") {
            return None;
        }
        let next = self.kind_at(1);
        let next_word = self.word_at(1);
        let starts_value = matches!(
            next,
            Some(
                TokenKind::LeftParen
                    | TokenKind::Dot
                    | TokenKind::DoubleColon
                    | TokenKind::LeftSquare
                    | TokenKind::Colon
                    | TokenKind::EqualEqual
                    | TokenKind::NotEqual
                    | TokenKind::Plus
                    | TokenKind::Minus
            )
        ) || next_word.is_some_and(|w| OPERATOR_WORDS.contains(&w));
        if starts_value {
            return None;
        }
        self.name().ok()
    }

    /// Whether a setter starts here: `=`, or a `set_to` that's refused.
    fn at_setter(&self) -> bool {
        self.is(TokenKind::Equal) || self.is_word("set_to")
    }

    /// Takes `=`, refusing `set_to` with `fix: =` (D283) and reading on as if it were `=`.
    fn take_setter(&mut self) -> Span {
        let index = self.at;
        let span = self.current_span();
        if self.is_word("set_to") {
            self.errors.push(Diagnostic::refused_fix(
                span,
                "=",
                "`=` assigns and `==` compares, so `set_to` is never written (D283)",
            ));
        }
        self.advance();
        self.mark_binary_word(index, index);
        span
    }

    /// `=`, a refused `set_to` or `:=`, or `@precog::set_to_best` (D283).
    fn setter(&mut self) -> Parse<Path> {
        let index = self.at;
        match self.kind() {
            Some(TokenKind::Equal | TokenKind::Word) if self.at_setter() => {
                let token = self.peek().ok_or(Reported)?;
                let span = self.take_setter();
                Ok(Path {
                    segments: vec![Segment {
                        joint: Joint::Start,
                        mark: Mark::None,
                        limb: false,
                        name: Name { text: self.text_of(token).to_string(), span },
                    }],
                    span,
                })
            }
            Some(TokenKind::ColonEqual) => {
                let token = self.advance().ok_or(Reported)?;
                self.errors.push(Diagnostic::refused_fix(
                    token.span,
                    "=",
                    "`=` gives a value, and there's no type inference in the base language (D277, D283)",
                ));
                Ok(Path {
                    segments: vec![Segment {
                        joint: Joint::Start,
                        mark: Mark::None,
                        limb: false,
                        name: Name { text: self.text_of(token).to_string(), span: token.span },
                    }],
                    span: token.span,
                })
            }
            Some(TokenKind::At) => {
                let path = self.path()?;
                self.mark_binary_word(index, self.at - 1);
                Ok(path)
            }
            _ => Err(self.error_expected("`=`")),
        }
    }

    fn let_statement(&mut self) -> Parse<Let> {
        let keyword_start = self.current_span();
        let constant = self.is_word("const");
        self.advance();
        let mut mutable = false;
        if self.is_word("mut") {
            mutable = true;
            self.advance();
        }
        let keyword = keyword_start.to(self.previous_span());
        let name = self.variable_name()?;
        let mut written_type = None;
        if self.is(TokenKind::Colon) {
            self.roles[self.at] = Role::LabelColon;
            self.advance();
            written_type = Some(self.parse_type()?);
        }
        // trace: D277
        const LET_WHY: &str = "every `let` and `const` writes its type, so a reader never has to work one out, and there's no type inference in the base language (D277)";
        if written_type.is_none() {
            if self.is(TokenKind::ColonEqual) {
                self.errors.push(Diagnostic::refused_fix(self.current_span(), "=", LET_WHY));
                self.advance();
                let value = self.expression()?;
                self.statement_end(&value)?;
                return Err(Reported);
            }
            if self.is(TokenKind::At) {
                self.errors.push(Diagnostic::not_designed(
                    self.current_span(),
                    "Precog's `@precog::set_to_best`, which needs its `rule` line, and `rule` lines aren't designed",
                    "not decided yet (standard 12.9)",
                ));
                self.recover();
                return Err(Reported);
            }
            if self.is(TokenKind::Equal) {
                self.errors.push(Diagnostic::refused(
                    self.current_span(),
                    "this `let` doesn't write its type",
                    LET_WHY,
                ));
                self.recover();
                return Err(Reported);
            }
        }
        let setter = self.setter()?;
        let value = self.expression()?;
        self.statement_end(&value)?;
        Ok(Let { keyword, mutable, constant, name, written_type, setter, value })
    }

    fn if_statement(&mut self) -> Parse<If> {
        let mut branches = Vec::new();
        let mut otherwise = None;
        // `if`, then any `else if`s.
        loop {
            let start = self.current_span();
            self.advance();
            if self.is_word("if") {
                self.advance();
            }
            let condition = self.condition()?;
            self.expect_block_colon()?;
            let body = self.block(&["end", "else"])?;
            branches.push(Branch { condition, body, span: start.to(self.previous_span()) });
            if self.is_word("else") {
                if self.word_at(1) == Some("if") {
                    continue;
                }
                let else_span = self.current_span();
                self.advance();
                if !self.is(TokenKind::Colon) {
                    let found = self.found();
                    let span = self.current_span();
                    self.errors.push(Diagnostic::refused(
                        span,
                        format!("expected `if` or `:` after `else`, but found {found}"),
                        STATEMENT_WHY,
                    ));
                    // Read on as if the `:` were there, so the rest of the chain isn't misread.
                    self.block(&["end", "else"])?;
                    self.statements_after_else(else_span)?;
                    let end = self.closer()?;
                    return Err(self.reported_after(end));
                }
                self.expect_block_colon()?;
                let body = self.block(&["end", "else"])?;
                otherwise = Some((else_span, body));
                self.statements_after_else(else_span)?;
            }
            break;
        }
        let end = self.closer()?;
        Ok(If { branches, otherwise, end })
    }

    /// A broken statement that has already read its whole block.
    fn reported_after(&mut self, _end: Closer) -> Reported {
        Reported
    }

    /// Anything but `end if` after `else:` is refused, and read on so the chain still closes (D126).
    fn statements_after_else(&mut self, else_span: Span) -> Parse<()> {
        while self.is_word("else") {
            let span = self.current_span();
            self.errors.push(
                Diagnostic::refused(
                    span,
                    "nothing follows `else:` but `end if`",
                    "`else if` and `else:` continue an `if`, and one `end if` closes the chain (D126)",
                )
                .with_because(else_span, "the `else:` is here"),
            );
            self.advance();
            if self.is_word("if") {
                self.advance();
                let _ = self.condition();
            }
            if self.is(TokenKind::Colon) {
                self.advance();
            }
            let _ = self.block(&["end", "else"])?;
        }
        Ok(())
    }

    /// A plain condition, or the test, `$s: status.failed`.
    fn condition(&mut self) -> Parse<Condition> {
        let subject = self.expression()?;
        if self.is(TokenKind::Colon) && !self.colon_ends_line() {
            self.roles[self.at] = Role::LabelColon;
            self.advance();
            let written_type = self.parse_type()?;
            let binding = if self.is(TokenKind::Dollar) { Some(self.variable_name()?) } else { None };
            return Ok(Condition::Test { subject, written_type, binding });
        }
        Ok(Condition::Plain(subject))
    }

    /// Whether the `:` here is the last thing on its line, opening a block.
    fn colon_ends_line(&self) -> bool {
        self.starts_line_at(self.at + 1)
    }

    fn match_statement(&mut self) -> Parse<Match> {
        self.advance();
        let subject = self.expression()?;
        self.expect_block_colon()?;
        let arms = self.arms()?;
        let end = self.closer()?;
        Ok(Match { subject, arms, end })
    }

    /// `case` arms until `end`.
    fn arms(&mut self) -> Parse<Vec<Arm>> {
        let mut arms = Vec::new();
        while self.is_word("case") {
            let start = self.current_span();
            self.advance();
            let pattern = self.path()?;
            let binding = if self.is(TokenKind::Dollar) { Some(self.variable_name()?) } else { None };
            self.expect_block_colon()?;
            let body = self.block(&["end", "case"])?;
            arms.push(Arm { pattern, binding, body, span: start.to(self.previous_span()) });
        }
        if !self.is_word("end") {
            return Err(self.error_expected("`case` or `end`"));
        }
        Ok(arms)
    }

    fn for_statement(&mut self) -> Parse<For> {
        self.advance();
        let label = if self.is(TokenKind::Word) { Some(self.name()?) } else { None };
        let name = self.variable_name()?;
        self.expect_word("in")?;
        let ownership = self.ownership_word()?;
        let source = self.expression()?;
        self.expect_block_colon()?;
        let body = self.block(&["end"])?;
        let end = self.closer()?;
        Ok(For { label, name, ownership, source, body, end })
    }

    fn shield(&mut self) -> Parse<Shield> {
        self.advance();
        let name = self.name()?;
        self.expect_block_colon()?;
        let mut reason = None;
        if self.is_word("reason") {
            self.advance();
            reason = Some(self.primary()?);
            if let Some(role) = self.at.checked_sub(1).and_then(|last| self.roles.get_mut(last)) {
                *role = Role::LineEnd;
            }
        }
        let body = self.block(&["end"])?;
        let end = self.closer()?;
        Ok(Shield { name, reason, body, end })
    }

    /// `[label: value, …]` or `[$x = value, …]`, as in `labels` and `defaults`.
    fn label_list(&mut self) -> Parse<Vec<Label>> {
        self.expect(TokenKind::LeftSquare, "[")?;
        let mut labels = Vec::new();
        while !self.is(TokenKind::RightSquare) && self.peek().is_some() {
            let start = self.current_span();
            let mut value = self.expression()?;
            let mut sets = false;
            let mut target = Path { segments: Vec::new(), span: value.span };
            if self.at_setter() {
                self.take_setter();
                sets = true;
                let fallback = self.expression()?;
                if let ExpressionKind::Variable(name) = &value.kind {
                    target = Path {
                        segments: vec![Segment {
                            joint: Joint::Start,
                            mark: Mark::None,
                            limb: false,
                            name: name.clone(),
                        }],
                        span: name.span,
                    };
                }
                value = fallback;
            }
            labels.push(Label { name: target, sets, value, span: start.to(self.previous_span()) });
            if self.is(TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RightSquare, "]")?;
        Ok(labels)
    }

    /// An ownership word, if one is here: `Give`, `Copy`, `Lend` or `Lend.mut`.
    fn ownership_word(&mut self) -> Parse<Option<(Ownership, Span)>> {
        // The parameter's words at a call: the call says `Give` and `Lend.mut` (D2).
        let mistaken = match self.word() {
            Some("Given") if self.kind_at(1) == Some(TokenKind::Dollar) || self.word_at(1).is_some() => {
                Some((Ownership::Give, self.current_span(), "Give"))
            }
            Some("Lent") if self.kind_at(1) == Some(TokenKind::Dot) && self.word_at(2) == Some("mut") => {
                let start = self.current_span();
                self.advance();
                self.advance();
                Some((Ownership::LendMut, start.to(self.current_span()), "Lend.mut"))
            }
            _ => None,
        };
        if let Some((ownership, span, fixed)) = mistaken {
            self.errors.push(Diagnostic::refused_fix(
                span,
                fixed,
                "the call and the parameter each have their own word (D2)",
            ));
            self.advance();
            return Ok(Some((ownership, span)));
        }
        let ownership = match self.word() {
            Some("Give") => Ownership::Give,
            Some("Copy") => Ownership::Copy,
            Some("Lend") => {
                if self.kind_at(1) == Some(TokenKind::Dot) && self.word_at(2) == Some("mut") {
                    let start = self.current_span();
                    self.advance();
                    self.advance();
                    self.advance();
                    return Ok(Some((Ownership::LendMut, start.to(self.previous_span()))));
                }
                Ownership::Lend
            }
            _ => return Ok(None),
        };
        // A word followed by `(` or `::` is a call or path, not an ownership word.
        if matches!(self.kind_at(1), Some(TokenKind::LeftParen | TokenKind::DoubleColon)) {
            return Ok(None);
        }
        let span = self.current_span();
        self.advance();
        Ok(Some((ownership, span)))
    }

    // ----- Types -----

    pub fn parse_type(&mut self) -> Parse<Type> {
        let start = self.current_span();
        if self.is(TokenKind::Question) {
            // `?item`: the `?` marks the name, so it touches it and not what's before (D234).
            self.roles[self.at] = Role::Prefix;
            self.advance();
            let name = self.name()?;
            let name = Name { text: name.text, span: start.to(name.span) };
            return Ok(Type { kind: TypeKind::Parameter(name), span: start.to(self.previous_span()) });
        }
        if self.is_word("Lend") && (self.word_at(1) == Some("from") || self.kind_at(1) == Some(TokenKind::Dot)) {
            let keyword = self.path()?;
            if !self.is_word("from") {
                return Err(self.error_expected("`from`"));
            }
            self.advance();
            let sources_start = self.current_span();
            let mut sources = Vec::new();
            if self.is(TokenKind::LeftAngle) {
                self.advance();
                loop {
                    sources.push(self.borrow_source()?);
                    if self.is(TokenKind::Comma) {
                        self.advance();
                        continue;
                    }
                    break;
                }
                self.expect(TokenKind::RightAngle, ">")?;
            } else {
                sources.push(self.borrow_source()?);
            }
            let sources_span = sources_start.to(self.previous_span());
            let target = self.parse_type()?;
            return Ok(Type {
                kind: TypeKind::Borrow { keyword, sources, sources_span, target: Box::new(target) },
                span: start.to(self.previous_span()),
            });
        }
        if !matches!(self.kind(), Some(TokenKind::Word | TokenKind::At)) {
            return Err(self.error_expected("a type"));
        }
        let path = self.path()?;
        let mut fill = Vec::new();
        let mut fill_span = None;
        if self.is(TokenKind::LeftAngle) {
            let fill_start = self.current_span();
            self.advance();
            while !self.is(TokenKind::RightAngle) && self.peek().is_some() {
                fill.push(self.type_argument()?);
                if self.is(TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            let end = self.expect(TokenKind::RightAngle, ">")?;
            fill_span = Some(fill_start.to(end));
        }
        let mut labels = Vec::new();
        let mut labels_span = None;
        if self.is(TokenKind::LeftSquare) && !self.gap_before_current() {
            let labels_start = self.current_span();
            labels = self.bracket_labels()?;
            labels_span = Some(labels_start.to(self.previous_span()));
        }
        Ok(Type {
            kind: TypeKind::Named { path, fill, fill_span, labels, labels_span },
            span: start.to(self.previous_span()),
        })
    }

    /// A borrow's source: `$r`, `` `text` `` or `program`.
    fn borrow_source(&mut self) -> Parse<Name> {
        match self.kind() {
            Some(TokenKind::Dollar) => self.variable_name(),
            Some(TokenKind::Backtick) => self.source_name(),
            _ => self.name(),
        }
    }

    fn type_argument(&mut self) -> Parse<TypeArgument> {
        let start = self.current_span();
        // `` `text`: $article `` fills a source; `key: str` labels a type.
        if self.is(TokenKind::Backtick) {
            let label = self.source_name()?;
            if self.is(TokenKind::Colon) {
                self.roles[self.at] = Role::LabelColon;
            }
            self.expect(TokenKind::Colon, ":")?;
            let value = if self.is(TokenKind::Backtick) {
                let name = self.source_name()?;
                Expression { span: name.span, kind: ExpressionKind::Variable(name) }
            } else {
                self.or_expression()?
            };
            return Ok(TypeArgument {
                label: Some(label),
                value: TypeArgumentValue::Value(value),
                span: start.to(self.previous_span()),
            });
        }
        let mut label = None;
        if self.is(TokenKind::Word) && self.kind_at(1) == Some(TokenKind::Colon) {
            label = Some(self.name()?);
            self.roles[self.at] = Role::LabelColon;
            self.advance();
        }
        let written_type = self.parse_type()?;
        Ok(TypeArgument { label, value: TypeArgumentValue::Type(written_type), span: start.to(self.previous_span()) })
    }

    /// Whether whitespace comes before the current token.
    fn gap_before_current(&self) -> bool {
        let Some(token) = self.peek() else {
            return false;
        };
        let Some(previous) = self.at.checked_sub(1).and_then(|index| self.tokens.get(index)) else {
            return false;
        };
        previous.span.end != token.span.start
    }

    /// `[label: value, name = value, …]`.
    fn bracket_labels(&mut self) -> Parse<Vec<Label>> {
        self.expect(TokenKind::LeftSquare, "[")?;
        let mut labels = Vec::new();
        while !self.is(TokenKind::RightSquare) && self.peek().is_some() {
            labels.push(self.label()?);
            if self.is(TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RightSquare, "]")?;
        Ok(labels)
    }

    fn label(&mut self) -> Parse<Label> {
        let start = self.current_span();
        if !self.label_ahead() {
            let value = self.expression()?;
            return Ok(Label {
                name: Path { segments: Vec::new(), span: start },
                sets: false,
                span: value.span,
                value,
            });
        }
        let name = self.path()?;
        let sets = if self.is(TokenKind::Colon) {
            self.roles[self.at] = Role::LabelColon;
            self.advance();
            false
        } else if self.at_setter() {
            self.take_setter();
            true
        } else {
            return Err(self.error_expected_after(":"));
        };
        let value = self.expression()?;
        Ok(Label { name, sets, value, span: start.to(self.previous_span()) })
    }

    /// Whether a label, `name:` or `name =`, starts here.
    fn label_ahead(&self) -> bool {
        let mut ahead = 0;
        loop {
            match self.kind_at(ahead) {
                Some(TokenKind::Word | TokenKind::Integer) => ahead += 1,
                _ => return false,
            }
            match self.kind_at(ahead) {
                Some(TokenKind::Dot | TokenKind::DoubleColon) => ahead += 1,
                Some(TokenKind::Colon) => return true,
                Some(TokenKind::Equal) => return true,
                Some(TokenKind::Word) => return self.word_at(ahead) == Some("set_to"),
                _ => return false,
            }
        }
    }

    // ----- Expressions -----

    fn mark_binary_word(&mut self, first: usize, last: usize) {
        if let Some(role) = self.roles.get_mut(first) {
            *role = Role::BinaryStart { last };
        }
    }

    pub fn expression(&mut self) -> Parse<Expression> {
        let value = self.or_expression()?;
        self.no_symbol_operator()?;
        Ok(value)
    }

    /// Operators are words, so a symbol left after a value is refused with the word (D43, D48, D49, D278).
    fn no_symbol_operator(&mut self) -> Parse<()> {
        const ORDER_WHY: &str = "`below`, `above`, `at_least` and `at_most` say their direction outright, and `<` and `>` only mean brackets (D278)";
        const WORDS_WHY: &str = "`at_least` and `at_most` are words, and there is no `<=` or `>=` (D49)";
        let span = self.current_span();
        let spaced = |word: &str, tight: bool| if tight { format!(" {word} ") } else { word.to_string() };
        let tight = !self.gap_before_current();
        let error = match self.kind() {
            Some(TokenKind::LeftAngle) => Diagnostic::refused_fix(span, spaced("below", tight), ORDER_WHY),
            Some(TokenKind::RightAngle) => Diagnostic::refused_fix(span, spaced("above", tight), ORDER_WHY),
            Some(TokenKind::LessEqual) => Diagnostic::refused_fix(span, "at_most", WORDS_WHY),
            Some(TokenKind::GreaterEqual) => Diagnostic::refused_fix(span, "at_least", WORDS_WHY),
            Some(TokenKind::Star) => Diagnostic::refused_fix(span, "x", "multiply is `x`, as in `$a x $b` (D43)"),
            Some(TokenKind::Slash) => Diagnostic::refused(
                span,
                "divide has a narrowing, and no symbol",
                "division has no single right answer, so you pick one (D48)",
            ),
            _ => return Ok(()),
        };
        self.errors.push(error);
        // Past the symbol, so recovering doesn't take a `<` for an open bracket.
        self.advance();
        Err(Reported)
    }

    fn or_expression(&mut self) -> Parse<Expression> {
        let mut left = self.and_expression()?;
        while self.is_word("or") {
            let (operator, operator_span) = self.operator_word("or", BinaryOperator::Or);
            let right = self.and_expression()?;
            left = binary(operator, operator_span, left, right);
        }
        Ok(left)
    }

    fn and_expression(&mut self) -> Parse<Expression> {
        let mut left = self.not_expression()?;
        while self.is_word("and") {
            let (operator, operator_span) = self.operator_word("and", BinaryOperator::And);
            let right = self.not_expression()?;
            left = binary(operator, operator_span, left, right);
        }
        Ok(left)
    }

    /// Reads an operator word and its narrowing, like `div.trunc`, marking it binary.
    fn operator_word(
        &mut self,
        _word: &str,
        make: impl Fn(Option<String>) -> BinaryOperator,
    ) -> (BinaryOperator, Span) {
        let first = self.at;
        let start = self.current_span();
        self.advance();
        let mut narrowing = None;
        if self.is(TokenKind::Dot) && self.kind_at(1) == Some(TokenKind::Word) {
            self.advance();
            if let Some(token) = self.advance() {
                narrowing = Some(self.text_of(token).to_string());
            }
        }
        self.mark_binary_word(first, self.at - 1);
        (make(narrowing), start.to(self.previous_span()))
    }

    fn not_expression(&mut self) -> Parse<Expression> {
        if self.is_word("not") {
            let start = self.current_span();
            self.advance();
            let operand = self.not_expression()?;
            let span = start.to(operand.span);
            return Ok(Expression {
                kind: ExpressionKind::Unary {
                    operator: UnaryOperator::Not,
                    operand: Box::new(operand),
                    operator_span: start,
                },
                span,
            });
        }
        self.comparison()
    }

    fn comparison_operator(&self) -> Option<BinaryOperator> {
        match self.kind()? {
            TokenKind::EqualEqual => Some(BinaryOperator::Equal),
            TokenKind::NotEqual => Some(BinaryOperator::NotEqual),
            TokenKind::Word => match self.word()? {
                "below" => Some(BinaryOperator::Below),
                "above" => Some(BinaryOperator::Above),
                "at_least" => Some(BinaryOperator::AtLeast),
                "at_most" => Some(BinaryOperator::AtMost),
                "lasts_at_least" => Some(BinaryOperator::LastsAtLeast),
                _ => None,
            },
            _ => None,
        }
    }

    fn comparison(&mut self) -> Parse<Expression> {
        let first = self.additive()?;
        let mut operands = vec![first];
        let mut operators = Vec::new();
        while let Some(operator) = self.comparison_operator() {
            let index = self.at;
            let span = self.current_span();
            self.advance();
            self.mark_binary_word(index, index);
            operators.push((operator, span));
            operands.push(self.additive()?);
        }
        if operators.len() == 1 {
            let right = operands.pop().ok_or(Reported)?;
            let left = operands.pop().ok_or(Reported)?;
            let (operator, span) = operators.pop().ok_or(Reported)?;
            return Ok(binary(operator, span, left, right));
        }
        if operators.is_empty() {
            return operands.pop().ok_or(Reported);
        }
        let span = operands.first().map_or(self.end_span(), |first| first.span).to(self.previous_span());
        Ok(Expression { kind: ExpressionKind::Chain { operands, operators }, span })
    }

    fn bit_operator(&self) -> Option<BinaryOperator> {
        match self.word()? {
            "bit_and" => Some(BinaryOperator::BitAnd),
            "bit_or" => Some(BinaryOperator::BitOr),
            "bit_xor" => Some(BinaryOperator::BitXor),
            "shl" => Some(BinaryOperator::ShiftLeft),
            "shr" => Some(BinaryOperator::ShiftRight),
            _ => None,
        }
    }

    /// Bit operators bind tightest of the binary operators, and mixing them with anything is refused by the checker
    /// (D116).
    fn bits(&mut self) -> Parse<Expression> {
        let mut left = self.unary()?;
        while let Some(operator) = self.bit_operator() {
            let index = self.at;
            let span = self.current_span();
            self.advance();
            self.mark_binary_word(index, index);
            let right = self.unary()?;
            left = binary(operator, span, left, right);
        }
        Ok(left)
    }

    fn additive(&mut self) -> Parse<Expression> {
        let mut left = self.multiplicative()?;
        loop {
            let operator = match self.kind() {
                Some(TokenKind::Plus) => BinaryOperator::Add,
                Some(TokenKind::Minus) => BinaryOperator::Subtract,
                Some(TokenKind::Word) if self.is_word("plus") && self.kind_at(1) == Some(TokenKind::Dot) => {
                    let (_, span) = self.operator_word("plus", |_| BinaryOperator::WrappingAdd);
                    let right = self.multiplicative()?;
                    left = binary(BinaryOperator::WrappingAdd, span, left, right);
                    continue;
                }
                _ => break,
            };
            let index = self.at;
            let span = self.current_span();
            self.advance();
            self.mark_binary_word(index, index);
            let right = self.multiplicative()?;
            left = binary(operator, span, left, right);
        }
        Ok(left)
    }

    fn multiplicative(&mut self) -> Parse<Expression> {
        let mut left = self.bits()?;
        loop {
            let (operator, span) = match self.word() {
                Some("x") => {
                    let index = self.at;
                    let span = self.current_span();
                    self.advance();
                    self.mark_binary_word(index, index);
                    (BinaryOperator::Multiply, span)
                }
                Some("div") => self.operator_word("div", BinaryOperator::Divide),
                Some("mod") => self.operator_word("mod", BinaryOperator::Remainder),
                _ => break,
            };
            let right = self.bits()?;
            left = binary(operator, span, left, right);
        }
        Ok(left)
    }

    fn unary(&mut self) -> Parse<Expression> {
        if self.is(TokenKind::Minus) {
            let start = self.current_span();
            self.roles[self.at] = Role::Prefix;
            self.advance();
            let operand = self.unary()?;
            let span = start.to(operand.span);
            return Ok(Expression {
                kind: ExpressionKind::Unary {
                    operator: UnaryOperator::Negate,
                    operand: Box::new(operand),
                    operator_span: start,
                },
                span,
            });
        }
        if self.is_word("bit_not") {
            let start = self.current_span();
            self.advance();
            let operand = self.unary()?;
            let span = start.to(operand.span);
            return Ok(Expression {
                kind: ExpressionKind::Unary {
                    operator: UnaryOperator::BitNot,
                    operand: Box::new(operand),
                    operator_span: start,
                },
                span,
            });
        }
        self.postfix_expression()
    }

    /// A value and what follows it tightly: fields, calls, methods, `[ ]`, `as`, `?`, `??` and `otherwise`.
    pub fn postfix_expression(&mut self) -> Parse<Expression> {
        let mut value = self.primary()?;
        loop {
            match self.kind() {
                Some(TokenKind::Dot) => match self.kind_at(1) {
                    Some(TokenKind::Percent | TokenKind::DoublePercent | TokenKind::At) => {
                        self.advance();
                        let method = self.path()?;
                        value = self.method_call(value, method)?;
                    }
                    Some(TokenKind::Word) if self.kind_at(2) == Some(TokenKind::DoubleColon) => {
                        self.advance();
                        let method = self.path()?;
                        value = self.method_call(value, method)?;
                    }
                    Some(TokenKind::Word) => {
                        self.advance();
                        let field = self.name()?;
                        let span = value.span.to(field.span);
                        value = Expression { kind: ExpressionKind::Field { base: Box::new(value), field }, span };
                    }
                    _ => break,
                },
                Some(TokenKind::LeftParen) if !self.gap_before_current() => {
                    let (arguments, arguments_span) = self.arguments()?;
                    let span = value.span.to(arguments_span);
                    value = Expression {
                        kind: ExpressionKind::Call { callee: Box::new(value), arguments, arguments_span },
                        span,
                    };
                }
                Some(TokenKind::LeftSquare) if !self.gap_before_current() => {
                    let bracket_start = self.current_span();
                    // After a value, brackets get one item, by one position (D133).
                    let labels = if matches!(value.kind, ExpressionKind::Path(_)) {
                        self.bracket_labels()?
                    } else {
                        self.advance();
                        let label = self.label()?;
                        if !self.is(TokenKind::RightSquare) {
                            return Err(self.error_expected("`]`"));
                        }
                        self.advance();
                        vec![label]
                    };
                    let bracket_span = bracket_start.to(self.previous_span());
                    let span = value.span.to(bracket_span);
                    value = Expression {
                        kind: ExpressionKind::Bracket { base: Box::new(value), labels, bracket_span },
                        span,
                    };
                }
                Some(TokenKind::Question) => {
                    let mark = self.current_span();
                    self.advance();
                    let span = value.span.to(mark);
                    value = Expression { kind: ExpressionKind::Pass { value: Box::new(value), mark }, span };
                }
                Some(TokenKind::DoubleQuestion) => {
                    let index = self.at;
                    let mark = self.current_span();
                    self.advance();
                    self.mark_binary_word(index, index);
                    let fallback = self.instead_value()?;
                    let span = value.span.to(fallback.span);
                    value = Expression {
                        kind: ExpressionKind::Instead { value: Box::new(value), fallback: Box::new(fallback), mark },
                        span,
                    };
                }
                Some(TokenKind::Word) if self.is_word("as") => {
                    let index = self.at;
                    let keyword = self.path()?;
                    self.mark_binary_word(index, self.at - 1);
                    let written_type = self.parse_type()?;
                    let span = value.span.to(written_type.span);
                    value =
                        Expression { kind: ExpressionKind::As { value: Box::new(value), keyword, written_type }, span };
                }
                Some(TokenKind::Word) if self.is_word("otherwise") => {
                    let keyword = self.current_span();
                    let index = self.at;
                    self.advance();
                    let handling = if self.is(TokenKind::Colon) {
                        self.roles[self.at] = Role::BlockColon;
                        self.advance();
                        let arms = self.arms()?;
                        let end = self.closer()?;
                        Handling::Arms { arms, end }
                    } else {
                        self.mark_binary_word(index, index);
                        Handling::Handler(self.path()?)
                    };
                    let span = value.span.to(self.previous_span());
                    value = Expression {
                        kind: ExpressionKind::Otherwise { value: Box::new(value), keyword, handling },
                        span,
                    };
                }
                _ => break,
            }
        }
        Ok(value)
    }

    /// What follows `??`: one value, with its own postfix parts, or a unary `-` that the checker refuses.
    fn instead_value(&mut self) -> Parse<Expression> {
        if self.is(TokenKind::Minus) {
            return self.unary();
        }
        let value = self.primary()?;
        let mut value = value;
        while self.is(TokenKind::Dot) && self.kind_at(1) == Some(TokenKind::Word) {
            self.advance();
            let field = self.name()?;
            let span = value.span.to(field.span);
            value = Expression { kind: ExpressionKind::Field { base: Box::new(value), field }, span };
        }
        if self.is(TokenKind::LeftParen) && !self.gap_before_current() {
            let (arguments, arguments_span) = self.arguments()?;
            let span = value.span.to(arguments_span);
            value =
                Expression { kind: ExpressionKind::Call { callee: Box::new(value), arguments, arguments_span }, span };
        }
        Ok(value)
    }

    fn method_call(&mut self, receiver: Expression, method: Path) -> Parse<Expression> {
        if !self.is(TokenKind::LeftParen) {
            return Err(self.error_expected_after("("));
        }
        let (arguments, arguments_span) = self.arguments()?;
        let span = receiver.span.to(arguments_span);
        Ok(Expression {
            kind: ExpressionKind::MethodCall { receiver: Box::new(receiver), method, arguments, arguments_span },
            span,
        })
    }

    fn arguments(&mut self) -> Parse<(Vec<Argument>, Span)> {
        let start = self.current_span();
        self.expect(TokenKind::LeftParen, "(")?;
        let mut arguments = Vec::new();
        while !self.is(TokenKind::RightParen) && self.peek().is_some() {
            let argument_start = self.current_span();
            let mut label = None;
            if self.is(TokenKind::Word)
                && self.kind_at(1) == Some(TokenKind::Colon)
                && self.kind_at(2) != Some(TokenKind::Colon)
            {
                label = Some(self.name()?);
                self.roles[self.at] = Role::LabelColon;
                self.advance();
            } else if self.is(TokenKind::Word) && self.kind_at(1) == Some(TokenKind::Dot) && self.label_with_narrowing()
            {
                // `at.screen.pixels: 5`, a label carrying a unit.
                let path = self.path()?;
                label = Some(Name { text: path.text(), span: path.span });
                self.roles[self.at] = Role::LabelColon;
                self.advance();
            }
            if self.is(TokenKind::Percent) {
                // trace: D5
                self.errors.push(Diagnostic::refused(
                    self.current_span(),
                    "`%` goes only on a method call",
                    "`%` goes only on a method call, never on an argument (D5)",
                ));
                self.advance();
                self.expression()?;
                return Err(Reported);
            }
            let ownership = self.ownership_word()?;
            let value = self.expression()?;
            arguments.push(Argument { label, ownership, value, span: argument_start.to(self.previous_span()) });
            if self.is(TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        let end = self.expect(TokenKind::RightParen, ")")?;
        Ok((arguments, start.to(end)))
    }

    /// Whether a dotted path here ends in a label's `:`.
    fn label_with_narrowing(&self) -> bool {
        let mut ahead = 0;
        loop {
            match (self.kind_at(ahead), self.kind_at(ahead + 1)) {
                (Some(TokenKind::Word), Some(TokenKind::Dot | TokenKind::DoubleColon)) => ahead += 2,
                (Some(TokenKind::Word), Some(TokenKind::Colon)) => {
                    return self.kind_at(ahead + 2) != Some(TokenKind::Colon);
                }
                _ => return false,
            }
        }
    }

    fn primary(&mut self) -> Parse<Expression> {
        let start = self.current_span();
        let Some(token) = self.peek() else {
            return Err(self.error_expected("a value"));
        };
        match token.kind {
            TokenKind::Integer | TokenKind::Float => {
                self.advance();
                let text = self.text_of(token).to_string();
                let literal = Expression {
                    kind: if token.kind == TokenKind::Integer {
                        ExpressionKind::Integer(text)
                    } else {
                        ExpressionKind::Float(text)
                    },
                    span: token.span,
                };
                Ok(self.unit_after(literal)?)
            }
            TokenKind::Tilde => {
                self.roles[self.at] = Role::Prefix;
                self.advance();
                let literal = if self.is(TokenKind::Minus) {
                    self.roles[self.at] = Role::Prefix;
                    let minus = self.current_span();
                    self.advance();
                    let number = self.number_literal()?;
                    let span = minus.to(number.span);
                    Expression {
                        kind: ExpressionKind::Unary {
                            operator: UnaryOperator::Negate,
                            operand: Box::new(number),
                            operator_span: minus,
                        },
                        span,
                    }
                } else {
                    self.number_literal()?
                };
                let span = start.to(literal.span);
                let inexact =
                    Expression { kind: ExpressionKind::Inexact { literal: Box::new(literal), exponent: None }, span };
                self.unit_after(inexact)
            }
            TokenKind::Text => {
                self.advance();
                Ok(Expression { kind: ExpressionKind::Text(self.text_of(token).to_string()), span: token.span })
            }
            TokenKind::Dollar => {
                let name = self.variable_name()?;
                Ok(Expression { span: name.span, kind: ExpressionKind::Variable(name) })
            }
            TokenKind::LeftParen => {
                self.advance();
                let inner = self.expression()?;
                let end = self.expect(TokenKind::RightParen, ")")?;
                Ok(Expression { kind: ExpressionKind::Group(Box::new(inner)), span: start.to(end) })
            }
            TokenKind::LeftSquare => {
                self.advance();
                let mut items = Vec::new();
                while !self.is(TokenKind::RightSquare) && self.peek().is_some() {
                    items.push(self.expression()?);
                    if self.is(TokenKind::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                let end = self.expect(TokenKind::RightSquare, "]")?;
                Ok(Expression { kind: ExpressionKind::List(items), span: start.to(end) })
            }
            TokenKind::At
                if self.kind_at(1) == Some(TokenKind::Word) && self.kind_at(2) != Some(TokenKind::DoubleColon) =>
            {
                self.builtin()
            }
            TokenKind::Word | TokenKind::At => self.word_primary(),
            _ => Err(self.error_expected("a value")),
        }
    }

    fn number_literal(&mut self) -> Parse<Expression> {
        match self.peek() {
            Some(token) if matches!(token.kind, TokenKind::Integer | TokenKind::Float) => {
                self.advance();
                let text = self.text_of(token).to_string();
                Ok(Expression {
                    kind: if token.kind == TokenKind::Integer {
                        ExpressionKind::Integer(text)
                    } else {
                        ExpressionKind::Float(text)
                    },
                    span: token.span,
                })
            }
            _ => Err(self.error_expected("a number")),
        }
    }

    /// A unit after a number, on the same line: `3600 time.seconds`, `5 items`.
    fn unit_after(&mut self, value: Expression) -> Parse<Expression> {
        if let Some(word) = self.word()
            && !self.starts_line()
            && !OPERATOR_WORDS.contains(&word)
            && !matches!(word, "else" | "case" | "returns" | "fails" | "end")
        {
            let unit = self.path()?;
            let span = value.span.to(unit.span);
            return Ok(Expression { kind: ExpressionKind::WithUnit { value: Box::new(value), unit }, span });
        }
        Ok(value)
    }

    fn builtin(&mut self) -> Parse<Expression> {
        let start = self.current_span();
        self.advance();
        let name = self.name()?;
        let mut fill = Vec::new();
        if self.is(TokenKind::LeftAngle) {
            self.advance();
            while !self.is(TokenKind::RightAngle) && self.peek().is_some() {
                fill.push(self.type_argument()?);
                if self.is(TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RightAngle, ">")?;
        }
        let mut arguments = Vec::new();
        if self.is(TokenKind::LeftParen) {
            arguments = self.arguments()?.0;
        }
        let name = Name { text: name.text, span: start.to(name.span) };
        Ok(Expression { kind: ExpressionKind::Builtin { name, fill, arguments }, span: start.to(self.previous_span()) })
    }

    /// A value starting with a word: a keyword value, an ownership word, or a path.
    fn word_primary(&mut self) -> Parse<Expression> {
        let start = self.current_span();
        match self.word() {
            Some("true") => {
                self.advance();
                return Ok(Expression { kind: ExpressionKind::Bool(true), span: start });
            }
            Some("false") => {
                self.advance();
                return Ok(Expression { kind: ExpressionKind::Bool(false), span: start });
            }
            Some("empty") if !self.followed_by_path_joint() && self.kind_at(1) != Some(TokenKind::LeftParen) => {
                self.advance();
                return Ok(Expression { kind: ExpressionKind::Empty, span: start });
            }
            Some("inf") if !self.followed_by_path_joint() && self.kind_at(1) != Some(TokenKind::LeftParen) => {
                self.advance();
                return self.unit_after(Expression { kind: ExpressionKind::Infinity, span: start });
            }
            Some("not") if !self.followed_by_path_joint() => {
                // `not` sits below comparisons and maths, so inside them it needs `( )` (D131).
                self.errors.push(Diagnostic::refused(
                    start,
                    "`not` sits below comparisons and maths, so here it needs `( )`",
                    "`not` sits just above `and`, and `( )` groups (D116, D131)",
                ));
                self.advance();
                let _ = self.unary()?;
                return Err(Reported);
            }
            Some("result") if !self.followed_by_path_joint() && self.kind_at(1) != Some(TokenKind::LeftParen) => {
                self.advance();
                return Ok(Expression { kind: ExpressionKind::Result, span: start });
            }
            Some("all") if !self.followed_by_path_joint() && self.kind_at(1) != Some(TokenKind::LeftParen) => {
                self.advance();
                let value = self.unary()?;
                let span = start.to(value.span);
                return Ok(Expression { kind: ExpressionKind::All(Box::new(value)), span });
            }
            Some("bake") if !self.followed_by_path_joint() && !self.starts_line_at(self.at + 1) => {
                self.advance();
                let value = self.postfix_expression()?;
                let span = start.to(value.span);
                return Ok(Expression { kind: ExpressionKind::Bake(Box::new(value)), span });
            }
            Some("exponent")
                if matches!(
                    self.kind_at(1),
                    Some(TokenKind::Tilde | TokenKind::Integer | TokenKind::Float | TokenKind::Minus)
                ) =>
            {
                let exponent = self.current_span();
                self.advance();
                let literal = if self.is(TokenKind::Minus) {
                    let minus = self.current_span();
                    self.roles[self.at] = Role::Prefix;
                    self.advance();
                    let number = self.number_literal()?;
                    let span = minus.to(number.span);
                    Expression {
                        kind: ExpressionKind::Unary {
                            operator: UnaryOperator::Negate,
                            operand: Box::new(number),
                            operator_span: minus,
                        },
                        span,
                    }
                } else {
                    self.primary()?
                };
                let span = start.to(literal.span);
                let kind = match literal.kind {
                    ExpressionKind::Inexact { literal, .. } => {
                        ExpressionKind::Inexact { literal, exponent: Some(exponent) }
                    }
                    other => ExpressionKind::Exponent(Box::new(Expression { kind: other, span: literal.span })),
                };
                return Ok(Expression { kind, span });
            }
            _ => {}
        }
        if let Some((ownership, ownership_span)) = self.ownership_word()? {
            let value = self.unary()?;
            let span = ownership_span.to(value.span);
            return Ok(Expression {
                kind: ExpressionKind::Owned { ownership, ownership_span, value: Box::new(value) },
                span,
            });
        }
        let path = self.path()?;
        Ok(Expression { span: path.span, kind: ExpressionKind::Path(path) })
    }

    fn followed_by_path_joint(&self) -> bool {
        matches!(self.kind_at(1), Some(TokenKind::DoubleColon | TokenKind::Dot))
            && !self.starts_line_at(self.at + 1)
            && self.peek_at(1).is_some_and(|next| self.peek().is_some_and(|this| this.span.end == next.span.start))
    }
}

fn binary(operator: BinaryOperator, operator_span: Span, left: Expression, right: Expression) -> Expression {
    let span = left.span.to(right.span);
    Expression {
        kind: ExpressionKind::Binary { operator, operator_span, left: Box::new(left), right: Box::new(right) },
        span,
    }
}

/// The base type names, which are keywords.
#[must_use]
pub fn is_type_word(word: &str) -> bool {
    matches!(
        word,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "f32"
            | "f64"
            | "bool"
            | "str"
            | "void"
            | "d32"
    )
}

/// Parses a file's tokens.
#[must_use]
pub fn parse(file: FileId, text: &str, tokens: &[Token]) -> (Parsed, Vec<Diagnostic>) {
    let mut parser = Parser::new(file, text, tokens);
    let parsed = parser.file();
    (Parsed { file: parsed, roles: parser.roles }, parser.errors)
}

/// Whether a value ends in a block closed by `end`, like `otherwise:`.
fn ends_in_block(value: &Expression) -> bool {
    matches!(&value.kind, ExpressionKind::Otherwise { handling: Handling::Arms { .. }, .. })
}
