//! Refuses what the standard refuses, and turns the rest into resolved code for the runner.
//! This file holds the declarations; `body.rs` holds statements and `expression.rs` expressions.

mod body;
mod expression;
mod generics;
mod hand;
mod operators;

use std::collections::{HashMap, HashSet};

use crate::ast::{
    Describe, FailureEntry, Fill, Function, FunctionKind, Item, ParameterWord, Path, Program,
    ShieldContent, Statement, StatementKind, TypeExpression, TypeKind, TypeName, UnitMeaning,
};
use crate::ir::{Code, Node};
use crate::report::{Outcome, Report, ReportExtra, Span};
use crate::syntax::is_clashing;
use crate::types::{FloatBound, FloatKind, FloatRules, IntegerKind, Sources, Type};

/// A struct's, union's or function's sources, and which lasts at least as long as which. (D171)
type SourceList = (Vec<String>, Vec<(String, String)>);

#[derive(Clone, Debug)]
struct StructInfo {
    must_give: bool,
    /// A generic struct's type parameters, each with what it needs. (D234)
    type_parameters: Vec<(String, Vec<String>)>,
    fields: Vec<(String, Type)>,
    /// Its sources in backticks, and which lasts at least as long as which. (D171)
    sources: Vec<String>,
    facts: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
struct UnionInfo {
    cases: Vec<(String, Vec<(String, Type)>)>,
    /// (D176)
    sources: Vec<String>,
    facts: Vec<(String, String)>,
}

/// A failure's identity is its full name, so `file_failed.denied` and `net_failed.denied` differ.
#[derive(Clone, Debug, PartialEq)]
struct Failure {
    short: String,
    full: String,
    data: Vec<(String, Type)>,
}

#[derive(Clone, Debug)]
struct ParameterInfo {
    name: String,
    label: Option<String>,
    word: ParameterWord,
    parameter_type: Type,
    default: Option<crate::ast::Expression>,
    /// The label says a unit, as in `at.screen.pixels`, so a plain number takes it. (D144)
    label_has_unit: bool,
}

/// `op <` for `reading` and `reading`, done by the function named `op < [reading, reading]`.
/// (D236)
#[derive(Clone, Debug)]
pub(super) struct OperatorInfo {
    pub(super) operator: String,
    pub(super) left: Type,
    pub(super) right: Type,
    pub(super) function: String,
    pub(super) result: Type,
}

#[derive(Clone, Debug)]
struct Signature {
    index: usize,
    name: String,
    kind: FunctionKind,
    /// A generic's type parameters, each with what it needs. (D234, D235)
    type_parameters: Vec<(String, Vec<String>)>,
    parameters: Vec<ParameterInfo>,
    any_order: bool,
    return_type: Type,
    failures: Vec<Failure>,
    /// ``fn shown <`t`>``: its own sources, and which lasts at least as long as which. (D171)
    sources: Vec<String>,
    source_facts: Vec<(String, String)>,
    /// `requires $list lasts_at_least $article;`, by parameter names. (D171)
    lasts: Vec<(String, String)>,
}

#[derive(Clone, Debug)]
struct UnitInfo {
    /// The base this unit traces back to, once worked out.
    base: Option<String>,
    target: Option<(String, Span)>,
    span: Span,
}

#[derive(Clone, Debug)]
struct GlobalInfo {
    index: usize,
    value_type: Type,
    node: Node,
}

#[derive(Clone, Debug)]
struct TogetherInfo {
    left: String,
    multiply: bool,
    right: String,
    result: String,
}

/// What ownership knows about a variable at one point in the code.
#[derive(Clone, Debug, Default)]
struct VariableState {
    given: Option<Span>,
    /// Given on one path and not on another, as a join of branches leaves it.
    given_on_some_path: bool,
    trust: Option<usize>,
    trust_ended: Option<Span>,
    /// In a `match` arm on this variable, where its data is read through the arm's name.
    matched: bool,
    /// `given` came from `finish`, so the duty is over, though nothing says the value is dead.
    finished: bool,
    /// For a stored borrow: where what it borrows was used in a way that ends the borrow.
    invalidated: Option<Span>,
    /// Fields moved out with `Give $l.file`. (D185)
    moved: Vec<MovedField>,
    /// Whether a `decl` variable has its value yet, on every path here. (D203)
    filled: Filled,
}

/// A `decl` variable's value: there on every path, on none, or only on some. (D203)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Filled {
    #[default]
    Yes,
    No,
    OnSomePaths,
}

/// A field moved out, where, and whether that happened on every path here. (D185)
#[derive(Clone, Debug, PartialEq, Eq)]
struct MovedField {
    path: Vec<usize>,
    span: Span,
    every_path: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Origin {
    Owned,
    GivenParameter,
    LentParameter,
    LentMutParameter,
    ForItem,
    Constant,
    /// A stored borrow, with whether it can change what it borrows.
    StoredBorrow,
    /// What a `for … in Give $xs` hasn't handed out yet, which is a duty while it runs. (D180)
    LoopRest,
}

impl Variable {
    fn name_shown(&self) -> String {
        format!("${}", self.name)
    }
}

#[derive(Clone, Debug)]
struct Variable {
    name: String,
    held_type: Type,
    mutable: bool,
    origin: Origin,
    declared: Span,
    must_give: bool,
    constant: Option<Node>,
    /// For a stored borrow: the variables it borrows.
    borrows: Vec<String>,
    /// For a stored borrow: which field of its source it holds, empty for the whole of it.
    borrowed_path: Vec<usize>,
}

/// One failure arm around the code being checked, for `instead` to find.
#[derive(Clone, Debug)]
struct ArmInfo {
    value_type: Type,
    scope_depth: usize,
    loop_depth: usize,
    /// What ownership knew at each `instead` in this arm.
    instead_states: Vec<Vec<VariableState>>,
}

/// One loop around the code being checked.
#[derive(Clone, Debug)]
struct LoopInfo {
    id: usize,
    name: Option<String>,
    /// How many scopes were open outside it, so a `break` knows which it leaves.
    scope_depth: usize,
    /// What ownership knew at each `break` that stops this loop.
    break_states: Vec<Vec<VariableState>>,
}

/// Facts about the function being checked.
#[allow(clippy::struct_excessive_bools)] // Each says what the code being checked is inside of.
struct FunctionContext {
    kind: FunctionKind,
    name: String,
    return_type: Type,
    failures: Option<Vec<Failure>>,
    scopes: Vec<HashMap<String, usize>>,
    variables: Vec<Variable>,
    state: Vec<VariableState>,
    in_contract: bool,
    in_ensures: bool,
    result_slot: usize,
    in_constant: usize,
    in_shield: usize,
    /// How many `on.target` chip branches the code being checked is in. (D233)
    in_chip: usize,
    call_span: Option<Span>,
    ensures_variables: Vec<(String, Span)>,
    /// The failure arms around the code being checked, innermost last.
    arms: Vec<ArmInfo>,
    /// The loops around the code being checked, innermost last.
    loops: Vec<LoopInfo>,
    next_loop_id: usize,
    /// Variables a `for` is reading in place right now.
    being_read: Vec<(String, Vec<usize>)>,
    /// Collections a `for … in Lend.mut` is changing through its items, so nothing else may use
    /// them meanwhile. (D180)
    being_changed: Vec<String>,
    /// Stored borrows made `.held`, which last until their block ends. (D169)
    held_borrows: Vec<usize>,
    /// The first use that a `.held` borrow forbids, and the borrow, found while noting uses.
    held_conflict: Option<(Span, usize)>,
    /// `requires $list lasts_at_least $article;` and the function's own source facts. (D171)
    lasts: Vec<(String, String)>,
    source_facts: Vec<(String, String)>,
    /// A call handing back a borrow may be checked now, for a stored borrow or a `return`. (D170)
    borrow_result_wanted: bool,
    /// `Give $l.file` is checking `$l`, which may still have fields moved out. (D185)
    moving_field: bool,
    /// What the last such call's result borrows, by the caller's names. (D170)
    borrowed_result: Option<BorrowedResult>,
    /// Variables a skipped `let` or `decl` made, which have their type and no value. (D215)
    skipped: HashSet<usize>,
    /// Names a broken `:=` or `const` line left with no known type, so lines using them can't
    /// be checked, and say nothing. (D215, D216)
    unknown: HashSet<String>,
    /// Whether a skipped statement was one that leaves, like `return`. (D215)
    skipped_exit: bool,
    /// The type a call.s result goes into, which can fill a generic.s type parameter. (D241)
    call_expected: Option<Type>,
}

/// What a returned borrow borrows at the call: the original values, and the names it went
/// through on the way, which also can't be used meanwhile. (D170, D174)
#[derive(Clone, Debug, Default)]
struct BorrowedResult {
    roots: Vec<String>,
    through: Vec<String>,
}

impl FunctionContext {
    fn outside_any_function() -> Self {
        Self {
            kind: FunctionKind::Plain,
            name: String::new(),
            return_type: Type::Void,
            failures: None,
            scopes: vec![HashMap::new()],
            variables: Vec::new(),
            state: Vec::new(),
            in_contract: false,
            in_ensures: false,
            result_slot: 0,
            in_constant: 0,
            in_shield: 0,
            in_chip: 0,
            call_span: None,
            ensures_variables: Vec::new(),
            arms: Vec::new(),
            loops: Vec::new(),
            next_loop_id: 0,
            being_read: Vec::new(),
            being_changed: Vec::new(),
            held_borrows: Vec::new(),
            held_conflict: None,
            borrow_result_wanted: false,
            moving_field: false,
            lasts: Vec::new(),
            source_facts: Vec::new(),
            borrowed_result: None,
            skipped: HashSet::new(),
            unknown: HashSet::new(),
            skipped_exit: false,
            call_expected: None,
        }
    }
}

pub struct Checker<'a> {
    source: &'a str,
    imports: HashSet<String>,
    limb_imports: HashSet<String>,
    aliases: HashMap<String, String>,
    /// What `import std::io::fmt_print;` brought in, without `std::`. (D198)
    imported_things: Vec<String>,
    /// `%append` from `use std::list.%append as %append;`, with its type and real name. (D178)
    pub(super) method_aliases: HashMap<String, (String, String, bool)>,
    structs: HashMap<String, StructInfo>,
    unions: HashMap<String, UnionInfo>,
    /// The sources a type may name here: a struct's while its fields are read, a function's while
    /// its signature and body are. (D171)
    allowed_sources: std::cell::RefCell<Vec<String>>,
    failure_sets: HashMap<String, Vec<Failure>>,
    handlers: HashMap<(String, String), Vec<crate::ast::Arm>>,
    units: HashMap<String, UnitInfo>,
    together: Vec<TogetherInfo>,
    signatures: HashMap<String, Signature>,
    /// What `type.alias` and `type.new` names stand for.
    named: HashMap<String, Type>,
    globals: HashMap<String, GlobalInfo>,
    global_nodes: Vec<Node>,
    compile_time: Vec<Node>,
    context: FunctionContext,
    /// The type parameters in scope, with what each needs, while a generic is read. (D234)
    generic: Vec<(String, Vec<String>)>,
    /// A generic struct's instances, like `pair<i32, str>`, made as types name them. (D234)
    instances: std::cell::RefCell<HashMap<String, StructInfo>>,
    /// Interfaces, type sets, and which types satisfy which interface. (D235)
    interfaces: HashMap<String, generics::InterfaceInfo>,
    type_sets: HashMap<String, Vec<Type>>,
    satisfied: Vec<(Type, String)>,
    /// Each type's own operators: which, for which types, and the function that does it. (D236)
    operators: Vec<OperatorInfo>,
    /// Every error found so far, since checking carries on past one. (D207, D215)
    errors: Vec<Report>,
    /// Set when an error sits where how to carry on isn't decided, like an `if`'s line or a
    /// function's header. Checking stops there, and what's still unwinding is already counted.
    stopped: bool,
    /// The packages the manifest needs, when there is a manifest. (D183)
    needs: Option<Vec<String>>,
}

/// Compile-time work's limit, standing in for D89's "such as through a step limit".
const WHY_TYPES: &str = "`:` doesn't convert, and `as` builds a new value of another type (D7)";

/// The largest array `all` will build here. Past this the oracle says so, and doesn't run out of
/// memory, which rule 6 forbids.
const ITEM_LIMIT: i128 = 16_000_000;

/// Checks the whole program, and gives every error it finds, in the order they appear.
/// trace: D72, D207, D215
pub fn check(
    source: &str,
    program: &Program,
    needs: Option<&[String]>,
) -> Result<Code, Vec<Report>> {
    let mut checker = Checker {
        source,
        imports: HashSet::new(),
        limb_imports: HashSet::new(),
        aliases: HashMap::new(),
        imported_things: Vec::new(),
        method_aliases: HashMap::new(),
        structs: HashMap::new(),
        unions: HashMap::new(),
        allowed_sources: std::cell::RefCell::new(Vec::new()),
        failure_sets: HashMap::new(),
        handlers: HashMap::new(),
        units: HashMap::new(),
        together: Vec::new(),
        signatures: HashMap::new(),
        named: HashMap::new(),
        globals: HashMap::new(),
        global_nodes: Vec::new(),
        compile_time: Vec::new(),
        context: FunctionContext::outside_any_function(),
        generic: Vec::new(),
        instances: std::cell::RefCell::new(HashMap::new()),
        interfaces: HashMap::new(),
        type_sets: HashMap::new(),
        satisfied: Vec::new(),
        operators: Vec::new(),
        errors: Vec::new(),
        stopped: false,
        needs: needs.map(<[String]>::to_vec),
    };
    let result = checker.check_program(program);
    let stopped = checker.stopped;
    let mut errors = checker.errors;
    match result {
        Ok(code) if errors.is_empty() => Ok(code),
        Ok(_) => Err(in_order(errors)),
        Err(report) => {
            if !stopped {
                errors.push(*report);
            }
            Err(in_order(errors))
        }
    }
}

/// Errors in the order they appear in the file, each once. (D207)
pub fn in_order(mut errors: Vec<Report>) -> Vec<Report> {
    errors.sort_by_key(|report| report.span.start);
    errors.dedup_by(|later, earlier| {
        later.span == earlier.span && later.what == earlier.what && later.why == earlier.why
    });
    errors
}

/// The items of a program, with those inside shields included.
fn every_item(program: &Program) -> Vec<&Item> {
    let mut items = Vec::new();
    for item in &program.items {
        items.push(item);
        if let Item::Shield(shield) = item {
            for content in &shield.items {
                if let ShieldContent::Item(inner) = content {
                    items.push(inner.as_ref());
                }
            }
        }
    }
    items
}

impl Checker<'_> {
    fn check_program(&mut self, program: &Program) -> Outcome<Code> {
        self.read_imports(program)?;
        let items = every_item(program);
        self.check_shields(program)?;
        self.declare_type_names(&items)?;
        self.declare_units(&items)?;
        self.declare_named_types(&items)?;
        self.declare_requirements(&items)?;
        self.declare_type_contents(&items)?;
        self.declare_failures(&items)?;
        self.declare_handlers(&items);
        let functions = Self::functions_of(&items);
        self.declare_signatures(&functions)?;
        self.declare_satisfies(&items)?;
        self.refuse_short_names_that_clash(program)?;
        self.check_globals(&items)?;

        let mut compiled = Vec::new();
        // An error in a function's body was counted where it was found, and checking went on
        // with the next statement. One in its header stops checking, since how to carry on
        // past it isn't decided. (D207, D215)
        for function in &functions {
            match self.check_function(function) {
                Ok(code) => compiled.push(code),
                Err(report) => {
                    if !self.stopped {
                        let body_start = function
                            .body
                            .first()
                            .map_or(function.end_span.start, |statement| statement.span.start);
                        self.stopped = report.span.start < body_start;
                        self.errors.push(*report);
                    }
                    if self.stopped {
                        return Err(Report::refused(function.span, "", ""));
                    }
                }
            }
        }
        let main = self.find_main(&functions)?;
        Ok(Code {
            main,
            functions: compiled,
            globals: std::mem::take(&mut self.global_nodes),
            compile_time: std::mem::take(&mut self.compile_time),
        })
    }

    fn functions_of<'b>(items: &[&'b Item]) -> Vec<&'b Function> {
        items
            .iter()
            .filter_map(|item| match item {
                Item::Function(function) => Some(function),
                _ => None,
            })
            .collect()
    }

    fn find_main(&self, functions: &[&Function]) -> Outcome<usize> {
        // trace: D23
        functions
            .iter()
            .position(|function| function.kind == FunctionKind::Main)
            .ok_or_else(|| {
                Report::refused(
                    Span::new(0, self.source.len().min(1)),
                    "this program has no `fn.main`",
                    "a program starts at `fn.main` (D11)",
                )
            })
    }

    /// One `import` line: a package's half, both halves, or one thing. (D82, D187, D188, D198)
    fn read_import(&mut self, import: &crate::ast::Import) -> Outcome<()> {
        // Each package a program uses has a `need` line in its manifest. (D183, D186, D228)
        // trace: D183, D228
        let package = import
            .name
            .text
            .split("::")
            .next()
            .unwrap_or(&import.name.text);
        if let Some(needs) = &self.needs {
            if matches!(package, "std" | "csup") && !needs.iter().any(|need| need == package) {
                return Err(Report::refused(
                    import.name.span,
                    format!("`{package}` has no `need` line in `CyJet.siml`"),
                    "each package a program uses has a `need` line in its manifest, with its version (D183, D228)",
                ));
            }
        }
        // trace: D198
        if let Some((kit, thing)) = import.name.text.split_once("::") {
            if import.limb || import.whole || kit != "std" {
                return Err(Report::not_designed(
                    import.name.span,
                    "importing one thing with `@`, with `pkg`, or from a kit other than `std`",
                    "12.2",
                ));
            }
            if !matches!(thing, "io::fmt_print" | "units::time") {
                return Err(Report::not_designed(
                    import.name.span,
                    format!("`std::{thing}`, since `std`'s full contents aren't chosen"),
                    "",
                ));
            }
            self.imported_things.push(thing.to_string());
            return Ok(());
        }
        // trace: D188, D187
        if import.whole {
            match import.name.text.as_str() {
                "std" => {
                    self.imports.insert("std".to_string());
                    self.limb_imports.insert("std".to_string());
                }
                "csup" => {
                    self.limb_imports.insert("csup".to_string());
                }
                other => {
                    return Err(Report::not_designed(
                        import.name.span,
                        format!("how the package `{other}` is found"),
                        "12.9",
                    ));
                }
            }
            return Ok(());
        }
        if import.limb {
            match import.name.text.as_str() {
                "csup" | "std" => {
                    self.limb_imports.insert(import.name.text.clone());
                }
                other => {
                    return Err(Report::not_designed(
                        import.name.span,
                        format!("how the limb `@{other}` is found"),
                        "12.9",
                    ));
                }
            }
        } else if import.name.text == "std" {
            self.imports.insert("std".to_string());
        } else {
            return Err(Report::not_designed(
                import.name.span,
                format!("how the kit `{}` is found", import.name.text),
                "12.9",
            ));
        }
        Ok(())
    }

    /// `use` only shortens a path with `::` into a kit or limb, so nothing in the base language,
    /// and no `inline.…`, has a short name. The fix removes the line. (D232, D227)
    /// trace: D232, D227
    fn refuse_inline_use(declared: &crate::ast::Use) -> Outcome<()> {
        if let Some(report) = expression::inline_spelling(&declared.path.text, declared.path.span) {
            return Err(report);
        }
        let text = &declared.path.text;
        if text.starts_with("inline.") || !text.contains("::") {
            return Err(Report::refused(
                declared.span,
                format!("`{text}` is in the base language, so it has no shorter name"),
                "`use` only shortens a path with `::` into a kit or limb, so every `inline.…` stays written in full (D232)",
            )
            .fix(""));
        }
        Ok(())
    }

    fn read_imports(&mut self, program: &Program) -> Outcome<()> {
        for import in &program.imports {
            self.read_import(import)?;
        }
        for declared in &program.uses {
            Self::refuse_inline_use(declared)?;
            // A limb works the same way, and a file still has to import it.
            // trace: D155, D82
            if let Some(rest) = declared.path.text.strip_prefix('@') {
                let limb = rest.split("::").next().unwrap_or(rest);
                if limb == "csup" || limb == "std" {
                    self.require_limb(limb, declared.path.span)?;
                    // trace: D225
                    if rest.starts_with("csup::extern") {
                        return Err(Report::not_designed(
                            declared.path.span,
                            "how a C function is declared and called through `@csup`",
                            "12.5",
                        ));
                    }
                } else {
                    return Err(Report::not_designed(
                        declared.path.span,
                        format!("how the limb `@{limb}` is found"),
                        "12.9",
                    ));
                }
            }
            // trace: D150
            let last_word = declared
                .path
                .text
                .rsplit(['.', ':'])
                .next()
                .unwrap_or_default()
                .trim_start_matches('%');
            let marks = declared
                .path
                .text
                .split_once(".%")
                .map_or(0, |(_, rest)| 1 + usize::from(rest.starts_with('%')));
            let mark = "%".repeat(marks);
            if declared.quoted {
                // trace: D151
                if !crate::syntax::could_be_called(&declared.short.text) {
                    return Err(Report::refused(
                        declared.short.span,
                        format!("nothing could use `\"{}\"` as a name", declared.short.text),
                        "the quotes only skip the check against the path, so a quoted name still has to be usable (D151)",
                    ));
                }
            } else if let Some(function) = narrowed_function(&declared.path.text) {
                // trace: D220
                let bare = function.rsplit("::").next().unwrap_or(function);
                return Err(Report::refused(
                    declared.span,
                    format!("`{}` alone doesn't say it's `{function}`'s narrowing", declared.short.text),
                    "a function's narrowing means something only next to its function, so it isn't shortened to its bare name, though a quoted name is your own choice (D220, D150)",
                )
                .fix(format!("use {function} as {bare};")));
            } else if declared.short.text != last_word {
                return Err(Report::refused(
                    declared.short.span,
                    format!("`{mark}{}` doesn't match `{mark}{last_word}`, the end of this path", declared.short.text),
                    "an unquoted name has to match the last word of its path, and a quoted one is your own choice (D150)",
                ));
            }
            // trace: D178
            if declared.method {
                let (owner, _) = declared
                    .path
                    .text
                    .split_once(".%")
                    .unwrap_or((&declared.path.text, ""));
                // `append` changes its receiver, so its path says `%%`. (D211)
                // trace: D211
                if owner == "std::list" && last_word == "append" && marks != 2 {
                    return Err(Report::refused(
                        declared.path.span,
                        "`append` changes its receiver, so it's `.%%append`",
                        "`.%` lends a receiver read-only and `.%%` lends it to change (D211)",
                    )
                    .fix("std::list.%%append"));
                }
                self.method_aliases.insert(
                    declared.short.text.clone(),
                    (owner.to_string(), last_word.to_string(), marks == 2),
                );
                continue;
            }
            // trace: D93
            self.aliases
                .insert(declared.short.text.clone(), declared.path.text.clone());
        }
        // trace: D198
        if !self.imported_things.is_empty() && self.imports.contains("std") {
            return Err(Report::not_designed(
                program
                    .imports
                    .first()
                    .map_or(Span::new(0, 0), |import| import.name.span),
                "importing one thing alongside `import std;`",
                "12.2",
            ));
        }
        Ok(())
    }

    /// ``<`text`, `table`: lasts_at_least `text`>``: each source has its own name, and no fact is
    /// written twice. (D171)
    /// trace: D171
    fn read_source_declarations(declared: &[crate::ast::SourceDeclaration]) -> Outcome<SourceList> {
        let mut sources: Vec<String> = Vec::new();
        for source in declared {
            if sources.contains(&source.name.text) {
                return Err(Report::refused(
                    source.name.span,
                    format!("`{}` is named twice", source.name.text),
                    "each source has its own name (D171)",
                ));
            }
            sources.push(source.name.text.clone());
        }
        let mut facts: Vec<(String, String)> = Vec::new();
        for source in declared {
            for shorter in &source.lasts_at_least {
                if !sources.contains(&shorter.text) {
                    return Err(Report::refused(
                        shorter.span,
                        format!("`{}` isn't one of these sources", shorter.text),
                        "`lasts_at_least` compares two declared sources (D171)",
                    ));
                }
                let fact = (source.name.text.clone(), shorter.text.clone());
                if facts.contains(&fact) || fact.0 == fact.1 {
                    return Err(Report::refused(
                        shorter.span,
                        format!(
                            "`{}` lasts at least as long as `{}` is already known",
                            fact.0, fact.1
                        ),
                        "the same `lasts_at_least` fact isn't written twice (D171)",
                    ));
                }
                facts.push(fact);
            }
        }
        Ok((sources, facts))
    }

    /// `requires $list lasts_at_least $article;` names two parameters, and says each fact once.
    /// trace: D171
    fn read_lasts(
        written: &[crate::ast::LastsAtLeast],
        parameters: &[ParameterInfo],
    ) -> Outcome<Vec<(String, String)>> {
        let mut facts: Vec<(String, String)> = Vec::new();
        for fact in written {
            for name in [&fact.longer, &fact.shorter] {
                if !parameters
                    .iter()
                    .any(|parameter| parameter.name == name.text)
                {
                    return Err(Report::refused(
                        name.span,
                        format!("`${}` isn't a parameter", name.text),
                        "`requires … lasts_at_least …` compares two parameters (D171)",
                    ));
                }
            }
            let pair = (fact.longer.text.clone(), fact.shorter.text.clone());
            if facts.contains(&pair) || pair.0 == pair.1 {
                return Err(Report::refused(
                    fact.span,
                    format!(
                        "`${}` lasts at least as long as `${}` is already known",
                        pair.0, pair.1
                    ),
                    "the same `lasts_at_least` fact isn't written twice (D171)",
                ));
            }
            facts.push(pair);
        }
        Ok(facts)
    }

    /// A returned struct holding borrows names parameters as its sources, like a returned borrow.
    /// trace: D171, D170
    fn check_holding_claim(
        return_type: &Type,
        parameters: &[ParameterInfo],
        span: Span,
    ) -> Outcome<()> {
        if let Type::Borrow {
            sources: Sources::Named(_),
            ..
        } = return_type
        {
            return Err(Report::not_designed(
                span,
                "returning a borrow from a function's own source",
                "",
            ));
        }
        let Type::Holding { fills, .. } = return_type else {
            return Ok(());
        };
        let mut names = Vec::new();
        for fill in fills {
            if let Some(name) = fill.strip_prefix('$') {
                names.push(name.to_string());
            } else {
                return Err(Report::not_designed(
                    span,
                    format!("returning a struct whose source is filled with `{fill}`"),
                    "",
                ));
            }
        }
        let claim = Type::Borrow {
            mutable: false,
            sources: Sources::Variables(names),
            inner: Box::new(Type::Struct(String::new())),
        };
        Self::check_borrow_claim(&claim, parameters, span)
    }

    /// A struct's declaration, or a generic struct's instance, like `pair<i32, str>`. (D234)
    fn struct_info(&self, name: &str) -> Option<StructInfo> {
        self.structs
            .get(name)
            .cloned()
            .or_else(|| self.instances.borrow().get(name).cloned())
    }

    /// The sources a struct or union declares, by its name.
    pub(super) fn sources_of(&self, base: &Type) -> (Vec<String>, Vec<(String, String)>) {
        match base {
            Type::Struct(name) => self
                .struct_info(name)
                .map(|info| (info.sources, info.facts))
                .unwrap_or_default(),
            Type::Union(name) => self
                .unions
                .get(name)
                .map(|info| (info.sources.clone(), info.facts.clone()))
                .unwrap_or_default(),
            _ => (Vec::new(), Vec::new()),
        }
    }

    /// `-> Lend from <$a, $b> reading` names where the borrow comes from, and that has to be a
    /// parameter that outlives the call. (D170)
    /// trace: D170
    fn check_borrow_claim(
        return_type: &Type,
        parameters: &[ParameterInfo],
        span: Span,
    ) -> Outcome<()> {
        let Type::Borrow {
            mutable,
            sources,
            inner,
        } = return_type
        else {
            return Ok(());
        };
        // trace: D123
        if inner.is_always_copied() {
            return Err(Report::refused(
                span,
                format!("a borrow of `{inner}`, which is always copied"),
                "plain numbers and `bool` are always copied, so there's nothing to borrow (D170, D123)",
            ));
        }
        let names = match sources {
            Sources::Variables(names) => names,
            Sources::Named(_) => {
                return Err(Report::not_designed(
                    span,
                    "returning a borrow from a function's own source",
                    "",
                ));
            }
            Sources::Program => {
                return Err(Report::not_designed(
                    span,
                    "returning a borrow `from program`",
                    "12.8",
                ));
            }
        };
        for name in names {
            let Some(parameter) = parameters.iter().find(|parameter| parameter.name == *name)
            else {
                return Err(Report::refused(
                    span,
                    format!("`${name}` isn't a parameter, and a returned borrow comes from one"),
                    "a function hands back a borrow only of a parameter, a named source or `program` (D170)",
                ));
            };
            match parameter.word {
                ParameterWord::Given => {
                    return Err(Report::refused(
                        span,
                        format!("`${name}` is given to this function, so it ends when the function does"),
                        "a returned borrow can't come from a `Given` parameter, which ends with the function (D170)",
                    ));
                }
                ParameterWord::Bare if *mutable => {
                    return Err(Report::refused(
                        span,
                        format!("`${name}` is only lent to read, so the function can't hand back `Lend.mut` of it"),
                        "`Lend.mut` comes only from what can change, here a `Lent.mut` parameter (D170, D2)",
                    ));
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// A short name that's also a name of yours has no stated winner.
    fn refuse_short_names_that_clash(&self, program: &Program) -> Outcome<()> {
        for declared in &program.uses {
            let name = &declared.short.text;
            // trace: D178
            if declared.method {
                let (owner, _, _) = self.method_aliases.get(name).cloned().unwrap_or_default();
                if self.signatures.contains_key(&format!("{owner}.%{name}")) {
                    return Err(Report::refused(
                        declared.short.span,
                        format!("`%{name}` is also a method of yours on `{owner}`"),
                        "a short kit method and one of your own with the same name on the same type clash (D178)",
                    ));
                }
                continue;
            }
            if self.signatures.contains_key(name)
                || self.structs.contains_key(name)
                || self.unions.contains_key(name)
            {
                return Err(Report::not_designed(
                    declared.short.span,
                    format!("`use … as {name}` when `{name}` is also declared in this file"),
                    "",
                ));
            }
        }
        Ok(())
    }

    /// trace: D34
    fn check_shields(&mut self, program: &Program) -> Outcome<()> {
        for item in &program.items {
            if let Item::Shield(shield) = item {
                if shield.reason.is_none() {
                    return Err(Report::refused(
                        shield.name.span,
                        format!("the shield `{}` has no reason", shield.name.text),
                        "a shield is a named block with a written reason (D34)",
                    ));
                }
                for content in &shield.items {
                    if let ShieldContent::UntaggedUnion { path, .. } = content {
                        if path.text != "@csup::union.untagged" {
                            return Err(Report::not_designed(
                                path.span,
                                format!("`{}`", path.text),
                                "12.9",
                            ));
                        }
                        self.require_limb("csup", path.span)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// trace: D82
    fn require_std(&self, rest: &str, span: Span) -> Outcome<()> {
        if self.imports.contains("std") {
            return Ok(());
        }
        // `import std::io::fmt_print;` imports only that, with its narrowings. (D198, D199)
        // trace: D198, D199
        if !self.imported_things.is_empty() {
            let reached = self
                .imported_things
                .iter()
                .any(|thing| rest == thing.as_str() || rest.starts_with(&format!("{thing}.")));
            if reached {
                return Ok(());
            }
            let things: Vec<String> = self
                .imported_things
                .iter()
                .map(|thing| format!("`std::{thing}`"))
                .collect();
            return Err(Report::refused(
                span,
                format!("this file imports only {} from `std`", things.join(" and ")),
                "importing one thing imports only that, with its narrowings, unlike `import std;` (D198, D199)",
            ));
        }
        Err(Report::refused(
            span,
            "this file uses `std` without importing it",
            "everything gets imported, including `std` (D82)",
        )
        .fix_elsewhere("import std;"))
    }

    /// trace: D82
    fn require_limb(&self, limb: &str, span: Span) -> Outcome<()> {
        if self.limb_imports.contains(limb) {
            return Ok(());
        }
        Err(Report::refused(
            span,
            format!("this file uses `@{limb}` without importing it"),
            "everything gets imported, including limbs (D82)",
        )
        .fix_elsewhere(format!("import @{limb};")))
    }

    fn declare_type_names(&mut self, items: &[&Item]) -> Outcome<()> {
        for item in items {
            let name = match item {
                Item::Struct(declaration) => &declaration.name,
                Item::Union(declaration) => &declaration.name,
                _ => continue,
            };
            if self.structs.contains_key(&name.text) || self.unions.contains_key(&name.text) {
                return Err(Report::not_designed(
                    name.span,
                    format!("declaring `{}` twice", name.text),
                    "",
                ));
            }
            match item {
                Item::Struct(declaration) => {
                    let (sources, facts) = Self::read_source_declarations(&declaration.sources)?;
                    self.structs.insert(
                        name.text.clone(),
                        StructInfo {
                            must_give: declaration.must_give,
                            type_parameters: Vec::new(),
                            fields: Vec::new(),
                            sources,
                            facts,
                        },
                    );
                }
                Item::Union(declaration) => {
                    let (sources, facts) = Self::read_source_declarations(&declaration.sources)?;
                    self.unions.insert(
                        name.text.clone(),
                        UnionInfo {
                            cases: Vec::new(),
                            sources,
                            facts,
                        },
                    );
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// trace: D109
    fn declare_units(&mut self, items: &[&Item]) -> Outcome<()> {
        for item in items {
            let Item::UnitSet(set) = item else { continue };
            for member in &set.members {
                let prefix = format!("{}.", set.family.text);
                if !member.path.text.starts_with(&prefix) {
                    return Err(Report::refused(
                        member.path.span,
                        format!("this unit is in the family `{}`", set.family.text),
                        "a unit always says its family (D112)",
                    )
                    .fix(format!("{}{}", prefix, member.path.text)));
                }
                // trace: D109
                if self.units.contains_key(&member.path.text) {
                    return Err(Report::refused(
                        member.path.span,
                        format!("`{}` is declared twice", member.path.text),
                        "every unit says what it is, once (D109)",
                    ));
                }
                let (base, target) = match &member.meaning {
                    UnitMeaning::Base => (Some(member.path.text.clone()), None),
                    UnitMeaning::Conversion { unit } => {
                        (None, Some((unit.text.clone(), unit.span)))
                    }
                    UnitMeaning::Unsaid => {
                        return Err(Report::refused(
                            member.path.span,
                            "this unit says neither `base` nor `=`",
                            "every unit says what it is, `: base` or `=` with a conversion (D109)",
                        ));
                    }
                };
                self.units.insert(
                    member.path.text.clone(),
                    UnitInfo {
                        base,
                        target,
                        span: member.path.span,
                    },
                );
            }
        }
        self.trace_units_to_their_bases()?;
        for item in items {
            if let Item::Together(line) = item {
                let left = self.resolve_unit(&line.left)?;
                let right = self.resolve_unit(&line.right)?;
                let result = self.resolve_unit(&line.result)?;
                // trace: D111
                self.together.push(TogetherInfo {
                    left,
                    multiply: line.operator == crate::ast::TogetherOperator::Multiply,
                    right,
                    result,
                });
            }
        }
        Ok(())
    }

    /// Follows each `=` back to a base. A loop never gets there, and is refused. (D109)
    fn trace_units_to_their_bases(&mut self) -> Outcome<()> {
        let mut names: Vec<(usize, String)> = self
            .units
            .iter()
            .map(|(name, info)| (info.span.start, name.clone()))
            .collect();
        // Source order, so which of several errors comes first doesn't depend on hashing.
        names.sort();
        for (_, name) in names {
            let mut seen = vec![name.clone()];
            let mut current = name.clone();
            let base = loop {
                let Some(info) = self.units.get(&current) else {
                    break None;
                };
                if let Some(base) = &info.base {
                    break Some(base.clone());
                }
                let Some((target, target_span)) = info.target.clone() else {
                    break None;
                };
                let target = self.canonical(&target);
                // trace: D109
                if target.starts_with("std::") {
                    let resolved = self.resolve_unit(&Path {
                        text: target,
                        span: target_span,
                    })?;
                    break Some(resolved);
                }
                if !self.units.contains_key(&target) {
                    return Err(Report::refused(
                        target_span,
                        format!("the unit `{target}` isn't known"),
                        "`=` points at a unit, in any family (D109)",
                    ));
                }
                if seen.contains(&target) {
                    let span = self.units.get(&name).map_or(target_span, |unit| unit.span);
                    return Err(Report::refused(
                        span,
                        format!("`{name}` never reaches a `base`"),
                        "loops and units with no base are refused (D109)",
                    ));
                }
                seen.push(target.clone());
                current = target;
            };
            if let Some(info) = self.units.get_mut(&name) {
                info.base = base;
            }
        }
        Ok(())
    }

    fn declare_type_contents(&mut self, items: &[&Item]) -> Outcome<()> {
        for item in items {
            match item {
                Item::Struct(declaration) => {
                    let (sources, _) =
                        self.sources_of(&Type::Struct(declaration.name.text.clone()));
                    self.allowed_sources.borrow_mut().clone_from(&sources);
                    // trace: D234, D239
                    let type_parameters =
                        self.read_type_parameters(&declaration.type_parameters)?;
                    self.generic.clone_from(&type_parameters);
                    let mut fields: Vec<(String, Type)> = Vec::new();
                    for field in &declaration.fields {
                        // trace: D57
                        if fields.iter().any(|(name, _)| *name == field.name.text) {
                            return Err(Report::refused(
                                field.name.span,
                                format!(
                                    "`{}` has two fields called `{}`",
                                    declaration.name.text, field.name.text
                                ),
                                "a field read is a plain `.`, so a field's name says which (D57)",
                            ));
                        }
                        let field_type = self.resolve_type(&field.written_type);
                        let field_type = match field_type {
                            Ok(found) => found,
                            Err(report) => {
                                self.generic.clear();
                                return Err(report);
                            }
                        };
                        // A struct holding a `must_Give` value is one too, and says so.
                        // trace: D177, D3
                        if self.is_must_give(&field_type) {
                            if !declaration.must_give {
                                return Err(Report::refused(
                                    field.written_type.span,
                                    format!(
                                        "`{}` is `{field_type}`, which is `must_Give`, so `{}` says so",
                                        field.name.text, declaration.name.text
                                    ),
                                    "a struct holding a `must_Give` value is `must_Give` itself, and nothing is implicit (D177, D3)",
                                )
                                .fix_elsewhere(format!("struct.must_Give {}", declaration.name.text)));
                            }
                        } else {
                            self.refuse_undecided_content(&field_type, field.written_type.span)?;
                        }
                        Self::check_borrow_in_a_field(&field_type, field.written_type.span)?;
                        fields.push((field.name.text.clone(), field_type));
                    }
                    self.allowed_sources.borrow_mut().clear();
                    let types: Vec<&Type> = fields.iter().map(|(_, found)| found).collect();
                    Self::refuse_unused_sources(&sources, &types, &declaration.sources)?;
                    self.generic.clear();
                    if let Some(info) = self.structs.get_mut(&declaration.name.text) {
                        info.fields = fields;
                        info.type_parameters = type_parameters;
                    }
                }
                Item::Union(declaration) => {
                    let (sources, _) = self.sources_of(&Type::Union(declaration.name.text.clone()));
                    self.allowed_sources.borrow_mut().clone_from(&sources);
                    let mut cases: Vec<(String, Vec<(String, Type)>)> = Vec::new();
                    for case in &declaration.cases {
                        // trace: D33
                        if cases.iter().any(|(name, _)| *name == case.name.text) {
                            return Err(Report::refused(
                                case.name.span,
                                format!(
                                    "`{}` has two cases called `{}`",
                                    declaration.name.text, case.name.text
                                ),
                                "a union's cases are narrowings, so a case's name says which (D33)",
                            ));
                        }
                        let mut fields = Vec::new();
                        for field in &case.fields {
                            let field_type = self.resolve_type(&field.written_type)?;
                            // trace: D176
                            self.refuse_undecided_content(&field_type, field.written_type.span)?;
                            Self::check_borrow_in_a_field(&field_type, field.written_type.span)?;
                            fields.push((field.name.text.clone(), field_type));
                        }
                        cases.push((case.name.text.clone(), fields));
                    }
                    self.allowed_sources.borrow_mut().clear();
                    let types: Vec<&Type> = cases
                        .iter()
                        .flat_map(|(_, fields)| fields.iter().map(|(_, found)| found))
                        .collect();
                    Self::refuse_unused_sources(&sources, &types, &declaration.sources)?;
                    if let Some(info) = self.unions.get_mut(&declaration.name.text) {
                        info.cases = cases;
                    }
                }
                _ => {}
            }
        }
        for item in items {
            if let Item::Struct(declaration) = item {
                self.refuse_struct_holding_itself(declaration)?;
            }
        }
        Ok(())
    }

    /// A `must_Give` value in a union case, or a stored borrow in a field, isn't decided yet.
    fn refuse_undecided_content(&self, content: &Type, span: Span) -> Outcome<()> {
        if self.is_must_give(content) {
            return Err(Report::not_designed(
                span,
                "a `must_Give` value in a `union.tagged` case",
                "12.8",
            ));
        }
        Ok(())
    }

    /// A field's borrow comes from one of its struct's sources, by name. (D171, D176)
    /// trace: D171, D176
    fn check_borrow_in_a_field(field_type: &Type, span: Span) -> Outcome<()> {
        let Type::Borrow {
            mutable, sources, ..
        } = field_type
        else {
            return Ok(());
        };
        if matches!(sources, Sources::Program) {
            return Err(Report::not_designed(
                span,
                "a field borrowing `from program`",
                "12.8",
            ));
        }
        if !matches!(sources, Sources::Named(_)) {
            return Err(Report::refused(
                span,
                "a borrow in a field names one of its type's sources, in backticks",
                "a struct names the sources of its borrows, as in ``Lend from `text` reading`` (D171)",
            ));
        }
        if *mutable {
            return Err(Report::not_designed(
                span,
                "a `Lend.mut` borrow inside a struct or union",
                "",
            ));
        }
        Ok(())
    }

    /// Every source is used by a field or case. (D171, D176)
    fn refuse_unused_sources(
        sources: &[String],
        types: &[&Type],
        declared: &[crate::ast::SourceDeclaration],
    ) -> Outcome<()> {
        for (source, declaration) in sources.iter().zip(declared) {
            if !types.iter().any(|found| uses_source(found, source)) {
                return Err(Report::refused(
                    declaration.name.span,
                    format!("`{source}` isn't used by any field"),
                    "each source is used by a field, or it says nothing (D171)",
                ));
            }
        }
        Ok(())
    }

    fn refuse_struct_holding_itself(
        &self,
        declaration: &crate::ast::StructDeclaration,
    ) -> Outcome<()> {
        let mut stack = vec![declaration.name.text.clone()];
        let mut seen = HashSet::new();
        while let Some(name) = stack.pop() {
            let Some(info) = self.struct_info(&name) else {
                continue;
            };
            for (_, field_type) in &info.fields {
                if let Type::Struct(inner) = field_type {
                    if *inner == declaration.name.text {
                        return Err(Report::not_designed(
                            declaration.name.span,
                            format!("`{inner}` holding itself directly"),
                            "",
                        ));
                    }
                    if seen.insert(inner.clone()) {
                        stack.push(inner.clone());
                    }
                }
            }
        }
        Ok(())
    }

    /// trace: D47
    fn declare_failures(&mut self, items: &[&Item]) -> Outcome<()> {
        for item in items {
            let Item::FailureSet(set) = item else {
                continue;
            };
            let mut members: Vec<Failure> = Vec::new();
            for member in &set.members {
                // trace: D47
                if members.iter().any(|other| other.full == member.path.text) {
                    return Err(Report::refused(
                        member.path.span,
                        format!("`{}` is in this set twice", member.path.text),
                        "a named set is a list of failures, each with its own name (D47)",
                    ));
                }
                let prefix = format!("{}.", set.name.text);
                let Some(short) = member.path.text.strip_prefix(&prefix) else {
                    return Err(Report::refused(
                        member.path.span,
                        format!("this failure is in the set `{}`", set.name.text),
                        "a named set's failures say the set's name (D47)",
                    )
                    .fix(format!("{}{}", prefix, member.path.text)));
                };
                let mut data = Vec::new();
                for field in &member.data {
                    data.push((
                        field.name.text.clone(),
                        self.resolve_type(&field.written_type)?,
                    ));
                }
                members.push(Failure {
                    short: short.to_string(),
                    full: member.path.text.clone(),
                    data,
                });
            }
            self.failure_sets.insert(set.name.text.clone(), members);
        }
        Ok(())
    }

    fn declare_handlers(&mut self, items: &[&Item]) {
        for item in items {
            let Item::HandlerSet(set) = item else {
                continue;
            };
            for handler in &set.handlers {
                self.handlers.insert(
                    (set.name.text.clone(), handler.name.text.clone()),
                    handler.arms.clone(),
                );
            }
        }
    }

    fn resolve_failure_list(&self, entries: &[FailureEntry]) -> Outcome<Vec<Failure>> {
        let mut failures: Vec<Failure> = Vec::new();
        let mut short_in_full: Vec<(String, bool)> = Vec::new();
        for entry in entries {
            let text = &entry.path.text;
            let mut found = Vec::new();
            let written_in_full = text.contains('.');
            if let Some(members) = self.failure_sets.get(text) {
                found.extend(members.iter().cloned());
            } else if let Some((set, short)) = text.split_once('.') {
                let member = self
                    .failure_sets
                    .get(set)
                    .and_then(|members| members.iter().find(|member| member.short == short));
                let Some(member) = member else {
                    return Err(Report::refused(
                        entry.path.span,
                        format!("`{text}` isn't in a set"),
                        "a failure with a full name is in a named set (D47)",
                    ));
                };
                found.push(member.clone());
            } else {
                let mut data = Vec::new();
                for field in &entry.data {
                    data.push((
                        field.name.text.clone(),
                        self.resolve_type(&field.written_type)?,
                    ));
                }
                found.push(Failure {
                    short: text.clone(),
                    full: text.clone(),
                    data,
                });
            }
            for failure in found {
                // trace: D47
                let clashes = failures
                    .iter()
                    .any(|other| other.short == failure.short && other.full != failure.full);
                let others_in_full = short_in_full
                    .iter()
                    .filter(|(short, _)| *short == failure.short)
                    .all(|(_, in_full)| *in_full);
                if clashes && !(written_in_full && others_in_full) {
                    return Err(Report::refused(
                        entry.path.span,
                        format!("two failures here are called `{}`", failure.short),
                        "two sets sharing a failure's name need the full name in a list (D47)",
                    ));
                }
                if !failures.iter().any(|other| other.full == failure.full) {
                    short_in_full.push((failure.short.clone(), written_in_full));
                    failures.push(failure);
                }
            }
        }
        Ok(failures)
    }

    #[allow(clippy::too_many_lines)] // Every check on a signature, top to bottom.
    fn declare_signatures(&mut self, functions: &[&Function]) -> Outcome<()> {
        for (index, function) in functions.iter().enumerate() {
            if self.signatures.contains_key(&function.name.text) {
                return Err(Report::not_designed(
                    function.name.span,
                    format!("declaring `{}` twice", function.name.text),
                    "",
                ));
            }
            // trace: D234
            let type_parameters = self.read_type_parameters(&function.type_parameters)?;
            self.generic.clone_from(&type_parameters);
            // trace: D171
            let (function_sources, function_facts) =
                Self::read_source_declarations(&function.sources)?;
            self.allowed_sources
                .borrow_mut()
                .clone_from(&function_sources);
            let mut parameters = Vec::new();
            for parameter in &function.parameters {
                let parameter_type = self.resolve_type(&parameter.written_type)?;
                // A parameter names the sources of a borrow inside it with the function's own.
                // trace: D171
                if let Type::Holding { fills, .. } = &parameter_type {
                    if fills.iter().any(|fill| !fill.starts_with('`')) {
                        return Err(Report::not_designed(
                            parameter.written_type.span,
                            "a parameter whose sources are filled with anything but the function's own",
                            "",
                        ));
                    }
                }
                // A bare parameter already is a borrow, and `Lent.mut` the changing one.
                // trace: D175
                if let Type::Borrow { mutable, .. } = &parameter_type {
                    let inner_text = match &parameter.written_type.kind {
                        TypeKind::StoredBorrow { inner, .. } => self.text_of(inner.span),
                        TypeKind::Named { .. } | TypeKind::Parameter(_) => "",
                    };
                    let fix = if *mutable {
                        format!("Lent.mut {inner_text}")
                    } else {
                        inner_text.to_string()
                    };
                    return Err(Report::refused(
                        parameter.written_type.span,
                        "a parameter isn't typed as a stored borrow",
                        "a bare parameter is already lent, and `Lent.mut` already is the changing one (D175)",
                    )
                    .fix(fix));
                }
                // A number takes `Lent.mut`, since that says it changes, and no other word.
                // trace: D2, D184
                if parameter.word == ParameterWord::Given && parameter_type.is_always_copied() {
                    return Err(Report::refused(
                        parameter.word_span,
                        format!("`{parameter_type}` is always copied, and takes no word"),
                        "plain numbers and `bool` are always copied, and take no word (D2, D123)",
                    ));
                }
                // trace: D86
                if parameter.word == ParameterWord::LentMut
                    && matches!(function.kind, FunctionKind::Pure | FunctionKind::PureBaked)
                {
                    return Err(Report::refused(
                        parameter.word_span,
                        "a pure function can't take `Lent.mut`",
                        "a pure function changes nothing outside itself (D86)",
                    ));
                }
                parameters.push(ParameterInfo {
                    name: parameter.name.text.clone(),
                    label: None,
                    word: parameter.word,
                    parameter_type,
                    default: None,
                    label_has_unit: false,
                });
            }
            let mut any_order = false;
            if let Some(labels) = &function.labels {
                any_order = labels.any_order;
                for (parameter, label) in &labels.entries {
                    let Some(info) = parameters
                        .iter_mut()
                        .find(|info| info.name == parameter.text)
                    else {
                        return Err(Report::refused(
                            parameter.span,
                            format!("`${}` isn't a parameter here", parameter.text),
                            "a label is declared for a parameter (D114)",
                        ));
                    };
                    // trace: D144
                    if let Some((_, unit_text)) = label.text.split_once('.') {
                        let unit = self.resolve_unit(&Path {
                            text: unit_text.to_string(),
                            span: label.span,
                        })?;
                        let parameter_unit = match &info.parameter_type {
                            Type::Integer { unit, .. } | Type::Float { unit, .. } => unit.as_ref(),
                            _ => None,
                        };
                        if parameter_unit != Some(&unit) {
                            return Err(Report::refused(
                                label.span,
                                format!(
                                    "this label says `{unit}`, and `${}` is `{}`",
                                    info.name, info.parameter_type
                                ),
                                "a label can carry the unit its parameter has (D144)",
                            ));
                        }
                        info.label_has_unit = true;
                    }
                    info.label = Some(label.text.clone());
                }
            }
            for (parameter, value) in &function.defaults {
                let Some(info) = parameters
                    .iter_mut()
                    .find(|info| info.name == parameter.text)
                else {
                    return Err(Report::refused(
                        parameter.span,
                        format!("`${}` isn't a parameter here", parameter.text),
                        "only labelled parameters can have defaults (D114)",
                    ));
                };
                // trace: D114
                if info.label.is_none() {
                    return Err(Report::refused(
                        parameter.span,
                        format!(
                            "`${}` has no label, so it can't have a default",
                            parameter.text
                        ),
                        "only labelled parameters can have defaults (D114)",
                    ));
                }
                let is_bool_literal = matches!(
                    &value.kind,
                    crate::ast::ExpressionKind::Path(path)
                        if path.text == "true" || path.text == "false"
                );
                if !is_bool_literal
                    && !matches!(
                        value.kind,
                        crate::ast::ExpressionKind::Integer { .. }
                            | crate::ast::ExpressionKind::Float { .. }
                            | crate::ast::ExpressionKind::Text(_)
                            | crate::ast::ExpressionKind::Empty
                            | crate::ast::ExpressionKind::WithUnit { .. }
                    )
                {
                    return Err(Report::not_designed(
                        value.span,
                        "a default that isn't a literal, and when it's worked out",
                        "",
                    ));
                }
                info.default = Some(value.clone());
            }
            self.check_method_declaration(function, &parameters)?;
            let return_type = self.resolve_type(&function.return_type)?;
            self.allowed_sources.borrow_mut().clear();
            Self::check_borrow_claim(&return_type, &parameters, function.return_type.span)?;
            Self::check_holding_claim(&return_type, &parameters, function.return_type.span)?;
            let lasts = Self::read_lasts(&function.lasts, &parameters)?;

            let failures = match &function.fails {
                Some(entries) => self.resolve_failure_list(entries)?,
                None => Vec::new(),
            };
            if function.kind == FunctionKind::Main {
                Self::check_main_signature(function, &return_type, !failures.is_empty())?;
            }
            if let Some(operator) = &function.operator {
                self.declare_operator(function, operator, &parameters, &return_type)?;
            }
            self.generic.clear();
            self.signatures.insert(
                function.name.text.clone(),
                Signature {
                    index,
                    name: function.name.text.clone(),
                    kind: function.kind,
                    type_parameters,
                    parameters,
                    any_order,
                    return_type,
                    failures,
                    sources: function_sources,
                    source_facts: function_facts,
                    lasts,
                },
            );
        }
        Ok(())
    }

    /// A method's first parameter is its receiver, which is its type and only borrows.
    /// trace: D130, D8
    fn check_method_declaration(
        &self,
        function: &Function,
        parameters: &[ParameterInfo],
    ) -> Outcome<()> {
        let Some((type_name, _)) = function.name.text.split_once(".%") else {
            return Ok(());
        };
        if let Some(found) = self.named.get(type_name) {
            if !matches!(found, Type::New { .. }) {
                return Err(Report::not_designed(
                    function.name.span,
                    format!("a method on `{type_name}`, which is another name for `{found}`"),
                    "",
                ));
            }
        }
        let Some(declared) = self.resolve_user_type(type_name) else {
            if is_clashing(type_name) || looks_like_number_type(type_name) {
                return Err(Report::not_designed(
                    function.name.span,
                    format!("a method on `{type_name}`, which isn't your own type"),
                    "",
                ));
            }
            return Err(Report::refused(
                function.name.span,
                format!("the type `{type_name}` isn't known"),
                "a method is declared with its type in the name (D130)",
            ));
        };
        let Some((receiver, written)) = parameters.first().zip(function.parameters.first()) else {
            return Err(Report::refused(
                function.name.span,
                format!("`{}` has no receiver", function.name.text),
                "a method's first parameter is its receiver (D130)",
            ));
        };
        if receiver.parameter_type != declared {
            return Err(Report::refused(
                written.written_type.span,
                format!("this method is on `{type_name}`, so its receiver is one"),
                "a method's first parameter is its receiver (D130)",
            ));
        }
        if receiver.word == ParameterWord::Given {
            return Err(Report::refused(
                written.word_span,
                "a method's receiver can't be `Given`",
                "methods only borrow, and anything that consumes its value is a plain function (D8, D130)",
            ));
        }
        if receiver.label.is_some() || receiver.default.is_some() {
            return Err(Report::refused(
                written.name.span,
                "a method's receiver is written before `.%`, so it has no label or default",
                "a method's first parameter is its receiver (D130)",
            ));
        }
        // `%%` in the name lends the receiver to change, and `%` lends it read-only. (D211)
        // trace: D211
        let changes = function.name.text.contains(".%%");
        let lent_mut = receiver.word == ParameterWord::LentMut;
        if changes != lent_mut {
            let (type_part, method) = function
                .name
                .text
                .split_once(".%")
                .unwrap_or((&function.name.text, ""));
            let method = method.trim_start_matches('%');
            let (what, fixed) = if lent_mut {
                (
                    "its receiver is `Lent.mut`, so its name says `%%`",
                    format!("{type_part}.%%{method}"),
                )
            } else {
                (
                    "its receiver is only lent to read, so its name says `%`",
                    format!("{type_part}.%{method}"),
                )
            };
            let mut report = Report::refused(
                function.name.span,
                format!("`{}`: {what}", function.name.text),
                "`%` lends a method's receiver read-only and `%%` lends it to change, and the declaration matches (D211)",
            );
            report = report.fix(fixed);
            return Err(report);
        }
        Ok(())
    }

    fn check_main_signature(
        function: &Function,
        return_type: &Type,
        has_failures: bool,
    ) -> Outcome<()> {
        // trace: D11
        if *return_type != Type::Void {
            return Err(Report::refused(
                function.return_type.span,
                "`fn.main` returns `void`",
                "a program starts at `fn.main`, and `main` says `return void;` (D11)",
            ));
        }
        if let Some(parameter) = function.parameters.first() {
            return Err(Report::not_designed(
                parameter.name.span,
                "how a program reads its arguments",
                "",
            ));
        }
        if has_failures {
            return Err(Report::not_designed(
                function.span,
                "`fn.main` listing failures",
                "12.6",
            ));
        }
        Ok(())
    }

    /// Constants can read each other in any order, so one is checked once those it reads are.
    /// trace: D83, D120
    fn check_globals(&mut self, items: &[&Item]) -> Outcome<()> {
        let mut pending: Vec<&Statement> = items
            .iter()
            .filter_map(|item| match item {
                Item::Constant(statement) => Some(statement),
                _ => None,
            })
            .collect();
        let names: Vec<String> = pending
            .iter()
            .filter_map(|statement| match &statement.kind {
                StatementKind::Constant { name, .. }
                | StatementKind::Inferred {
                    constant: true,
                    name,
                    ..
                } => Some(name.text.clone()),
                _ => None,
            })
            .collect();
        while !pending.is_empty() {
            let before = pending.len();
            let mut later = Vec::new();
            for statement in pending {
                let (StatementKind::Constant { value, .. }
                | StatementKind::Inferred {
                    constant: true,
                    value,
                    ..
                }) = &statement.kind
                else {
                    continue;
                };
                let mut reads = Vec::new();
                body::collect_variables(value, &mut reads);
                let waiting = reads
                    .iter()
                    .any(|(name, _)| names.contains(name) && !self.globals.contains_key(name));
                if waiting {
                    later.push(statement);
                } else {
                    self.check_global(statement)?;
                }
            }
            if later.len() == before {
                let span = later
                    .first()
                    .map_or(Span::new(0, 0), |statement| statement.span);
                return Err(Report::refused(
                    span,
                    "these constants read each other in a loop",
                    "a `const`'s value must be fully known at compile time (D83)",
                ));
            }
            pending = later;
        }
        Ok(())
    }

    fn check_global(&mut self, statement: &Statement) -> Outcome<()> {
        let (name, value_type, node) = match &statement.kind {
            StatementKind::Constant {
                name,
                written_type,
                value,
            } => {
                let value_type = self.resolve_type(written_type)?;
                self.context = FunctionContext::outside_any_function();
                let node = self.check_constant_value(value, &value_type)?;
                (name, value_type, node)
            }
            // trace: D216
            StatementKind::Inferred {
                constant: true,
                name,
                value,
                ..
            } => {
                self.context = FunctionContext::outside_any_function();
                let (node, value_type) = self.infer_value(value, true)?;
                (name, value_type, node)
            }
            _ => return Ok(()),
        };
        if self.globals.contains_key(&name.text) {
            return Err(Report::not_designed(
                name.span,
                format!("declaring `${}` twice", name.text),
                "",
            ));
        }
        let index = self.global_nodes.len();
        self.global_nodes.push(node.clone());
        self.compile_time.push(node.clone());
        self.globals.insert(
            name.text.clone(),
            GlobalInfo {
                index,
                value_type,
                node,
            },
        );
        Ok(())
    }

    /// Applies `use` to a path's first piece. (D93)
    fn canonical(&self, text: &str) -> String {
        let cut = text.find(['.', ':']).unwrap_or(text.len());
        let (first, rest) = text.split_at(cut);
        match self.aliases.get(first) {
            Some(full) => format!("{full}{rest}"),
            None => text.to_string(),
        }
    }

    fn is_must_give(&self, value_type: &Type) -> bool {
        match value_type {
            Type::Struct(name) => self.struct_info(name).is_some_and(|info| info.must_give),
            Type::Holding { base, .. } => self.is_must_give(base),
            Type::List { item, .. } | Type::Array { item, .. } => self.is_must_give(item),
            Type::New { underlying, .. } => self.is_must_give(underlying),
            _ => false,
        }
    }

    /// Resolves a unit to its one written form, such as `money.cents`. (D112)
    fn resolve_unit(&self, path: &Path) -> Outcome<String> {
        if path.text == "items" {
            return Ok("items".to_string());
        }
        let text = self.canonical(&path.text);
        if let Some(rest) = text.strip_prefix("std::") {
            self.require_std(rest, path.span)?;
            // trace: D199
            if rest == "units::time.seconds" {
                return Ok(text);
            }
            if let Some(old) = rest.strip_prefix("time.") {
                return Err(Report::refused(
                    path.span,
                    format!("`std::{rest}` is in `std`'s `units` group"),
                    "a unit family in `std` sits in the `units` group (D199, D196)",
                )
                .fix(format!("std::units::time.{old}")));
            }
            return Err(Report::not_designed(
                path.span,
                format!("the unit `{text}`, since the full list of units `std` has isn't chosen"),
                "12.2",
            ));
        }
        if text.contains("::") {
            return Err(Report::not_designed(
                path.span,
                format!("the unit `{text}`, from another kit"),
                "12.9",
            ));
        }
        if self.units.contains_key(&text) {
            return Ok(text);
        }
        // trace: D112
        if text.contains('.') {
            return Err(Report::refused(
                path.span,
                format!("the unit `{text}` isn't known"),
                "a unit is defined in a collection, and a unit from a kit has its path (D109, D112)",
            ));
        }
        Err(Report::refused(
            path.span,
            format!("the bare unit `{text}` doesn't say its family"),
            "a unit always says its family, or its path from a kit, unless `use` shortens it (D112)",
        ))
    }

    /// Names from `type.alias` and `type.new`, resolved once what they name is, in any order.
    /// trace: D134, D120
    fn declare_named_types(&mut self, items: &[&Item]) -> Outcome<()> {
        let mut pending: Vec<&TypeName> = Vec::new();
        let mut family_kinds: HashMap<String, bool> = HashMap::new();
        for item in items {
            match item {
                Item::TypeName(name) => pending.push(name),
                Item::TypeFamily(members) => {
                    for member in members {
                        if let Some(family) = &member.family {
                            let kind = family_kinds.entry(family.clone()).or_insert(member.new);
                            if *kind != member.new {
                                return Err(Report::refused(
                                    member.name.span,
                                    format!("`{family}` is already declared with the other kind"),
                                    "a family doesn't mix `type.alias` and `type.new` (D134)",
                                ));
                            }
                        }
                        pending.push(member);
                    }
                }
                _ => {}
            }
        }
        let names: Vec<String> = pending.iter().map(|name| name.name.text.clone()).collect();
        for (index, declared) in pending.iter().enumerate() {
            let text = &declared.name.text;
            if self.structs.contains_key(text)
                || self.unions.contains_key(text)
                || names.iter().take(index).any(|name| name == text)
            {
                return Err(Report::not_designed(
                    declared.name.span,
                    format!("declaring `{text}` twice"),
                    "",
                ));
            }
        }
        while !pending.is_empty() {
            let before = pending.len();
            let mut later = Vec::new();
            for declared in pending {
                let mut reads = Vec::new();
                self.collect_type_names(&declared.written_type, &mut reads);
                let waiting = reads
                    .iter()
                    .any(|read| names.contains(read) && !self.named.contains_key(read));
                if waiting {
                    later.push(declared);
                } else {
                    self.declare_named_type(declared)?;
                }
            }
            if later.len() == before {
                let first = later.first().copied();
                return Err(match first {
                    Some(TypeName {
                        new: true, name, ..
                    }) => Report::not_designed(
                        name.span,
                        format!("a `type.new` that holds itself, as `{}` does", name.text),
                        "",
                    ),
                    Some(TypeName { name, .. }) => Report::refused(
                        name.span,
                        format!("`{}` leads back to itself", name.text),
                        "`type.alias` gives a type another name, so it has to end at a type (D134)",
                    ),
                    None => Report::refused(Span::new(0, 0), "these types lead back", "(D134)"),
                });
            }
            pending = later;
        }
        Ok(())
    }

    fn declare_named_type(&mut self, declared: &TypeName) -> Outcome<()> {
        let target = self.resolve_type(&declared.written_type)?;
        if matches!(target, Type::Borrow { .. }) {
            return Err(Report::not_designed(
                declared.written_type.span,
                "another name for a stored borrow",
                "",
            ));
        }
        let text = declared.name.text.clone();
        let named = if declared.new {
            Type::New {
                name: text.clone(),
                underlying: Box::new(target),
            }
        } else {
            target
        };
        self.named.insert(text, named);
        Ok(())
    }

    fn collect_type_names(&self, written: &TypeExpression, out: &mut Vec<String>) {
        match &written.kind {
            TypeKind::Named { path, fill, .. } => {
                out.push(self.canonical(&path.text));
                for entry in fill {
                    self.collect_type_names(&entry.written_type, out);
                }
            }
            TypeKind::StoredBorrow { inner, .. } => self.collect_type_names(inner, out),
            TypeKind::Parameter(_) => {}
        }
    }

    /// Follows `type.alias` to the struct or union behind a name, in `name` or `name.case`,
    /// because an alias is fully interchangeable. (D134)
    pub(super) fn through_aliases(&self, text: &str) -> String {
        let declared = |found: Option<&Type>| match found {
            Some(Type::Struct(name) | Type::Union(name)) => Some(name.clone()),
            _ => None,
        };
        if let Some(name) = declared(self.named.get(text)) {
            return name;
        }
        if let Some((head, tail)) = text.rsplit_once('.') {
            if let Some(name) = declared(self.named.get(head)) {
                return format!("{name}.{tail}");
            }
        }
        text.to_string()
    }

    fn resolve_user_type(&self, name: &str) -> Option<Type> {
        if let Some(found) = self.named.get(name) {
            return Some(found.clone());
        }
        if self.structs.contains_key(name) {
            Some(Type::Struct(name.to_string()))
        } else if self.unions.contains_key(name) {
            Some(Type::Union(name.to_string()))
        } else {
            None
        }
    }

    /// A type, with a struct's or union's sources filled in when it holds borrows. (D171, D176)
    fn resolve_type(&self, written: &TypeExpression) -> Outcome<Type> {
        let plain = self.resolve_plain_type(written)?;
        let TypeKind::Named { sources: fills, .. } = &written.kind else {
            return Ok(plain);
        };
        let (declared, _) = self.sources_of(&plain);
        if declared.is_empty() && fills.is_empty() {
            return Ok(plain);
        }
        // trace: D171
        if declared.is_empty() {
            return Err(Report::refused(
                written.span,
                format!("`{plain}` holds no borrows, so it has no sources to fill"),
                "a struct that holds a borrow names its sources, and others have none (D171)",
            ));
        }
        let order = declared
            .iter()
            .map(|name| format!("`{name}`"))
            .collect::<Vec<_>>()
            .join(", then ");
        for (index, source) in declared.iter().enumerate() {
            let Some(fill) = fills.get(index) else {
                return Err(Report::refused(
                    written.span,
                    format!("`{plain}` has the sources {order}, and this leaves out `{source}`"),
                    "a type fills every source, in the declared order, and leaving one out is refused (D171)",
                ));
            };
            if fill.source.text != *source {
                return Err(Report::refused(
                    fill.source.span,
                    format!("`{plain}`'s sources go in their declared order: {order}"),
                    "a type fills every source, in the declared order (D171)",
                ));
            }
        }
        if let Some(extra) = fills.get(declared.len()) {
            return Err(Report::refused(
                extra.source.span,
                format!("`{plain}` has no source `{}`", extra.source.text),
                "a type fills the sources its struct declares (D171)",
            ));
        }
        let mut values = Vec::new();
        for fill in fills {
            values.push(match &fill.value {
                crate::ast::SourceValue::Variable(name) => format!("${}", name.text),
                crate::ast::SourceValue::Program => "program".to_string(),
                crate::ast::SourceValue::Source(name) => {
                    if !self.allowed_sources.borrow().contains(&name.text) {
                        return Err(Self::not_a_source_here(name));
                    }
                    format!("`{}`", name.text)
                }
            });
        }
        Ok(Type::Holding {
            base: Box::new(plain),
            sources: declared,
            fills: values,
        })
    }

    fn not_a_source_here(name: &crate::ast::Name) -> Box<Report> {
        Report::refused(
            name.span,
            format!("`{}` isn't a source here", name.text),
            "a source is declared in backticks on the struct, union or function it belongs to (D171)",
        )
    }

    #[allow(clippy::too_many_lines)] // One arm per way a type can be spelled.
    fn resolve_plain_type(&self, written: &TypeExpression) -> Outcome<Type> {
        let (path, fill, describe) = match &written.kind {
            // trace: D234
            TypeKind::Parameter(name) => {
                if self.generic.iter().any(|(found, _)| *found == name.text) {
                    return Ok(Type::Parameter(name.text.clone()));
                }
                return Err(Report::refused(
                    name.span,
                    format!("`?{}` isn't a type parameter here", name.text),
                    "a type parameter is declared in `< >` straight after its function's or struct's name (D234)",
                ));
            }
            TypeKind::StoredBorrow {
                mutable,
                held,
                sources,
                inner,
            } => {
                // trace: D169
                if *held {
                    return Err(Report::not_designed(
                        written.span,
                        "`held` on a borrow that isn't stored in a `let`",
                        "",
                    ));
                }
                let sources = match sources {
                    crate::ast::BorrowSources::Program => Sources::Program,
                    // trace: D171
                    crate::ast::BorrowSources::Source(name) => {
                        if !self.allowed_sources.borrow().contains(&name.text) {
                            return Err(Self::not_a_source_here(name));
                        }
                        Sources::Named(name.text.clone())
                    }
                    crate::ast::BorrowSources::Variables(names) => {
                        Sources::Variables(names.iter().map(|name| name.text.clone()).collect())
                    }
                };
                return Ok(Type::Borrow {
                    mutable: *mutable,
                    sources,
                    inner: Box::new(self.resolve_type(inner)?),
                });
            }
            TypeKind::Named {
                path,
                fill,
                describe,
                ..
            } => (path, fill, describe),
        };
        let text = self.canonical(&path.text);
        if let Some(mine) = text.strip_prefix("my::") {
            if !is_clashing(mine) {
                return Err(Report::not_designed(
                    path.span,
                    format!("`my::` on `{mine}`, which doesn't clash"),
                    "",
                ));
            }
            return self.resolve_user_type(mine).ok_or_else(|| {
                Report::refused(
                    path.span,
                    format!("the type `{mine}` isn't declared"),
                    "`my::` reaches a name of yours (D75)",
                )
            });
        }
        if text.starts_with('@') {
            return Err(Report::not_designed(
                path.span,
                format!("the limb type `{text}`"),
                "12.9",
            ));
        }
        if let Some(rest) = text.strip_prefix("std::") {
            self.require_std(rest, path.span)?;
            return self.resolve_std_type(rest, written, path, fill, describe);
        }
        // trace: D134
        if !is_clashing(&text) {
            if let Some(found) = self.named.get(&text) {
                if !fill.is_empty() || !describe.is_empty() {
                    return Err(Report::not_designed(
                        written.span,
                        format!("filling in or describing `{text}`"),
                        "12.5",
                    ));
                }
                return Ok(found.clone());
            }
            if let Some((head, _)) = text.split_once('.') {
                if !is_clashing(head) && self.named.contains_key(head) {
                    return Err(Report::not_designed(
                        path.span,
                        format!("a narrowing on `{head}`, a name from `type`"),
                        "",
                    ));
                }
            }
            // Sorted, so which one a message names can't depend on how a map is laid out.
            let mut family_members: Vec<&String> = self
                .named
                .keys()
                .filter(|name| name.rsplit('.').next() == Some(text.as_str()) && name.contains('.'))
                .collect();
            family_members.sort();
            if !family_members.is_empty() && self.resolve_user_type(&text).is_none() {
                let report = Report::refused(
                    path.span,
                    format!("`{text}` is in a family, so it's used with it"),
                    "a family's types are always used with the family (D134)",
                );
                return Err(match family_members.as_slice() {
                    [only] => report.fix((*only).clone()),
                    _ => report,
                });
            }
        }
        let (head, narrowing) = match text.split_once('.') {
            Some((head, narrowing)) => (head, Some(narrowing)),
            None => (text.as_str(), None),
        };
        match head {
            "void" | "bool" if narrowing.is_some() => {
                return Err(Report::not_designed(
                    path.span,
                    format!("the narrowing `{text}`"),
                    "",
                ));
            }
            "void" => return Ok(Type::Void),
            "bool" => return Ok(Type::Bool),
            "str" => {
                return match narrowing {
                    None => Ok(Type::Str),
                    // trace: D122
                    Some("with_unit") => Err(Report::not_designed(
                        path.span,
                        "how a unit prints with `as str.with_unit`",
                        "12.3",
                    )),
                    // trace: D51
                    Some("utf8") => Err(Report::refused(
                        path.span,
                        "`str.utf8` is the default",
                        "a default the language fixes can't be written out (D51)",
                    )
                    .fix("str")),
                    Some(other) => Err(Report::not_designed(
                        path.span,
                        format!("`str.{other}`, and how its string literals work"),
                        "12.2",
                    )),
                };
            }
            "i32" | "i64" | "u8" | "u32" | "u64" | "f32" | "f64" => {
                // trace: D54
                if narrowing == Some("decimal") {
                    return Err(Report::refused(
                        path.span,
                        format!("`{text}` doesn't exist"),
                        "a narrowing means \"this, made more specific\", so a decimal float is `d32`, not `f32.decimal` (D54)",
                    ));
                }
                // trace: D158
                if head.starts_with('f') {
                    if let Some(word) = narrowing {
                        let first = word.split('.').next().unwrap_or(word);
                        if let Some(rules) = FloatRules::from_narrowing(first) {
                            if first != word {
                                return Err(Report::not_designed(
                                    path.span,
                                    format!("`{first}` together with another narrowing"),
                                    "12.2",
                                ));
                            }
                            return Self::resolve_float_type(head, None, rules, describe, written);
                        }
                    }
                }
                let unit = match narrowing {
                    Some(unit_text) => Some(self.resolve_unit(&Path {
                        text: unit_text.to_string(),
                        span: path.span,
                    })?),
                    None => None,
                };
                return Self::resolve_number_type(head, unit, describe, written);
            }
            _ => {}
        }
        if narrowing.is_none() {
            // trace: D234
            if self
                .structs
                .get(head)
                .is_some_and(|info| !info.type_parameters.is_empty())
            {
                if !describe.is_empty() {
                    return Err(Report::not_designed(
                        written.span,
                        format!("describing `{head}`, a generic struct"),
                        "",
                    ));
                }
                return self.struct_instance(head, fill, written.span);
            }
            if let Some(found) = self.resolve_user_type(head) {
                if !fill.is_empty() || !describe.is_empty() {
                    return Err(Report::not_designed(
                        written.span,
                        format!("filling in or describing `{head}`, which generics would do"),
                        "12.5",
                    ));
                }
                return Ok(found);
            }
        } else if self.unions.get(head).is_some_and(|info| {
            info.cases
                .iter()
                .any(|(case, _)| Some(case.as_str()) == narrowing)
        }) {
            return Err(Report::not_designed(
                path.span,
                format!("`{text}` as a variable's type, outside the test"),
                "",
            ));
        }
        // trace: D10
        if text == "int.signed.32_bit" {
            return Err(Report::refused(
                path.span,
                "`int.signed.32_bit` doesn't exist",
                "a 32-bit signed integer is `i32`, with one spelling and no alias (D10)",
            )
            .fix("i32"));
        }
        // trace: D55
        if head == "d32" {
            return Err(Report::not_designed(
                path.span,
                "which limb `d32` comes from",
                "12.9",
            ));
        }
        if looks_like_number_type(head) {
            return Err(Report::not_designed(
                path.span,
                format!("the type `{head}`, since the full list isn't chosen"),
                "12.2",
            ));
        }
        Err(Report::refused(
            path.span,
            format!("the type `{text}` isn't known"),
            "a type is built in, or declared with `struct` or `union.tagged` (D33, D57)",
        ))
    }

    fn resolve_float_type(
        head: &str,
        unit: Option<String>,
        rules: FloatRules,
        describe: &[Describe],
        written: &TypeExpression,
    ) -> Outcome<Type> {
        let kind = if head == "f32" {
            FloatKind::F32
        } else {
            FloatKind::F64
        };
        let bound = Self::resolve_float_bound(kind, unit.as_ref(), rules, describe, written)?;
        Ok(Type::Float {
            kind,
            bound,
            unit,
            rules,
        })
    }

    /// `f64[min: -40.0, max: 125.0]`, whose numbers are ordinary float literals.
    /// trace: D158, D59
    fn resolve_float_bound(
        kind: FloatKind,
        unit: Option<&String>,
        rules: FloatRules,
        describe: &[Describe],
        written: &TypeExpression,
    ) -> Outcome<Option<(FloatBound, FloatBound)>> {
        if describe.is_empty() {
            return Ok(None);
        }
        if unit.is_some() {
            return Err(Report::not_designed(
                written.span,
                "a bound on a number with a unit",
                "",
            ));
        }
        if let Some(word) = rules.word() {
            return Err(Report::not_designed(
                written.span,
                format!("a bound on `{}.{word}`", kind.word()),
                "12.2",
            ));
        }
        let plain = Type::Float {
            kind,
            bound: None,
            unit: None,
            rules: FloatRules::Plain,
        };
        let mut low = None;
        let mut high = None;
        for entry in describe {
            let slot = match entry.label.text.as_str() {
                "min" => &mut low,
                "max" => &mut high,
                other => {
                    return Err(Report::refused(
                        entry.label.span,
                        format!("a float has no `{other}`"),
                        "`f64[min: -40.0, max: 125.0]` is still 64 bits, and holds only that range (D158)",
                    ));
                }
            };
            if matches!(entry.value.kind, crate::ast::ExpressionKind::Integer { .. }) {
                return Err(Report::not_designed(
                    entry.value.span,
                    "an integer literal where a float is needed",
                    "",
                ));
            }
            if !matches!(entry.value.kind, crate::ast::ExpressionKind::Float { .. }) {
                return Err(Report::not_designed(
                    entry.value.span,
                    "a bound that isn't a literal",
                    "",
                ));
            }
            let value = expression::float_literal_value(&entry.value, &plain)?;
            *slot = Some(value);
        }
        match (low, high) {
            (Some(low), Some(high)) => {
                if low > high {
                    return Err(Report::refused(
                        written.span,
                        "`min` is above `max`, so nothing fits",
                        "a bounded float holds only its range (D158)",
                    ));
                }
                Ok(Some((FloatBound::new(low), FloatBound::new(high))))
            }
            _ => Err(Report::not_designed(
                written.span,
                "a bound with only `min` or only `max`",
                "",
            )),
        }
    }

    fn resolve_number_type(
        head: &str,
        unit: Option<String>,
        describe: &[Describe],
        written: &TypeExpression,
    ) -> Outcome<Type> {
        if head == "f32" || head == "f64" {
            return Self::resolve_float_type(head, unit, FloatRules::Plain, describe, written);
        }
        let kind = match head {
            "i64" => IntegerKind::I64,
            "u8" => IntegerKind::U8,
            "u32" => IntegerKind::U32,
            "u64" => IntegerKind::U64,
            _ => IntegerKind::I32,
        };
        let bound = Self::resolve_bound(kind, unit.as_ref(), describe, written)?;
        Ok(Type::Integer { kind, bound, unit })
    }

    /// trace: D20
    fn resolve_bound(
        kind: IntegerKind,
        unit: Option<&String>,
        describe: &[Describe],
        written: &TypeExpression,
    ) -> Outcome<Option<(i128, i128)>> {
        if describe.is_empty() {
            return Ok(None);
        }
        if unit.is_some() {
            return Err(Report::not_designed(
                written.span,
                "a bound on a number with a unit",
                "",
            ));
        }
        let mut low = None;
        let mut high = None;
        for entry in describe {
            let Some(value) = expression::literal_integer(&entry.value) else {
                return Err(Report::not_designed(
                    entry.value.span,
                    "a bound that isn't a literal",
                    "",
                ));
            };
            match entry.label.text.as_str() {
                "min" => low = Some(value),
                "max" => high = Some(value),
                other => {
                    return Err(Report::refused(
                        entry.label.span,
                        format!("an integer has no `{other}`"),
                        "`i32[min: 0, max: 100]` is still 32 bits, and holds only that range (D20)",
                    ));
                }
            }
            if !kind.fits(value) {
                return Err(Report::refused(
                    entry.value.span,
                    format!("`{value}` doesn't fit `{}`", kind.word()),
                    "a literal that can't fit is a compile error (D20)",
                ));
            }
        }
        match (low, high) {
            (Some(low), Some(high)) => {
                if low > high {
                    return Err(Report::refused(
                        written.span,
                        "`min` is above `max`, so nothing fits",
                        "a bounded integer holds only its range (D20)",
                    ));
                }
                Ok(Some((low, high)))
            }
            _ => Err(Report::not_designed(
                written.span,
                "a bound with only `min` or only `max`",
                "",
            )),
        }
    }

    #[allow(clippy::too_many_lines)] // Lists, arrays and maps all read their labels here.
    fn resolve_std_type(
        &self,
        rest: &str,
        written: &TypeExpression,
        path: &Path,
        fill: &[Fill],
        describe: &[Describe],
    ) -> Outcome<Type> {
        match rest {
            "list" | "array" => {
                let [Fill {
                    label: None,
                    written_type,
                }] = fill
                else {
                    return Err(Report::refused(
                        written.span,
                        format!("`std::{rest}` fills in one type, bare, in `< >`"),
                        "a collection's types go in `< >`, and one type can go bare (D24, D25)",
                    ));
                };
                // A collection can hold `must_Give` items, and `for … in Give` hands them out.
                // trace: D180, D177
                let item = Box::new(self.resolve_type(written_type)?);
                let (mut len, mut min, mut max) = (None, None, None);
                // trace: D140
                let mut len_type = IntegerKind::U64;
                for entry in describe {
                    if entry.label.text == "len.type" {
                        len_type = Self::read_length_type(&entry.value)?;
                        continue;
                    }
                    let value = Self::read_item_count(&entry.value)?;
                    match entry.label.text.as_str() {
                        "len" => len = Some((value, entry)),
                        "len.min" => min = Some((value, entry)),
                        "len.max" => max = Some((value, entry)),
                        // trace: D13
                        "len.always" => {
                            return Err(Report::refused(
                                entry.label.span,
                                "`len.always` doesn't exist",
                                "bare `len` is exact, so there is no `len.always` (D13)",
                            )
                            .fix("len"));
                        }
                        other => {
                            return Err(Report::not_designed(
                                entry.label.span,
                                format!("the label `{other}` on `std::{rest}`"),
                                "12.2",
                            ));
                        }
                    }
                }
                if rest == "list" {
                    // trace: D13
                    if let Some((_, entry)) = len {
                        return Err(Report::refused(
                            entry.label.span,
                            "a list has no bare `len`, since exactly that many is an `array`",
                            "a list with a bare `len` is refused in favour of `array` (D13)",
                        ));
                    }
                    // trace: D11
                    let Some((max_value, max_entry)) = max else {
                        return Err(Report::refused(
                            path.span,
                            "a list always writes its `len.max`, with `inf` for unbounded",
                            "nothing goes unsaid about being empty or unbounded (D11)",
                        ));
                    };
                    let min_value = min.and_then(|(value, _)| value);
                    if let Some((None, entry)) = min {
                        return Err(Report::refused(
                            entry.value.span,
                            "`len.min` can't be `inf`",
                            "`len.min` is a floor (D13)",
                        ));
                    }
                    if let (Some(low), Some(high)) = (min_value, max_value) {
                        if low > high {
                            return Err(Report::refused(
                                max_entry.value.span,
                                "`len.min` is above `len.max`, so nothing fits",
                                "a list holds `len.min` to `len.max` items (D13)",
                            ));
                        }
                    }
                    Self::refuse_count_too_big(max_value, len_type, max_entry)?;
                    return Ok(Type::List {
                        item,
                        min: min_value,
                        max: max_value,
                        len_type,
                    });
                }
                if let Some((_, entry)) = min.or(max) {
                    return Err(Report::refused(
                        entry.label.span,
                        "an array has exactly `len` items",
                        "bare `len` is exact, and `len.min` and `len.max` are a list's (D13)",
                    ));
                }
                let Some((value, entry)) = len else {
                    return Err(Report::refused(
                        path.span,
                        "an array says its `len`",
                        "an array has exactly `len` items (D13)",
                    ));
                };
                let Some(value) = value else {
                    return Err(Report::refused(
                        entry.value.span,
                        "an array can't have `inf` items",
                        "an array has exactly `len` items (D13)",
                    ));
                };
                Self::refuse_count_too_big(Some(value), len_type, entry)?;
                Ok(Type::Array {
                    item,
                    len: value,
                    len_type,
                })
            }
            // trace: D138
            "map" => Err(Report::refused(
                path.span,
                "there's no bare `std::map`, so pick `std::map.hash` or `std::map.ordered`",
                "no default is obvious, so you pick (D48, D138)",
            )),
            "map.hash" | "map.ordered" => {
                let mut key = None;
                let mut value = None;
                for entry in fill {
                    match entry.label.as_ref().map(|label| label.text.as_str()) {
                        Some("key") => key = Some(self.resolve_type(&entry.written_type)?),
                        Some("val") => value = Some(self.resolve_type(&entry.written_type)?),
                        // trace: D25
                        _ => {
                            return Err(Report::refused(
                                entry.written_type.span,
                                "a map's types get the labels `key` and `val`",
                                "one type can go bare, and more than one must have labels (D25)",
                            ));
                        }
                    }
                }
                let (Some(key), Some(value)) = (key, value) else {
                    return Err(Report::refused(
                        written.span,
                        "a map fills in `key` and `val`",
                        "a map's types get labels (D24, D25)",
                    ));
                };
                Ok(Type::Map {
                    ordered: rest == "map.ordered",
                    key: Box::new(key),
                    value: Box::new(value),
                })
            }
            // trace: D24
            other if other.starts_with("list.") || other.starts_with("array.") => {
                Err(Report::refused(
                    path.span,
                    format!("`std::{other}` is the older spelling"),
                    "a collection's types go in `< >` (D24)",
                ))
            }
            other => Err(Report::not_designed(
                path.span,
                format!("the type `std::{other}`"),
                "12.2",
            )),
        }
    }

    /// `len.type: u32`, which says what `$xs.len` is and how the length is stored. (D140)
    fn read_length_type(value: &crate::ast::Expression) -> Outcome<IntegerKind> {
        let crate::ast::ExpressionKind::Path(path) = &value.kind else {
            return Err(Report::refused(
                value.span,
                "`len.type` is an integer type",
                "a list can choose its length's type with `len.type` (D140)",
            ));
        };
        match path.text.as_str() {
            "i32" => Ok(IntegerKind::I32),
            "i64" => Ok(IntegerKind::I64),
            "u8" => Ok(IntegerKind::U8),
            "u32" => Ok(IntegerKind::U32),
            "u64" => Ok(IntegerKind::U64),
            "str" | "bool" | "void" | "f32" | "f64" => Err(Report::refused(
                value.span,
                format!("`{}` can't count a length", path.text),
                "a list can choose its length's type with `len.type` (D140)",
            )),
            other => Err(Report::not_designed(
                value.span,
                format!("`len.type: {other}`"),
                "12.2",
            )),
        }
    }

    /// trace: D140
    fn refuse_count_too_big(
        count: Option<i128>,
        len_type: IntegerKind,
        entry: &Describe,
    ) -> Outcome<()> {
        if count.is_some_and(|count| !len_type.fits(count)) {
            return Err(Report::refused(
                entry.value.span,
                format!("`{}` can't count this many", len_type.word()),
                "a length that doesn't fit its `len.type` is refused (D140)",
            ));
        }
        Ok(())
    }

    /// `8 items` or `inf items`, where the unit is required. `None` is `inf`. (D101)
    fn read_item_count(value: &crate::ast::Expression) -> Outcome<Option<i128>> {
        use crate::ast::ExpressionKind;
        let ExpressionKind::WithUnit { number, unit } = &value.kind else {
            // trace: D101
            let shown = match &value.kind {
                ExpressionKind::Inf => "inf items".to_string(),
                ExpressionKind::Integer { digits, .. } => format!("{digits} items"),
                _ => {
                    return Err(Report::not_designed(
                        value.span,
                        "a length that isn't a literal",
                        "",
                    ));
                }
            };
            return Err(Report::refused(
                value.span,
                "a length says its unit",
                "the language asks for units on things that have them, like lengths (D101)",
            )
            .fix(shown));
        };
        if unit.text != "items" {
            return Err(Report::refused(
                unit.span,
                format!("a length is in `items`, not `{}`", unit.text),
                "the language asks for units on things that have them, like lengths (D101)",
            )
            .fix("items"));
        }
        match &number.kind {
            ExpressionKind::Inf => Ok(None),
            _ => match expression::literal_integer(number) {
                Some(count) if count >= 0 => Ok(Some(count)),
                _ => Err(Report::refused(
                    number.span,
                    "a length is a whole number of items",
                    "`len`, `len.min` and `len.max` count items (D13)",
                )),
            },
        }
    }

    fn mismatch(span: Span, expected: &Type, found: &Type) -> Box<Report> {
        // trace: D134
        if matches!(expected, Type::New { .. }) || matches!(found, Type::New { .. }) {
            return Report::refused(
                span,
                format!("this is `{found}`, where `{expected}` is needed"),
                "a `type.new` is a different type, so it doesn't mix with another, or with what it's made from (D134)",
            );
        }
        // trace: D100
        if let (
            Type::Integer {
                kind: left_kind,
                unit: left_unit,
                ..
            },
            Type::Integer {
                kind: right_kind,
                unit: right_unit,
                ..
            },
        ) = (expected, found)
        {
            if left_kind == right_kind && left_unit != right_unit {
                return Report::refused(
                    span,
                    format!("this is `{found}`, where `{expected}` is needed"),
                    "the compiler checks units, and mixing them is refused (D100, D101)",
                );
            }
        }
        Report::refused(
            span,
            format!("this is `{found}`, where `{expected}` is needed"),
            WHY_TYPES,
        )
    }

    /// The code as typed, for a `fix:`.
    fn text_of(&self, span: Span) -> &str {
        self.source.get(span.start..span.end).unwrap_or("")
    }
}

/// `i16`, `u32`, `f128` and such: number types nobody has chosen yet. (12.2)
fn looks_like_number_type(word: &str) -> bool {
    let mut characters = word.chars();
    matches!(characters.next(), Some('i' | 'u' | 'f' | 'd'))
        && !characters.as_str().is_empty()
        && characters.all(|character| character.is_ascii_digit())
}

/// Whether a type borrows from the named source somewhere in it. (D171)
fn uses_source(found: &Type, source: &str) -> bool {
    match found {
        Type::Borrow {
            sources: Sources::Named(name),
            ..
        } => name == source,
        Type::Holding { fills, .. } => fills.iter().any(|fill| *fill == format!("`{source}`")),
        Type::List { item, .. } | Type::Array { item, .. } => uses_source(item, source),
        _ => false,
    }
}

/// The function a `use` path narrows, as `std::io::fmt_print` in `std::io::fmt_print.stored`.
/// A unit's narrowing, like `std::units::time.seconds`, isn't a function's, and keeps D150. The
/// one function in `std` the oracle knows is the print (D194). Whether D220 covers a limb
/// function's narrowing waits on how C functions come through `@csup` (D225).
/// trace: D220
fn narrowed_function(path: &str) -> Option<&str> {
    let (function, _) = path.rsplit_once('.')?;
    (function == "std::io::fmt_print").then_some(function)
}

impl Checker<'_> {
    /// A type's own operator takes two values, at least one of them a struct or union, since
    /// an operator is never defined for built-in types. Its values are lent, as the operator
    /// shows no word. `==` and `<` give a `bool`, which the others follow from. (D236, D237)
    /// trace: D236, D237
    fn declare_operator(
        &mut self,
        function: &crate::ast::Function,
        operator: &str,
        parameters: &[ParameterInfo],
        return_type: &Type,
    ) -> Outcome<()> {
        let [left, right] = parameters else {
            return Err(Report::refused(
                function.name.span,
                format!("`op {operator}` takes two values"),
                "a type's own operator is a function named by it, taking the values on each side (D236)",
            ));
        };
        let own = |found: &Type| matches!(found, Type::Struct(_) | Type::Union(_));
        if !own(&left.parameter_type) && !own(&right.parameter_type) {
            return Err(Report::refused(
                function.name.span,
                format!(
                    "`op {operator}` here is for `{}` and `{}`, which are built in",
                    left.parameter_type, right.parameter_type
                ),
                "a struct or union can define an operator for itself, but an operator is never defined for, or changed on, built-in types (D236)",
            ));
        }
        if let Some(parameter) = function
            .parameters
            .iter()
            .find(|parameter| parameter.word != ParameterWord::Bare)
        {
            return Err(Report::not_designed(
                parameter.word_span,
                "an operator's value that's `Given` or `Lent.mut`, since the operator shows no word",
                "",
            ));
        }
        if matches!(operator, "==" | "<") && *return_type != Type::Bool {
            return Err(Report::refused(
                function.return_type.span,
                format!("`op {operator}` gives a `bool`"),
                "`!=`, `>`, `at_least` and `at_most` follow from `==` and `<`, so those compare and give a `bool` (D237)",
            ));
        }
        self.operators.push(OperatorInfo {
            operator: operator.to_string(),
            left: left.parameter_type.clone(),
            right: right.parameter_type.clone(),
            function: function.name.text.clone(),
            result: return_type.clone(),
        });
        Ok(())
    }
}
