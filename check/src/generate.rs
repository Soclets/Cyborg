//! Random programs, from the syntax tree in `tree.rs`. Every program is valid by the standard's
//! rules as written: typed `let`s, `=` for values, words for operators, spacing as 1.14 says,
//! loops that always end, and calls only to functions declared earlier, so nothing recurses.
//! What it doesn't generate is listed in `coverage.rs`, since all passing doesn't mean covered.

use std::fmt::Write as _;

use crate::random::Random;
use crate::tree::{
    Arguments, Arm, Comparison, Ending, Expression, FailureDecl, FailureFamily, Filling, Function,
    Generics, Handling, Kind, Layout, MethodDecl, NamedType, Operator, ParamKind, Position,
    Procedure, Program, Statement, StructDecl, Style, Tokens, Type, UnionDecl, UnitFamily,
};

/// How big a program gets.
pub struct Limits {
    pub functions: i128,
    pub statements: i128,
    pub depth: usize,
}

pub const LIMITS: Limits = Limits {
    functions: 3,
    statements: 8,
    depth: 3,
};

struct Scope {
    variables: Vec<(String, Type)>,
    /// Arrays, with their item type and length.
    arrays: Vec<(String, Kind, i128)>,
    /// Bounded lists, with their item type and `len.max`.
    lists: Vec<(String, Kind, i128)>,
    /// Struct variables, with which struct each is.
    records: Vec<(String, usize)>,
    /// Union variables, with which union each is.
    cased: Vec<(String, usize)>,
    /// Fields a `match` arm's name reads: the name, the field and its kind.
    case_fields: Vec<(String, String, Kind)>,
}

/// The float units a program declares: two bases, the length with prefixes, units made of them
/// with `per` and `x`, a base with an offset and a logarithmic unit, whose values are points or
/// spans (2.16).
#[derive(Clone, Copy)]
struct FloatUnits {
    length: &'static str,
    kilo: &'static str,
    milli: &'static str,
    time: &'static str,
    speed: &'static str,
    accel: &'static str,
    area: &'static str,
    kelvin: &'static str,
    celsius: &'static str,
    milliwatts: &'static str,
    decibels: &'static str,
}

/// What goes between a print's holes: plain text, braces, a tab, and every escape (1.13).
const TEXT_PIECES: [&str; 15] = [
    "",
    " ",
    ": ",
    "\\{",
    "\\}",
    "{tab}",
    "x=",
    "\\\\",
    "\\\"",
    "\\'",
    "\\r",
    "\\0",
    "\\x41",
    "\\u00e9",
    "\\U0001F642",
];

/// `std`'s radians, which every program can use, as it can degrees (2.16).
const RADIANS: &str = "std::units::angle.radians";

#[allow(clippy::struct_excessive_bools)] // Independent switches for what's being generated now.
struct Generator<'random> {
    random: &'random mut Random,
    next_name: usize,
    /// The failures the function being made lists, which `?` adds to; none in `main`.
    failing: Option<Vec<FailureDecl>>,
    named: Vec<NamedType>,
    structs: Vec<StructDecl>,
    unions: Vec<UnionDecl>,
    procedures: Vec<Procedure>,
    methods: Vec<MethodDecl>,
    unit_families: Vec<UnitFamily>,
    /// Generics, when the program declares them (5.8).
    generics: Option<Generics>,
    /// A `must_Give` struct, when the program declares one (6.9).
    tokens: Option<Tokens>,
    /// A named set of failures and its handlers, when the program declares one (8.3, 8.7).
    failure_family: Option<FailureFamily>,
    /// What the function being made returns, for an `otherwise` arm that returns (8.6).
    returns: Option<Type>,
    /// Names for other things: a `type.new` over the first struct, an alias for a bounded list,
    /// and one for a bounded string, with what each needs (2.18).
    wrap: Option<String>,
    list_alias: Option<(String, Kind, i128)>,
    text_alias: Option<(String, i128)>,
    named_others: Vec<(String, bool, String)>,
    /// The first struct, and the kind of `nest`'s list, when `nest` and `view` are declared
    /// (7.1).
    nest: Option<(String, Kind)>,
    /// The first struct and its first field, when `rwarm` is declared (6.10).
    warm: Option<(String, String)>,
    /// The kinds whose `trio_…` some loop calls, so it's declared (4.4).
    trios: Vec<Kind>,
    /// Whether `main` has `$pool` and `$events` (D346, D392).
    runners: bool,
    /// Whether `tag`, `tpick` and `tcount` are declared (2.9).
    strings: bool,
    /// Whether `main` has `$err` and `$buf` too (1.13).
    writers: bool,
    /// Whether the print being made is `.stored`, whose floats take no shapes (1.13).
    stored_print: bool,
    /// The kinds whose `fn.pure.baked` function something calls, so it's declared (5.11).
    baked: Vec<Kind>,
    /// The kinds whose list-view functions some call uses, so they're declared (2.10).
    views: Vec<Kind>,
    /// Whether the function being made has a heap of its own, so its lists can grow (2.10).
    has_heap: bool,
    /// `std`'s degrees as the program writes them: in full, or `degrees` after a `use` (1.11).
    degrees: &'static str,
    /// `use` lines, each a path and its short name (1.11).
    uses: Vec<(String, String)>,
    /// Whether the recursive functions are declared (5.1).
    recursion: bool,
    /// The names of the loops around what's being generated, innermost last (4.8).
    loop_names: Vec<String>,
    /// A C union, when the program declares one (9.6, D398).
    untagged: Option<(String, Vec<(String, Kind)>)>,
    /// Float units, when the program has them.
    float_units: Option<FloatUnits>,
    /// Every unit, with how many of its family's base it is.
    units: Vec<(&'static str, i128)>,
    /// Every constant's name, since only constants go into another.
    constants: Vec<String>,
    /// Struct variables handed over with `Give`, which are gone from then on.
    given_away: Vec<String>,
    /// Making `main`, where `$heap` and any `$arena` are, for lists that grow.
    in_main: bool,
    arena: Option<i128>,
    /// Lists that grow, with the allocator each takes memory from.
    growable: Vec<(String, String)>,
    /// Making a pure function, which calls only pure ones (5.9).
    pure_only: bool,
    /// Making a contract, which calls nothing here (5.5).
    no_calls: bool,
    functions: Vec<Function>,
}

/// A program from a seed: the same seed always gives the same program.
#[allow(clippy::too_many_lines)] // Each kind of declaration in turn, then `main`.
pub fn program(seed: u64) -> Program {
    let mut random = Random::new(seed);
    let mut generator = Generator {
        random: &mut random,
        next_name: 0,
        failing: None,
        named: Vec::new(),
        structs: Vec::new(),
        unions: Vec::new(),
        procedures: Vec::new(),
        methods: Vec::new(),
        unit_families: Vec::new(),
        generics: None,
        tokens: None,
        untagged: None,
        loop_names: Vec::new(),
        recursion: false,
        failure_family: None,
        returns: None,
        degrees: "std::units::angle.degrees",
        views: Vec::new(),
        baked: Vec::new(),
        has_heap: false,
        writers: false,
        strings: false,
        runners: false,
        trios: Vec::new(),
        warm: None,
        nest: None,
        wrap: None,
        list_alias: None,
        text_alias: None,
        named_others: Vec::new(),
        stored_print: false,
        uses: Vec::new(),
        float_units: None,
        units: Vec::new(),
        constants: Vec::new(),
        given_away: Vec::new(),
        in_main: false,
        arena: None,
        growable: Vec::new(),
        pure_only: false,
        no_calls: false,
        functions: Vec::new(),
    };
    generator.named_types();
    generator.unit_families();
    generator.float_unit_families();
    for _ in 0..generator.random.between(0, 2) {
        let declared = generator.struct_decl();
        generator.structs.push(declared);
    }
    for index in 0..generator.structs.len() {
        generator.method_decls(index);
    }
    generator.generics();
    generator.tokens();
    generator.untagged_union();
    generator.recursion = generator.random.chance(40);
    generator.failure_family();
    generator.writers = generator.random.chance(30);
    generator.other_named_types();
    generator.runners = generator.random.chance(40);
    if generator.random.chance(50) {
        let kind = generator.integer_kind();
        generator.nest = generator
            .structs
            .first()
            .map(|declared| (declared.name.clone(), kind));
    }
    if generator.random.chance(50) {
        generator.warm = generator.structs.first().and_then(|declared| {
            let (field, _) = declared.fields.first()?;
            Some((declared.name.clone(), field.clone()))
        });
    }
    generator.strings = generator.random.chance(50);
    // A unit from a kit is shortened with `use`, keeping its matching name (1.11, 2.16).
    if generator.random.chance(40) {
        generator.uses.push((
            "std::units::angle.degrees".to_string(),
            "degrees".to_string(),
        ));
        generator.degrees = "degrees";
    }
    if generator.random.chance(40) {
        generator.uses.push((
            "std::units::time.seconds".to_string(),
            "seconds".to_string(),
        ));
    }
    for _ in 0..generator.random.between(0, 2) {
        let declared = generator.union_decl();
        generator.unions.push(declared);
    }
    // Declarations work in any order, so functions are printed shuffled, and a call can reach
    // one declared later (D287).
    for at in (1..generator.functions.len()).rev() {
        let other = usize::try_from(generator.random.between(0, at as i128)).unwrap_or(0);
        generator.functions.swap(at, other);
    }
    for _ in 0..generator.random.between(0, 2) {
        let procedure = generator.procedure();
        generator.procedures.push(procedure);
    }
    let count = generator.random.between(0, LIMITS.functions);
    for _ in 0..count {
        let function = generator.function();
        generator.functions.push(function);
    }
    let mut scope = Scope {
        variables: Vec::new(),
        arrays: Vec::new(),
        lists: Vec::new(),
        records: Vec::new(),
        cased: Vec::new(),
        case_fields: Vec::new(),
    };
    // An arena with room for a few lists, so some programs run it out (D389).
    if generator.random.chance(40) {
        generator.arena = Some(generator.random.between(16, 256));
    }
    generator.in_main = true;
    let prelude = generator.float_prelude(&mut scope);
    let mut main = generator.block(&mut scope, LIMITS.statements, 0, (true, false));
    main.splice(0..0, prelude);
    // `main` ends by printing what it made, so a program's work shows in its output.
    for statement in main.clone() {
        if let Statement::Let {
            name,
            value_type:
                Type::Integer(_)
                | Type::Float(_)
                | Type::Bounded(..)
                | Type::OtherFloat(..)
                | Type::BoundedFloat(..),
            ..
        }
        | Statement::LetOtherwise { name, .. }
        | Statement::Task { name, .. } = statement
        {
            main.push(Statement::Print(Expression::Variable(name)));
        } else if let Statement::Let {
            name,
            value_type: Type::FloatUnit(_),
            ..
        } = statement
        {
            // A value in a unit prints with its unit (1.13).
            main.push(Statement::Format {
                writer: "out",
                stored: false,
                text: "{unit}".to_string(),
                values: vec![Expression::Variable(name)],
            });
        }
    }
    let fewer_brackets = generator.random.chance(50);
    Program {
        unit_families: generator.unit_families,
        generics: generator.generics,
        tokens: generator.tokens,
        untagged: generator.untagged,
        recursion: generator.recursion,
        failure_family: generator.failure_family,
        uses: generator.uses,
        views: generator.views,
        baked: generator.baked,
        writers: generator.writers,
        named_others: generator.named_others,
        runners: generator.runners,
        trios: generator.trios,
        warm: generator.warm,
        nest: generator.nest,
        strings: generator.strings,
        methods: generator.methods,
        arena: generator.arena,
        named: generator.named,
        structs: generator.structs,
        unions: generator.unions,
        procedures: generator.procedures,
        functions: generator.functions,
        main,
        fewer_brackets,
    }
}

impl Generator<'_> {
    /// With float units, `main` starts with a length, a time, points and spans, and angles, so
    /// the units made from them have something to be made from (2.16).
    fn float_prelude(&mut self, scope: &mut Scope) -> Vec<Statement> {
        let mut prelude = Vec::new();
        if let Some(units) = self.float_units {
            let leak = |text: String| -> &'static str { Box::leak(text.into_boxed_str()) };
            for unit in [
                units.length,
                units.time,
                leak(format!("{}.point", units.celsius)),
                leak(format!("{}.span", units.celsius)),
                leak(format!("{}.point", units.decibels)),
                leak(format!("{}.span", units.decibels)),
                self.degrees,
                RADIANS,
            ] {
                let name = self.name("fw");
                let value = self.float_unit_value(scope, unit, LIMITS.depth);
                scope.variables.push((name.clone(), Type::FloatUnit(unit)));
                prelude.push(Statement::Let {
                    name,
                    value_type: Type::FloatUnit(unit),
                    value,
                });
            }
        }
        prelude
    }

    /// A comparison chain of three values, usually all going one way, like `$x below 5 at_most
    /// 9`, and now and then changing direction, like `$a below $b above $c`, which means each
    /// comparison and stops at the first false (3.9). The first is a variable, so each value
    /// after it has a typed side to take its type from.
    fn chain(&mut self, scope: &Scope, depth: usize) -> Option<Expression> {
        let integers: Vec<(String, Type)> = scope
            .variables
            .iter()
            .filter(|(_, found)| matches!(found, Type::Integer(_)))
            .cloned()
            .collect();
        let (name, Type::Integer(kind)) = self.random.pick(&integers).cloned()? else {
            return None;
        };
        let directions: [&[Comparison]; 3] = [
            &[Comparison::Below, Comparison::AtMost],
            &[Comparison::Above, Comparison::AtLeast],
            &[Comparison::Equal],
        ];
        let direction = if self.random.chance(20) {
            &Comparison::ALL[..]
        } else {
            *self.random.pick(&directions)?
        };
        let comparisons = vec![*self.random.pick(direction)?, *self.random.pick(direction)?];
        let middle = self.integer(scope, kind, depth - 1);
        let last = self.integer(scope, kind, depth - 1);
        Some(Expression::Chain {
            values: vec![Expression::Variable(name), middle, last],
            comparisons,
        })
    }

    fn name(&mut self, prefix: &str) -> String {
        self.next_name += 1;
        format!("{prefix}{}", self.next_name)
    }

    fn integer_type(&mut self) -> Type {
        Type::Integer(*self.random.pick(&Kind::ALL).unwrap_or(&Kind::I32))
    }

    #[allow(clippy::too_many_lines)] // A function's parts in the order they're written.
    fn function(&mut self) -> Function {
        let name = self.name("fun");
        let count = self.random.between(1, 3);
        let mut scope = Scope {
            variables: Vec::new(),
            arrays: Vec::new(),
            lists: Vec::new(),
            records: Vec::new(),
            cased: Vec::new(),
            case_fields: Vec::new(),
        };
        let mut parameters = Vec::new();
        for _ in 0..count {
            let parameter = self.name("p");
            let mut parameter_type = self.integer_type();
            // Now and then a bounded parameter, which a call's value is checked against (2.5).
            if let (true, Type::Integer(kind)) = (self.random.chance(20), parameter_type) {
                parameter_type = self.bounded_type_of(kind);
            }
            scope.variables.push((parameter.clone(), parameter_type));
            parameters.push((parameter, parameter_type));
        }
        // The last few parameters may have labels, some with defaults (5.2, 5.4).
        let labelled = usize::try_from(self.random.between(0, count)).unwrap_or(0);
        let mut labels = Vec::new();
        let mut defaults = Vec::new();
        let first_labelled = parameters.len() - labelled;
        for (parameter, parameter_type) in parameters.iter_mut().skip(first_labelled) {
            // A label can carry a unit, so a plain number passed to it takes that unit (2.16).
            if let (Type::Integer(kind), Some((unit, _))) =
                (*parameter_type, self.random.pick(&self.units).copied())
            {
                // Two parameters can't share a label, so each unit labels one.
                let taken = labels
                    .iter()
                    .any(|(_, label)| *label == format!("at.{unit}"));
                if !taken && self.random.chance(30) {
                    *parameter_type = Type::Unit(kind, unit, None);
                    if let Some(entry) = scope
                        .variables
                        .iter_mut()
                        .find(|(name, _)| name == parameter)
                    {
                        entry.1 = *parameter_type;
                    }
                    labels.push((parameter.clone(), format!("at.{unit}")));
                    continue;
                }
            }
            labels.push((parameter.clone(), self.name("lb")));
            if let (true, Type::Integer(kind)) = (self.random.chance(40), parameter_type) {
                defaults.push((parameter.clone(), self.literal(*kind)));
            }
        }
        let any_order = labels.len() > 1 && self.random.chance(40);
        let result = self.integer_type();
        // Some functions are pure, which never fail here, since whether one may isn't decided,
        // and call only pure functions (5.9, 12.5).
        let pure = self.random.chance(25);
        self.pure_only = pure;
        // Contracts read the parameters and literals, and call only pure functions (5.5).
        let was_pure = self.pure_only;
        self.pure_only = true;
        let requires = self
            .random
            .chance(30)
            .then(|| self.expression(&scope, Type::Bool, 2));
        self.pure_only = was_pure;
        self.no_calls = true;
        let ensures = match (self.random.chance(30), result) {
            (true, Type::Integer(kind)) => {
                let comparison = *self
                    .random
                    .pick(&Comparison::ALL)
                    .unwrap_or(&Comparison::AtLeast);
                Some((comparison, self.literal(kind)))
            }
            _ => None,
        };
        self.no_calls = false;
        // Some functions can fail, each with a test that decides when (8.1, 8.2).
        let mut own = Vec::new();
        let mut body = Vec::new();
        if !pure && self.random.chance(40) {
            for _ in 0..self.random.between(1, 2) {
                let failure = self.name("bad");
                let carried = if self.random.chance(50) {
                    self.random.pick(&parameters).cloned()
                } else {
                    None
                };
                let data = carried.as_ref().and_then(|(_, found)| match found {
                    Type::Integer(kind) => Some(*kind),
                    Type::Bool
                    | Type::Float(_)
                    | Type::Bounded(..)
                    | Type::Text(_)
                    | Type::Simd(..)
                    | Type::Secret(_)
                    | Type::Unit(..)
                    | Type::FloatUnit(_)
                    | Type::OtherFloat(..)
                    | Type::BoundedFloat(..)
                    | Type::SecretBool => None,
                });
                let test = self.expression(&scope, Type::Bool, LIMITS.depth);
                body.push(Statement::If {
                    test,
                    then: vec![Statement::Fail {
                        name: failure.clone(),
                        data: data.and(carried.map(|(name, _)| Expression::Variable(name))),
                    }],
                    otherwise: Vec::new(),
                });
                own.push(FailureDecl {
                    name: failure,
                    data,
                    family: None,
                });
            }
        }
        // Now and then the function lists the named set, so it can fail with any of them, and
        // fails with one when a test says (8.1, 8.3).
        if let (false, true, Some(family)) =
            (pure, self.random.chance(30), self.failure_family.clone())
        {
            if let Some(member) = self.random.pick(&family.members).cloned() {
                let test = self.expression(&scope, Type::Bool, LIMITS.depth);
                let data = member.data.map(|kind| self.literal(kind));
                body.push(Statement::If {
                    test,
                    then: vec![Statement::Fail {
                        name: member.name,
                        data,
                    }],
                    otherwise: Vec::new(),
                });
            }
            own.extend(family.members);
        }
        self.returns = Some(result);
        self.failing = Some(own);
        // Now and then a heap of its own, so lists made here can grow (2.10).
        self.has_heap = !pure && self.random.chance(30);
        if self.has_heap {
            body.insert(0, Statement::Heap);
        }
        body.extend(self.block(&mut scope, 3, 1, (false, false)));
        self.has_heap = false;
        let returned = self.expression(&scope, result, LIMITS.depth);
        let failures = self.failing.take().unwrap_or_default();
        self.pure_only = false;
        self.returns = None;
        Function {
            name,
            parameters,
            labels,
            any_order,
            defaults,
            pure,
            requires,
            ensures,
            result,
            failures,
            body,
            returned,
        }
    }

    /// A block's statements. Only `main` prints, and only a loop's blocks `break`.
    fn block(
        &mut self,
        scope: &mut Scope,
        most: i128,
        depth: usize,
        (prints, in_loop): (bool, bool),
    ) -> Vec<Statement> {
        let declared_before = scope.variables.len();
        let arrays_before = scope.arrays.len();
        let lists_before = scope.lists.len();
        let records_before = scope.records.len();
        let cased_before = scope.cased.len();
        let case_fields_before = scope.case_fields.len();
        let count = self.random.between(1, most);
        let mut statements = Vec::new();
        for _ in 0..count {
            statements.push(self.statement(scope, depth, (prints, in_loop)));
        }
        // What a block declares ends with it.
        scope.variables.truncate(declared_before);
        scope.arrays.truncate(arrays_before);
        scope.lists.truncate(lists_before);
        scope.records.truncate(records_before);
        scope.cased.truncate(cased_before);
        scope.case_fields.truncate(case_fields_before);
        statements
    }

    #[allow(clippy::too_many_lines)] // One arm per kind of statement, read top to bottom.
    fn statement(
        &mut self,
        scope: &mut Scope,
        depth: usize,
        (prints, in_loop): (bool, bool),
    ) -> Statement {
        // 10 and 11 are for the kinds of statement added later, so they share fairly.
        let roll = self.random.between(0, 12);
        let integers: Vec<String> = scope
            .variables
            .iter()
            // A loop's counter, `$i…`, and a plain parameter, `$p…`, can't change (D184).
            .filter(|(name, value_type)| {
                matches!(
                    value_type,
                    Type::Integer(_)
                        | Type::Float(_)
                        | Type::Bounded(..)
                        | Type::Text(_)
                        | Type::Simd(..)
                        | Type::Secret(_)
                        | Type::Unit(_, _, None)
                        | Type::FloatUnit(_)
                ) && !name.starts_with('i')
                    && !name.starts_with('x')
                    && !name.starts_with('k')
                    && !name.starts_with('p')
            })
            .map(|(name, _)| name.clone())
            .collect();
        match roll {
            0..=3 if !self.unions.is_empty() && self.random.chance(15) => {
                self.union_statement(scope, 2, (prints, in_loop))
            }
            0..=3 if self.random.chance(8) => self.constant(scope),
            0..=3 if !self.units.is_empty() && self.random.chance(15) => self.unit_let(scope),
            0..=3 if self.float_units.is_some() && self.random.chance(35) => {
                self.float_unit_let(scope)
            }
            0..=3 if !self.structs.is_empty() && self.random.chance(15) => {
                self.record_statement(scope)
            }
            0..=3 if !self.named.is_empty() && self.random.chance(10) => self.named_let(scope),
            0..=3 if !self.named_others.is_empty() && self.random.chance(12) => self
                .other_named_let(scope, in_loop)
                .unwrap_or_else(|| self.list_statement(scope)),
            0..=3 if self.random.chance(10) => self.text_let(scope),
            0..=3 if self.random.chance(8) => {
                let value_type = Type::Secret(self.integer_kind());
                let value = self.expression(scope, value_type, LIMITS.depth);
                let name = self.name("sec");
                scope.variables.push((name.clone(), value_type));
                Statement::Let {
                    name,
                    value_type,
                    value,
                }
            }
            // The kinds of statement added later share 10 and 11 evenly.
            10 | 11 => match self.random.between(0, 22) {
                0 if prints => self
                    .reveal(scope)
                    .unwrap_or_else(|| self.list_statement(scope)),
                1 if !self.pure_only => self
                    .pointer(scope, prints)
                    .unwrap_or_else(|| self.list_statement(scope)),
                2 if !self.pure_only => self
                    .assembly(scope)
                    .unwrap_or_else(|| self.list_statement(scope)),
                3 if self.untagged.is_some() => self
                    .untagged_statement(scope)
                    .unwrap_or_else(|| self.list_statement(scope)),
                4 => self
                    .token(scope, depth)
                    .unwrap_or_else(|| self.list_statement(scope)),
                // Maps are made and ended; putting and getting isn't decided (2.12).
                5 if self.in_main => {
                    let key = if self.random.chance(30) {
                        "str".to_string()
                    } else {
                        self.integer_kind().word().to_string()
                    };
                    Statement::Map {
                        name: self.name("mp"),
                        kind: if self.random.chance(50) {
                            "hash"
                        } else {
                            "ordered"
                        },
                        key,
                        value: self.integer_kind(),
                    }
                }
                6 if prints && self.writers => Statement::Flush,
                7 if prints && self.strings => self.strings_statement(scope),
                // A string on the heap, from text or a number (2.9).
                8 if prints => {
                    // A number comes from a variable, whose type says what it is.
                    let numbers: Vec<String> = scope
                        .variables
                        .iter()
                        .filter(|(_, found)| matches!(found, Type::Integer(_)))
                        .map(|(name, _)| name.clone())
                        .collect();
                    // `empty` doesn't say what it's empty of here, so the text has something in it.
                    let from = match self.random.pick(&numbers).cloned() {
                        Some(name) if self.random.chance(50) => Expression::Variable(name),
                        _ => match self.text(scope, None) {
                            Expression::Text(text) if text.is_empty() => {
                                Expression::Text("o".to_string())
                            }
                            other => other,
                        },
                    };
                    Statement::HeapText {
                        name: self.name("hs"),
                        from,
                    }
                }
                9 => self
                    .simd_statement(scope)
                    .unwrap_or_else(|| self.list_statement(scope)),
                10 if !scope.arrays.is_empty() || !scope.lists.is_empty() => self
                    .chunks(scope)
                    .unwrap_or_else(|| self.list_statement(scope)),
                11 if !scope.lists.is_empty() => self
                    .list_view(scope)
                    .unwrap_or_else(|| self.list_statement(scope)),
                12 => self
                    .seconds(scope)
                    .unwrap_or_else(|| self.list_statement(scope)),
                13 => self
                    .generic_box(scope)
                    .unwrap_or_else(|| self.list_statement(scope)),
                14 if self.generics.is_some() => self
                    .generic_pair(scope)
                    .unwrap_or_else(|| self.list_statement(scope)),
                15 if prints => self
                    .split_tasks(scope)
                    .unwrap_or_else(|| self.list_statement(scope)),
                16 if !self.pure_only => self
                    .shared(scope)
                    .unwrap_or_else(|| self.list_statement(scope)),
                17 => self
                    .fresh_loop(scope)
                    .unwrap_or_else(|| self.list_statement(scope)),
                18 => self
                    .borrow_struct(scope)
                    .unwrap_or_else(|| self.list_statement(scope)),
                19 => self
                    .defer_statement(scope, prints)
                    .unwrap_or_else(|| self.list_statement(scope)),
                20 => self
                    .nest_statement(scope)
                    .unwrap_or_else(|| self.list_statement(scope)),
                21 => self
                    .secret_collections(scope)
                    .unwrap_or_else(|| self.list_statement(scope)),
                22 => {
                    let value = self.secret_bool(scope, 2);
                    let name = self.name("sb");
                    scope.variables.push((name.clone(), Type::SecretBool));
                    Statement::Let {
                        name,
                        value_type: Type::SecretBool,
                        value,
                    }
                }
                _ => self.list_statement(scope),
            },
            0..=3 if self.random.chance(15) => self
                .generic_struct(scope)
                .unwrap_or_else(|| self.list_statement(scope)),
            0..=3 if self.random.chance(10) => {
                let value_type = Type::Simd(self.integer_kind(), self.random.between(1, 5));
                let value = self.expression(scope, value_type, LIMITS.depth);
                let name = self.name("sv");
                scope.variables.push((name.clone(), value_type));
                Statement::Let {
                    name,
                    value_type,
                    value,
                }
            }
            0..=3 if self.random.chance(8) => self.declare(scope),
            0..=3 if prints && self.random.chance(10) => self
                .task(scope)
                .unwrap_or_else(|| self.list_statement(scope)),
            4 | 5 if prints && !self.procedures.is_empty() && self.random.chance(50) => self
                .call_procedure(scope, in_loop)
                .unwrap_or_else(|| self.list_statement(scope)),
            6 if depth < 2 && self.random.chance(25) => {
                self.test_statement(scope, depth, (prints, in_loop))
            }
            9 if depth < 2 && !scope.cased.is_empty() && self.random.chance(60) => {
                self.union_statement(scope, depth, (prints, in_loop))
            }
            0..=3 => {
                if self.random.chance(35) {
                    // The kind of a function that can fail, so the call has failures to handle.
                    let kinds: Vec<Kind> = self
                        .functions
                        .iter()
                        .filter(|function| !function.failures.is_empty())
                        .filter_map(|function| match function.result {
                            Type::Integer(kind) => Some(kind),
                            _ => None,
                        })
                        .collect();
                    if let Some(kind) = self.random.pick(&kinds).copied() {
                        for _ in 0..3 {
                            if let Some(statement) = self.let_otherwise(scope, kind, in_loop) {
                                return statement;
                            }
                        }
                    }
                }
                let value_type = match self.random.between(0, 99) {
                    0..=19 => Type::Bool,
                    20..=29 => Type::Float(self.random.chance(50)),
                    30..=32 => Type::OtherFloat(
                        self.random.chance(50),
                        if self.random.chance(50) {
                            "ieee"
                        } else {
                            "fast"
                        },
                    ),
                    33..=35 => {
                        let low = self.random.between(-50, 10);
                        Type::BoundedFloat(
                            self.random.chance(50),
                            low,
                            low + self.random.between(1, 100),
                        )
                    }
                    36..=47 => self.bounded_type(),
                    _ => self.integer_type(),
                };
                let ieee: Vec<String> = match value_type {
                    Type::Float(wide) => scope
                        .variables
                        .iter()
                        .filter(|(_, found)| *found == Type::OtherFloat(wide, "ieee"))
                        .map(|(name, _)| name.clone())
                        .collect(),
                    _ => Vec::new(),
                };
                let value = match value_type {
                    Type::Bounded(kind, low, high) => self.bounded_value(scope, kind, low, high),
                    // A `.ieee` float crosses into `.stable` with `as`, which fails on NaN or
                    // infinity (2.6).
                    Type::Float(_) if !ieee.is_empty() && self.random.chance(50) => {
                        Expression::Convert {
                            variable: self.random.pick(&ieee).cloned().unwrap_or_default(),
                            narrowing: "",
                            target: value_type,
                        }
                    }
                    _ => self.expression(scope, value_type, LIMITS.depth),
                };
                let name = self.name("v");
                scope.variables.push((name.clone(), value_type));
                Statement::Let {
                    name,
                    value_type,
                    value,
                }
            }
            4 | 5 if !integers.is_empty() => {
                let name = self.random.pick(&integers).cloned().unwrap_or_default();
                let value_type = scope
                    .variables
                    .iter()
                    .find(|(found, _)| *found == name)
                    .map_or(Type::Integer(Kind::I32), |(_, found)| *found);
                let mut value = self.expression(scope, value_type, LIMITS.depth);
                // A bare literal that can't fit a bounded integer is refused, so it's one that
                // fits; anything else is checked while running (2.5).
                // A negated literal, as in `-7`, is a literal too.
                let literal = match &value {
                    Expression::Integer(number) => Some(*number),
                    Expression::Negate(inner) => match inner.as_ref() {
                        Expression::Integer(number) => Some(-number),
                        _ => None,
                    },
                    _ => None,
                };
                if let (Type::Bounded(_, low, high), Some(number)) = (value_type, literal) {
                    if !(low..=high).contains(&number) {
                        value = Expression::Integer(self.random.between(low, high));
                    }
                }
                Statement::Change { name, value }
            }
            6 if depth < 2 => {
                let test = self.expression(scope, Type::Bool, LIMITS.depth);
                let mut then = self.block(scope, 3, depth + 1, (prints, in_loop));
                // A `break` ends a block, so nothing after it goes unreached (4.8).
                if in_loop && self.random.chance(30) {
                    // Now and then a named loop around, stopped with every loop inside it.
                    let named = self.random.pick(&self.loop_names.clone()).cloned();
                    match named {
                        Some(name) if self.random.chance(40) => {
                            then.push(Statement::BreakNamed(name));
                        }
                        _ => then.push(Statement::Break),
                    }
                }
                let otherwise = if self.random.chance(50) {
                    self.block(scope, 3, depth + 1, (prints, in_loop))
                } else {
                    Vec::new()
                };
                Statement::If {
                    test,
                    then,
                    otherwise,
                }
            }
            7 if depth < 2 => {
                let counter = self.name("i");
                let rounds = self.random.between(0, 5);
                scope
                    .variables
                    .push((counter.clone(), Type::Integer(Kind::I32)));
                let body = self.block(scope, 3, depth + 1, (prints, true));
                Statement::While {
                    counter,
                    rounds,
                    body,
                }
            }
            8 if depth < 2 => self.range_loop(scope, depth, prints),
            9 if depth < 2 && self.random.chance(30) => self.loop_over(scope, depth, prints),
            9 if self.random.chance(30) => self.list_statement(scope),
            9 if self.random.chance(50) => self.array_statement(scope),
            9 if self.random.chance(50) => self.record_statement(scope),
            9 => self.union_statement(scope, depth, (prints, in_loop)),
            _ if prints && self.random.chance(40) => self.format_print(scope),
            _ if prints => {
                // A print's value is an integer variable, so its type is never left for a
                // literal to guess, and how a `bool` prints isn't decided (12.3).
                let printable: Vec<String> = scope
                    .variables
                    .iter()
                    .filter(|(_, value_type)| {
                        matches!(
                            value_type,
                            Type::Integer(_) | Type::Float(_) | Type::Bounded(..)
                        )
                    })
                    .map(|(name, _)| name.clone())
                    .collect();
                if let Some(name) = self.random.pick(&printable).cloned() {
                    Statement::Print(Expression::Variable(name))
                } else {
                    {
                        let value_type = self.integer_type();
                        let value = self.expression(scope, value_type, LIMITS.depth);
                        let name = self.name("v");
                        scope.variables.push((name.clone(), value_type));
                        Statement::Let {
                            name,
                            value_type,
                            value,
                        }
                    }
                }
            }
            _ => {
                let value_type = self.integer_type();
                let value = self.expression(scope, value_type, LIMITS.depth);
                let name = self.name("v");
                scope.variables.push((name.clone(), value_type));
                Statement::Let {
                    name,
                    value_type,
                    value,
                }
            }
        }
    }

    /// `for` through an array, reading or changing each item, or `forever` with a counter that
    /// stops it. The array is out of reach in the body, which a `for` holds it for (4.4, 4.7).
    fn loop_over(&mut self, scope: &mut Scope, depth: usize, prints: bool) -> Statement {
        let collections: Vec<(String, Kind, i128)> =
            scope.arrays.iter().chain(&scope.lists).cloned().collect();
        let Some((array, kind, _)) = self.random.pick(&collections).cloned() else {
            let counter = self.name("i");
            let rounds = self.random.between(0, 4);
            scope
                .variables
                .push((counter.clone(), Type::Integer(Kind::I32)));
            let body = self.block(scope, 3, depth + 1, (prints, true));
            return Statement::Forever {
                counter,
                rounds,
                body,
            };
        };
        let changes = self.random.chance(50);
        // The item's name starts `x`, so it's never chosen for `change`; one lent to change is
        // changed once in the body instead.
        let item = self.name("x");
        // What the loop goes through is locked while it runs, so the body doesn't touch any.
        let held_arrays = std::mem::take(&mut scope.arrays);
        let held_lists = std::mem::take(&mut scope.lists);
        let held_variables = scope.variables.len();
        scope.variables.push((item.clone(), Type::Integer(kind)));
        let name = self.random.chance(30).then(|| self.name("lp"));
        self.loop_names.extend(name.clone());
        let mut body = self.block(scope, 3, depth + 1, (prints, true));
        self.break_named(scope, &mut body);
        if name.is_some() {
            self.loop_names.pop();
        }
        if changes {
            let value = self.expression(scope, Type::Integer(kind), LIMITS.depth);
            body.insert(
                0,
                Statement::Change {
                    name: item.clone(),
                    value,
                },
            );
        }
        scope.variables.truncate(held_variables);
        scope.arrays = held_arrays;
        scope.lists = held_lists;
        Statement::ForEach {
            name,
            item,
            array,
            changes,
            body,
        }
    }

    /// Now and then a loop's body ends with `if …: break name;`, naming this loop or one
    /// around it, which stops it and every loop inside (4.8).
    fn break_named(&mut self, scope: &Scope, body: &mut Vec<Statement>) {
        let Some(name) = self.random.pick(&self.loop_names.clone()).cloned() else {
            return;
        };
        if self.random.chance(50) {
            let test = self.boolean(scope, 2);
            body.push(Statement::If {
                test,
                then: vec![Statement::BreakNamed(name)],
                otherwise: Vec::new(),
            });
        }
    }

    /// A print of one to three variables, with plain, numbered or named holes, shapes that
    /// apply to each value's type, and text between them with escaped braces and tabs (1.13).
    fn format_print(&mut self, scope: &Scope) -> Statement {
        let printable: Vec<(String, Type)> = scope
            .variables
            .iter()
            .filter(|(_, found)| {
                matches!(
                    found,
                    Type::Integer(_)
                        | Type::Float(_)
                        | Type::Bounded(..)
                        | Type::Text(_)
                        | Type::Unit(..)
                        | Type::FloatUnit(_)
                )
            })
            .cloned()
            .collect();
        let style = self.random.between(0, 2);
        // `.stored` prints floats exactly, and whether `places:` or `form:` go with it isn't
        // decided, so its floats take no shapes (1.13).
        let stored = self.random.chance(10);
        self.stored_print = stored;
        // A named hole can read a struct's field, as in `{$r.celsius}` (1.13).
        let mut printable = printable;
        if style == 2 {
            for (record, index) in &scope.records {
                if self.given_away.contains(record) {
                    continue;
                }
                if let Some(declared) = self.structs.get(*index) {
                    for (field, kind) in &declared.fields {
                        printable.push((format!("{record}.{field}"), Type::Integer(*kind)));
                    }
                }
            }
        }
        let mut chosen = Vec::new();
        for _ in 0..self.random.between(1, 3) {
            if let Some(found) = self.random.pick(&printable).cloned() {
                // A string holding its bytes prints through a named hole here, since how one
                // goes in as a value isn't shown (1.13).
                if style != 2 && matches!(found.1, Type::Text(Some(_))) {
                    continue;
                }
                // A unit prints with the `unit` shape, which a numbered hole doesn't take here.
                if style == 1 && matches!(found.1, Type::Unit(..) | Type::FloatUnit(_)) {
                    continue;
                }
                chosen.push(found);
            }
        }
        let mut text = String::new();
        let mut values = Vec::new();
        let mut order: Vec<usize> = (0..chosen.len()).collect();
        if style == 1 {
            for at in (1..order.len()).rev() {
                let other = usize::try_from(self.random.between(0, at as i128)).unwrap_or(0);
                order.swap(at, other);
            }
        }
        for index in &order {
            let Some((name, value_type)) = chosen.get(*index).cloned() else {
                continue;
            };
            text.push_str(self.random.pick(&TEXT_PIECES).copied().unwrap_or(""));
            let shapes = self.shapes(value_type);
            match style {
                // Numbered holes take no shapes here, since whether they can isn't shown.
                1 => {
                    let _ = write!(text, "{{{}}}", index + 1);
                }
                2 => {
                    if shapes.is_empty() {
                        let _ = write!(text, "{{${name}}}");
                    } else {
                        let _ = write!(text, "{{${name}, {shapes}}}");
                    }
                }
                _ => {
                    let _ = write!(text, "{{{shapes}}}");
                }
            }
        }
        if style != 2 {
            values = chosen
                .into_iter()
                .map(|(name, _)| Expression::Variable(name))
                .collect();
        }
        // Most go to `$out`, and in a `main` with more writers, some to `$err` or `$buf` (1.13).
        let writer = if self.writers && self.in_main {
            *self
                .random
                .pick(&["out", "out", "err", "buf"])
                .unwrap_or(&"out")
        } else {
            "out"
        };
        self.stored_print = false;
        Statement::Format {
            writer,
            stored,
            text,
            values,
        }
    }

    /// A width's unit: columns, East Asian columns, graphemes or bytes, which all agree on what
    /// the generator prints. What a point is isn't decided (1.13, 12.2).
    fn width_unit(&mut self) -> &'static str {
        self.random
            .pick(&["cols", "cols", "cols.east_asian", "graphemes", "bytes"])
            .copied()
            .unwrap_or("cols")
    }

    /// Shapes that apply to a value of this type: a width with its unit and an alignment, a
    /// base with or without `zeros`, `sign` on a signed number, `places:` or `form: science` on
    /// a float, or none (1.13).
    fn shapes(&mut self, value_type: Type) -> String {
        let signed = match value_type {
            Type::Integer(kind) | Type::Bounded(kind, ..) => kind.range().0 < 0,
            _ => true,
        };
        let mut shapes: Vec<String> = Vec::new();
        let width = self.random.between(1, 12);
        match value_type {
            Type::Unit(..) | Type::FloatUnit(_) => return "unit".to_string(),
            Type::Text(_) => {
                if self.random.chance(50) {
                    let align = *self
                        .random
                        .pick(&["left", "right", "center"])
                        .unwrap_or(&"left");
                    shapes.push(format!("align: {align}"));
                    shapes.push(format!("width: {width} {}", self.width_unit()));
                }
                return shapes.join(", ");
            }
            Type::Float(_) if self.stored_print => {}
            Type::Float(_) => match self.random.between(0, 3) {
                0 => shapes.push(format!("places: {}", self.random.between(0, 4))),
                1 => {
                    shapes.push("form: science".to_string());
                    if self.random.chance(50) {
                        shapes.push(format!("places: {}", self.random.between(0, 3)));
                    }
                }
                2 => {}
                _ => {
                    let align = *self
                        .random
                        .pick(&["left", "right", "center"])
                        .unwrap_or(&"left");
                    shapes.push(format!("align: {align}"));
                    shapes.push(format!("width: {width} {}", self.width_unit()));
                }
            },
            _ => match self.random.between(0, 3) {
                0 => {
                    let base = *self
                        .random
                        .pick(&["bin", "oct", "hex", "hex.upper"])
                        .unwrap_or(&"hex");
                    shapes.push(format!("base: {base}"));
                    if self.random.chance(50) {
                        shapes.push("zeros".to_string());
                        shapes.push(format!("width: {width} {}", self.width_unit()));
                    }
                }
                1 => {
                    let align = *self
                        .random
                        .pick(&["left", "right", "center"])
                        .unwrap_or(&"left");
                    shapes.push(format!("align: {align}"));
                    shapes.push(format!("width: {width} {}", self.width_unit()));
                }
                2 => {}
                _ => {
                    if self.random.chance(50) {
                        shapes.push("zeros".to_string());
                        shapes.push(format!("width: {width} {}", self.width_unit()));
                    }
                }
            },
        }
        if signed && self.random.chance(20) {
            shapes.push("sign".to_string());
        }
        shapes.join(", ")
    }

    /// A secret value: a literal or plain value, which becomes secret where it goes, a secret
    /// variable, a choice by `.%pick` on two secrets compared, or two joined by an operator that
    /// takes the same time for every value (2.22).
    /// A `bool.secret`: two secrets compared, or ones joined with `and.both`, `or.both` or
    /// `not`, which always work out both sides (2.22).
    fn secret_bool(&mut self, scope: &Scope, depth: usize) -> Expression {
        let secrets: Vec<(String, Type)> = scope
            .variables
            .iter()
            .filter(|(_, found)| matches!(found, Type::Secret(_)))
            .cloned()
            .collect();
        if depth > 0 && self.random.chance(40) {
            if let Some(variable) = self.variable_of(scope, Type::SecretBool) {
                return match self.random.between(0, 2) {
                    0 => Expression::Not(Box::new(variable)),
                    roll => {
                        let other = self.secret_bool(scope, depth - 1);
                        Expression::Logic {
                            and: roll == 1,
                            both: true,
                            left: Box::new(variable),
                            right: Box::new(other),
                        }
                    }
                };
            }
        }
        if let Some((name, value_type)) = self.random.pick(&secrets).cloned() {
            let other = self
                .variable_of(scope, value_type)
                .unwrap_or_else(|| Expression::Variable(name.clone()));
            let comparison = *self
                .random
                .pick(&Comparison::ALL)
                .unwrap_or(&Comparison::Equal);
            return Expression::Compare {
                comparison,
                left: Box::new(Expression::Variable(name)),
                right: Box::new(other),
            };
        }
        self.variable_of(scope, Type::SecretBool)
            .unwrap_or(Expression::Bool(true))
    }

    /// Secrets in an array, from secrets here, one read back (2.22).
    fn secret_collections(&mut self, scope: &mut Scope) -> Option<Statement> {
        let secrets: Vec<(String, Kind)> = scope
            .variables
            .iter()
            .filter_map(|(name, found)| match found {
                Type::Secret(kind) => Some((name.clone(), *kind)),
                _ => None,
            })
            .collect();
        let (first, kind) = self.random.pick(&secrets)?.clone();
        let second = self.secret(scope, kind, 1);
        let items = vec![Expression::Variable(first), second];
        // A `simd` of secrets is decided, and the oracle doesn't run one yet (D286).
        let array = self.name("sa");
        let read = self.name("sec");
        let word = kind.word();
        scope.variables.push((read.clone(), Type::Secret(kind)));
        Some(Statement::Sequence(vec![
            Statement::LetNamed {
                name: array.clone(),
                written: format!("std::coll::array<{word}.secret>[len: 2 items]"),
                value: Expression::Lanes(items),
                back: None,
            },
            Statement::Let {
                name: read,
                value_type: Type::Secret(kind),
                value: Expression::Item {
                    array,
                    position: Position::Pos(2),
                },
            },
        ]))
    }

    fn secret(&mut self, scope: &Scope, kind: Kind, depth: usize) -> Expression {
        let value_type = Type::Secret(kind);
        let roll = self.random.between(0, 9);
        if depth == 0 || roll < 4 {
            if roll < 2 {
                if let Some(variable) = self.variable_of(scope, value_type) {
                    return variable;
                }
            }
            return self.integer(scope, kind, depth.min(1));
        }
        if roll == 4 {
            // A `bool.secret` here chooses, or a comparison of two secrets made in place.
            if let (Some(test), Some(if_true), Some(if_false)) = (
                self.variable_of(scope, Type::SecretBool),
                self.variable_of(scope, value_type),
                self.variable_of(scope, value_type),
            ) {
                return Expression::Pick {
                    test: Box::new(test),
                    if_true: Box::new(if_true),
                    if_false: Box::new(if_false),
                };
            }
            if let (Some(left), Some(right)) = (
                self.variable_of(scope, value_type),
                self.variable_of(scope, value_type),
            ) {
                let comparison = *self
                    .random
                    .pick(&Comparison::ALL)
                    .unwrap_or(&Comparison::Equal);
                // Each side is a secret variable, so `.%pick` has a type to give back.
                let if_true = self
                    .variable_of(scope, value_type)
                    .unwrap_or_else(|| left.clone());
                let if_false = self
                    .variable_of(scope, value_type)
                    .unwrap_or_else(|| right.clone());
                return Expression::Pick {
                    test: Box::new(Expression::Compare {
                        comparison,
                        left: Box::new(left),
                        right: Box::new(right),
                    }),
                    if_true: Box::new(if_true),
                    if_false: Box::new(if_false),
                };
            }
        }
        // A shift by a public amount takes the same time for every value (2.22).
        if roll == 5 {
            if let Some(variable) = self.variable_of(scope, value_type) {
                return Expression::Binary {
                    operator: Operator::ShiftRight,
                    left: Box::new(variable),
                    right: Box::new(Expression::Integer(self.random.between(0, kind.bits() - 1))),
                };
            }
        }
        let operator = *self
            .random
            .pick(&[
                Operator::AndBit,
                Operator::OrBit,
                Operator::XorBit,
                Operator::PlusWrap,
                Operator::MinusWrap,
                Operator::TimesWrap,
                Operator::PlusSaturate,
                Operator::MinusSaturate,
                Operator::TimesSaturate,
            ])
            .unwrap_or(&Operator::XorBit);
        // One side is a secret variable when there is one, so the maths is on a secret.
        let left = self
            .variable_of(scope, value_type)
            .unwrap_or_else(|| self.integer(scope, kind, 1));
        let right = self.secret(scope, kind, depth - 1);
        Expression::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// A shield going through a raw or volatile pointer to an integer variable, reading or
    /// writing it, made with `.of` and the word that says its access, and moved on and back
    /// with `.%moved`, whose unit is always written. `.at(address:)` points at hardware, which
    /// running a generated program can't have (9.8).
    fn pointer(&mut self, scope: &Scope, prints: bool) -> Option<Statement> {
        let changeable = |name: &String| {
            !name.starts_with('i')
                && !name.starts_with('p')
                && !name.starts_with('x')
                && !name.starts_with('k')
        };
        let integers: Vec<(String, Kind)> = scope
            .variables
            .iter()
            .filter_map(|(name, found)| match found {
                Type::Integer(kind) => Some((name.clone(), *kind)),
                _ => None,
            })
            .collect();
        let (target, kind) = self.random.pick(&integers)?.clone();
        // `Lend.mut` needs a variable that can change.
        let mutable = changeable(&target) && self.random.chance(70);
        let others: Vec<String> = integers
            .iter()
            .filter(|(name, found)| *found == kind && *name != target && changeable(name))
            .map(|(name, _)| name.clone())
            .collect();
        let read_into = self.random.pick(&others).cloned();
        if read_into.is_none() && !prints {
            return None;
        }
        let writes = (mutable && self.random.chance(70)).then(|| {
            let (low, high) = kind.range();
            self.random.between(low.max(-50), high.min(50))
        });
        let moves = self
            .random
            .chance(50)
            .then(|| (self.random.between(1, 3), self.name("q"), self.name("q")));
        Some(Statement::Pointer {
            shield: self.name("sh"),
            family: if self.random.chance(50) {
                "raw"
            } else {
                "volatile"
            },
            writes,
            kind,
            pointer: self.name("q"),
            target,
            moves,
            read_into,
            mutable,
        })
    }

    /// One aarch64 instruction on `i64` variables, in a shield in an `on.target aarch64:`
    /// branch, and the same sum in plain maths under `else:`, so a build for that chip and one
    /// for any other give the same value. The registers wrap, so the maths does too (9.4).
    fn assembly(&mut self, scope: &mut Scope) -> Option<Statement> {
        let wide: Vec<String> = scope
            .variables
            .iter()
            .filter(|(_, found)| *found == Type::Integer(Kind::I64))
            .map(|(name, _)| name.clone())
            .collect();
        let mut reads = Vec::new();
        for _ in 0..3 {
            reads.push(self.random.pick(&wide)?.clone());
        }
        let variable = |at: usize| Box::new(Expression::Variable(reads[at].clone()));
        let pair = |operator| Expression::Binary {
            operator,
            left: variable(0),
            right: variable(1),
        };
        let (instruction, otherwise) = match self.random.between(0, 6) {
            0 => ("add x0, x0, x1", pair(Operator::PlusWrap)),
            1 => ("sub x0, x0, x1", pair(Operator::MinusWrap)),
            2 => ("mul x0, x0, x1", pair(Operator::TimesWrap)),
            3 => ("and x0, x0, x1", pair(Operator::AndBit)),
            4 => ("orr x0, x0, x1", pair(Operator::OrBit)),
            5 => ("eor x0, x0, x1", pair(Operator::XorBit)),
            _ => (
                "madd x0, x0, x1, x2",
                Expression::Binary {
                    operator: Operator::PlusWrap,
                    left: Box::new(pair(Operator::TimesWrap)),
                    right: variable(2),
                },
            ),
        };
        if !instruction.contains("x2") {
            reads.truncate(2);
        }
        let result = self.name("k");
        scope
            .variables
            .push((result.clone(), Type::Integer(Kind::I64)));
        Some(Statement::Assembly {
            shield: self.name("sh"),
            result,
            instruction,
            reads,
            otherwise,
        })
    }

    /// Now and then generics: which integer types say they're `ordered`, and the first struct
    /// with its own operators, so it's `ordered` and `addable` too (3.10, 5.8).
    fn generics(&mut self) {
        if !self.random.chance(50) {
            return;
        }
        let ordered: Vec<Kind> = Kind::ALL
            .iter()
            .copied()
            .filter(|_| self.random.chance(60))
            .collect();
        let structure = self
            .structs
            .first()
            .filter(|declared| !declared.fields.is_empty())
            .cloned()
            .filter(|_| self.random.chance(70));
        let lines = ordered.len() + 2 + if structure.is_some() { 2 } else { 0 };
        let easter = usize::try_from(self.random.between(0, 3 * lines as i128)).unwrap_or(0);
        self.generics = Some(Generics {
            ordered,
            structure,
            easter,
            unit_parameter: self.float_units.is_some(),
        });
    }

    /// Now and then a C union with two to four integer fields (9.6, D398).
    fn untagged_union(&mut self) {
        if !self.random.chance(40) {
            return;
        }
        let name = self.name("cu");
        let mut fields = Vec::new();
        for _ in 0..self.random.between(2, 4) {
            let field = self.name("cf");
            fields.push((field, self.integer_kind()));
        }
        self.untagged = Some((name, fields));
    }

    /// A C union built with one field and `rest = all 0`, and read as any field, into a variable
    /// of that kind that can change (9.6, D398, D402).
    fn untagged_statement(&mut self, scope: &Scope) -> Option<Statement> {
        if self.pure_only || self.no_calls {
            return None;
        }
        let (union, fields) = self.untagged.clone()?;
        let (written, written_kind) = self.random.pick(&fields)?.clone();
        // `rest = all 0` zeroes what the written field doesn't cover, so any field reads (D402).
        let (read, read_kind) = self.random.pick(&fields)?.clone();
        let targets: Vec<String> = scope
            .variables
            .iter()
            .filter(|(name, found)| {
                *found == Type::Integer(read_kind)
                    && !name.starts_with('i')
                    && !name.starts_with('p')
                    && !name.starts_with('x')
                    && !name.starts_with('k')
            })
            .map(|(name, _)| name.clone())
            .collect();
        let read_into = self.random.pick(&targets)?.clone();
        let value = self.integer(scope, written_kind, LIMITS.depth);
        Some(Statement::Untagged {
            shield: self.name("sh"),
            name: self.name("cn"),
            union,
            written: (written, value),
            read,
            read_into,
        })
    }

    /// Now and then a `must_Give` struct with one to three integer fields, and its holder (6.9).
    fn tokens(&mut self) {
        if !self.random.chance(50) {
            return;
        }
        let name = self.name("tok");
        let count = self.random.between(1, 3);
        let mut fields = Vec::new();
        for _ in 0..count {
            let field = self.name("tf");
            fields.push((field, self.integer_kind()));
        }
        self.tokens = Some(Tokens {
            name,
            fields,
            holder: self.name("hold"),
        });
    }

    /// A `must_Give` value made and handed off on every path: ended straight away, by a
    /// `defer`, in both arms of an `if`, passed through a function first, or inside a holder
    /// that's closed. A field may be read on the way, which lends it and doesn't count (6.9,
    /// 6.12).
    fn token(&mut self, scope: &Scope, depth: usize) -> Option<Statement> {
        if self.pure_only || self.no_calls {
            return None;
        }
        let tokens = self.tokens.clone()?;
        let fields: Vec<(String, Expression)> = tokens
            .fields
            .iter()
            .map(|(field, kind)| (field.clone(), self.integer(scope, *kind, LIMITS.depth)))
            .collect();
        let union = self
            .random
            .pick(&scope.cased)
            .cloned()
            .and_then(|(variable, index)| Some((variable, self.unions.get(index)?.clone())));
        let ending = match self.random.between(0, 6) {
            6 if union.is_some() => {
                let (variable, declared) = union.clone()?;
                Ending::Matched {
                    variable,
                    union: declared.name,
                    cases: declared.cases.into_iter().map(|(case, _)| case).collect(),
                }
            }
            5 => Ending::Looped {
                count: self.random.between(2, 4),
                item: self.name("tk"),
            },
            0 => Ending::Direct,
            1 => Ending::Deferred,
            2 => Ending::Branches(self.boolean(scope, depth.min(2))),
            3 => {
                let kind = tokens.fields.first().map_or(Kind::I32, |(_, kind)| *kind);
                Ending::Passed {
                    made_from: self.integer(scope, kind, LIMITS.depth),
                    passed: self.name("tk"),
                }
            }
            _ => Ending::Held,
        };
        // `Looped` reads inside its loop, through the item.
        // A field read into a variable of its kind that can change.
        let read_into = if matches!(ending, Ending::Held) {
            None
        } else {
            let (field, kind) = self.random.pick(&tokens.fields)?.clone();
            let targets: Vec<String> = scope
                .variables
                .iter()
                .filter(|(name, found)| {
                    *found == Type::Integer(kind)
                        && !name.starts_with('i')
                        && !name.starts_with('p')
                        && !name.starts_with('x')
                        && !name.starts_with('k')
                })
                .map(|(name, _)| name.clone())
                .collect();
            self.random
                .pick(&targets)
                .cloned()
                .map(|target| (target, field))
        };
        Some(Statement::Token {
            name: self.name("tk"),
            struct_name: tokens.name,
            holder: tokens.holder,
            fields,
            ending,
            read_into,
        })
    }

    /// Whether a generic function can be called here: not from a pure function, since generic
    /// ones aren't pure, and not from a contract, which calls nothing.
    fn generic_calls(&self) -> Option<Generics> {
        if self.pure_only || self.no_calls {
            return None;
        }
        self.generics.clone()
    }

    /// An integer from a generic call: `gmax` on an `ordered` type, `gsum` on one in `whole`,
    /// `gtwice` on either of those, or `gkeep` on any. One argument is a variable, so the call
    /// knows its type from it, and numbers go bare (5.8, D394).
    fn generic_integer(&mut self, scope: &Scope, kind: Kind, depth: usize) -> Option<Expression> {
        let generics = self.generic_calls()?;
        let Some(Expression::Variable(name)) = self.variable_of(scope, Type::Integer(kind)) else {
            return None;
        };
        let whole = matches!(kind, Kind::I32 | Kind::I64);
        let ordered = generics.ordered.contains(&kind);
        let other = self.integer(scope, kind, depth.saturating_sub(1));
        let call = |function: &str, arguments: Vec<Expression>| Expression::Call {
            function: function.to_string(),
            arguments: arguments
                .into_iter()
                .map(|argument| (None, argument))
                .collect(),
            handling: Handling::Crash,
        };
        let variable = Expression::Variable(name);
        Some(match self.random.between(0, 3) {
            0 if ordered => call("gmax", vec![variable, other]),
            1 if whole => call("gsum", vec![variable, other]),
            2 if whole => call("gtwice", vec![variable]),
            _ => call("gkeep", vec![variable]),
        })
    }

    /// A `bool` from a generic call, `gsame`, on two integers of an `ordered` type or two of the
    /// struct lent, or from the struct's own `below` or `==` (3.10, 5.8).
    fn generic_boolean(&mut self, scope: &Scope, depth: usize) -> Option<Expression> {
        let generics = self.generic_calls()?;
        if let Some(structure) = &generics.structure {
            let index = self
                .structs
                .iter()
                .position(|declared| declared.name == structure.name)?;
            let live: Vec<String> = scope
                .records
                .iter()
                .filter(|(name, found)| *found == index && !self.given_away.contains(name))
                .map(|(name, _)| name.clone())
                .collect();
            if let (true, Some(left), Some(right)) = (
                self.random.chance(80),
                self.random.pick(&live).cloned(),
                self.random.pick(&live).cloned(),
            ) {
                return Some(match self.random.between(0, 2) {
                    0 => Expression::Call {
                        function: "gsame".to_string(),
                        arguments: vec![
                            (None, Expression::Lend(left)),
                            (None, Expression::Lend(right)),
                        ],
                        handling: Handling::Crash,
                    },
                    roll => Expression::Compare {
                        comparison: if roll == 1 {
                            Comparison::Below
                        } else {
                            Comparison::Equal
                        },
                        left: Box::new(Expression::Variable(left)),
                        right: Box::new(Expression::Variable(right)),
                    },
                });
            }
        }
        let kind = *self.random.pick(&generics.ordered)?;
        let variable = self.variable_of(scope, Type::Integer(kind))?;
        let other = self.integer(scope, kind, depth.saturating_sub(1));
        Some(Expression::Call {
            function: "gsame".to_string(),
            arguments: vec![(None, variable), (None, other)],
            handling: Handling::Crash,
        })
    }

    /// The struct with its own operators made from two of it: `gmax`, `gtwice` or its own `+`
    /// (3.10, 5.8).
    fn generic_struct(&mut self, scope: &mut Scope) -> Option<Statement> {
        let generics = self.generic_calls()?;
        let structure = generics.structure?;
        let index = self
            .structs
            .iter()
            .position(|declared| declared.name == structure.name)?;
        let live: Vec<String> = scope
            .records
            .iter()
            .filter(|(name, found)| *found == index && !self.given_away.contains(name))
            .map(|(name, _)| name.clone())
            .collect();
        let left = self.random.pick(&live)?.clone();
        let right = self.random.pick(&live)?.clone();
        let value = match self.random.between(0, 3) {
            3 => {
                let kind = structure
                    .fields
                    .first()
                    .map_or(Kind::I32, |(_, kind)| *kind);
                Expression::Binary {
                    operator: Operator::DivideFloor,
                    left: Box::new(Expression::Variable(left)),
                    right: Box::new(self.integer(scope, kind, 1)),
                }
            }
            0 => Expression::Call {
                function: "gmax".to_string(),
                arguments: vec![
                    (None, Expression::Lend(left)),
                    (None, Expression::Lend(right)),
                ],
                handling: Handling::Crash,
            },
            1 => Expression::Call {
                function: "gtwice".to_string(),
                arguments: vec![(None, Expression::Lend(left))],
                handling: Handling::Crash,
            },
            _ => Expression::Binary {
                operator: *self
                    .random
                    .pick(&[Operator::Add, Operator::Subtract, Operator::Multiply])
                    .unwrap_or(&Operator::Add),
                left: Box::new(Expression::Variable(left)),
                right: Box::new(Expression::Variable(right)),
            },
        };
        let name = self.name("r");
        scope.records.push((name.clone(), index));
        Some(Statement::LetStructValue {
            name,
            struct_name: structure.name,
            value,
        })
    }

    /// A `gpair<…, …>` holding an integer and an integer or a struct, given to `gfirst`, whose
    /// first goes into a variable that can change (5.8).
    fn generic_pair(&mut self, scope: &Scope) -> Option<Statement> {
        self.generic_calls()?;
        let targets: Vec<(String, Kind)> = scope
            .variables
            .iter()
            .filter_map(|(name, found)| match found {
                Type::Integer(kind)
                    if !name.starts_with('i')
                        && !name.starts_with('p')
                        && !name.starts_with('x')
                        && !name.starts_with('k') =>
                {
                    Some((name.clone(), *kind))
                }
                _ => None,
            })
            .collect();
        let (read_into, left) = self.random.pick(&targets)?.clone();
        let first = self.integer(scope, left, LIMITS.depth);
        let (right, second) = match self.structs.first().cloned() {
            Some(declared) if self.random.chance(60) => {
                let fields = declared
                    .fields
                    .iter()
                    .map(|(field, kind)| {
                        (
                            field.clone(),
                            self.field_value(scope, &declared, field, *kind),
                        )
                    })
                    .collect();
                (
                    declared.name.clone(),
                    Expression::Build {
                        struct_name: declared.name,
                        fields,
                    },
                )
            }
            _ => {
                let kind = self.integer_kind();
                (kind.word().to_string(), self.literal(kind))
            }
        };
        Some(Statement::GenericPair {
            name: self.name("gp"),
            left,
            right,
            first,
            second,
            read_into,
        })
    }

    /// A `gbox<…>` holding an integer, its `held` read back into a variable that can change
    /// (5.8).
    fn generic_box(&mut self, scope: &Scope) -> Option<Statement> {
        self.generic_calls()?;
        let targets: Vec<(String, Kind)> = scope
            .variables
            .iter()
            .filter_map(|(name, found)| match found {
                Type::Integer(kind)
                    if !name.starts_with('i')
                        && !name.starts_with('p')
                        && !name.starts_with('x')
                        && !name.starts_with('k') =>
                {
                    Some((name.clone(), *kind))
                }
                _ => None,
            })
            .collect();
        let (read_into, kind) = self.random.pick(&targets)?.clone();
        let value = self.integer(scope, kind, LIMITS.depth);
        Some(Statement::GenericBox {
            name: self.name("gb"),
            kind,
            value,
            read_into,
        })
    }

    /// `decl`, or `decl mut`, given its value once on each path: in both arms of an `if`, in
    /// every arm of a `match` on a union, or straight after. One without `mut` is named `k…`,
    /// so nothing changes it after (2.19).
    fn declare(&mut self, scope: &mut Scope) -> Statement {
        let kind = self.integer_kind();
        let mutable = self.random.chance(50);
        let name = self.name(if mutable { "d" } else { "k" });
        let union = self
            .random
            .pick(&scope.cased)
            .cloned()
            .and_then(|(variable, index)| Some((variable, self.unions.get(index)?.clone())));
        let statement = match (self.random.between(0, 2), union) {
            (0, Some((variable, declared))) => {
                let mut arms = Vec::new();
                for (case, _) in declared.cases {
                    let value = self.expression(scope, Type::Integer(kind), LIMITS.depth);
                    arms.push((
                        case,
                        None,
                        vec![Statement::Assign {
                            name: name.clone(),
                            value,
                        }],
                    ));
                }
                Statement::Sequence(vec![
                    Statement::Decl {
                        name: name.clone(),
                        mutable,
                        kind,
                    },
                    Statement::Match {
                        variable,
                        union_name: declared.name,
                        arms,
                    },
                ])
            }
            (1, _) => {
                let value = self.expression(scope, Type::Integer(kind), LIMITS.depth);
                Statement::Sequence(vec![
                    Statement::Decl {
                        name: name.clone(),
                        mutable,
                        kind,
                    },
                    Statement::Assign {
                        name: name.clone(),
                        value,
                    },
                ])
            }
            _ => {
                let test = self.expression(scope, Type::Bool, LIMITS.depth);
                let then_value = self.expression(scope, Type::Integer(kind), LIMITS.depth);
                let else_value = self.expression(scope, Type::Integer(kind), LIMITS.depth);
                Statement::DeclAssign {
                    name: name.clone(),
                    mutable,
                    kind,
                    test,
                    then_value,
                    else_value,
                }
            }
        };
        scope.variables.push((name, Type::Integer(kind)));
        statement
    }

    /// `std`'s seconds as integer points and spans: a point and a span, the point moved by the
    /// span, the distance between the two points, taken off its unit into a variable that can
    /// change (2.16).
    fn seconds(&mut self, scope: &Scope) -> Option<Statement> {
        let seconds = if self
            .uses
            .iter()
            .any(|(path, _)| path == "std::units::time.seconds")
        {
            "seconds"
        } else {
            "std::units::time.seconds"
        };
        let targets: Vec<String> = scope
            .variables
            .iter()
            .filter(|(name, found)| {
                *found == Type::Integer(Kind::I64)
                    && !name.starts_with('i')
                    && !name.starts_with('p')
                    && !name.starts_with('x')
                    && !name.starts_with('k')
            })
            .map(|(name, _)| name.clone())
            .collect();
        let target = self.random.pick(&targets)?.clone();
        let leak = |text: String| -> &'static str { Box::leak(text.into_boxed_str()) };
        let point = Type::Unit(Kind::I64, leak(format!("{seconds}.point")), None);
        let span = Type::Unit(Kind::I64, leak(format!("{seconds}.span")), None);
        let [at, by, later, between] = [
            self.name("ts"),
            self.name("ts"),
            self.name("ts"),
            self.name("ts"),
        ];
        let literal = |number| Expression::WithUnit {
            number,
            unit: seconds,
        };
        let variable = |name: &String| Box::new(Expression::Variable(name.clone()));
        Some(Statement::Sequence(vec![
            Statement::Let {
                name: at.clone(),
                value_type: point,
                value: literal(self.random.between(0, 100_000)),
            },
            Statement::Let {
                name: by.clone(),
                value_type: span,
                value: literal(self.random.between(0, 3600)),
            },
            Statement::Let {
                name: later.clone(),
                value_type: point,
                value: Expression::Binary {
                    operator: Operator::Add,
                    left: variable(&at),
                    right: variable(&by),
                },
            },
            Statement::Let {
                name: between.clone(),
                value_type: span,
                value: Expression::Binary {
                    operator: Operator::Subtract,
                    left: variable(&later),
                    right: variable(&at),
                },
            },
            Statement::Change {
                name: target,
                value: Expression::Convert {
                    variable: between,
                    narrowing: "",
                    target: Type::Integer(Kind::I64),
                },
            },
        ]))
    }

    /// A list read through a view: summed by `vsum_…`, which takes any list, changed by
    /// `vbump_…`, which takes a `.mut` view, split into halves read one by one, or its `.len`
    /// taken off its unit (2.10, D379, D382).
    fn list_view(&mut self, scope: &Scope) -> Option<Statement> {
        if self.pure_only || self.no_calls {
            return None;
        }
        let (list, kind, _) = self.random.pick(&scope.lists)?.clone();
        let changeable = |kind: Kind| -> Vec<String> {
            scope
                .variables
                .iter()
                .filter(|(name, found)| {
                    *found == Type::Integer(kind)
                        && !name.starts_with('i')
                        && !name.starts_with('p')
                        && !name.starts_with('x')
                        && !name.starts_with('k')
                })
                .map(|(name, _)| name.clone())
                .collect()
        };
        let call = |function: &str, argument: Expression| Expression::Call {
            function: format!("{function}_{}", kind.word()),
            arguments: vec![(None, argument)],
            handling: Handling::Crash,
        };
        if !self.views.contains(&kind) {
            self.views.push(kind);
        }
        let statement = match self.random.between(0, 3) {
            0 => Statement::Call(call("vbump", Expression::LendMut(list))),
            1 => {
                let target = self.random.pick(&changeable(kind))?.clone();
                let halves = self.name("h");
                let half = |side: &str| Expression::Field {
                    variable: halves.clone(),
                    field: side.to_string(),
                };
                Statement::Sequence(vec![
                    Statement::Split {
                        name: halves.clone(),
                        kind,
                        list,
                        at_end: self.random.chance(50),
                    },
                    Statement::Change {
                        name: target,
                        value: Expression::Binary {
                            operator: Operator::PlusWrap,
                            left: Box::new(call("vsum", half("first"))),
                            right: Box::new(call("vsum", half("second"))),
                        },
                    },
                ])
            }
            2 => {
                let target = self.random.pick(&changeable(Kind::U64))?.clone();
                Statement::Change {
                    name: target,
                    value: Expression::Convert {
                        variable: format!("{list}.len"),
                        narrowing: "",
                        target: Type::Integer(Kind::U64),
                    },
                }
            }
            _ => {
                let target = self.random.pick(&changeable(kind))?.clone();
                Statement::Change {
                    name: target,
                    value: call("vsum", Expression::Lend(list)),
                }
            }
        };
        Some(statement)
    }

    /// A `tag` holding a literal in its `str` field, printed through named holes, or a stored
    /// borrow of one of two views, from `tpick`, printed (2.9, 6.10).
    fn strings_statement(&mut self, scope: &Scope) -> Statement {
        let views: Vec<String> = scope
            .variables
            .iter()
            .filter(|(_, found)| *found == Type::Text(None))
            .map(|(name, _)| name.clone())
            .collect();
        if self.random.chance(50) {
            // Two views here, or two made for it from literals, which last the whole program.
            let mut made = Vec::new();
            let (left, right) = match (
                self.random.pick(&views).cloned(),
                self.random.pick(&views).cloned(),
            ) {
                (Some(left), Some(right)) if left != right => (left, right),
                _ => {
                    let mut pair = Vec::new();
                    for _ in 0..2 {
                        let name = self.name("tv");
                        let text = match self.text(scope, Some(12)) {
                            Expression::Text(text) if text.is_empty() => {
                                Expression::Text("o".to_string())
                            }
                            other => other,
                        };
                        made.push(Statement::Let {
                            name: name.clone(),
                            value_type: Type::Text(None),
                            value: text,
                        });
                        pair.push(name);
                    }
                    let right = pair.pop().unwrap_or_default();
                    let left = pair.pop().unwrap_or_default();
                    (left, right)
                }
            };
            let test = self.boolean(scope, 2);
            made.push(Statement::PickText {
                name: self.name("pt"),
                left,
                right,
                test,
            });
            return Statement::Sequence(made);
        }
        let name = self.name("g");
        let text = match self.text(scope, Some(12)) {
            Expression::Text(text) if text.is_empty() => Expression::Text("o".to_string()),
            other => other,
        };
        let number = self.integer(scope, Kind::I32, 2);
        Statement::Sequence(vec![
            Statement::LetStruct {
                name: name.clone(),
                struct_name: "tag".to_string(),
                fields: vec![("name".to_string(), text), ("n".to_string(), number)],
            },
            Statement::Format {
                writer: "out",
                stored: false,
                text: format!("{{${name}.name}} {{${name}.n}}"),
                values: Vec::new(),
            },
        ])
    }

    /// A `simd` from an array of its count with `as`, and back into an array; or one picked
    /// lane by lane between two by a comparison; or one shuffled from another (2.21).
    fn simd_statement(&mut self, scope: &mut Scope) -> Option<Statement> {
        let name = self.name("sv");
        let (value_type, value) = match self.random.between(0, 2) {
            0 => {
                let (array, kind, len) = self.random.pick(&scope.arrays)?.clone();
                let value_type = Type::Simd(kind, len);
                let back = self.name("a");
                let written = format!("std::coll::array<{}>[len: {len} items]", kind.word());
                let statement = Statement::Sequence(vec![
                    Statement::Let {
                        name: name.clone(),
                        value_type,
                        value: Expression::As {
                            variable: array,
                            target: value_type,
                        },
                    },
                    Statement::LetNamed {
                        name: back.clone(),
                        written: written.clone(),
                        value: Expression::AsNamed {
                            variable: name.clone(),
                            written,
                        },
                        back: None,
                    },
                ]);
                scope.variables.push((name, value_type));
                scope.arrays.push((back, kind, len));
                return Some(statement);
            }
            1 => {
                let vectors: Vec<(String, Type)> = scope
                    .variables
                    .iter()
                    .filter(|(_, found)| matches!(found, Type::Simd(..)))
                    .cloned()
                    .collect();
                let (left, value_type) = self.random.pick(&vectors)?.clone();
                let right = self
                    .variable_of(scope, value_type)
                    .unwrap_or_else(|| Expression::Variable(left.clone()));
                let comparison = *self
                    .random
                    .pick(&Comparison::ALL)
                    .unwrap_or(&Comparison::Below);
                (
                    value_type,
                    Expression::Pick {
                        test: Box::new(Expression::Compare {
                            comparison,
                            left: Box::new(Expression::Variable(left.clone())),
                            right: Box::new(right.clone()),
                        }),
                        if_true: Box::new(right),
                        if_false: Box::new(Expression::Variable(left)),
                    },
                )
            }
            _ => {
                let vectors: Vec<(String, Kind, i128)> = scope
                    .variables
                    .iter()
                    .filter_map(|(name, found)| match found {
                        Type::Simd(kind, count) => Some((name.clone(), *kind, *count)),
                        _ => None,
                    })
                    .collect();
                let (receiver, kind, count) = self.random.pick(&vectors)?.clone();
                let lanes = self.random.between(1, 5);
                let from_zero = self.random.chance(50);
                let picked = (0..lanes)
                    .map(|_| self.random.between(0, count - 1) + i128::from(!from_zero))
                    .collect();
                (
                    Type::Simd(kind, lanes),
                    Expression::Shuffle {
                        receiver,
                        lanes: picked,
                        from_zero,
                    },
                )
            }
        };
        scope.variables.push((name.clone(), value_type));
        Some(Statement::Let {
            name,
            value_type,
            value,
        })
    }

    /// A loop over an array or list in `simd` chunks, padded, exact where the array's length is
    /// a multiple of the lanes, or with its leftovers handed over after (2.21).
    fn chunks(&mut self, scope: &Scope) -> Option<Statement> {
        let collections: Vec<(String, Kind, i128, bool)> = scope
            .arrays
            .iter()
            .map(|(name, kind, len)| (name.clone(), *kind, *len, true))
            .chain(
                scope
                    .lists
                    .iter()
                    .map(|(name, kind, max)| (name.clone(), *kind, *max, false)),
            )
            .collect();
        let (array, kind, len, is_array) = self.random.pick(&collections)?.clone();
        let targets: Vec<String> = scope
            .variables
            .iter()
            .filter(|(name, found)| {
                *found == Type::Integer(kind)
                    && !name.starts_with('i')
                    && !name.starts_with('p')
                    && !name.starts_with('x')
                    && !name.starts_with('k')
            })
            .map(|(name, _)| name.clone())
            .collect();
        let target = self.random.pick(&targets)?.clone();
        let lanes = self.random.between(1, 4);
        // `exact` only where the length is known to be a multiple, which an array's is.
        let exact = is_array && len % lanes == 0;
        let mode = match self.random.between(0, 2) {
            0 if exact => "exact",
            1 => "leftovers",
            _ => "pad",
        };
        Some(Statement::Chunks {
            array,
            lanes,
            mode,
            chunk: self.name("xc"),
            leftover: self.name("xl"),
            target,
        })
    }

    /// A secret made public in a shield, into a plain variable that can change (2.22, 9.7).
    fn reveal(&mut self, scope: &Scope) -> Option<Statement> {
        let secrets: Vec<(String, Kind)> = scope
            .variables
            .iter()
            .filter_map(|(name, found)| match found {
                Type::Secret(kind) => Some((name.clone(), *kind)),
                _ => None,
            })
            .collect();
        let (secret, kind) = self.random.pick(&secrets)?.clone();
        let targets: Vec<String> = scope
            .variables
            .iter()
            .filter(|(name, found)| {
                *found == Type::Integer(kind)
                    && !name.starts_with('i')
                    && !name.starts_with('p')
                    && !name.starts_with('x')
                    && !name.starts_with('k')
            })
            .map(|(name, _)| name.clone())
            .collect();
        let target = self.random.pick(&targets)?.clone();
        Some(Statement::Reveal {
            shield: self.name("sh"),
            target,
            secret,
        })
    }

    /// A `simd` value for a `let` or `change`: its lanes, `all` of one value, a variable, or
    /// maths lane by lane on two variables of the type, since `[ ]` and `all` take their type only
    /// from where they go, and never one `simd` and one value, which isn't decided (2.21, 12.2).
    fn lanes(&mut self, scope: &Scope, kind: Kind, lanes: i128, depth: usize) -> Expression {
        let value_type = Type::Simd(kind, lanes);
        if depth > 0 && self.random.chance(50) {
            if let (Some(left), Some(right)) = (
                self.variable_of(scope, value_type),
                self.variable_of(scope, value_type),
            ) {
                let operator = *self
                    .random
                    .pick(&[
                        Operator::Add,
                        Operator::PlusWrap,
                        Operator::MinusWrap,
                        Operator::TimesWrap,
                        Operator::PlusSaturate,
                        Operator::AndBit,
                        Operator::XorBit,
                    ])
                    .unwrap_or(&Operator::PlusWrap);
                return Expression::Binary {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            }
        }
        if self.random.chance(20) {
            if let Some(variable) = self.variable_of(scope, value_type) {
                return variable;
            }
        }
        // An array of the same count crosses into a `simd` with `as`, which can't fail (2.21).
        let arrays: Vec<String> = scope
            .arrays
            .iter()
            .filter(|(_, found, len)| *found == kind && *len == lanes)
            .map(|(name, ..)| name.clone())
            .collect();
        if let (true, Some(variable)) = (self.random.chance(30), self.random.pick(&arrays).cloned())
        {
            return Expression::As {
                variable,
                target: value_type,
            };
        }
        // `.%pick` chooses each lane from one of two, by a lane-by-lane comparison (2.21).
        if let (true, Some(Expression::Variable(left)), Some(Expression::Variable(right))) = (
            depth > 0 && self.random.chance(20),
            self.variable_of(scope, value_type),
            self.variable_of(scope, value_type),
        ) {
            let comparison = *self
                .random
                .pick(&Comparison::ALL)
                .unwrap_or(&Comparison::Below);
            let if_false = self.lanes(scope, kind, lanes, depth - 1);
            return Expression::Pick {
                test: Box::new(Expression::Compare {
                    comparison,
                    left: Box::new(Expression::Variable(left.clone())),
                    right: Box::new(Expression::Variable(right)),
                }),
                if_true: Box::new(Expression::Variable(left)),
                if_false: Box::new(if_false),
            };
        }
        // `.%shuffle` lists, for each lane, which lane of another `simd` it comes from, which
        // can have another count and repeat (2.21).
        let sources: Vec<(String, i128)> = scope
            .variables
            .iter()
            .filter_map(|(name, found)| match found {
                Type::Simd(other, count) if *other == kind => Some((name.clone(), *count)),
                _ => None,
            })
            .collect();
        if let (true, Some((receiver, count))) =
            (self.random.chance(20), self.random.pick(&sources).cloned())
        {
            let from_zero = self.random.chance(50);
            let lanes = (0..lanes)
                .map(|_| {
                    let at = self.random.between(0, count - 1);
                    if from_zero {
                        at
                    } else {
                        at + 1
                    }
                })
                .collect();
            return Expression::Shuffle {
                receiver,
                lanes,
                from_zero,
            };
        }
        if self.random.chance(20) {
            return Expression::AllLanes(Box::new(self.literal(kind)));
        }
        Expression::Lanes(
            (0..lanes)
                .map(|_| self.integer(scope, kind, depth.min(1)))
                .collect(),
        )
    }

    /// A reading method called on a live struct variable, giving this kind (5.6).
    fn method_call(&mut self, scope: &Scope, kind: Kind) -> Option<Expression> {
        if self.no_calls || self.pure_only {
            return None;
        }
        let calls: Vec<(String, String)> = scope
            .records
            .iter()
            .filter(|(name, _)| !self.given_away.contains(name))
            .flat_map(|(variable, index)| {
                let struct_name = self
                    .structs
                    .get(*index)
                    .map(|declared| declared.name.clone())
                    .unwrap_or_default();
                self.methods
                    .iter()
                    .filter(move |method| {
                        method.changes.is_none()
                            && method.kind == kind
                            && method.struct_name == struct_name
                    })
                    .map(move |method| (variable.clone(), method.name.clone()))
            })
            .collect();
        // A union's `.%size` gives an `i32` (5.6), and so does `uwidth_…`, which it's lent to
        // (7.5).
        let mut calls = calls;
        let mut lent = Vec::new();
        if kind == Kind::I32 {
            for (variable, index) in &scope.cased {
                if let Some(declared) = self.unions.get(*index).filter(|declared| declared.sized) {
                    calls.push((variable.clone(), "size".to_string()));
                    lent.push((variable.clone(), declared.name.clone()));
                }
            }
        }
        if let (true, Some((variable, union))) =
            (self.random.chance(40), self.random.pick(&lent).cloned())
        {
            return Some(Expression::Call {
                function: format!("uwidth_{union}"),
                arguments: vec![(None, Expression::Lend(variable))],
                handling: Handling::Crash,
            });
        }
        let (receiver, name) = self.random.pick(&calls)?.clone();
        Some(Expression::CallMethod { receiver, name })
    }

    /// A `simd` variable of this kind, to read one value from: its lanes added in pairs,
    /// multiplied, or its least or greatest (2.21).
    fn reduction(&mut self, scope: &Scope, kind: Kind) -> Option<Expression> {
        let found: Vec<String> = scope
            .variables
            .iter()
            .filter(|(_, found)| matches!(found, Type::Simd(lane, _) if *lane == kind))
            .map(|(name, _)| name.clone())
            .collect();
        let variable = self.random.pick(&found)?.clone();
        let method = *self
            .random
            .pick(&["sum", "product", "min_val", "max_val"])
            .unwrap_or(&"sum");
        Some(Expression::Method {
            receiver: Box::new(Expression::Variable(variable)),
            method,
        })
    }

    /// A string variable: a view, from a literal or another view, or one holding up to some
    /// bytes inside it, from a literal that fits (2.9).
    fn text_let(&mut self, scope: &mut Scope) -> Statement {
        let value_type = if self.random.chance(50) {
            Type::Text(None)
        } else {
            Type::Text(Some(self.random.between(1, 22)))
        };
        let mut value = self.expression(scope, value_type, LIMITS.depth);
        // A number into a bounded string with `as`, which fails when its text doesn't fit; a
        // single value, since `as` goes on nothing else (2.14, 3.9).
        if matches!(value_type, Type::Text(Some(_))) && self.random.chance(40) {
            let integers: Vec<(String, Kind)> = scope
                .variables
                .iter()
                .filter_map(|(name, found)| match found {
                    Type::Integer(kind) => Some((name.clone(), *kind)),
                    _ => None,
                })
                .collect();
            if let Some((variable, kind)) = self.random.pick(&integers).cloned() {
                value = Expression::As {
                    variable,
                    target: value_type,
                };
                // The most bytes a number of this type can need, so a string that holds them
                // can't fail, and a `??` on it is refused (2.14).
                let needed = match kind {
                    Kind::U8 => 3,
                    Kind::I32 => 11,
                    Kind::U32 => 10,
                    Kind::I64 | Kind::U64 => 20,
                };
                let can_fail = matches!(value_type, Type::Text(Some(bytes)) if bytes < needed);
                // Now and then its failure is handled with text that fits instead (2.14, 8.5).
                if let (true, true, Type::Text(bytes)) =
                    (can_fail, self.random.chance(40), value_type)
                {
                    let instead = self.text(scope, bytes);
                    value = Expression::AsOrElse {
                        value: Box::new(value),
                        instead: Box::new(instead),
                    };
                }
            }
        }
        let name = self.name("t");
        scope.variables.push((name.clone(), value_type));
        Statement::Let {
            name,
            value_type,
            value,
        }
    }

    /// Text for a string of this type: a literal of letters, spaces or a two-byte `é`, since
    /// escapes in a string literal aren't decided (12.2), no longer than a bounded string holds, `empty`, or for a view now and then
    /// another view, which copies with no word (2.9, D368).
    fn text(&mut self, scope: &Scope, bytes: Option<i128>) -> Expression {
        if bytes.is_none() && self.random.chance(30) {
            if let Some(variable) = self.variable_of(scope, Type::Text(None)) {
                return variable;
            }
        }
        let room = usize::try_from(bytes.unwrap_or(12)).unwrap_or(12);
        let mut text = String::new();
        for _ in 0..self.random.between(0, 10) {
            let piece = *self
                .random
                .pick(&["a", "b", "k", "o", "z", " ", "é"])
                .unwrap_or(&"a");
            if text.len() + piece.len() > room {
                break;
            }
            text.push_str(piece);
        }
        Expression::Text(text)
    }

    /// `const`, from literals and constants alone, joined with operators that can't overflow or
    /// fail, so its value is known when compiling and nothing rests on how a compile-time
    /// overflow is reported (2.3).
    fn constant(&mut self, scope: &mut Scope) -> Statement {
        // Now and then a float constant, from a literal (2.3).
        if self.random.chance(20) {
            let value_type = Type::Float(self.random.chance(50));
            let value = self.float_literal();
            let name = self.name("k");
            scope.variables.push((name.clone(), value_type));
            return Statement::Const {
                name,
                value_type,
                value,
            };
        }
        let kind = self.integer_kind();
        let value = self.constant_value(scope, kind, 2);
        let name = self.name("k");
        scope.variables.push((name.clone(), Type::Integer(kind)));
        self.constants.push(name.clone());
        Statement::Const {
            name,
            value_type: Type::Integer(kind),
            value,
        }
    }

    /// A call to `bk_…`, the `fn.pure.baked` function for this kind, declared once it's used
    /// (5.11).
    fn baked_call(&mut self, kind: Kind, argument: Expression) -> Expression {
        if !self.baked.contains(&kind) {
            self.baked.push(kind);
        }
        Expression::Call {
            function: format!("bk_{}", kind.word()),
            arguments: vec![(None, argument)],
            handling: Handling::Crash,
        }
    }

    fn constant_value(&mut self, scope: &Scope, kind: Kind, depth: usize) -> Expression {
        if depth == 0 || self.random.chance(40) {
            let constants: Vec<String> = scope
                .variables
                .iter()
                .filter(|(name, found)| {
                    *found == Type::Integer(kind) && self.constants.contains(name)
                })
                .map(|(name, _)| name.clone())
                .collect();
            if self.random.chance(40) {
                if let Some(name) = self.random.pick(&constants).cloned() {
                    return Expression::Variable(name);
                }
            }
            return self.literal(kind);
        }
        // A constant can call a `fn.pure.baked` function (2.3, 5.11).
        if self.random.chance(25) {
            let argument = self.constant_value(scope, kind, depth - 1);
            return self.baked_call(kind, argument);
        }
        let operator = *self
            .random
            .pick(&[
                Operator::PlusWrap,
                Operator::MinusWrap,
                Operator::TimesWrap,
                Operator::AndBit,
                Operator::OrBit,
                Operator::XorBit,
            ])
            .unwrap_or(&Operator::PlusWrap);
        let left = self.constant_value(scope, kind, depth - 1);
        let right = self.constant_value(scope, kind, depth - 1);
        Expression::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// Up to three named integer types: aliases, new types, and now and then a family of new
    /// types (2.18).
    fn named_types(&mut self) {
        for _ in 0..self.random.between(0, 3) {
            let alias = self.random.chance(50);
            // A family of new types, or of aliases, stating its kind once (2.18).
            let family = self.random.chance(40).then(|| {
                let kind_word = if alias { "faa" } else { "fam" };
                self.name(kind_word)
            });
            let kind = self.integer_kind();
            let name = self.name(if alias { "al" } else { "nt" });
            self.named.push(NamedType {
                name,
                alias,
                family,
                kind,
            });
        }
    }

    /// Now and then a `type.new` over the first struct, an alias for a bounded list, and one for
    /// a bounded string (2.18).
    fn other_named_types(&mut self) {
        if let (true, Some(declared)) = (self.random.chance(40), self.structs.first()) {
            let struct_name = declared.name.clone();
            let name = self.name("wrap");
            self.named_others.push((name.clone(), false, struct_name));
            self.wrap = Some(name);
        }
        if self.random.chance(40) {
            let kind = self.integer_kind();
            let max = self.random.between(1, 6);
            let name = self.name("nums");
            self.named_others.push((
                name.clone(),
                true,
                format!("std::coll::list<{}>[len.max: {max} items]", kind.word()),
            ));
            self.list_alias = Some((name, kind, max));
        }
        if self.random.chance(40) {
            let bytes = self.random.between(2, 16);
            let name = self.name("txt");
            self.named_others
                .push((name.clone(), true, format!("str[len.max: {bytes} bytes]")));
            self.text_alias = Some((name, bytes));
        }
    }

    /// A value of a name for something other than an integer: a struct crossing into its
    /// `type.new` with `Copy` or `Give` and back out with `Give`, a bounded list or string
    /// through its alias, used as the list or string from then on (2.18).
    fn other_named_let(&mut self, scope: &mut Scope, in_loop: bool) -> Option<Statement> {
        match self.random.between(0, 2) {
            0 => {
                let new_type = self.wrap.clone()?;
                let index = 0;
                let live: Vec<String> = scope
                    .records
                    .iter()
                    .filter(|(name, found)| *found == index && !self.given_away.contains(name))
                    .map(|(name, _)| name.clone())
                    .collect();
                let source = self.random.pick(&live)?.clone();
                // Whether a pure function may `Copy` isn't decided, so one gives (12.5), and a
                // loop would give it away again each round, so in a loop it copies.
                if self.pure_only && in_loop {
                    return None;
                }
                let give = self.pure_only || (!in_loop && self.random.chance(40));
                if give {
                    self.given_away.push(source.clone());
                }
                let back = self.name("r");
                scope.records.push((back.clone(), index));
                Some(Statement::WrapStruct {
                    wrapped: self.name("w"),
                    new_type,
                    word: if give { "Give" } else { "Copy" },
                    source,
                    back,
                    struct_name: self.structs.first()?.name.clone(),
                })
            }
            1 => {
                let (alias, kind, max) = self.list_alias.clone()?;
                let count = self.random.between(0, max);
                let value = if count == 0 {
                    Expression::Text(String::new())
                } else {
                    Expression::Lanes(
                        (0..count)
                            .map(|_| self.expression(scope, Type::Integer(kind), LIMITS.depth))
                            .collect(),
                    )
                };
                let name = self.name("l");
                scope.lists.push((name.clone(), kind, max));
                Some(Statement::LetNamed {
                    name,
                    written: alias,
                    value,
                    back: None,
                })
            }
            _ => {
                let (alias, bytes) = self.text_alias.clone()?;
                let value = self.text(scope, Some(bytes));
                let name = self.name("t");
                scope
                    .variables
                    .push((name.clone(), Type::Text(Some(bytes))));
                Some(Statement::LetNamed {
                    name,
                    written: alias,
                    value,
                    back: None,
                })
            }
        }
    }

    /// A variable of a named type. An alias takes any value of its integer and is used as one
    /// from then on; a new type crosses in from a literal or with `as`, and back out into a
    /// plain variable with `as`, since it doesn't mix (2.18).
    fn named_let(&mut self, scope: &mut Scope) -> Statement {
        if self.random.chance(40) {
            if let Some(statement) = self.other_named_let(scope, true) {
                return statement;
            }
        }
        let Some(named) = self.random.pick(&self.named.clone()).cloned() else {
            return self.list_statement(scope);
        };
        let name = self.name("v");
        if named.alias {
            let value = self.expression(scope, Type::Integer(named.kind), LIMITS.depth);
            scope
                .variables
                .push((name.clone(), Type::Integer(named.kind)));
            return Statement::LetNamed {
                name,
                written: named.used(),
                value,
                back: None,
            };
        }
        let plain = self.variable_of(scope, Type::Integer(named.kind));
        let value = match plain {
            Some(Expression::Variable(variable)) if self.random.chance(60) => Expression::AsNamed {
                variable,
                written: named.used(),
            },
            _ => self.literal(named.kind),
        };
        let back = self.name("v");
        scope
            .variables
            .push((back.clone(), Type::Integer(named.kind)));
        Statement::LetNamed {
            name,
            written: named.used(),
            value,
            back: Some((back, named.kind)),
        }
    }

    /// The test: a union variable tested for one of its cases, whose fields the block reads, or
    /// a plain integer tested for a bound, which the block puts into a bounded variable with no
    /// check (4.2).
    fn test_statement(
        &mut self,
        scope: &mut Scope,
        depth: usize,
        (prints, in_loop): (bool, bool),
    ) -> Statement {
        let held = scope.case_fields.len();
        let declared = scope.variables.len();
        let mut body = Vec::new();
        let tested = match self.random.pick(&scope.cased).cloned() {
            Some((variable, index)) if self.random.chance(50) => {
                let union = self.unions.get(index).cloned();
                union.and_then(|union| {
                    let (case, fields) = self.random.pick(&union.cases)?.clone();
                    for (field, kind) in fields {
                        scope.case_fields.push((variable.clone(), field, kind));
                    }
                    Some((variable, format!("{}.{case}", union.name)))
                })
            }
            _ => None,
        };
        let (variable, written) = if let Some(found) = tested {
            found
        } else {
            let integers: Vec<(String, Kind)> = scope
                .variables
                .iter()
                .filter_map(|(name, found)| match found {
                    Type::Integer(kind) => Some((name.clone(), *kind)),
                    _ => None,
                })
                .collect();
            let Some((variable, kind)) = self.random.pick(&integers).cloned() else {
                return self.list_statement(scope);
            };
            let Type::Bounded(_, low, high) = self.bounded_type_of(kind) else {
                return self.list_statement(scope);
            };
            let bounded = Type::Bounded(kind, low, high);
            let name = self.name("v");
            body.push(Statement::Let {
                name: name.clone(),
                value_type: bounded,
                value: Expression::Variable(variable.clone()),
            });
            scope.variables.push((name, bounded));
            (variable, bounded.word())
        };
        // A block inside a loop is still in it, so nothing in it gives a value away (4.8).
        body.extend(self.block(scope, 2, depth + 1, (prints, in_loop)));
        scope.case_fields.truncate(held);
        scope.variables.truncate(declared);
        Statement::Test {
            variable,
            written,
            body,
        }
    }

    /// A bounded type over this kind.
    fn bounded_type_of(&mut self, kind: Kind) -> Type {
        let (low, high) = kind.range();
        let low = self.random.between(low.max(-50), high.min(50));
        let high = self.random.between(low, high.min(low + 100));
        Type::Bounded(kind, low, high)
    }

    /// A call to an integer function started as a task in `main`, then ended with
    /// `task.done`, whose failure, if any, stops the program (D356, D357, 8.9).
    fn task(&mut self, scope: &mut Scope) -> Option<Statement> {
        let kind = self.integer_kind();
        let (function, arguments, _) = self.plain_call(scope, kind, LIMITS.depth)?;
        let task = self.name("task");
        let name = self.name("v");
        scope.variables.push((name.clone(), Type::Integer(kind)));
        let runner = self.runner();
        Some(Statement::Task {
            task,
            name,
            kind,
            call: Expression::Call {
                function,
                arguments,
                handling: Handling::Crash,
            },
            runner,
        })
    }

    /// A stored borrow of a struct in scope: read through, `.held` in a block of its own,
    /// changed through with `Lend.mut`, or returned by `rwarm` from two (6.10).
    fn borrow_struct(&mut self, scope: &mut Scope) -> Option<Statement> {
        let live: Vec<(String, usize)> = scope
            .records
            .iter()
            .filter(|(name, _)| !self.given_away.contains(name))
            .cloned()
            .collect();
        let (record, index) = self.random.pick(&live)?.clone();
        let declared = self.structs.get(index)?.clone();
        let (field, kind) = self.random.pick(&declared.fields)?.clone();
        // `rwarm` reads the first field of the first struct, from two of them.
        let others: Vec<String> = live
            .iter()
            .filter(|(_, found)| *found == 0)
            .map(|(name, _)| name.clone())
            .collect();
        let warm = self.warm.clone().filter(|_| index == 0 && others.len() > 1);
        let how = match self.random.between(0, 3) {
            0 => "held",
            1 => "mut",
            2 if warm.is_some() => "returned",
            _ => "plain",
        };
        let (other, field, kind) = match (&warm, how) {
            (Some((_, warm_field)), "returned") => {
                let other = self.random.pick(&others)?.clone();
                let kind = declared
                    .fields
                    .iter()
                    .find(|(found, _)| found == warm_field)
                    .map_or(kind, |(_, found)| *found);
                (other, warm_field.clone(), kind)
            }
            _ => (record.clone(), field, kind),
        };
        // The value can't read the struct, which the `Lend.mut` holds until the write.
        let value = match declared.field_type(&field, kind) {
            Type::Bounded(_, low, high) => Expression::Integer(self.random.between(low, high)),
            _ => self.literal(kind),
        };
        // What's read goes into a new variable, named `k…` so nothing changes it after.
        let target = self.name("k");
        let declare = (how != "mut").then_some(kind);
        if how != "mut" && how != "held" {
            scope.variables.push((target.clone(), Type::Integer(kind)));
        }
        Some(Statement::BorrowStruct {
            how,
            view: self.name("bw"),
            records: (record, other),
            struct_name: declared.name,
            field,
            target,
            declare,
            value,
        })
    }

    /// A value for each of a struct's fields, each fitting its field (2.5).
    fn struct_fields(&mut self, scope: &Scope, declared: &StructDecl) -> Vec<(String, Expression)> {
        declared
            .fields
            .iter()
            .map(|(field, kind)| {
                (
                    field.clone(),
                    self.field_value(scope, declared, field, *kind),
                )
            })
            .collect()
    }

    /// A `nest`, holding a struct, an `i32` and a list, built whole, a field of its struct
    /// changed, and an item of its list read; or a `view`, holding a borrow of a struct, its
    /// field read through it (7.1, 6.10).
    fn nest_statement(&mut self, scope: &mut Scope) -> Option<Statement> {
        let (inner, kind) = self.nest.clone()?;
        let declared = self.structs.first()?.clone();
        let name = self.name("n");
        let read = self.name("k");
        if self.random.chance(40) {
            let live: Vec<String> = scope
                .records
                .iter()
                .filter(|(found, index)| *index == 0 && !self.given_away.contains(found))
                .map(|(found, _)| found.clone())
                .collect();
            // A struct already here, or one built for it.
            let mut made = Vec::new();
            let record = if let Some(record) = self.random.pick(&live).cloned() {
                record
            } else {
                let record = self.name("r");
                let fields = self.struct_fields(scope, &declared);
                made.push(Statement::LetStruct {
                    name: record.clone(),
                    struct_name: declared.name.clone(),
                    fields,
                });
                scope.records.push((record.clone(), 0));
                record
            };
            let (field, field_kind) = self.random.pick(&declared.fields)?.clone();
            scope
                .variables
                .push((read.clone(), Type::Integer(field_kind)));
            made.extend([
                Statement::LetNamed {
                    name: name.clone(),
                    written: format!("view<`src`: ${record}>"),
                    value: Expression::Build {
                        struct_name: "view".to_string(),
                        fields: vec![("of".to_string(), Expression::Lend(record))],
                    },
                    back: None,
                },
                Statement::Let {
                    name: read,
                    value_type: Type::Integer(field_kind),
                    value: Expression::Field {
                        variable: format!("{name}.of"),
                        field,
                    },
                },
            ]);
            return Some(Statement::Sequence(made));
        }
        let built = self.struct_fields(scope, &declared);
        let count = self.integer(scope, Kind::I32, 2);
        let items = (0..self.random.between(1, 3))
            .map(|_| self.literal(kind))
            .collect();
        let (field, field_kind) = self.random.pick(&declared.fields)?.clone();
        let changed = match declared.field_type(&field, field_kind) {
            Type::Bounded(_, low, high) => Expression::Integer(self.random.between(low, high)),
            _ => self.literal(field_kind),
        };
        scope.variables.push((read.clone(), Type::Integer(kind)));
        Some(Statement::Sequence(vec![
            Statement::LetStruct {
                name: name.clone(),
                struct_name: "nest".to_string(),
                fields: vec![
                    (
                        "inner".to_string(),
                        Expression::Build {
                            struct_name: inner,
                            fields: built,
                        },
                    ),
                    ("count".to_string(), count),
                    ("items".to_string(), Expression::Lanes(items)),
                ],
            },
            Statement::ChangeField {
                variable: format!("{name}.inner"),
                field,
                value: changed,
            },
            Statement::Let {
                name: read,
                value_type: Type::Integer(kind),
                value: Expression::Item {
                    array: format!("{name}.items"),
                    position: Position::Pos(1),
                },
            },
        ]))
    }

    /// `defer` on a print in `main`, or on a `change` to a variable that can change; it runs
    /// when the block ends, in reverse order (6.12).
    fn defer_statement(&mut self, scope: &Scope, prints: bool) -> Option<Statement> {
        if prints && self.random.chance(50) {
            let number = self.random.between(1, 99);
            return Some(Statement::Defer(Box::new(Statement::Format {
                writer: "out",
                stored: false,
                text: format!("deferred {number}"),
                values: Vec::new(),
            })));
        }
        let targets: Vec<(String, Kind)> = scope
            .variables
            .iter()
            .filter_map(|(name, found)| match found {
                Type::Integer(kind)
                    if !name.starts_with('i')
                        && !name.starts_with('p')
                        && !name.starts_with('x')
                        && !name.starts_with('k') =>
                {
                    Some((name.clone(), *kind))
                }
                _ => None,
            })
            .collect();
        let (name, kind) = self.random.pick(&targets)?.clone();
        let value = self.literal(kind);
        Some(Statement::Defer(Box::new(Statement::Change {
            name,
            value,
        })))
    }

    /// A loop over the fresh array `trio_…` returns, which takes no word (4.4, D396).
    fn fresh_loop(&mut self, scope: &Scope) -> Option<Statement> {
        if self.pure_only || self.no_calls {
            return None;
        }
        let targets: Vec<(String, Kind)> = scope
            .variables
            .iter()
            .filter_map(|(name, found)| match found {
                Type::Integer(kind)
                    if !name.starts_with('i')
                        && !name.starts_with('p')
                        && !name.starts_with('x')
                        && !name.starts_with('k') =>
                {
                    Some((name.clone(), *kind))
                }
                _ => None,
            })
            .collect();
        let (target, kind) = self.random.pick(&targets)?.clone();
        if !self.trios.contains(&kind) {
            self.trios.push(kind);
        }
        let argument = self.integer(scope, kind, 2);
        Some(Statement::FreshLoop {
            item: self.name("x"),
            call: Expression::Call {
                function: format!("trio_{}", kind.word()),
                arguments: vec![(None, argument)],
                handling: Handling::Crash,
            },
            target,
        })
    }

    /// Which of `main`'s runners a task goes on (D346, D356, D392).
    fn runner(&mut self) -> &'static str {
        if self.runners {
            self.random
                .pick(&["runner", "pool", "events"])
                .copied()
                .unwrap_or("runner")
        } else {
            "runner"
        }
    }

    /// A list split into halves, each summed by a task that reads it, joined and added (D343,
    /// D382).
    fn split_tasks(&mut self, scope: &Scope) -> Option<Statement> {
        let (list, kind, _) = self.random.pick(&scope.lists)?.clone();
        let targets: Vec<String> = scope
            .variables
            .iter()
            .filter(|(name, found)| {
                *found == Type::Integer(kind)
                    && !name.starts_with('i')
                    && !name.starts_with('p')
                    && !name.starts_with('x')
                    && !name.starts_with('k')
            })
            .map(|(name, _)| name.clone())
            .collect();
        let target = self.random.pick(&targets)?.clone();
        if !self.views.contains(&kind) {
            self.views.push(kind);
        }
        Some(Statement::SplitTasks {
            halves: self.name("h"),
            list,
            kind,
            at_end: self.random.chance(50),
            tasks: (self.name("task"), self.name("task")),
            runner: self.runner(),
            target,
        })
    }

    /// An atomic, used every way a task would, or one or two locks taken together (D358,
    /// D361).
    fn shared(&mut self, scope: &mut Scope) -> Option<Statement> {
        if self.random.chance(50) {
            let kind = *self
                .random
                .pick(&[Kind::U32, Kind::U64, Kind::I64])
                .unwrap_or(&Kind::U64);
            let (old, now) = (self.name("k"), self.name("k"));
            scope.variables.push((old.clone(), Type::Integer(kind)));
            scope.variables.push((now.clone(), Type::Integer(kind)));
            return Some(Statement::Atomic {
                name: self.name("at"),
                kind,
                numbers: (
                    self.random.between(0, 20),
                    self.random.between(0, 20),
                    self.random.between(0, 40),
                ),
                old,
                now,
                acquire: self.random.chance(40),
            });
        }
        let targets: Vec<(String, Kind)> = scope
            .variables
            .iter()
            .filter_map(|(name, found)| match found {
                Type::Integer(kind)
                    if !name.starts_with('i')
                        && !name.starts_with('p')
                        && !name.starts_with('x')
                        && !name.starts_with('k') =>
                {
                    Some((name.clone(), *kind))
                }
                _ => None,
            })
            .collect();
        let (target, kind) = self.random.pick(&targets)?.clone();
        let count = self.random.between(1, 2);
        let mut locks = Vec::new();
        let mut bound = Vec::new();
        for _ in 0..count {
            let (low, high) = kind.range();
            locks.push((
                self.name("lk"),
                self.random.between(low.max(0), high.min(50)),
            ));
            bound.push(self.name("lb"));
        }
        Some(Statement::Locks {
            locks,
            bound,
            kind,
            target,
        })
    }

    /// A void function: integers lent to change, which its body changes, plain integers, and
    /// structs `Lent` or `Given`, whose fields it reads (5.3, 6.1, 6.5, D393).
    fn procedure(&mut self) -> Procedure {
        let name = self.name("proc");
        let mut scope = Scope {
            variables: Vec::new(),
            arrays: Vec::new(),
            lists: Vec::new(),
            records: Vec::new(),
            cased: Vec::new(),
            case_fields: Vec::new(),
        };
        let mut parameters = Vec::new();
        let mut changed = Vec::new();
        for _ in 0..self.random.between(1, 3) {
            // A parameter that changes is `$q…`; the others are `$p…`, which `change` never
            // picks, since a parameter can't change unless it's lent to (D184).
            let roll = self.random.between(0, 2);
            let parameter = self.name(if roll == 0 { "q" } else { "p" });
            let kind = match roll {
                0 => {
                    let kind = self.integer_kind();
                    changed.push((parameter.clone(), kind));
                    ParamKind::Changed(kind)
                }
                1 if !self.structs.is_empty() => {
                    let count = i128::try_from(self.structs.len()).unwrap_or(1);
                    let index = usize::try_from(self.random.between(0, count - 1)).unwrap_or(0);
                    let struct_name = self
                        .structs
                        .get(index)
                        .map(|declared| declared.name.clone())
                        .unwrap_or_default();
                    // Its fields are read, never changed, since it isn't lent to change.
                    if let Some(declared) = self.structs.get(index) {
                        for (field, kind) in &declared.fields {
                            scope
                                .case_fields
                                .push((parameter.clone(), field.clone(), *kind));
                        }
                    }
                    ParamKind::Record {
                        struct_name,
                        given: self.random.chance(50),
                    }
                }
                _ => {
                    let kind = self.integer_kind();
                    scope
                        .variables
                        .push((parameter.clone(), Type::Integer(kind)));
                    ParamKind::Plain(kind)
                }
            };
            parameters.push((parameter, kind));
        }
        let mut body = Vec::new();
        for (parameter, kind) in changed {
            let value = self.expression(&scope, Type::Integer(kind), 2);
            body.push(Statement::Change {
                name: parameter.clone(),
                value,
            });
        }
        body.extend(self.block(&mut scope, 2, 1, (false, false)));
        Procedure {
            name,
            parameters,
            body,
        }
    }

    /// A call to a void function as a statement: a different variable lent to change for each
    /// integer it changes, literals for plain integers, and structs lent or handed over with
    /// `Give` or `Copy`, a given one gone afterwards (6.1, 6.2, 6.5).
    fn call_procedure(&mut self, scope: &Scope, in_loop: bool) -> Option<Statement> {
        let procedure = self.random.pick(&self.procedures.clone())?.clone();
        let mut changeable: Vec<(String, Kind)> = scope
            .variables
            .iter()
            .filter(|(name, _)| {
                !name.starts_with('i')
                    && !name.starts_with('p')
                    && !name.starts_with('x')
                    && !name.starts_with('k')
            })
            .filter_map(|(name, found)| match found {
                Type::Integer(kind) => Some((name.clone(), *kind)),
                _ => None,
            })
            .collect();
        let mut records: Vec<(String, usize)> = scope
            .records
            .iter()
            .filter(|(name, _)| !self.given_away.contains(name))
            .cloned()
            .collect();
        let mut arguments = Vec::new();
        for (_, kind) in &procedure.parameters {
            match kind {
                ParamKind::Changed(kind) => {
                    let at = changeable.iter().position(|(_, found)| found == kind)?;
                    let (name, _) = changeable.remove(at);
                    arguments.push(format!("Lend.mut ${name}"));
                }
                ParamKind::Plain(kind) => {
                    arguments.push(
                        crate::tree::Style {
                            fewer_brackets: false,
                        }
                        .grouped(&self.literal(*kind)),
                    );
                }
                ParamKind::Record { struct_name, given } => {
                    let at = records.iter().position(|(_, index)| {
                        self.structs
                            .get(*index)
                            .is_some_and(|declared| declared.name == *struct_name)
                    })?;
                    let (name, _) = records.remove(at);
                    if *given {
                        // In a loop, `Give` would hand it over again on the next round.
                        if !in_loop && self.random.chance(50) {
                            arguments.push(format!("Give ${name}"));
                            self.given_away.push(name);
                        } else {
                            arguments.push(format!("Copy ${name}"));
                        }
                    } else {
                        arguments.push(format!("Lend ${name}"));
                    }
                }
            }
        }
        Some(Statement::CallProcedure {
            name: procedure.name,
            arguments,
        })
    }

    /// A tagged union of two or three cases, some carrying integer data (7.5).
    fn union_decl(&mut self) -> UnionDecl {
        let name = self.name("un");
        let cases = (0..self.random.between(2, 3))
            .map(|_| {
                let case = self.name("cs");
                let fields = if self.random.chance(50) {
                    (0..self.random.between(1, 2))
                        .map(|_| (self.name("gd"), self.integer_kind()))
                        .collect()
                } else {
                    Vec::new()
                };
                (case, fields)
            })
            .collect();
        UnionDecl {
            name,
            cases,
            sized: self.random.chance(50),
        }
    }

    /// A new union value, or a `match` on one in scope, whose arms cover every case in any
    /// order, an arm with data now and then naming it to read it (4.3, 7.6).
    fn union_statement(
        &mut self,
        scope: &mut Scope,
        depth: usize,
        (prints, in_loop): (bool, bool),
    ) -> Statement {
        if depth < 2 {
            if let Some((variable, index)) = self.random.pick(&scope.cased).cloned() {
                if self.random.chance(60) {
                    if let Some(declared) = self.unions.get(index).cloned() {
                        let mut cases = declared.cases.clone();
                        // A shuffle, so arms aren't always in the declared order.
                        for at in (1..cases.len()).rev() {
                            let other =
                                usize::try_from(self.random.between(0, at as i128)).unwrap_or(0);
                            cases.swap(at, other);
                        }
                        let mut arms = Vec::new();
                        for (case, fields) in cases {
                            let named = (!fields.is_empty() && self.random.chance(70))
                                .then(|| self.name("m"));
                            let held = scope.case_fields.len();
                            if let Some(name) = &named {
                                for (field, kind) in &fields {
                                    scope.case_fields.push((name.clone(), field.clone(), *kind));
                                }
                            }
                            let body = self.block(scope, 2, depth + 1, (prints, in_loop));
                            scope.case_fields.truncate(held);
                            arms.push((case, named, body));
                        }
                        return Statement::Match {
                            variable,
                            union_name: declared.name,
                            arms,
                        };
                    }
                }
            }
        }
        let count = i128::try_from(self.unions.len()).unwrap_or(0);
        if count == 0 {
            return self.array_statement(scope);
        }
        let index = usize::try_from(self.random.between(0, count - 1)).unwrap_or(0);
        let Some(declared) = self.unions.get(index).cloned() else {
            return self.array_statement(scope);
        };
        let Some((case, fields)) = self.random.pick(&declared.cases).cloned() else {
            return self.array_statement(scope);
        };
        let fields = fields
            .iter()
            .map(|(field, kind)| {
                (
                    field.clone(),
                    self.expression(scope, Type::Integer(*kind), LIMITS.depth),
                )
            })
            .collect();
        let name = self.name("w");
        scope.cased.push((name.clone(), index));
        Statement::LetCase {
            name,
            union_name: declared.name,
            case,
            fields,
        }
    }

    /// Methods on a struct: now and then one that reads, giving back a value from its fields,
    /// and one that changes a field by a value it's given (5.6).
    fn method_decls(&mut self, index: usize) {
        let Some(declared) = self.structs.get(index).cloned() else {
            return;
        };
        let mut scope = Scope {
            variables: Vec::new(),
            arrays: Vec::new(),
            lists: Vec::new(),
            records: Vec::new(),
            cased: Vec::new(),
            case_fields: declared
                .fields
                .iter()
                .map(|(field, kind)| ("r".to_string(), field.clone(), *kind))
                .collect(),
        };
        if self.random.chance(50) {
            let kind = self.integer_kind();
            let value = self.integer(&scope, kind, 2);
            let name = self.name("mt");
            self.methods.push(MethodDecl {
                struct_name: declared.name.clone(),
                name,
                changes: None,
                labelled: false,
                kind,
                value,
            });
        }
        if let (true, Some((field, kind))) = (
            self.random.chance(40),
            self.random.pick(&declared.fields).cloned(),
        ) {
            scope
                .variables
                .push(("by".to_string(), Type::Integer(kind)));
            // A bounded field gets a value that fits, or one checked while running (2.5).
            let value = self.field_value(&scope, &declared, &field, kind);
            let name = self.name("mt");
            self.methods.push(MethodDecl {
                struct_name: declared.name,
                name,
                changes: Some(field),
                labelled: self.random.chance(50),
                kind,
                value,
            });
        }
    }

    /// Up to two families of units, each a base and up to three more, each a whole number of
    /// the one before, so every unit in a family is a whole number of the smaller ones (2.16).
    fn unit_families(&mut self) {
        for _ in 0..self.random.between(0, 2) {
            let family = self.name("fu");
            let mut units = Vec::new();
            let base = format!("{family}.{}", self.name("b"));
            self.units
                .push((Box::leak(base.clone().into_boxed_str()), 1));
            let mut factor = 1;
            let mut previous = base.clone();
            units.push((base, ": base".to_string()));
            for _ in 0..self.random.between(1, 3) {
                let count = self.random.between(2, 12);
                factor *= count;
                let unit = format!("{family}.{}", self.name("m"));
                self.units
                    .push((Box::leak(unit.clone().into_boxed_str()), factor));
                units.push((unit.clone(), format!(" = {count} {previous}")));
                previous = unit;
            }
            self.unit_families.push(UnitFamily {
                keyword: "fam.unit",
                name: family,
                units,
            });
        }
    }

    /// Now and then float units: two bases in one family, a family of units made of them with
    /// `per` and `x`, and a base with an offset, whose values say `.point` or `.span` (2.16).
    fn float_unit_families(&mut self) {
        if !self.random.chance(50) {
            return;
        }
        let leak = |text: String| -> &'static str { Box::leak(text.into_boxed_str()) };
        let bases = self.name("fv");
        let length = leak(format!("{bases}.{}", self.name("a")));
        let time = leak(format!("{bases}.{}", self.name("t")));
        let made = self.name("fc");
        let speed = leak(format!("{made}.{}", self.name("sp")));
        let area = leak(format!("{made}.{}", self.name("ar")));
        let scale = self.name("fo");
        let kelvin = leak(format!("{scale}.{}", self.name("k")));
        let celsius = leak(format!("{scale}.{}", self.name("c")));
        let offset = *self
            .random
            .pick(&["~-273.15", "32.5", "-40.0", "100.25"])
            .unwrap_or(&"32.5");
        // A prefix family, which the length opts into with `with`, making its prefixed units by
        // joining the names.
        let prefixes = self.name("fp");
        let short = length.split('.').nth(1).unwrap_or("a");
        let kilo = leak(format!("{bases}.kilo{short}"));
        let milli = leak(format!("{bases}.milli{short}"));
        self.unit_families.push(UnitFamily {
            keyword: "fam.prefix",
            name: prefixes.clone(),
            units: vec![
                (format!("{prefixes}.kilo"), " = 1000".to_string()),
                (format!("{prefixes}.milli"), " = 1 per 1000".to_string()),
            ],
        });
        self.unit_families.push(UnitFamily {
            keyword: "fam.unit",
            name: bases,
            units: vec![
                (length.to_string(), format!(": base with {prefixes}")),
                (time.to_string(), ": base".to_string()),
            ],
        });
        let accel = leak(format!("{made}.{}", self.name("ac")));
        self.unit_families.push(UnitFamily {
            keyword: "fam.unit",
            name: made,
            units: vec![
                (speed.to_string(), format!(" = {length} per {time}")),
                (accel.to_string(), format!(" = {speed} per {time}")),
                (area.to_string(), format!(" = {length} x {length}")),
            ],
        });
        // A logarithmic unit, whose levels are points and gains spans.
        let power = self.name("fl");
        let milliwatts = leak(format!("{power}.{}", self.name("w")));
        let decibels = leak(format!("{power}.{}", self.name("db")));
        self.unit_families.push(UnitFamily {
            keyword: "fam.unit",
            name: power,
            units: vec![
                (milliwatts.to_string(), ": base".to_string()),
                (
                    decibels.to_string(),
                    format!(" = decibels of 1 {milliwatts}"),
                ),
            ],
        });
        self.unit_families.push(UnitFamily {
            keyword: "fam.unit",
            name: scale,
            units: vec![
                (kelvin.to_string(), ": base".to_string()),
                (celsius.to_string(), format!(" = {kelvin} offset {offset}")),
            ],
        });
        self.float_units = Some(FloatUnits {
            length,
            kilo,
            milli,
            time,
            speed,
            accel,
            area,
            kelvin,
            celsius,
            milliwatts,
            decibels,
        });
    }

    /// A float variable in a unit: a length or time, a speed or area made of them with `div` or
    /// `x`, or a point or span on the offset scale (2.16).
    fn float_unit_let(&mut self, scope: &mut Scope) -> Statement {
        let Some(units) = self.float_units else {
            return self.list_statement(scope);
        };
        let leak = |text: String| -> &'static str { Box::leak(text.into_boxed_str()) };
        let has = |unit: &str| {
            scope
                .variables
                .iter()
                .any(|(_, found)| matches!(found, Type::FloatUnit(written) if *written == unit))
        };
        let celsius_point = leak(format!("{}.point", units.celsius));
        // Lengths and times first, and points before spans, so the units made from them have
        // something to be made from.
        let roll = if !has(units.length) {
            0
        } else if !has(units.time) && self.random.chance(70) {
            1
        } else if !has(celsius_point) && self.random.chance(50) {
            4
        } else {
            // Mostly the units made from others, now that there's something to make them from.
            *self
                .random
                .pick(&[2, 3, 5, 5, 4, 0, 1, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
                .unwrap_or(&2)
        };
        let celsius_span = leak(format!("{}.span", units.celsius));
        // A speed needs a time, and a point plus a span needs a span.
        let roll = match roll {
            2 if !has(units.time) => 1,
            4 if has(celsius_point) && !has(celsius_span) => 5,
            other => other,
        };
        let unit = match roll {
            0 => units.length,
            1 => units.time,
            10 if has(units.speed) => units.accel,
            2 | 10 => units.speed,
            3 => units.area,
            4 => celsius_point,
            5 => celsius_span,
            6 => leak(format!("{}.point", units.kelvin)),
            7 => leak(format!("{}.span", units.kelvin)),
            8 => units.kilo,
            9 => units.milli,
            11 => leak(format!("{}.point", units.decibels)),
            12 => leak(format!("{}.span", units.decibels)),
            13 => leak(format!("{}.point", units.milliwatts)),
            14 => self.degrees,
            _ => RADIANS,
        };
        let value_type = Type::FloatUnit(unit);
        let value = self.float_unit_value(scope, unit, LIMITS.depth);
        let name = self.name("fw");
        scope.variables.push((name.clone(), value_type));
        Statement::Let {
            name,
            value_type,
            value,
        }
    }

    /// A value in a float unit: a variable in it, a literal, two added, a speed or area made
    /// from a length and a time, a point plus a span or one point minus another, or a point
    /// converted across the offset (2.16).
    #[allow(clippy::too_many_lines)] // One branch per kind of unit, read top to bottom.
    fn float_unit_value(&mut self, scope: &Scope, unit: &'static str, depth: usize) -> Expression {
        let Some(units) = self.float_units else {
            return self.float_literal();
        };
        let of = |wanted: &str| -> Vec<String> {
            scope
                .variables
                .iter()
                .filter(
                    |(_, found)| matches!(found, Type::FloatUnit(written) if *written == wanted),
                )
                .map(|(name, _)| name.clone())
                .collect()
        };
        let same = of(unit);
        let roll = self.random.between(0, 9);
        if roll < 2 {
            if let Some(name) = self.random.pick(&same).cloned() {
                return Expression::Variable(name);
            }
        }
        let bare = unit
            .strip_suffix(".point")
            .or_else(|| unit.strip_suffix(".span"))
            .unwrap_or(unit);
        let point = unit.strip_suffix(".point").is_some();
        let span = unit.strip_suffix(".span").is_some();
        let leak = |text: String| -> &'static str { Box::leak(text.into_boxed_str()) };
        // Maths makes a combination of units, which storing gives its declared name, and which
        // compares with a value of that name too (2.16, D397).
        if unit == units.accel {
            if let (Some(speed), Some(time)) = (
                self.random.pick(&of(units.speed)).cloned(),
                self.random.pick(&of(units.time)).cloned(),
            ) {
                return Expression::Binary {
                    operator: Operator::Divide,
                    left: Box::new(Expression::Variable(speed)),
                    right: Box::new(Expression::Variable(time)),
                };
            }
            return Expression::FloatWithUnit {
                number: self.positive_float(),
                unit,
            };
        }
        // A length converted from the same length with another prefix, or an angle from the
        // other angle.
        let lengths = [units.length, units.kilo, units.milli];
        let angles = [self.degrees, RADIANS];
        let kin: &[&str] = if lengths.contains(&unit) {
            &lengths
        } else if angles.contains(&unit) {
            &angles
        } else {
            &[]
        };
        if !kin.is_empty() && roll < 5 {
            let others: Vec<String> = kin
                .iter()
                .filter(|other| **other != unit)
                .flat_map(|other| of(other))
                .collect();
            if let Some(source) = self.random.pick(&others).cloned() {
                return Expression::Convert {
                    variable: source,
                    narrowing: "",
                    target: Type::FloatUnit(unit),
                };
            }
        }
        if unit == units.speed || unit == units.area {
            let lengths = of(units.length);
            let times = of(units.time);
            if unit == units.speed {
                if let (Some(length), Some(time)) = (
                    self.random.pick(&lengths).cloned(),
                    self.random.pick(&times).cloned(),
                ) {
                    return Expression::Binary {
                        operator: Operator::Divide,
                        left: Box::new(Expression::Variable(length)),
                        right: Box::new(Expression::Variable(time)),
                    };
                }
            } else if let (Some(left), Some(right)) = (
                self.random.pick(&lengths).cloned(),
                self.random.pick(&lengths).cloned(),
            ) {
                return Expression::Binary {
                    operator: Operator::Multiply,
                    left: Box::new(Expression::Variable(left)),
                    right: Box::new(Expression::Variable(right)),
                };
            }
            // A speed or area with no length and time to make it from is written as one.
            return Expression::FloatWithUnit {
                number: self.positive_float(),
                unit,
            };
        }
        // `glarger` keeps whatever unit its unit parameter is filled with (5.8, 2.16).
        let unit_parameter = self
            .generic_calls()
            .is_some_and(|generics| generics.unit_parameter);
        if unit_parameter && !point && !span && self.random.chance(25) {
            if let (Some(left), Some(right)) = (
                self.random.pick(&same).cloned(),
                self.random.pick(&same).cloned(),
            ) {
                return Expression::Call {
                    function: "glarger".to_string(),
                    arguments: vec![
                        (None, Expression::Variable(left)),
                        (None, Expression::Variable(right)),
                    ],
                    handling: Handling::Crash,
                };
            }
        }
        if depth > 0 && roll < 8 {
            if point {
                // A point plus a span, or a point converted from the other scale.
                let spans = of(leak(format!("{bare}.span")));
                let points = of(unit);
                if let (Some(at), Some(by)) = (
                    self.random.pick(&points).cloned(),
                    self.random.pick(&spans).cloned(),
                ) {
                    return Expression::Binary {
                        operator: Operator::Add,
                        left: Box::new(Expression::Variable(at)),
                        right: Box::new(Expression::Variable(by)),
                    };
                }
                let other = match bare {
                    found if found == units.celsius => units.kelvin,
                    found if found == units.kelvin => units.celsius,
                    found if found == units.decibels => units.milliwatts,
                    _ => units.decibels,
                };
                if let Some(source) = self
                    .random
                    .pick(&of(leak(format!("{other}.point"))))
                    .cloned()
                {
                    return Expression::Convert {
                        variable: source,
                        narrowing: "",
                        target: Type::FloatUnit(unit),
                    };
                }
            } else if span
                && (bare == units.celsius || bare == units.kelvin)
                && self.random.chance(30)
            {
                // A span converted from the other scale, which only scales it.
                let other = if bare == units.celsius {
                    units.kelvin
                } else {
                    units.celsius
                };
                if let Some(source) = self
                    .random
                    .pick(&of(leak(format!("{other}.span"))))
                    .cloned()
                {
                    return Expression::Convert {
                        variable: source,
                        narrowing: "",
                        target: Type::FloatUnit(unit),
                    };
                }
            } else if span {
                // One point minus another, or two spans added.
                let points = of(leak(format!("{bare}.point")));
                if let (Some(later), Some(earlier)) = (
                    self.random.pick(&points).cloned(),
                    self.random.pick(&points).cloned(),
                ) {
                    return Expression::Binary {
                        operator: Operator::Subtract,
                        left: Box::new(Expression::Variable(later)),
                        right: Box::new(Expression::Variable(earlier)),
                    };
                }
                if let (Some(left), Some(right)) = (
                    self.random.pick(&same).cloned(),
                    self.random.pick(&same).cloned(),
                ) {
                    return Expression::Binary {
                        operator: Operator::Add,
                        left: Box::new(Expression::Variable(left)),
                        right: Box::new(Expression::Variable(right)),
                    };
                }
            } else if let (Some(left), Some(right)) = (
                self.random.pick(&same).cloned(),
                self.random.pick(&same).cloned(),
            ) {
                return Expression::Binary {
                    operator: Operator::Add,
                    left: Box::new(Expression::Variable(left)),
                    right: Box::new(Expression::Variable(right)),
                };
            }
        }
        // A literal says its unit's name, with no `.point` or `.span`, which only a declared type
        // says, so a point or span compared with something is compared with a variable.
        if point || span {
            if let Some(name) = self.random.pick(&same).cloned() {
                return Expression::Variable(name);
            }
        }
        Expression::FloatWithUnit {
            number: self.positive_float(),
            unit: leak(bare.to_string()),
        }
    }

    /// A float literal that isn't negative, exact in quarters.
    fn positive_float(&mut self) -> String {
        let whole = self.random.between(0, 40);
        let quarter = *self.random.pick(&["0", "25", "5", "75"]).unwrap_or(&"5");
        format!("{whole}.{quarter}")
    }

    /// A variable with a unit, its unit taken off with `as` (2.16).
    fn unit_taken_off(&mut self, scope: &Scope, kind: Kind) -> Option<Expression> {
        let found: Vec<String> = scope
            .variables
            .iter()
            .filter(|(_, found)| matches!(found, Type::Unit(other, ..) if *other == kind))
            .map(|(name, _)| name.clone())
            .collect();
        let variable = self.random.pick(&found)?.clone();
        Some(Expression::Convert {
            variable,
            narrowing: "",
            target: Type::Integer(kind),
        })
    }

    /// How many of its base a unit is.
    fn unit_factor(&self, unit: &str) -> i128 {
        self.units
            .iter()
            .find(|(found, _)| *found == unit)
            .map_or(1, |(_, factor)| *factor)
    }

    /// The units in the same family as this one, with their factors.
    fn same_family(&self, unit: &str) -> Vec<(&'static str, i128)> {
        let family = unit.split('.').next().unwrap_or(unit);
        self.units
            .iter()
            .filter(|(found, _)| found.split('.').next() == Some(family))
            .copied()
            .collect()
    }

    /// A variable with a unit, now and then bounded by values written in a unit that converts
    /// to it exactly, and a first value of that unit (2.16, 2.5).
    fn unit_let(&mut self, scope: &mut Scope) -> Statement {
        let Some((mut unit, mut factor)) = self.random.pick(&self.units.clone()).copied() else {
            return self.list_statement(scope);
        };
        let mut kind = *self
            .random
            .pick(&[Kind::I32, Kind::I64, Kind::U32, Kind::U64])
            .unwrap_or(&Kind::I32);
        // Usually the family and kind of a unit variable already here, so the two can convert.
        let existing: Vec<(Kind, &'static str)> = scope
            .variables
            .iter()
            .filter_map(|(_, found)| match found {
                Type::Unit(kind, unit, _) => Some((*kind, *unit)),
                _ => None,
            })
            .collect();
        if let (true, Some((found_kind, found_unit))) =
            (self.random.chance(70), self.random.pick(&existing).copied())
        {
            if let Some((other, other_factor)) =
                self.random.pick(&self.same_family(found_unit)).copied()
            {
                kind = found_kind;
                unit = other;
                factor = other_factor;
            }
        }
        let mut bounds = None;
        if self.random.chance(30) {
            // A bound in a unit that's a whole number of this one converts exactly.
            let bigger: Vec<(&'static str, i128)> = self
                .same_family(unit)
                .into_iter()
                .filter(|(_, found)| found % factor == 0)
                .collect();
            if let (Some((low_unit, low_factor)), Some((high_unit, high_factor))) = (
                self.random.pick(&bigger).copied(),
                self.random.pick(&bigger).copied(),
            ) {
                let low = self.random.between(0, 5);
                let high_count = self.random.between(1, 20);
                let low_in = low * low_factor / factor;
                let high_in = high_count * high_factor / factor;
                if low_in <= high_in && kind.range().1 >= high_in {
                    let written: &'static str = Box::leak(
                        format!("min: {low} {low_unit}, max: {high_count} {high_unit}")
                            .into_boxed_str(),
                    );
                    bounds = Some((written, low_in, high_in));
                }
            }
        }
        let value_type = Type::Unit(kind, unit, bounds);
        let mut value = self.unit_value(scope, kind, unit, LIMITS.depth);
        // Often a variable in another unit of the family, converted (2.16).
        let sources: Vec<(String, i128)> = scope
            .variables
            .iter()
            .filter_map(|(name, found)| match found {
                Type::Unit(other, written, _) if *other == kind && *written != unit => {
                    let source = self.unit_factor(written);
                    let same = self
                        .same_family(unit)
                        .iter()
                        .any(|(member, _)| member == written);
                    same.then(|| (name.clone(), source))
                }
                _ => None,
            })
            .collect();
        if let (true, Some((variable, source))) =
            (self.random.chance(60), self.random.pick(&sources).cloned())
        {
            let narrowing = if source % factor == 0 {
                ""
            } else {
                *self
                    .random
                    .pick(&[".trunc", ".floor", ".exact"])
                    .unwrap_or(&".trunc")
            };
            value = Expression::Convert {
                variable,
                narrowing,
                target: Type::Unit(kind, unit, None),
            };
        }
        // A bare literal that can't fit a bounded type is refused, so it's one that fits;
        // anything else is checked while running (2.5).
        if let Some((_, low, high)) = bounds {
            if matches!(value, Expression::WithUnit { .. }) || self.random.chance(50) {
                value = Expression::WithUnit {
                    number: self.random.between(low, high.min(low + 1000)),
                    unit,
                };
            }
        }
        let name = self.name("uv");
        scope.variables.push((name.clone(), value_type));
        Statement::Let {
            name,
            value_type,
            value,
        }
    }

    /// A value in a unit: a number with the unit, a variable in it, two added with `+` or a
    /// `.wrap` or `.saturate` word, or a variable in another unit of the family converted with
    /// `as` to a smaller unit, or `as.trunc`, `as.floor` or `as.exact` to a bigger one (2.16).
    fn unit_value(
        &mut self,
        scope: &Scope,
        kind: Kind,
        unit: &'static str,
        depth: usize,
    ) -> Expression {
        let roll = self.random.between(0, 9);
        if roll < 3 {
            let found: Vec<String> = scope
                .variables
                .iter()
                .filter(|(_, found)| {
                    matches!(found, Type::Unit(other, written, _) if *other == kind && *written == unit)
                })
                .map(|(name, _)| name.clone())
                .collect();
            if let Some(name) = self.random.pick(&found).cloned() {
                return Expression::Variable(name);
            }
        }
        if roll == 3 || roll == 4 {
            let factor = self.unit_factor(unit);
            let family = self.same_family(unit);
            let found: Vec<(String, i128)> = scope
                .variables
                .iter()
                .filter_map(|(name, found)| match found {
                    Type::Unit(other, written, _) if *other == kind && *written != unit => family
                        .iter()
                        .find(|(member, _)| member == written)
                        .map(|(_, source)| (name.clone(), *source)),
                    _ => None,
                })
                .collect();
            if let Some((variable, source)) = self.random.pick(&found).cloned() {
                let narrowing = if source % factor == 0 {
                    ""
                } else {
                    *self
                        .random
                        .pick(&[".trunc", ".floor", ".exact"])
                        .unwrap_or(&".trunc")
                };
                return Expression::Convert {
                    variable,
                    narrowing,
                    target: Type::Unit(kind, unit, None),
                };
            }
        }
        if depth > 0 && (roll == 5 || roll == 6) {
            let operator = *self
                .random
                .pick(&[
                    Operator::Add,
                    Operator::PlusWrap,
                    Operator::MinusWrap,
                    Operator::PlusSaturate,
                ])
                .unwrap_or(&Operator::PlusWrap);
            let left = self.unit_value(scope, kind, unit, depth - 1);
            let right = self.unit_value(scope, kind, unit, depth - 1);
            return Expression::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Expression::WithUnit {
            number: self.random.between(0, 60),
            unit,
        }
    }

    /// A struct with one to four integer fields, in any of the layouts (7.1).
    fn struct_decl(&mut self) -> StructDecl {
        let name = self.name("rec");
        let layout = *self.random.pick(&Layout::ALL).unwrap_or(&Layout::Written);
        let fields: Vec<(String, Kind)> = (0..self.random.between(1, 4))
            .map(|_| (self.name("fd"), self.integer_kind()))
            .collect();
        // Now and then a bounded field, which every value put in it is checked against (2.5).
        let mut bounds = Vec::new();
        for (field, kind) in &fields {
            if let (true, Type::Bounded(_, low, high)) =
                (self.random.chance(20), self.bounded_type_of(*kind))
            {
                bounds.push((field.clone(), low, high));
            }
        }
        StructDecl {
            name,
            layout,
            fields,
            bounds,
        }
    }

    /// A value for a struct's field: one that fits a bounded field, or any of its kind (2.5).
    fn field_value(
        &mut self,
        scope: &Scope,
        declared: &StructDecl,
        field: &str,
        kind: Kind,
    ) -> Expression {
        match declared.field_type(field, kind) {
            Type::Bounded(kind, low, high) => self.bounded_value(scope, kind, low, high),
            _ => self.expression(scope, Type::Integer(kind), LIMITS.depth),
        }
    }

    /// A bounded integer type, its range inside its kind's (2.5).
    fn bounded_type(&mut self) -> Type {
        let kind = self.integer_kind();
        let (low, high) = kind.range();
        let low = self.random.between(low.max(-50), high.min(50));
        let high = self.random.between(low, high.min(low + 100));
        Type::Bounded(kind, low, high)
    }

    /// A first value for a bounded integer: a literal it holds, which needs no check, or a
    /// variable of its kind, which is checked while running (2.5).
    fn bounded_value(&mut self, scope: &Scope, kind: Kind, low: i128, high: i128) -> Expression {
        if self.random.chance(40) {
            if let Some(variable) = self.variable_of(scope, Type::Integer(kind)) {
                return variable;
            }
        }
        Expression::Integer(self.random.between(low, high))
    }

    fn integer_kind(&mut self) -> Kind {
        *self.random.pick(&Kind::ALL).unwrap_or(&Kind::I32)
    }

    /// A new struct, built with `[ ]`, or a change to a field of one in scope (7.2, 7.3).
    fn record_statement(&mut self, scope: &mut Scope) -> Statement {
        // The struct with its own operators is often made from two of it (3.10, 5.8).
        if self.random.chance(35) {
            if let Some(made) = self.generic_struct(scope) {
                return made;
            }
        }
        let live: Vec<(String, usize)> = scope
            .records
            .iter()
            .filter(|(name, _)| !self.given_away.contains(name))
            .cloned()
            .collect();
        if let Some((variable, index)) = self.random.pick(&live).cloned() {
            let struct_name = self
                .structs
                .get(index)
                .map(|declared| declared.name.clone());
            let changing: Vec<MethodDecl> = self
                .methods
                .iter()
                .filter(|method| {
                    method.changes.is_some() && Some(&method.struct_name) == struct_name.as_ref()
                })
                .cloned()
                .collect();
            // A pure function calls only pure ones, and a method isn't one (5.9).
            if let (true, Some(method)) = (
                !self.pure_only && self.random.chance(40),
                self.random.pick(&changing).cloned(),
            ) {
                let value = self.literal(method.kind);
                return Statement::CallChanging {
                    receiver: variable,
                    name: method.name,
                    value,
                    labelled: method.labelled,
                };
            }
            if self.random.chance(50) {
                if let Some(declared) = self.structs.get(index).cloned() {
                    if let Some((field, kind)) = self.random.pick(&declared.fields).cloned() {
                        let mut value = self.field_value(scope, &declared, &field, kind);
                        // Whether a field can change to a value that calls a method on its own
                        // struct isn't decided, so that value is a literal instead.
                        let written = Style {
                            fewer_brackets: false,
                        }
                        .expression(&value);
                        if written.contains(&format!("${variable}.%")) {
                            value = self.literal(kind);
                        }
                        return Statement::ChangeField {
                            variable,
                            field,
                            value,
                        };
                    }
                }
            }
        }
        let count = i128::try_from(self.structs.len()).unwrap_or(0);
        if count == 0 {
            return self.array_statement(scope);
        }
        let index = usize::try_from(self.random.between(0, count - 1)).unwrap_or(0);
        let Some(declared) = self.structs.get(index).cloned() else {
            return self.array_statement(scope);
        };
        let fields = declared
            .fields
            .iter()
            .map(|(field, kind)| {
                (
                    field.clone(),
                    self.field_value(scope, &declared, field, *kind),
                )
            })
            .collect();
        let name = self.name("r");
        scope.records.push((name.clone(), index));
        Statement::LetStruct {
            name,
            struct_name: declared.name,
            fields,
        }
    }

    /// A new bounded list, from `[ ]` or `empty`, an `append` to one in scope, which stops the
    /// program when it's full, or a change to one of its items, which is checked while
    /// running (2.10, 2.17, 8.9).
    fn list_statement(&mut self, scope: &mut Scope) -> Statement {
        if !scope.lists.is_empty() && self.random.chance(30) {
            if let Some(statement) = self.list_view(scope) {
                return statement;
            }
        }
        if let Some((list, kind, max)) = self.random.pick(&scope.lists).cloned() {
            if self.random.chance(50) {
                // The value doesn't read the list, since reading a list in the call that lends
                // it to change isn't decided.
                let lists = std::mem::take(&mut scope.lists);
                let value = self.expression(scope, Type::Integer(kind), LIMITS.depth);
                scope.lists = lists;
                let allocator = self
                    .growable
                    .iter()
                    .find(|(found, _)| *found == list)
                    .map(|(_, allocator)| allocator.clone());
                return Statement::Append {
                    handled: self.random.chance(30),
                    list,
                    value,
                    allocator,
                };
            }
            if self.random.chance(50) {
                let position = if self.random.chance(50) {
                    Position::Pos(self.random.between(1, max))
                } else {
                    Position::Offset(self.random.between(0, max - 1))
                };
                let value = self.expression(scope, Type::Integer(kind), LIMITS.depth);
                return Statement::ChangeItem {
                    array: list,
                    position,
                    value,
                };
            }
        }
        let kind = self.integer_kind();
        let max = self.random.between(1, 6);
        let count = self.random.between(0, max);
        let items = (0..count)
            .map(|_| self.expression(scope, Type::Integer(kind), LIMITS.depth))
            .collect();
        let name = self.name("l");
        scope.lists.push((name.clone(), kind, max));
        // In `main`, or a function with a heap, a list may grow, on the heap or in `main`'s
        // arena (D321, D330).
        let allocator = ((self.in_main || self.has_heap) && self.random.chance(40)).then(|| {
            if self.in_main && self.arena.is_some() && self.random.chance(60) {
                "arena".to_string()
            } else {
                "heap".to_string()
            }
        });
        if let Some(allocator) = &allocator {
            self.growable.push((name.clone(), allocator.clone()));
        }
        // Now and then it says how it grows: only what each append needs, or in steps (2.10).
        let grow = allocator
            .as_ref()
            .and_then(|_| match self.random.between(0, 5) {
                0 => Some("exact".to_string()),
                1 => Some(format!("by {} items", self.random.between(1, 8))),
                2 => Some(format!("by {} kib", self.random.between(1, 2))),
                _ => None,
            });
        Statement::LetList {
            name,
            kind,
            max,
            items,
            allocator,
            grow,
        }
    }

    /// A new array, or a change to an item of one in scope.
    fn array_statement(&mut self, scope: &mut Scope) -> Statement {
        if self.random.chance(20) {
            if let Some(statement) = self.chunks(scope) {
                return statement;
            }
        }
        if let Some((array, kind, len)) = self.random.pick(&scope.arrays).cloned() {
            if self.random.chance(60) {
                let position = self.position(scope, len);
                let value = self.expression(scope, Type::Integer(kind), LIMITS.depth);
                return Statement::ChangeItem {
                    array,
                    position,
                    value,
                };
            }
        }
        let kind = *self.random.pick(&Kind::ALL).unwrap_or(&Kind::I32);
        let len = self.random.between(1, 5);
        let filling = if self.random.chance(30) {
            Filling::All(self.literal(kind))
        } else {
            Filling::Items(
                (0..len)
                    .map(|_| self.expression(scope, Type::Integer(kind), LIMITS.depth))
                    .collect(),
            )
        };
        let name = self.name("a");
        scope.arrays.push((name.clone(), kind, len));
        Statement::LetArray {
            name,
            kind,
            len,
            filling,
        }
    }

    /// An item's place: a literal inside the array, counted either way, or now and then a
    /// variable's value, which may be outside it and is checked while running.
    fn position(&mut self, scope: &Scope, len: i128) -> Position {
        let integers: Vec<String> = scope
            .variables
            .iter()
            .filter(|(_, found)| matches!(found, Type::Integer(_)))
            .map(|(name, _)| name.clone())
            .collect();
        if self.random.chance(20) {
            if let Some(name) = self.random.pick(&integers).cloned() {
                return Position::PosOf(name);
            }
        }
        if self.random.chance(50) {
            Position::Pos(self.random.between(1, len))
        } else {
            Position::Offset(self.random.between(0, len - 1))
        }
    }

    /// `for` over a literal range that always reaches its end exactly, since a range that can't
    /// is refused and one that steps past it is a reading (4.5, 12.1). Its counter isn't used,
    /// since what type it has isn't stated.
    fn range_loop(&mut self, scope: &mut Scope, depth: usize, prints: bool) -> Statement {
        let counter = self.name("i");
        let from = self.random.between(-5, 5);
        let step = if self.random.chance(50) {
            None
        } else {
            Some(*self.random.pick(&[1, 2, 3, -1, -2]).unwrap_or(&1))
        };
        let inclusive = self.random.chance(50);
        let lowest = i128::from(!inclusive);
        let rounds = self.random.between(lowest, 4);
        // Now and then the end lies past the last step, which the range stops short of (4.5).
        let by: i128 = step.unwrap_or(1);
        let past = self.random.between(0, by.abs() - 1) * by.signum();
        let mut end = Expression::Integer(from + by * rounds + past);
        // Or the end is only known while running, kept small, so the range may run zero rounds
        // with no error (4.5).
        if by == 1 && self.random.chance(25) {
            if let Some(Expression::Variable(variable)) =
                self.variable_of(scope, Type::Integer(Kind::I32))
            {
                end = Expression::Binary {
                    operator: Operator::RemainderTrunc,
                    left: Box::new(Expression::Variable(variable)),
                    right: Box::new(Expression::Integer(6)),
                };
            }
        }
        // A range of literals counts in `i32`, and its counter can be read inside (4.5).
        let held_variables = scope.variables.len();
        scope
            .variables
            .push((counter.clone(), Type::Integer(Kind::I32)));
        let name = self.random.chance(30).then(|| self.name("lp"));
        self.loop_names.extend(name.clone());
        let mut body = self.block(scope, 3, depth + 1, (prints, true));
        self.break_named(scope, &mut body);
        if name.is_some() {
            self.loop_names.pop();
        }
        scope.variables.truncate(held_variables);
        Statement::For {
            name,
            counter,
            from,
            end,
            inclusive,
            step,
            body,
        }
    }

    /// A value of this type. Comparisons and calls only go where a typed value is around, so a
    /// literal never has to guess its type.
    fn expression(&mut self, scope: &Scope, value_type: Type, depth: usize) -> Expression {
        match value_type {
            Type::Bool => self.boolean(scope, depth),
            // Into a bounded integer, anything not a literal is checked while running (2.5).
            Type::Integer(kind) | Type::Bounded(kind, ..) => self.integer(scope, kind, depth),
            Type::Text(bytes) => self.text(scope, bytes),
            Type::Simd(kind, lanes) => self.lanes(scope, kind, lanes, depth),
            Type::Secret(kind) => self.secret(scope, kind, depth),
            Type::SecretBool => self.secret_bool(scope, depth),
            Type::Unit(kind, unit, _) => self.unit_value(scope, kind, unit, depth),
            Type::FloatUnit(unit) => self.float_unit_value(scope, unit, depth),
            Type::Float(wide) => self.float(scope, wide, depth),
            Type::OtherFloat(wide, rules) => self.other_float(scope, wide, rules, depth),
            // Into a bounded float, a literal that fits, or a value checked while running (2.6).
            Type::BoundedFloat(wide, low, high) => {
                if self.random.chance(40) {
                    if let Some(variable) = self.variable_of(scope, Type::Float(wide)) {
                        return variable;
                    }
                }
                // Strictly inside, so a quarter either way still fits.
                if high - low < 2 {
                    return Expression::Float(format!("{low}.0"));
                }
                let whole = self.random.between(low + 1, high - 1);
                let quarter = *self.random.pick(&["0", "25", "5", "75"]).unwrap_or(&"5");
                Expression::Float(format!("{whole}.{quarter}"))
            }
        }
    }

    /// A `.ieee` or `.fast` float: a literal, a variable of the type, or maths on two, where a
    /// `.ieee` one now and then divides by zero, making infinity or NaN (2.6).
    fn other_float(
        &mut self,
        scope: &Scope,
        wide: bool,
        rules: &'static str,
        depth: usize,
    ) -> Expression {
        let value_type = Type::OtherFloat(wide, rules);
        let roll = self.random.between(0, 9);
        if depth == 0 || roll < 4 {
            if self.random.chance(50) {
                if let Some(variable) = self.variable_of(scope, value_type) {
                    return variable;
                }
            }
            return self.float_literal();
        }
        if roll == 4 && rules == "ieee" {
            let top = self.other_float(scope, wide, rules, depth - 1);
            return Expression::Binary {
                operator: Operator::Divide,
                left: Box::new(top),
                right: Box::new(Expression::Float("0.0".to_string())),
            };
        }
        let operator = *self
            .random
            .pick(&[
                Operator::Add,
                Operator::Subtract,
                Operator::Multiply,
                Operator::Divide,
            ])
            .unwrap_or(&Operator::Add);
        let left = self.other_float(scope, wide, rules, depth - 1);
        let right = self.other_float(scope, wide, rules, depth - 1);
        Expression::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn variable_of(&mut self, scope: &Scope, value_type: Type) -> Option<Expression> {
        // A bounded value goes into its plain integer with no check (2.5).
        let found: Vec<&String> = scope
            .variables
            .iter()
            .filter(|(_, found)| {
                *found == value_type
                    || matches!((value_type, found), (Type::Integer(kind), Type::Bounded(bounded, ..)) if kind == *bounded)
            })
            .map(|(name, _)| name)
            .collect();
        self.random
            .pick(&found)
            .map(|name| Expression::Variable((*name).clone()))
    }

    fn literal(&mut self, kind: Kind) -> Expression {
        let (low, high) = kind.range();
        let value = if self.random.chance(70) {
            self.random.between(low.max(-20), high.min(20))
        } else {
            self.random.between(low, high)
        };
        Expression::Integer(value)
    }

    #[allow(clippy::too_many_lines)] // One roll per kind of integer value, read top to bottom.
    fn integer(&mut self, scope: &Scope, kind: Kind, depth: usize) -> Expression {
        let roll = self.random.between(0, 9);
        if depth == 0 || roll < 3 {
            return match self.variable_of(scope, Type::Integer(kind)) {
                Some(variable) if self.random.chance(60) => variable,
                _ => self.literal(kind),
            };
        }
        if roll == 3 {
            if let Some(call) = self.call(scope, kind, depth) {
                return call;
            }
        }
        if roll == 4 && self.random.chance(40) {
            if let Some(call) = self.generic_integer(scope, kind, depth) {
                return call;
            }
        }
        // A `fn.pure.baked` function, called while running, or baked in place with `bake`
        // from a value known while compiling (5.11).
        if roll == 4 && !self.no_calls && self.random.chance(30) {
            if self.random.chance(50) {
                let argument = self.constant_value(scope, kind, 1);
                let call = self.baked_call(kind, argument);
                return Expression::Bake(Box::new(call));
            }
            let argument = self.integer(scope, kind, depth.saturating_sub(1));
            return self.baked_call(kind, argument);
        }
        // `tcount` takes a `str`: a literal, or a view, which passes with no word (2.9, D368).
        if roll == 4
            && kind == Kind::I32
            && self.strings
            && !self.no_calls
            && !self.pure_only
            && self.random.chance(40)
        {
            {
                let text = match self.text(scope, None) {
                    Expression::Text(text) if text.is_empty() => Expression::Text("o".to_string()),
                    other => other,
                };
                let number = self.integer(scope, kind, depth.saturating_sub(1));
                return Expression::Call {
                    function: "tcount".to_string(),
                    arguments: vec![(None, text), (None, number)],
                    handling: Handling::Crash,
                };
            }
        }
        // A recursive call, kept to a few levels by `mod.trunc` (5.1).
        if roll == 4
            && kind == Kind::U64
            && self.recursion
            && !self.pure_only
            && !self.no_calls
            && self.random.chance(50)
        {
            let inner = self.integer(scope, kind, depth.saturating_sub(1));
            let function = *self
                .random
                .pick(&["rsum", "reven", "rodd"])
                .unwrap_or(&"rsum");
            return Expression::Call {
                function: function.to_string(),
                arguments: vec![(
                    None,
                    Expression::Binary {
                        operator: Operator::RemainderTrunc,
                        left: Box::new(inner),
                        right: Box::new(Expression::Integer(30)),
                    },
                )],
                handling: Handling::Crash,
            };
        }
        if roll == 5 {
            let arrays: Vec<(String, i128)> = scope
                .arrays
                .iter()
                .chain(&scope.lists)
                .filter(|(_, found, _)| *found == kind)
                .map(|(name, _, len)| (name.clone(), *len))
                .collect();
            if let Some((array, len)) = self.random.pick(&arrays).cloned() {
                let position = self.position(scope, len);
                return Expression::Item { array, position };
            }
        }
        if roll == 7 {
            if let Some(reduction) = self.reduction(scope, kind) {
                return reduction;
            }
        }
        if roll == 8 {
            if let Some(call) = self.method_call(scope, kind) {
                return call;
            }
        }
        if roll == 9 {
            if let Some(stripped) = self.unit_taken_off(scope, kind) {
                return stripped;
            }
        }
        if roll == 6 {
            let fields: Vec<(String, String)> = scope
                .records
                .iter()
                .filter(|(name, _)| !self.given_away.contains(name))
                .flat_map(|(variable, index)| {
                    self.structs
                        .get(*index)
                        .map(|declared| declared.fields.clone())
                        .unwrap_or_default()
                        .into_iter()
                        .filter(|(_, found)| *found == kind)
                        .map(move |(field, _)| (variable.clone(), field))
                })
                .collect();
            let mut fields = fields;
            fields.extend(
                scope
                    .case_fields
                    .iter()
                    .filter(|(_, _, found)| *found == kind)
                    .map(|(variable, field, _)| (variable.clone(), field.clone())),
            );
            if let Some((variable, field)) = self.random.pick(&fields).cloned() {
                return Expression::Field { variable, field };
            }
        }
        if roll == 4 && kind.range().0 < 0 {
            let inner = self.integer(scope, kind, depth - 1);
            return Expression::Negate(Box::new(inner));
        }
        let mut operator = *self.random.pick(&Operator::ALL).unwrap_or(&Operator::Add);
        // Whether plain `-` and `x` follow `+`'s overflow row isn't decided (12.3), so they're
        // usually their `.wrap` forms here, to keep more programs in scope.
        if self.random.chance(70) {
            operator = match operator {
                Operator::Subtract => Operator::MinusWrap,
                Operator::Multiply => Operator::TimesWrap,
                other => other,
            };
        }
        // Shifts stay on unsigned types, by a literal amount below the width, since shifting a
        // negative or by too much isn't decided (12.1, 12.3).
        if operator.is_shift() {
            if kind.range().0 < 0 {
                return self.literal(kind);
            }
            let left = self.integer(scope, kind, depth - 1);
            let amount = self.random.between(0, kind.bits() - 1);
            return Expression::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(Expression::Integer(amount)),
            };
        }
        let left = self.integer(scope, kind, depth - 1);
        // Dividing by zero is a check every time, so a divisor is usually a literal that isn't
        // zero, to reach more of what follows.
        let right = if operator.divides() && self.random.chance(70) {
            let (_, high) = kind.range();
            Expression::Integer(self.random.between(1, high.min(20)))
        } else {
            self.integer(scope, kind, depth - 1)
        };
        Expression::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// A call to a function made earlier with this result. One that can fail is handled with
    /// `?` where the function being made can pass its failures up, `??` and a value, or not at
    /// all, so a failure stops the program (8.4, 8.5, 8.9).
    /// A float: a literal, exact like `1.25` or marked inexact like `~0.1`, a variable, or
    /// `+`, `-`, `x` or `div` on two (2.6, 2.7, 3.2, 3.3).
    fn float(&mut self, scope: &Scope, wide: bool, depth: usize) -> Expression {
        let roll = self.random.between(0, 9);
        if depth == 0 || roll < 4 {
            if self.random.chance(50) {
                if let Some(variable) = self.variable_of(scope, Type::Float(wide)) {
                    return variable;
                }
            }
            return self.float_literal();
        }
        if roll == 4 {
            let inner = self.float(scope, wide, depth - 1);
            return Expression::Negate(Box::new(inner));
        }
        // A `.ieee` float crosses into `.stable` with `as`, which fails on NaN or infinity (2.6).
        if roll == 6 || self.random.chance(15) {
            let found: Vec<String> = scope
                .variables
                .iter()
                .filter(|(_, found)| *found == Type::OtherFloat(wide, "ieee"))
                .map(|(name, _)| name.clone())
                .collect();
            if let Some(variable) = self.random.pick(&found).cloned() {
                return Expression::Convert {
                    variable,
                    narrowing: "",
                    target: Type::Float(wide),
                };
            }
        }
        // `std::math::sin` takes an angle in radians (2.16).
        if roll == 5 && wide {
            let angles: Vec<String> = scope
                .variables
                .iter()
                .filter(|(_, found)| *found == Type::FloatUnit(RADIANS))
                .map(|(name, _)| name.clone())
                .collect();
            if let Some(angle) = self.random.pick(&angles).cloned() {
                return Expression::Call {
                    function: "std::math::sin".to_string(),
                    arguments: vec![(None, Expression::Variable(angle))],
                    handling: Handling::Crash,
                };
            }
        }
        let operator = *self
            .random
            .pick(&[
                Operator::Add,
                Operator::Subtract,
                Operator::Multiply,
                Operator::Divide,
            ])
            .unwrap_or(&Operator::Add);
        let left = self.float(scope, wide, depth - 1);
        let right = self.float(scope, wide, depth - 1);
        Expression::Binary {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    /// A float literal: quarters are exact in both widths and take no mark, and tenths that
    /// aren't halves can't be held exactly, so they say `~` (2.7).
    fn float_literal(&mut self) -> Expression {
        // Now and then with an exponent, exact or marked `~`, never negative, which isn't decided
        // (2.7, D165, 12.2).
        if self.random.chance(8) {
            let written = *self
                .random
                .pick(&[
                    "exponent 1.25e2",
                    "exponent 7.75e3",
                    "exponent -3.5e1",
                    "exponent ~6.02214076e23",
                ])
                .unwrap_or(&"exponent 1.25e2");
            return Expression::Float(written.to_string());
        }
        let whole = self.random.between(-20, 20);
        let sign = if whole < 0 { "-" } else { "" };
        let whole = whole.abs();
        if self.random.chance(60) {
            let quarter = *self.random.pick(&["0", "25", "5", "75"]).unwrap_or(&"5");
            Expression::Float(format!("{sign}{whole}.{quarter}"))
        } else {
            let tenth = *self
                .random
                .pick(&["1", "2", "3", "4", "6", "7", "8", "9"])
                .unwrap_or(&"1");
            Expression::Float(format!("~{sign}{whole}.{tenth}"))
        }
    }

    fn call(&mut self, scope: &Scope, kind: Kind, depth: usize) -> Option<Expression> {
        let (function, arguments, failures) = self.plain_call(scope, kind, depth)?;
        let handling = if failures.is_empty() {
            Handling::Crash
        } else if self.failing.is_some() && self.random.chance(40) {
            if let Some(failing) = &mut self.failing {
                for failure in failures {
                    if !failing.iter().any(|found| found.name == failure.name) {
                        failing.push(failure);
                    }
                }
            }
            Handling::Pass
        } else if self.random.chance(70) {
            Handling::Fallback(Box::new(self.literal(kind)))
        } else {
            Handling::Crash
        };
        Some(Expression::Call {
            function,
            arguments,
            handling,
        })
    }

    /// A function made earlier with this result, its arguments, and what it can fail with.
    fn plain_call(
        &mut self,
        scope: &Scope,
        kind: Kind,
        depth: usize,
    ) -> Option<(String, Arguments, Vec<FailureDecl>)> {
        if self.no_calls {
            return None;
        }
        let candidates: Vec<Function> = self
            .functions
            .iter()
            .filter(|function| function.result == Type::Integer(kind))
            .filter(|function| function.pure || !self.pure_only)
            .cloned()
            .collect();
        let function = self.random.pick(&candidates)?.clone();
        let mut unlabelled = Vec::new();
        let mut labelled = Vec::new();
        for (parameter, parameter_type) in &function.parameters {
            let label = function
                .labels
                .iter()
                .find(|(found, _)| found == parameter)
                .map(|(_, label)| label.clone());
            let defaulted = function
                .defaults
                .iter()
                .any(|(found, _)| found == parameter);
            // A label with a default can be left off (5.2).
            if label.is_some() && defaulted && self.random.chance(40) {
                continue;
            }
            let value = match (parameter_type, &label) {
                // A label carrying a unit takes a plain number, which gets the label's unit.
                (Type::Unit(kind, ..), Some(label)) if label.starts_with("at.") => {
                    self.literal(*kind)
                }
                // A literal for a bounded parameter fits it, and anything else is checked (2.5).
                (Type::Bounded(kind, low, high), _) => {
                    self.bounded_value(scope, *kind, *low, *high)
                }
                _ => self.expression(scope, *parameter_type, depth.saturating_sub(1)),
            };
            match label {
                Some(label) => labelled.push((Some(label), value)),
                None => unlabelled.push((None, value)),
            }
        }
        // `labels.any_order` lets the labelled ones come in any order (5.2).
        if function.any_order {
            for at in (1..labelled.len()).rev() {
                let other = usize::try_from(self.random.between(0, at as i128)).unwrap_or(0);
                labelled.swap(at, other);
            }
        }
        unlabelled.extend(labelled);
        Some((function.name, unlabelled, function.failures))
    }

    /// `let` from a call that can fail, with an `otherwise:` arm for each failure, each using
    /// a value `instead` (8.6).
    fn let_otherwise(&mut self, scope: &mut Scope, kind: Kind, in_loop: bool) -> Option<Statement> {
        let (function, arguments, failures) = self.plain_call(scope, kind, LIMITS.depth)?;
        if failures.is_empty() {
            return None;
        }
        // A call that can fail only with the named set can use one of its handlers, the one
        // that returns marked `*`, and only where `return void;` fits (8.7, 8.8).
        let handler = self.failure_family.clone().and_then(|family| {
            let only_family = failures
                .iter()
                .all(|failure| failure.family.as_ref() == Some(&family.name));
            if !only_family || !self.random.chance(50) {
                return None;
            }
            Some(if self.in_main && self.random.chance(30) {
                format!("{}.*bail", family.handler)
            } else {
                format!("{}.fallback", family.handler)
            })
        });
        let mut arms = Vec::new();
        for failure in failures {
            arms.push(self.arm(scope, kind, in_loop, failure));
        }
        let name = self.name("v");
        scope.variables.push((name.clone(), Type::Integer(kind)));
        Some(Statement::LetOtherwise {
            name,
            value_type: Type::Integer(kind),
            call: Expression::Call {
                function,
                arguments,
                handling: Handling::Crash,
            },
            arms,
            handler,
        })
    }

    /// What one `otherwise` arm does: gives a value instead, now and then the failure's own
    /// data, or returns, breaks out of the loop around, or fails with one of the function's
    /// own failures (8.6).
    fn arm(
        &mut self,
        scope: &Scope,
        kind: Kind,
        in_loop: bool,
        failure: FailureDecl,
    ) -> (String, Option<String>, Arm) {
        if failure.data == Some(kind) && self.random.chance(40) {
            let named = self.name("f");
            let read = Expression::Field {
                variable: named.clone(),
                field: "got".to_string(),
            };
            return (failure.name, Some(named), Arm::Instead(read));
        }
        let own = self.failing.clone().unwrap_or_default();
        let arm = match self.random.between(0, 9) {
            0 if self.in_main => Arm::Return(None),
            0 => match self.returns {
                Some(returns) => Arm::Return(Some(self.expression(scope, returns, 2))),
                None => Arm::Instead(self.literal(kind)),
            },
            1 if in_loop => Arm::Break,
            2 if !own.is_empty() => match self.random.pick(&own).cloned() {
                Some(raised) => {
                    let data = raised.data.map(|carried| self.literal(carried));
                    Arm::Fail(raised.name, data)
                }
                None => Arm::Instead(self.literal(kind)),
            },
            _ => Arm::Instead(self.literal(kind)),
        };
        (failure.name, None, arm)
    }

    /// Now and then a named set of failures, two or three, some carrying data, and its
    /// handlers (8.3, 8.7).
    fn failure_family(&mut self) {
        if !self.random.chance(40) {
            return;
        }
        let name = self.name("fs");
        let mut members = Vec::new();
        for _ in 0..self.random.between(2, 3) {
            let member = format!("{name}.{}", self.name("fm"));
            let data = self.random.chance(50).then(|| self.integer_kind());
            members.push(FailureDecl {
                name: member,
                data,
                family: Some(name.clone()),
            });
        }
        self.failure_family = Some(FailureFamily {
            name,
            members,
            handler: self.name("hs"),
        });
    }

    fn boolean(&mut self, scope: &Scope, depth: usize) -> Expression {
        let roll = self.random.between(0, 9);
        if depth == 0 || roll < 2 {
            return match self.variable_of(scope, Type::Bool) {
                Some(variable) => variable,
                None => Expression::Bool(self.random.chance(50)),
            };
        }
        if roll < 4 {
            let and = self.random.chance(50);
            let both = self.random.chance(25);
            let left = self.boolean(scope, depth - 1);
            let right = self.boolean(scope, depth - 1);
            return Expression::Logic {
                and,
                both,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        if roll == 4 {
            if self.random.chance(70) {
                if let Some(generic) = self.generic_boolean(scope, depth) {
                    return generic;
                }
            }
            return Expression::Not(Box::new(self.boolean(scope, depth - 1)));
        }
        if roll == 5 {
            if let Some(chain) = self.chain(scope, depth) {
                return chain;
            }
        }
        // Comparing two `simd` values gives a `bool` for each lane, read with `.%any_true()` or
        // `.%all_true()` (2.21).
        if roll == 6 {
            let found: Vec<(String, Type)> = scope
                .variables
                .iter()
                .filter(|(_, found)| matches!(found, Type::Simd(..)))
                .cloned()
                .collect();
            if let Some((name, Type::Simd(kind, lanes))) = self.random.pick(&found).cloned() {
                let comparison = *self
                    .random
                    .pick(&Comparison::ALL)
                    .unwrap_or(&Comparison::Equal);
                let right = self
                    .variable_of(scope, Type::Simd(kind, lanes))
                    .unwrap_or(Expression::Variable(name.clone()));
                let method = if self.random.chance(50) {
                    "any_true"
                } else {
                    "all_true"
                };
                return Expression::Method {
                    receiver: Box::new(Expression::Compare {
                        comparison,
                        left: Box::new(Expression::Variable(name)),
                        right: Box::new(right),
                    }),
                    method,
                };
            }
        }
        // A comparison needs a typed side: a variable of some number type.
        let numbers: Vec<(String, Type)> = scope
            .variables
            .iter()
            .filter(|(_, found)| {
                matches!(
                    found,
                    Type::Integer(_) | Type::Float(_) | Type::Unit(..) | Type::FloatUnit(_)
                )
            })
            .cloned()
            .collect();
        let Some((name, value_type)) = self.random.pick(&numbers).cloned() else {
            return Expression::Bool(self.random.chance(50));
        };
        let comparison = *self
            .random
            .pick(&Comparison::ALL)
            .unwrap_or(&Comparison::Equal);
        let right = self.expression(scope, value_type, depth - 1);
        Expression::Compare {
            comparison,
            left: Box::new(Expression::Variable(name)),
            right: Box::new(right),
        }
    }
}
