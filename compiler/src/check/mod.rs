//! The checker: everything the standard refuses, and the checked program for code generation.
//!
//! Checking goes item by item, then statement by statement. After an error, only that statement is skipped
//! (standard 11.7), and a skipped `let` still declares its variable, with the type it wrote.

pub mod format;
pub mod types;

mod expressions;
mod library;
mod statements;

use std::collections::HashMap;

use crate::checked;
use crate::diagnostic::Diagnostic;
use crate::manifest::Manifest;
use crate::source::{FileId, Span};
use crate::syntax::{self, ItemKind, Joint, Path};

use types::{IntegerKind, Type};

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

/// A variable in scope.
#[derive(Clone, Debug)]
pub struct Variable {
    pub value_type: Type,
    pub mutable: bool,
    pub local: checked::LocalId,
    pub declared: Span,
    /// For a `must_Give` value, like a buffered writer: where it was given away or finished, if it was.
    pub given: Option<Span>,
}

/// What's in scope while checking one function's body.
#[derive(Default)]
pub struct Body {
    pub scopes: Vec<HashMap<String, Variable>>,
    pub locals: Vec<checked::MachineType>,
    pub returns: Type,
    pub next_loop: checked::LoopId,
}

impl Default for Type {
    fn default() -> Self {
        Type::Void
    }
}

impl Body {
    #[must_use]
    pub fn lookup(&self, name: &str) -> Option<&Variable> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut Variable> {
        self.scopes.iter_mut().rev().find_map(|scope| scope.get_mut(name))
    }

    pub fn declare(&mut self, name: &str, variable: Variable) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), variable);
        }
    }

    pub fn new_local(&mut self, machine_type: checked::MachineType) -> checked::LocalId {
        self.locals.push(machine_type);
        self.locals.len() - 1
    }
}

pub struct Checker<'a> {
    pub file: FileId,
    pub text: &'a str,
    pub manifest: &'a Manifest,
    pub target: crate::target::Target,
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
    text: &str,
    manifest: &Manifest,
    parsed: &syntax::File,
    target: crate::target::Target,
) -> (Option<checked::Program>, Vec<Diagnostic>) {
    let mut checker = Checker {
        file,
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
        for item in &file.items {
            if let ItemKind::Function(function) = &item.kind
                && let Ok(signature) = self.signature(function)
            {
                if function.keyword.text() == "fn.main" {
                    main = Some(signature.id);
                }
                self.functions.insert(signature.name.clone(), signature.clone());
                bodies.push((function, signature));
            }
        }
        for (function, signature) in bodies {
            self.function_body(function, &signature);
        }
        if let Some(id) = main { self.program.entry = id } else {
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
        let mut body = Body { scopes: vec![HashMap::new()], returns: signature.returns.clone(), ..Body::default() };
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
                name,
                Variable {
                    value_type: value_type.clone(),
                    mutable: false,
                    local,
                    declared: parameter.name.span,
                    given: None,
                },
            );
        }
        let statements = self.block(&mut body, &function.body);
        if let Some(target) = self.program.functions.get_mut(signature.id) {
            target.parameters = parameters;
            target.locals = body.locals;
            target.body = statements;
        }
    }

    /// The type a written type means.
    pub fn resolve_type(&mut self, written: &syntax::Type) -> Check<Type> {
        match &written.kind {
            syntax::TypeKind::Named { path, fill, labels, .. } => {
                let text = self.resolve_path(path);
                if let Some(kind) = IntegerKind::from_name(&text) {
                    if !fill.is_empty() {
                        return Err(self.not_yet(written.span, format!("`{text}` filled in with `< >`")));
                    }
                    let bounds = self.integer_bounds(kind, labels)?;
                    return Ok(Type::Integer { kind, bounds });
                }
                match text.as_str() {
                    "bool" => Ok(Type::Bool),
                    "str" => Ok(Type::Str),
                    "void" => Ok(Type::Void),
                    "f64.stable" | "f64.fast" | "f64.ieee" | "f32.stable" | "f32.fast" | "f32.ieee" => {
                        let bits = if text.starts_with("f64") { 64 } else { 32 };
                        let kind = match text.rsplit('.').next() {
                            Some("fast") => types::FloatKind::Fast,
                            Some("ieee") => types::FloatKind::Ieee,
                            _ => types::FloatKind::Stable,
                        };
                        Ok(Type::Float { bits, kind })
                    }
                    "os::stdout.buffered"
                    | "os::stdout.unbuffered"
                    | "os::stderr.buffered"
                    | "os::stderr.unbuffered" => {
                        let stream =
                            if text.starts_with("os::stdout") { checked::Stream::Out } else { checked::Stream::Error };
                        Ok(Type::Writer(types::WriterKind { stream, buffered: text.ends_with(".buffered") }))
                    }
                    _ => Err(self.not_yet(written.span, format!("the type `{text}`"))),
                }
            }
            _ => Err(self.not_yet(written.span, "this kind of type".to_string())),
        }
    }

    fn integer_bounds(&mut self, kind: IntegerKind, labels: &[syntax::Label]) -> Check<Option<types::Bounds>> {
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
            return Err(self.not_yet(labels[0].span, "a bound with only `min` or only `max`".to_string()));
        };
        let _ = kind;
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

    /// Something this compiler doesn't handle yet. Reported as `NotDesignedYet` with the reason that the compiler
    /// hasn't reached it, so it's never mistaken for a decision.
    pub fn not_yet(&mut self, span: Span, what: String) -> Reported {
        self.error(Diagnostic::not_designed(span, what, "this compiler doesn't handle it yet"))
    }
}
