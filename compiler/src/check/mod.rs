//! The checker: everything the standard refuses, and the checked program for code generation.
//!
//! Checking goes item by item, then statement by statement. After an error, only that statement is skipped
//! (standard 11.7), and a skipped `let` still declares its variable, with the type it wrote.

pub mod aggregates;
pub mod bake;
pub mod contracts;
pub mod failures;
pub mod floats;
pub mod format;
pub mod types;

mod expressions;
pub mod library;
pub mod names;
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
#[derive(Clone, Debug, Default)]
pub struct Signature {
    pub name: String,
    pub id: checked::FunctionId,
    pub parameters: Vec<(String, Type, syntax::ParameterOwnership)>,
    pub returns: Type,
    pub span: Span,
    /// The failures it lists, and its result's layout if there are any (standard 8).
    pub failures: Vec<failures::FailureInfo>,
    pub result: Option<failures::ResultLayout>,
    /// Its labels, defaults and contracts (D114).
    pub top: contracts::TopOfBody,
    pub purity: Purity,
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
    /// A `const` in a body, fixed at compile time (D83).
    Constant,
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
    /// For a union, the case it's known to hold here, inside the test or a `match` arm (D9).
    pub known_case: Option<usize>,
}

/// A loop being checked, for `break` (D128).
#[derive(Clone, Debug)]
pub struct Loop {
    pub id: checked::LoopId,
    pub label: Option<String>,
    /// Variable states at each `break` out of it, joined after it.
    pub breaks: Option<Vec<Vec<State>>>,
    /// How many variables were in scope as it started, so a `break` knows which ones it leaves behind.
    pub variables: usize,
}

/// The `why` for something nobody has decided yet.
pub const NOT_DECIDED: &str = "not decided yet, and the standard doesn't say";

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
    /// The function's failures, and its result's layout, for `fail` and `?`.
    pub failures: Vec<failures::FailureInfo>,
    pub result: Option<failures::ResultLayout>,
    /// The `otherwise:` arms around this, innermost last, for `instead`.
    pub handlers: Vec<failures::Handler>,
    /// The function's labels, defaults and contracts, for `ensures` at each `return`.
    pub top: contracts::TopOfBody,
    /// The value being returned, for `result` in `ensures`.
    pub result_value: Option<(checked::LocalId, Type)>,
    /// The opener of the loop being checked, which its checked form keeps.
    pub loop_opener: Span,
    /// Whether this is a `fn.pure` or `fn.pure.baked` function, whose code is held to more (D86, D91).
    pub purity: Purity,
    /// Whether a contract's condition is being checked, which only calls pure functions (D86).
    pub in_contract: bool,
    /// `const`s declared in this body, by their locals, with their values (D83).
    pub known: HashMap<checked::LocalId, bake::Constant>,
    /// The variables a `match` arm is inside, since reading a case's field through one isn't decided (D139).
    pub matching: Vec<String>,
    /// What the statement being checked does to ownership, so far (D285).
    pub events: Vec<crate::flow::Event>,
    /// The calls whose arguments are being checked, innermost last, which is where a use is said to be.
    pub calls: Vec<Span>,
    /// Parameters that are only lent, bare or `Lent.mut`, so their duties are the caller's (D2).
    pub lent: Vec<checked::LocalId>,
}

/// How pure a function is (D86, D91).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Purity {
    #[default]
    Plain,
    Pure,
    Baked,
}

impl Default for Type {
    fn default() -> Self {
        Type::Void
    }
}

impl Body {
    /// Where a use is said to be: the call around it, or the fallback.
    #[must_use]
    pub fn site(&self, fallback: Span) -> Span {
        self.calls.last().copied().unwrap_or(fallback)
    }

    /// What was recorded since `mark` only happens on some paths, like the right side of `and` (D124).
    pub fn maybe_since(&mut self, mark: usize) {
        let events = self.events.split_off(mark.min(self.events.len()));
        if !events.is_empty() {
            self.events.push(crate::flow::Event::Maybe(events));
        }
    }

    /// Takes back a read of `local` just recorded, since the hand-off around it says what happened instead.
    pub fn take_read(&mut self, local: checked::LocalId) {
        if let Some(crate::flow::Event::Read { place, .. }) = self.events.last()
            && place.local == local
        {
            self.events.pop();
        }
    }

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
    pub aggregates: aggregates::Aggregates,
    /// The program's `const`s, with their types and values (D83).
    pub constants: HashMap<String, (Type, bake::Constant)>,
    pub failure_sets: failures::FailureSets,
    /// Handler narrowings, by their full name, with their arms (D46).
    pub handlers: HashMap<String, Vec<syntax::Arm>>,
    /// `type.alias` and `type.new` (D134).
    pub type_names: names::TypeNames,
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
        aggregates: aggregates::Aggregates::default(),
        constants: HashMap::new(),
        failure_sets: failures::FailureSets::default(),
        handlers: HashMap::new(),
        type_names: names::TypeNames::default(),
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
        self.declare_type_names(file);
        self.declare_aggregates(file);
        self.resolve_type_names();
        self.declare_failure_sets(file);
        self.declare_handlers(file);
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
                if self.functions.contains_key(&signature.name) && signature.name != "main" {
                    let span = function.name.as_ref().map_or(function.header_span, |name| name.span);
                    self.error(Diagnostic::not_designed(
                        span,
                        format!("declaring `{}` twice", signature.name),
                        "not decided yet, and the standard doesn't say",
                    ));
                    header_failed = true;
                    continue;
                }
                self.functions.insert(signature.name.clone(), signature.clone());
                bodies.push((function, signature));
            }
        }
        // How checking carries on past an error in a function's header isn't decided, so it stops there (D215).
        if header_failed {
            return;
        }
        // Baked functions first, so `const`s and `bake` can run them, then every other body (D91).
        let (baked, others): (Vec<_>, Vec<_>) =
            bodies.into_iter().partition(|(_, signature)| signature.purity == Purity::Baked);
        for (function, signature) in baked {
            self.function_body(function, &signature);
        }
        for item in &file.items {
            if let ItemKind::Const(constant) = &item.kind {
                self.global_constant(constant);
            }
        }
        for (function, signature) in others {
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

    /// A path into `std`, `os` or a limb needs its import (D82, D271). The fix adds the import at the top.
    pub fn require_import(&mut self, path: &str, span: Span) -> Check<()> {
        let (package, why) = if path.starts_with("std::") {
            ("std", "everything gets imported, including `std` (D82)")
        } else if path.starts_with("os::") {
            ("os", "a program that uses `os` says `need os` in its manifest and `import os;` in its files (D271, D82)")
        } else if let Some(limb) = path.strip_prefix('@').and_then(|rest| rest.split("::").next()) {
            let key = format!("@{limb}");
            if self.imports.iter().any(|import| *import == key) {
                return Ok(());
            }
            let shown = format!("import @{limb};");
            let edit = crate::diagnostic::Edit { span: Span::new(self.file, 0, 0), replacement: format!("{shown}\n") };
            return Err(self.error(
                Diagnostic::refused_fix(span, shown, "everything gets imported, including limbs (D82)")
                    .with_edits(vec![edit]),
            ));
        } else {
            return Ok(());
        };
        if self.imports.iter().any(|import| import == package || *import == format!("pkg {package}")) {
            return Ok(());
        }
        let shown = format!("import {package};");
        let edit = crate::diagnostic::Edit { span: Span::new(self.file, 0, 0), replacement: format!("{shown}\n") };
        Err(self.error(Diagnostic::refused_fix(span, shown, why).with_edits(vec![edit])))
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
            if value_type.is_copied()
                && matches!(parameter.ownership, syntax::ParameterOwnership::Given)
                && let Some(span) = parameter.ownership_span
            {
                return Err(self.error(Diagnostic::refused(
                    span,
                    format!("`{value_type}` is always copied, and takes no word"),
                    "plain numbers and `bool` are always copied, and take no word (D2, D123)",
                )));
            }
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
        self.method_declaration(function, &name, &parameters)?;
        let failures = match &function.fails {
            Some(list) => self.fail_list(list)?,
            None => Vec::new(),
        };
        let result = if failures.is_empty() { None } else { Some(self.result_layout(&returns, &failures)) };
        let id = self.program.functions.len();
        let symbol = if name == "main" { "main".to_string() } else { format!("cyborg.{name}") };
        let machine_returns = result.map_or_else(|| returns.machine(), failures::result_machine_type);
        self.program.functions.push(checked::Function {
            symbol,
            parameters: Vec::new(),
            returns: machine_returns,
            locals: Vec::new(),
            body: Vec::new(),
        });
        let top = self.top_of_body(function);
        let purity = match keyword.as_str() {
            "fn.pure" => Purity::Pure,
            "fn.pure.baked" => Purity::Baked,
            _ => Purity::Plain,
        };
        if purity != Purity::Plain {
            if function.fails.is_some() {
                let end = function.returns.as_ref().map_or(function.header_span.end, |written| written.span.end);
                return Err(self.error(Diagnostic::not_designed(
                    Span { file: self.file, start: function.header_span.start, end },
                    "whether a `fn.pure` may fail",
                    "not decided yet (standard 12.5)",
                )));
            }
            if let Some(parameter) = function
                .parameters
                .list
                .iter()
                .find(|parameter| parameter.ownership == syntax::ParameterOwnership::LentMut)
            {
                let span = parameter.ownership_span.unwrap_or(parameter.span);
                return Err(self.error(Diagnostic::refused(
                    Span { file: span.file, start: span.start, end: span.start + 4 },
                    "a pure function can't take `Lent.mut`",
                    "a pure function changes nothing outside itself (D86)",
                )));
            }
        }
        Ok(Signature { name, id, parameters, returns, span: function.header_span, failures, result, top, purity })
    }

    fn function_body(&mut self, function: &syntax::Function, signature: &Signature) {
        let errors_before = self.errors.len();
        let mut body = Body {
            scopes: vec![Vec::new()],
            returns: signature.returns.clone(),
            function_name: signature.name.clone(),
            is_main: function.keyword.text() == "fn.main",
            failures: signature.failures.clone(),
            result: signature.result,
            top: signature.top.clone(),
            purity: signature.purity,
            ..Body::default()
        };
        let mut parameters = Vec::new();
        let mut starts = Vec::new();
        for (parameter, (name, value_type, ownership)) in function.parameters.list.iter().zip(&signature.parameters) {
            let local = body.new_local(value_type.machine());
            let passing = if *ownership == syntax::ParameterOwnership::Given
                || (value_type.is_copied() && *ownership != syntax::ParameterOwnership::LentMut)
            {
                checked::Passing::Value
            } else {
                checked::Passing::Address
            };
            parameters.push(checked::Parameter { local, passing });
            if *ownership != syntax::ParameterOwnership::Given {
                body.lent.push(local);
            }
            if !value_type.is_copied() {
                let origin = match ownership {
                    syntax::ParameterOwnership::Given => crate::flow::Origin::Given,
                    syntax::ParameterOwnership::LentMut => crate::flow::Origin::LentMut,
                    _ => crate::flow::Origin::Lent,
                };
                starts.push(crate::flow::Event::Bind {
                    local,
                    name: name.clone(),
                    declared: parameter.name.span,
                    must_give: self.must_give(value_type),
                    origin,
                });
            }
            body.declare(
                Variable {
                    name: name.clone(),
                    value_type: value_type.clone(),
                    mutable: *ownership == syntax::ParameterOwnership::LentMut,
                    origin: Origin::Parameter,
                    local,
                    declared: parameter.name.span,
                    given: None,
                    known_case: None,
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
        // A function with `requires` is told where it was called from, for its crash message (D29).
        let mut prelude = Vec::new();
        if contracts::needs_site(&signature.top) {
            let site = body.new_local(checked::MachineType::Integer { bits: 32, signed: true });
            parameters.push(checked::Parameter { local: site, passing: checked::Passing::Value });
            prelude = self.requires_checks(&mut body, &signature.top, site);
        }
        let (statements, leaves) = self.function_block(&mut body, &function.body);
        let statements: Vec<checked::Statement> =
            std::iter::once(checked::Statement::Flow(starts)).chain(prelude).chain(statements).collect();
        if !leaves && !contains_broken(&function.body) {
            self.function_end(&body, function);
        }
        // Ownership is checked over every path, once the function is otherwise sound (D2, D285).
        if self.errors.len() == errors_before {
            let graph = crate::flow::lower(&statements);
            let found = crate::flow::check(&graph);
            self.errors.extend(found);
        }
        if let Some(target) = self.program.functions.get_mut(signature.id) {
            target.parameters = parameters;
            target.locals = body.locals;
            target.body = statements;
        }
    }

    /// Whether a value of this type carries a duty to be given away or finished (D2, D177).
    #[must_use]
    pub fn must_give(&self, value_type: &Type) -> bool {
        match value_type {
            Type::Struct { id, .. } => self.aggregates.structs.get(*id).is_some_and(|info| info.must_give),
            Type::New { inner, .. } => self.must_give(inner),
            _ => false,
        }
    }

    /// Records that a variable got a value, if it's one whose ownership is followed: anything but a plain number.
    pub fn bind_event(&self, body: &mut Body, local: checked::LocalId, name: &str, declared: Span, value_type: &Type) {
        if value_type.is_copied() || value_type.is_unknown() {
            return;
        }
        body.events.push(crate::flow::Event::Bind {
            local,
            name: name.to_string(),
            declared,
            must_give: self.must_give(value_type),
            origin: crate::flow::Origin::Local,
        });
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

    /// A `const` at the top of a file, worked out while compiling (D83, D91).
    fn global_constant(&mut self, constant: &syntax::Let) {
        let Some(written) = &constant.written_type else { return };
        let Ok(value_type) = self.resolve_type(written) else { return };
        let mut body = Body { scopes: vec![Vec::new()], ..Body::default() };
        if let Ok(value) = self.constant_value(&mut body, &constant.value, &value_type) {
            self.constants.insert(constant.name.text.clone(), (value_type, value));
        }
    }

    /// A value known at compile time: only known values and calls to `fn.pure.baked` functions (D83, D91).
    pub fn constant_value(
        &mut self,
        body: &mut Body,
        value: &syntax::Expression,
        into: &Type,
    ) -> Check<bake::Constant> {
        self.known_at_compile_time(
            body,
            value,
            "a `const`'s value must be fully known at compile time (D83, D91)",
            true,
        )?;
        let checked = self.value_into(body, value, into)?;
        self.run_at_compile_time(body, &checked, value.span)
    }

    /// Runs checked code at compile time.
    pub fn run_at_compile_time(
        &mut self,
        body: &Body,
        checked: &checked::Expression,
        span: Span,
    ) -> Check<bake::Constant> {
        let mut interpreter = bake::Interpreter::new(&self.program);
        interpreter.known.clone_from(&body.known);
        match interpreter.evaluate(checked) {
            Ok(value) => Ok(value),
            Err(bake::Stop::Refused(diagnostic)) => Err(self.error(*diagnostic)),
            Err(bake::Stop::StepLimit(span)) => Err(self.error(Diagnostic::refused(
                span,
                "compile-time work didn't finish within its step limit",
                "compile-time code always finishes, through a step limit that refuses it (D89, D91)",
            ))),
            Err(bake::Stop::NotBuilt(what)) => Err(self.not_yet(span, what)),
        }
    }

    /// Refuses a variable that isn't known at compile time, or a call to a function that isn't `fn.pure.baked`.
    pub fn known_at_compile_time(
        &mut self,
        body: &Body,
        value: &syntax::Expression,
        why: &str,
        constant: bool,
    ) -> Check<()> {
        match &value.kind {
            syntax::ExpressionKind::Variable(name) => {
                let known = body.find(&name.text).is_some_and(|index| body.variables[index].origin == Origin::Constant)
                    || (body.find(&name.text).is_none() && self.constants.contains_key(&name.text));
                if !known {
                    return Err(self.error(Diagnostic::refused(
                        name.span,
                        format!("`{}` isn't known at compile time", name.text),
                        "a `const`'s value must be fully known at compile time (D83, D91)",
                    )));
                }
                Ok(())
            }
            syntax::ExpressionKind::Call { callee, arguments, .. } => {
                if let syntax::ExpressionKind::Path(path) = &callee.kind {
                    let name = self.resolve_path(path);
                    if self.functions.get(&name).is_some_and(|signature| signature.purity != Purity::Baked) {
                        let span = if constant { value.span } else { value.span };
                        return Err(self.error(Diagnostic::refused(
                            span,
                            format!("`{name}` isn't `fn.pure.baked`"),
                            why,
                        )));
                    }
                }
                for argument in arguments {
                    self.known_at_compile_time(body, &argument.value, why, constant)?;
                }
                Ok(())
            }
            syntax::ExpressionKind::Binary { left, right, .. } => {
                self.known_at_compile_time(body, left, why, constant)?;
                self.known_at_compile_time(body, right, why, constant)
            }
            syntax::ExpressionKind::Unary { operand, .. } | syntax::ExpressionKind::Group(operand) => {
                self.known_at_compile_time(body, operand, why, constant)
            }
            // A crossing is the same data, and is known when its value is (D141), but text from `as` is built
            // while running.
            syntax::ExpressionKind::As { value: inner, written_type, .. } => {
                if let syntax::TypeKind::Named { path, .. } = &written_type.kind
                    && self.resolve_path(path) == "str"
                {
                    return Err(self.error(Diagnostic::refused(
                        value.span,
                        "this `as` isn't known at compile time",
                        "a `const`'s value must be fully known at compile time (D83, D91)",
                    )));
                }
                self.known_at_compile_time(body, inner, why, constant)
            }
            _ => Ok(()),
        }
    }

    /// A method is declared with its type and mark in its name, and its receiver first, held the way the mark says
    /// (D130, D211).
    fn method_declaration(
        &mut self,
        function: &syntax::Function,
        name: &str,
        parameters: &[(String, Type, syntax::ParameterOwnership)],
    ) -> Check<()> {
        let Some(path) = &function.name else { return Ok(()) };
        let Some((owner, method)) = name.rsplit_once('.') else { return Ok(()) };
        let mut owned_type = self.aggregates.names.get(owner).cloned();
        if let Some(named) = self.method_owner(owner, path.span) {
            owned_type = Some(named?);
        }
        let Some(owned_type) = owned_type else {
            if method.starts_with('%') {
                if IntegerKind::from_name(owner).is_some()
                    || matches!(owner, "bool" | "str")
                    || owner.starts_with('f') && owner.len() <= 3
                {
                    return Err(self.error(Diagnostic::not_designed(
                        path.span,
                        format!("a method on `{owner}`, which isn't your own type"),
                        "not decided yet, and the standard doesn't say",
                    )));
                }
                return Err(self.error(Diagnostic::refused(
                    path.span,
                    format!("the type `{owner}` isn't known"),
                    "a method is declared with its type in the name (D130)",
                )));
            }
            return Ok(());
        };
        if !method.starts_with('%') {
            return Err(self.error(Diagnostic::refused_fix(
                path.span,
                format!("{owner}.%{method}"),
                "a method is declared with its type in the name, as in `reading.%warm_up` (D130)",
            )));
        }
        let Some((first, receiver_type, ownership)) =
            parameters.first().map(|(_, value_type, ownership)| ((), value_type, ownership))
        else {
            return Err(self.error(Diagnostic::refused(
                path.span,
                format!("`{name}` has no receiver"),
                "a method's first parameter is its receiver (D130)",
            )));
        };
        let _ = first;
        let receiver = &function.parameters.list[0];
        let receiver_labelled = function.body.iter().any(|statement| match &statement.kind {
            syntax::StatementKind::Labels(labels) | syntax::StatementKind::Defaults(labels) => labels.iter().any(|label| {
                matches!(&label.value.kind, syntax::ExpressionKind::As { value, .. } if matches!(&value.kind, syntax::ExpressionKind::Variable(name) if name.text == receiver.name.text))
                    || label.name.segments.first().is_some_and(|segment| segment.name.text == receiver.name.text)
            }),
            _ => false,
        });
        if receiver_labelled {
            return Err(self.error(Diagnostic::refused(
                receiver.name.span,
                "a method's receiver is written before `.%`, so it has no label or default",
                "a method's first parameter is its receiver (D130)",
            )));
        }
        if *receiver_type != owned_type {
            return Err(self.error(Diagnostic::refused(
                receiver.written_type.span,
                format!("this method is on `{owner}`, so its receiver is one"),
                "a method's first parameter is its receiver (D130)",
            )));
        }
        let mark_why =
            "`%` lends a method's receiver read-only and `%%` lends it to change, and the declaration matches (D211)";
        let bare = method.trim_start_matches('%');
        match ownership {
            syntax::ParameterOwnership::Given => {
                return Err(self.error(Diagnostic::refused(
                    receiver.ownership_span.unwrap_or(receiver.span),
                    "a method's receiver can't be `Given`",
                    "methods only borrow, and anything that consumes its value is a plain function (D8, D130)",
                )));
            }
            syntax::ParameterOwnership::LentMut if !method.starts_with("%%") => {
                return Err(self.error(Diagnostic::refused_fix(path.span, format!("{owner}.%%{bare}"), mark_why)));
            }
            syntax::ParameterOwnership::Lent if method.starts_with("%%") => {
                return Err(self.error(Diagnostic::refused_fix(path.span, format!("{owner}.%{bare}"), mark_why)));
            }
            _ => {}
        }
        Ok(())
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
        if let Some(named) = self.named_type(written, &text) {
            return named;
        }
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
        if let Some((base, kind)) = text.rsplit_once('.')
            && IntegerKind::from_name(base).is_some()
            && matches!(kind, "ieee" | "stable" | "fast")
        {
            return Err(self.error(Diagnostic::refused(
                path.span,
                format!("`{base}` is an integer, and only a float says `.{kind}`"),
                "a float says whether it's `.stable`, `.fast` or `.ieee`, and an integer has no such kind (D257, D158)",
            )));
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
                self.require_import(&text, path.span)?;
                let stream = if text.starts_with("os::stdout") { checked::Stream::Out } else { checked::Stream::Error };
                Ok(Type::Writer(types::WriterKind { stream, buffered: text.ends_with(".buffered") }))
            }
            "os::stdout" | "os::stderr" => {
                self.require_import(&text, path.span)?;
                Err(self.error(Diagnostic::refused(
                    path.span,
                    format!("`{text}` says which kind it is"),
                    library::WRITER_KINDS_WHY,
                )))
            }
            _ if self.aggregates.names.contains_key(&text) => Ok(self.aggregates.names[&text].clone()),
            _ if let Some(reported) = self.bare_member(path, &text) => Err(reported),
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
