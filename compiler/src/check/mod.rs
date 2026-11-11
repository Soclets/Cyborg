//! The checker: everything the standard refuses, and the checked program for code generation.
//!
//! Checking goes item by item, then statement by statement. After an error, only that statement is skipped
//! (standard 11.7), and a skipped `let` still declares its variable, with the type it wrote.

pub mod floats;
pub mod format;
pub mod types;

mod expressions;
mod library;
mod statements;

use std::collections::HashMap;

use crate::checked;
use crate::diagnostic::Diagnostic;
use crate::manifest::Manifest;
use crate::source::{FileId, Sources, Span};
use crate::syntax::{self, ItemKind, Joint, Path};
use crate::target::{Mode, Target};

use types::{IntegerKind, Type};

/// Why a closer that leaves something out is refused.
pub const CLOSER_WHY: &str = "the closer repeats all of what it closes, including its narrowings and name (D33, D121)";

/// A check that's already reported its error.
#[derive(Debug, Clone, Copy)]
pub struct Reported;

pub type Check<T> = Result<T, Reported>;

/// A function's signature, as callers see it.
#[derive(Clone, Debug)]
pub struct Signature {
    pub name: String,
    pub id: checked::FunctionId,
    pub parameters: Vec<(String, Type, syntax::ParameterOwnership)>,
    pub returns: Type,
    pub span: Span,
}

/// Whether a variable has its value on the path being checked (D203).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Assigned,
    /// No value on any path yet.
    Unassigned,
    /// A value on some paths only.
    Maybe,
}

impl State {
    /// The state after two paths join.
    #[must_use]
    pub fn join(self, other: State) -> State {
        if self == other { self } else { State::Maybe }
    }
}

/// How a variable came to be, which decides the words its errors use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Let,
    /// A `let` whose value was refused, so it never got one (standard 11.7).
    RefusedLet,
    Decl,
    Parameter,
    LoopItem,
}

/// A variable.
#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub value_type: Type,
    pub mutable: bool,
    pub origin: Origin,
    pub local: checked::LocalId,
    pub declared: Span,
    /// For a `must_Give` value, like a buffered writer: where it was given away or finished, if it was.
    pub given: Option<Span>,
}

/// A loop being checked, for `break` (D128).
#[derive(Clone, Debug)]
pub struct Loop {
    pub id: checked::LoopId,
    pub label: Option<String>,
    /// Variable states at each `break` out of it, joined after it.
    pub breaks: Option<Vec<Vec<State>>>,
}

/// What's in scope while checking one function's body.
#[derive(Default)]
pub struct Body {
    pub variables: Vec<Variable>,
    /// Each variable's state on the path being checked.
    pub states: Vec<State>,
    /// Names in scope, innermost last.
    pub scopes: Vec<Vec<(String, usize)>>,
    pub locals: Vec<checked::MachineType>,
    pub returns: Type,
    pub function_name: String,
    pub is_main: bool,
    pub loops: Vec<Loop>,
    pub next_loop: checked::LoopId,
    /// The span of the statement being checked, which an error about a `decl` with no value points at.
    pub statement: Option<Span>,
}

impl Default for Type {
    fn default() -> Self {
        Type::Void
    }
}

impl Body {
    /// The variable a name means here, by its index.
    #[must_use]
    pub fn find(&self, name: &str) -> Option<usize> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.iter().rev().find(|(each, _)| each == name).map(|(_, index)| *index))
    }

    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&Variable> {
        self.find(name).and_then(|index| self.variables.get(index))
    }

    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Variable> {
        self.find(name).and_then(|index| self.variables.get_mut(index))
    }

    /// Declares a variable in the innermost scope, giving its index.
    pub fn declare(&mut self, variable: Variable, state: State) -> usize {
        let index = self.variables.len();
        if let Some(scope) = self.scopes.last_mut() {
            scope.push((variable.name.clone(), index));
        }
        self.variables.push(variable);
        self.states.push(state);
        index
    }

    pub fn new_local(&mut self, machine_type: checked::MachineType) -> checked::LocalId {
        self.locals.push(machine_type);
        self.locals.len() - 1
    }

    /// Forgets the variables declared after the first `count`, when their scope ends.
    pub fn forget_after(&mut self, count: usize) {
        self.variables.truncate(count);
        self.states.truncate(count);
    }

    /// Joins the states of two paths into this one.
    pub fn join_states(first: &[State], second: &[State]) -> Vec<State> {
        first.iter().zip(second).map(|(a, b)| a.join(*b)).collect()
    }
}

pub struct Checker<'a> {
    pub file: FileId,
    /// Every file, so a runtime check's message can be rendered while building.
    pub sources: &'a Sources,
    pub mode: Mode,
    pub text: &'a str,
    pub manifest: &'a Manifest,
    pub target: Target,
    pub errors: Vec<Diagnostic>,
    /// Packages and parts imported: `std`, `os`, `@std`, `csup`.
    pub imports: Vec<String>,
    /// Short names from `use`, to the path they stand for.
    pub uses: HashMap<String, String>,
    pub functions: HashMap<String, Signature>,
    pub program: checked::Program,
}

/// Checks a parsed file, giving the program if nothing was refused.
#[must_use]
pub fn check(
    file: FileId,
    sources: &Sources,
    manifest: &Manifest,
    parsed: &syntax::File,
    target: Target,
    mode: Mode,
) -> (Option<checked::Program>, Vec<Diagnostic>) {
    let text = &sources.get(file).text;
    let mut checker = Checker {
        file,
        sources,
        mode,
        text,
        manifest,
        target,
        errors: Vec::new(),
        imports: Vec::new(),
        uses: HashMap::new(),
        functions: HashMap::new(),
        program: checked::Program::default(),
    };
    checker.check_file(parsed);
    let failed = !checker.errors.is_empty();
    (if failed { None } else { Some(checker.program) }, checker.errors)
}

impl Checker<'_> {
    pub fn error(&mut self, diagnostic: Diagnostic) -> Reported {
        self.errors.push(diagnostic);
        Reported
    }

    #[must_use]
    pub fn slice(&self, span: Span) -> &str {
        self.text.get(span.range()).unwrap_or("")
    }

    fn check_file(&mut self, file: &syntax::File) {
        // Imports and short names first, then every signature, since declarations work in any order (D120).
        for item in &file.items {
            match &item.kind {
                ItemKind::Import(import) => self.import(import),
                ItemKind::Use(short) => self.use_item(short),
                _ => {}
            }
        }
        let mut bodies = Vec::new();
        let mut main = None;
        let mut header_failed = false;
        for item in &file.items {
            if let ItemKind::Function(function) = &item.kind {
                let Ok(signature) = self.signature(function) else {
                    header_failed = true;
                    continue;
                };
                if function.keyword.text() == "fn.main" {
                    main = Some(signature.id);
                }
                self.functions.insert(signature.name.clone(), signature.clone());
                bodies.push((function, signature));
            }
        }
        // How checking carries on past an error in a function's header isn't decided, so it stops there (D215).
        if header_failed {
            return;
        }
        for (function, signature) in bodies {
            self.function_body(function, &signature);
        }
        if let Some(id) = main {
            self.program.entry = id
        } else {
            let start = Span::new(self.file, 0, 1);
            self.error(Diagnostic::refused(
                start,
                "this program has no `fn.main`",
                "a program starts at `fn.main` (D11)",
            ));
        }
    }

    fn import(&mut self, import: &syntax::Import) {
        let Some(first) = import.path.first() else {
            return;
        };
        let package = first.name.text.clone();
        let key = if first.limb { format!("@{package}") } else { package.clone() };
        if matches!(package.as_str(), "std" | "os" | "csup") && !self.manifest.needs(&package) {
            self.error(Diagnostic::refused(
                first.name.span.to(import.path.span),
                format!("`{package}` has no `need` line in `CyJet.siml`"),
                "each package a program uses has a `need` line in its manifest, with its version (D183, D228)",
            ));
        }
        if package == "os" && !self.target.has_os() {
            // trace: D271
            self.error(Diagnostic::not_designed(
                first.name.span.to(import.path.span),
                "`os` on a board with no operating system, since a bare-metal program's writers aren't decided",
                "not decided yet (standard 12.2)",
            ));
        }
        self.imports.push(key);
    }

    fn use_item(&mut self, short: &syntax::Use) {
        self.uses.insert(short.short.text.clone(), short.path.text());
    }

    /// The full path a written path stands for, through any `use`.
    #[must_use]
    pub fn resolve_path(&self, path: &Path) -> String {
        let text = path.text();
        if let Some(first) = path.first()
            && first.joint == Joint::Start
            && !first.limb
            && let Some(full) = self.uses.get(&first.name.text)
        {
            let rest = &text[first.name.text.len()..];
            return format!("{full}{rest}");
        }
        text
    }

    fn signature(&mut self, function: &syntax::Function) -> Check<Signature> {
        let keyword = function.keyword.text();
        let name = match (&function.name, keyword.as_str()) {
            (_, "fn.main") => "main".to_string(),
            (Some(name), _) => name.text(),
            (None, _) => return Err(Reported),
        };
        let mut parameters = Vec::new();
        for parameter in &function.parameters.list {
            let value_type = self.resolve_type(&parameter.written_type)?;
            parameters.push((parameter.name.text.clone(), value_type, parameter.ownership));
        }
        let returns = match &function.returns {
            Some(written) => self.resolve_type(written)?,
            None => Type::Void,
        };
        if keyword == "fn.main" && returns != Type::Void {
            let span = function.returns.as_ref().map_or(function.header_span, |written| written.span);
            return Err(self.error(Diagnostic::refused(
                span,
                "`fn.main` returns `void`",
                "a program starts at `fn.main`, and `main` says `return void;` (D11)",
            )));
        }
        let id = self.program.functions.len();
        let symbol = if name == "main" { "main".to_string() } else { format!("cyborg.{name}") };
        self.program.functions.push(checked::Function {
            symbol,
            parameters: Vec::new(),
            returns: returns.machine(),
            locals: Vec::new(),
            body: Vec::new(),
        });
        Ok(Signature { name, id, parameters, returns, span: function.header_span })
    }

    fn function_body(&mut self, function: &syntax::Function, signature: &Signature) {
        let mut body = Body {
            scopes: vec![Vec::new()],
            returns: signature.returns.clone(),
            function_name: signature.name.clone(),
            is_main: function.keyword.text() == "fn.main",
            ..Body::default()
        };
        let mut parameters = Vec::new();
        for (parameter, (name, value_type, ownership)) in function.parameters.list.iter().zip(&signature.parameters) {
            let local = body.new_local(value_type.machine());
            let passing = if value_type.is_copied() && *ownership != syntax::ParameterOwnership::LentMut {
                checked::Passing::Value
            } else {
                checked::Passing::Address
            };
            parameters.push(checked::Parameter { local, passing });
            body.declare(
                Variable {
                    name: name.clone(),
                    value_type: value_type.clone(),
                    mutable: *ownership == syntax::ParameterOwnership::LentMut,
                    origin: Origin::Parameter,
                    local,
                    declared: parameter.name.span,
                    given: None,
                },
                State::Assigned,
            );
        }
        let keyword = function.keyword.text();
        let expected = match (&function.name, &function.operator) {
            (Some(name), _) => format!("end {keyword} {}", name.text()),
            (None, Some(operator)) => format!("end {keyword} {}", operator.text),
            (None, None) => format!("end {keyword}"),
        };
        self.check_closer(&function.end, &expected, CLOSER_WHY);
        let (statements, leaves) = self.function_block(&mut body, &function.body);
        if !leaves && !contains_broken(&function.body) {
            self.function_end(&body, function);
        }
        if let Some(target) = self.program.functions.get_mut(signature.id) {
            target.parameters = parameters;
            target.locals = body.locals;
            target.body = statements;
        }
    }

    /// A closer repeats what it closes, including its narrowings and name (D33, D121).
    pub fn check_closer(&mut self, closer: &syntax::Closer, expected: &str, why: &str) {
        let written: Vec<&str> = self.slice(closer.span).split_whitespace().collect();
        let expected_words: Vec<&str> = expected.split_whitespace().collect();
        if written != expected_words {
            if written.get(1) == Some(&"pub") {
                self.error(Diagnostic::not_designed(
                    closer.span,
                    "whether a closer repeats `pub`",
                    "not decided yet (standard 12.2)",
                ));
                return;
            }
            self.error(Diagnostic::refused_fix(closer.span, expected, why));
        }
    }

    /// A function whose body can run off its end (D11).
    fn function_end(&mut self, body: &Body, function: &syntax::Function) {
        let closer = function.end.span;
        if body.is_main {
            self.error(Diagnostic::refused(
                closer,
                "`fn.main` ends without `return void;`",
                "`main` says `return void;` (D11)",
            ));
        } else if body.returns == Type::Void {
            self.error(Diagnostic::not_designed(
                closer,
                "whether every `void` function says `return void;`",
                "not decided yet (standard 12.5)",
            ));
        } else if !body.returns.is_unknown() {
            self.error(Diagnostic::refused(
                closer,
                format!("`{}` can reach its end without a `return`", body.function_name),
                "`return` hands back a value (D11)",
            ));
        }
    }

    /// The type a written type means.
    pub fn resolve_type(&mut self, written: &syntax::Type) -> Check<Type> {
        let syntax::TypeKind::Named { path, fill, labels, .. } = &written.kind else {
            return Err(self.not_yet(written.span, "this kind of type".to_string()));
        };
        let text = self.resolve_path(path);
        if let Some(kind) = IntegerKind::from_name(&text) {
            if !matches!(text.as_str(), "i32" | "i64" | "u8" | "u32" | "u64") {
                return Err(self.error(Diagnostic::not_designed(
                    path.span,
                    format!("the type `{text}`, since the full list isn't chosen"),
                    "not decided yet (standard 12.2)",
                )));
            }
            if !fill.is_empty() {
                return Err(self.not_yet(written.span, format!("`{text}` filled in with `< >`")));
            }
            let bounds = self.integer_bounds(written.span, labels)?;
            return Ok(Type::Integer { kind, bounds });
        }
        if text == "int.signed.32_bit" {
            return Err(self.error(Diagnostic::refused_fix(
                path.span,
                "i32",
                "a 32-bit signed integer is `i32`, with one spelling and no alias (D10)",
            )));
        }
        if let Some(rest) = text.strip_prefix("f64").or_else(|| text.strip_prefix("f32")) {
            let bits = if text.starts_with("f64") { 64 } else { 32 };
            let kind = match rest {
                ".stable" => types::FloatKind::Stable,
                ".fast" => types::FloatKind::Fast,
                ".ieee" => types::FloatKind::Ieee,
                _ => {
                    return Err(self.error(Diagnostic::refused(
                        path.span,
                        format!("`f{bits}` says which kind of float it is: `.stable`, `.fast` or `.ieee`"),
                        "a float always says its kind straight after its type, since floats differ between machines in nearly every other language (D257)",
                    )));
                }
            };
            let bounds = self.float_bounds(written.span, bits, kind, labels)?;
            return Ok(Type::Float { bits, kind, bounds });
        }
        match text.as_str() {
            "bool" => Ok(Type::Bool),
            "str" => Ok(Type::Str),
            "void" => Ok(Type::Void),
            "os::stdout.buffered" | "os::stdout.unbuffered" | "os::stderr.buffered" | "os::stderr.unbuffered" => {
                let stream = if text.starts_with("os::stdout") { checked::Stream::Out } else { checked::Stream::Error };
                Ok(Type::Writer(types::WriterKind { stream, buffered: text.ends_with(".buffered") }))
            }
            _ if text.contains("::") || text.contains('.') => {
                Err(self.not_yet(written.span, format!("the type `{text}`")))
            }
            _ => Err(self.error(Diagnostic::refused(
                path.span,
                format!("the type `{text}` isn't known"),
                "a type is built in, or declared with `struct` or `union.tagged` (D33, D57)",
            ))),
        }
    }

    /// A float's bounds, `f64.stable[min: -40.0, max: 125.0]`, whose literals follow D59 (D158).
    fn float_bounds(
        &mut self,
        span: Span,
        bits: u32,
        kind: types::FloatKind,
        labels: &[syntax::Label],
    ) -> Check<Option<types::FloatBounds>> {
        if labels.is_empty() {
            return Ok(None);
        }
        if kind != types::FloatKind::Stable {
            return Err(self.error(Diagnostic::not_designed(
                span,
                format!("a bound on `f{bits}.{}`", kind.word()),
                "not decided yet (standard 12.2)",
            )));
        }
        let mut min = None;
        let mut max = None;
        let plain = Type::Float { bits, kind, bounds: None };
        for label in labels {
            let name = label.name.text();
            if name != "min" && name != "max" {
                return Err(self.error(Diagnostic::refused(
                    label.name.span,
                    format!("a float has no `{name}`"),
                    format!("`f{bits}[min: -40.0, max: 125.0]` is still {bits} bits, and holds only that range (D158)"),
                )));
            }
            if !matches!(
                label.value.kind,
                syntax::ExpressionKind::Float(_)
                    | syntax::ExpressionKind::Integer(_)
                    | syntax::ExpressionKind::Inexact { .. }
                    | syntax::ExpressionKind::Exponent(_)
                    | syntax::ExpressionKind::Unary { .. }
            ) {
                return Err(self.error(Diagnostic::not_designed(
                    label.value.span,
                    "a bound that isn't a literal",
                    "not decided yet, and the standard doesn't say",
                )));
            }
            let mut body = Body::default();
            let typed = self.expression(&mut body, &label.value, Some(&plain))?;
            let Some(value) = typed.float_literal else {
                return Err(self.error(Diagnostic::not_designed(
                    label.value.span,
                    "a bound that isn't a literal",
                    "not decided yet, and the standard doesn't say",
                )));
            };
            if name == "min" {
                min = Some(value);
            } else {
                max = Some(value);
            }
        }
        let (Some(min), Some(max)) = (min, max) else {
            return Err(self.error(Diagnostic::not_designed(
                span,
                "a bound with only `min` or only `max`",
                "not decided yet, and the standard doesn't say",
            )));
        };
        if min > max {
            return Err(self.error(Diagnostic::refused(
                span,
                "`min` is above `max`, so nothing fits",
                "a bounded float holds only its range (D158)",
            )));
        }
        Ok(Some(types::FloatBounds::new(min, max)))
    }

    fn integer_bounds(&mut self, span: Span, labels: &[syntax::Label]) -> Check<Option<types::Bounds>> {
        if labels.is_empty() {
            return Ok(None);
        }
        let mut min = None;
        let mut max = None;
        for label in labels {
            let value = self.constant_integer(&label.value)?;
            match label.name.text().as_str() {
                "min" => min = Some(value),
                "max" => max = Some(value),
                _ => {
                    return Err(self.not_yet(label.span, format!("the label `{}` on an integer", label.name.text())));
                }
            }
        }
        let (Some(min), Some(max)) = (min, max) else {
            return Err(self.error(Diagnostic::not_designed(
                span,
                "a bound with only `min` or only `max`",
                "not decided yet, and the standard doesn't say",
            )));
        };
        if min > max {
            return Err(self.error(Diagnostic::refused(
                span,
                "`min` is above `max`, so nothing fits",
                "a bounded integer holds only its range (D20)",
            )));
        }
        Ok(Some(types::Bounds { min, max }))
    }

    /// An integer known while compiling, such as a bound.
    pub fn constant_integer(&mut self, expression: &syntax::Expression) -> Check<i128> {
        match &expression.kind {
            syntax::ExpressionKind::Integer(text) => {
                text.replace('_', "").parse().map_err(|_| self.not_yet(expression.span, "this number".to_string()))
            }
            syntax::ExpressionKind::Unary { operator: syntax::UnaryOperator::Negate, operand, .. } => {
                Ok(-self.constant_integer(operand)?)
            }
            _ => Err(self.not_yet(expression.span, "a bound that isn't a whole number".to_string())),
        }
    }

    /// Something decided that this compiler doesn't handle yet, which is `NotBuiltYet`, never `NotDesignedYet`
    /// (D286).
    pub fn not_yet(&mut self, span: Span, what: String) -> Reported {
        self.error(Diagnostic::new(
            crate::diagnostic::Kind::NotBuiltYet,
            span,
            what,
            "this compiler doesn't handle it yet (D286)",
        ))
    }
}

/// Whether a block holds a statement the parser couldn't read, which might have been the `return`.
fn contains_broken(statements: &[syntax::Statement]) -> bool {
    statements.iter().any(|statement| match &statement.kind {
        syntax::StatementKind::Broken => true,
        syntax::StatementKind::If(chain) => {
            chain.branches.iter().any(|branch| contains_broken(&branch.body))
                || chain.otherwise.as_ref().is_some_and(|(_, body)| contains_broken(body))
        }
        syntax::StatementKind::While { body, .. } | syntax::StatementKind::Forever { body, .. } => {
            contains_broken(body)
        }
        syntax::StatementKind::For(each) => contains_broken(&each.body),
        _ => false,
    })
}

/// Whether a name of yours clashes with a keyword, a word operator, a base type or `main`, so it's written with `my::`
/// (D75, D214).
#[must_use]
pub fn clashes(name: &str) -> bool {
    matches!(
        name,
        "x" | "div"
            | "mod"
            | "below"
            | "above"
            | "at_least"
            | "at_most"
            | "and"
            | "or"
            | "not"
            | "bit_and"
            | "bit_or"
            | "bit_xor"
            | "bit_not"
            | "shl"
            | "shr"
            | "as"
            | "if"
            | "else"
            | "for"
            | "while"
            | "forever"
            | "break"
            | "return"
            | "let"
            | "mut"
            | "change"
            | "decl"
            | "assign"
            | "fn"
            | "struct"
            | "union"
            | "match"
            | "case"
            | "end"
            | "in"
            | "fail"
            | "fails"
            | "finish"
            | "otherwise"
            | "instead"
            | "empty"
            | "inf"
            | "true"
            | "false"
            | "void"
            | "bool"
            | "str"
            | "i32"
            | "i64"
            | "u8"
            | "u32"
            | "u64"
            | "main"
            | "range"
            | "all"
    )
}
