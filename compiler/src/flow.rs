//! The control-flow IR, and the ownership checks that run over it (D2, D129, D169, D185, D285).
//!
//! While checking, the checker records what each statement does to a variable's ownership as events, kept in the
//! checked program as `Statement::Flow`. Here each function is lowered to a graph of blocks of those events, and one
//! forward pass follows every path: what's been given away or finished, which `must_Give` duties are still open, and
//! which stored borrows were spoiled by a later use of what they borrow. The pass is small and plain on purpose, since
//! its lattice and transfer functions are what the Verus proofs cover (rule 13).

use std::collections::{BTreeMap, BTreeSet};

use crate::checked::{self, Block, ExpressionKind, LocalId, LoopId, OnFailure, Statement};
use crate::diagnostic::Diagnostic;
use crate::source::Span;

const GIVEN_WHY: &str = "a given value is dead afterwards (D2)";
const DUTY_WHY: &str = "a `must_Give` value is handed off with `Give` exactly once, or `finish` ends it (D2, D129)";
const BORROW_WHY: &str =
    "a stored `Lend.mut` means nothing else uses it meanwhile, and a `Lend` means it can't change (D2)";
const HELD_WHY: &str = "`.held` means nobody touches what it borrows for the whole block (D169)";
const PARTIAL_WHY: &str = "after a field is moved out, the value is incomplete (D185)";

/// A variable, or a field inside one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    pub local: LocalId,
    pub path: Vec<usize>,
}

/// How a variable came to hold its value, which decides whether it carries a duty and how errors name it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Local,
    Given,
    Lent,
    LentMut,
}

/// Where an event happened: the variable as written, the whole place, and the call or hand-off around it.
#[derive(Clone, Copy, Debug)]
pub struct Spot {
    /// `$b`.
    pub variable: Span,
    /// `$b.n`.
    pub place: Span,
    /// The call around it, or the variable when there's none: where a use is said to be.
    pub site: Span,
    /// The call around it, or the whole hand-off, such as `Give $s`: where a hand-off is said to be.
    pub marker: Span,
}

#[derive(Clone, Debug)]
pub enum Event {
    /// A variable gets a value.
    Bind {
        local: LocalId,
        name: String,
        declared: Span,
        must_give: bool,
        origin: Origin,
    },
    /// Reading it, or a `Copy` or `Lend` of it.
    Read {
        place: Place,
        spot: Spot,
    },
    Give {
        place: Place,
        spot: Spot,
    },
    /// `Lend.mut`, or a `%%` receiver.
    LendMut {
        place: Place,
        spot: Spot,
    },
    Change {
        place: Place,
        spot: Spot,
    },
    Finish {
        local: LocalId,
        spot: Spot,
    },
    /// A stored borrow made in `borrower`, from `sources`.
    Borrow {
        borrower: LocalId,
        sources: Vec<LocalId>,
        mutable: bool,
        held: bool,
    },
    /// What only happens on some paths, like the right side of `and`.
    Maybe(Vec<Event>),
    /// The block that declared these ends.
    ScopeEnd {
        locals: Vec<LocalId>,
    },
    /// Leaving the function, by `return` or `fail`, or leaving a loop's variables behind by `break`.
    Leave {
        span: Span,
        locals: Option<Vec<LocalId>>,
    },
}

/// A block of the graph: its events, and the blocks it can go to next.
#[derive(Clone, Debug, Default)]
pub struct Node {
    pub events: Vec<Event>,
    pub next: Vec<usize>,
}

/// One function as a control-flow graph. Block 0 is the start.
#[derive(Clone, Debug, Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
}

/// Lowers a function's checked body to its graph.
#[must_use]
pub fn lower(body: &Block) -> Graph {
    let mut lowering = Lowering { graph: Graph::default(), loops: Vec::new(), instead: Vec::new() };
    let start = lowering.node();
    lowering.block(body, Some(start));
    lowering.graph
}

struct Lowering {
    graph: Graph,
    /// Each open loop, with the block after it.
    loops: Vec<(LoopId, usize)>,
    /// Where `instead` goes, for each open set of arms.
    instead: Vec<usize>,
}

impl Lowering {
    fn node(&mut self) -> usize {
        self.graph.nodes.push(Node::default());
        self.graph.nodes.len() - 1
    }

    fn edge(&mut self, from: usize, to: usize) {
        if let Some(node) = self.graph.nodes.get_mut(from)
            && !node.next.contains(&to)
        {
            node.next.push(to);
        }
    }

    /// Lowers statements from `current`, giving the block where they end, or `None` once every path has left.
    fn block(&mut self, statements: &Block, mut current: Option<usize>) -> Option<usize> {
        for statement in statements {
            // Code after a `return` or `break` still gets a block, unreachable, so nothing in it is lost.
            let here = match current {
                Some(here) => here,
                None => self.node(),
            };
            current = self.statement(statement, here);
        }
        current
    }

    fn statement(&mut self, statement: &Statement, here: usize) -> Option<usize> {
        match statement {
            Statement::Flow(events) => Some(self.events(events, here)),
            Statement::If { condition, then, otherwise } => {
                let here = self.expression(condition, here);
                let then_start = self.node();
                let otherwise_start = self.node();
                self.edge(here, then_start);
                self.edge(here, otherwise_start);
                let then_end = self.block(then, Some(then_start));
                let otherwise_end = self.block(otherwise, Some(otherwise_start));
                self.join(&[then_end, otherwise_end])
            }
            Statement::Loop { id, body, .. } => {
                let head = self.node();
                let after = self.node();
                self.edge(here, head);
                self.loops.push((*id, after));
                if let Some(end) = self.block(body, Some(head)) {
                    self.edge(end, head);
                }
                self.loops.pop();
                Some(after)
            }
            Statement::Break(id) => {
                if let Some(&(_, after)) = self.loops.iter().rev().find(|(each, _)| each == id) {
                    self.edge(here, after);
                }
                None
            }
            Statement::Return(value) => {
                if let Some(value) = value {
                    self.expression(value, here);
                }
                None
            }
            Statement::Instead { value, .. } => {
                let here = self.expression(value, here);
                if let Some(&after) = self.instead.last() {
                    self.edge(here, after);
                }
                None
            }
            Statement::Let { value, .. }
            | Statement::AliasValue { value, .. }
            | Statement::Set { value, .. }
            | Statement::SetField { value, .. }
            | Statement::Evaluate(value) => Some(self.expression(value, here)),
            Statement::Check { condition, .. } | Statement::CheckAtSite { condition, .. } => {
                Some(self.expression(condition, here))
            }
            Statement::OpenWriter { .. }
            | Statement::Alias { .. }
            | Statement::Print { .. }
            | Statement::Flush { .. }
            | Statement::Done { .. } => Some(here),
        }
    }

    /// Adds events to the block, branching around those that only happen on some paths.
    fn events(&mut self, events: &[Event], mut here: usize) -> usize {
        for event in events {
            if let Event::Maybe(inner) = event {
                let branch = self.node();
                let after = self.node();
                self.edge(here, branch);
                self.edge(here, after);
                let end = self.events(inner, branch);
                self.edge(end, after);
                here = after;
            } else if let Some(node) = self.graph.nodes.get_mut(here) {
                node.events.push(event.clone());
            }
        }
        here
    }

    fn join(&mut self, ends: &[Option<usize>]) -> Option<usize> {
        let live: Vec<usize> = ends.iter().flatten().copied().collect();
        if live.is_empty() {
            return None;
        }
        let after = self.node();
        for end in live {
            self.edge(end, after);
        }
        Some(after)
    }

    /// The `otherwise:` arms inside an expression branch off and come back after it.
    fn expression(&mut self, expression: &checked::Expression, here: usize) -> usize {
        let mut here = here;
        for inner in children(expression) {
            here = self.expression(inner, here);
        }
        if let ExpressionKind::Handle { on_failure, .. } = &expression.kind
            && let OnFailure::Arms { arms, .. } = on_failure.as_ref()
        {
            let after = self.node();
            self.edge(here, after);
            self.instead.push(after);
            for (_, arm) in arms {
                let start = self.node();
                self.edge(here, start);
                if let Some(end) = self.block(arm, Some(start)) {
                    self.edge(end, after);
                }
            }
            self.instead.pop();
            here = after;
        }
        here
    }
}

/// The expressions directly inside one.
fn children(expression: &checked::Expression) -> Vec<&checked::Expression> {
    use ExpressionKind as Kind;
    match &expression.kind {
        Kind::Arithmetic { left, right, .. }
        | Kind::FloatArithmetic { left, right, .. }
        | Kind::Compare { left, right, .. }
        | Kind::And(left, right)
        | Kind::Or(left, right) => vec![left, right],
        Kind::Negate { operand: inner, .. }
        | Kind::Not(inner)
        | Kind::BitNot(inner)
        | Kind::TextOf(inner)
        | Kind::Tag(inner)
        | Kind::Convert(inner)
        | Kind::Field { base: inner, .. }
        | Kind::CaseField { base: inner, .. }
        | Kind::FromIeee { value: inner, .. }
        | Kind::FitFloat { value: inner, .. }
        | Kind::Fit { value: inner, .. } => vec![inner],
        Kind::Handle { call, .. } => vec![call],
        Kind::Call { arguments, .. } => arguments
            .iter()
            .filter_map(|argument| match argument {
                checked::Argument::Value(value) | checked::Argument::Temporary(value) => Some(value),
                _ => None,
            })
            .collect(),
        Kind::Build(fields) | Kind::BuildCase { fields, .. } => fields.iter().collect(),
        Kind::Bind { value, body, .. } => vec![value, body],
        _ => Vec::new(),
    }
}

/// What a variable holds on the paths reaching a point.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Holding {
    Alive,
    /// Given away or finished, on every path, or on some when `maybe`.
    Gone {
        finished: bool,
        at: Span,
        maybe: bool,
    },
    /// A field was moved out (D185).
    Partial {
        at: Span,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Loan {
    sources: Vec<LocalId>,
    mutable: bool,
    held: bool,
    /// Where something used what it borrows, after which it can't be used.
    spoiled: Option<Span>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct State {
    holding: BTreeMap<LocalId, Holding>,
    loans: BTreeMap<LocalId, Loan>,
}

/// What's known about each variable that doesn't change along a path.
#[derive(Clone, Debug)]
struct Variable {
    name: String,
    declared: Span,
    must_give: bool,
    origin: Origin,
}

fn join_holding(a: &Holding, b: &Holding) -> Holding {
    match (a, b) {
        (Holding::Alive, Holding::Alive) => Holding::Alive,
        (Holding::Gone { finished, at, maybe }, Holding::Gone { maybe: other, .. }) => {
            Holding::Gone { finished: *finished, at: *at, maybe: *maybe || *other }
        }
        (Holding::Gone { finished, at, .. }, _) | (_, Holding::Gone { finished, at, .. }) => {
            Holding::Gone { finished: *finished, at: *at, maybe: true }
        }
        (Holding::Partial { at }, _) | (_, Holding::Partial { at }) => Holding::Partial { at: *at },
    }
}

fn join(a: &State, b: &State) -> State {
    let mut joined = a.clone();
    for (local, holding) in &b.holding {
        let merged = match a.holding.get(local) {
            Some(mine) => join_holding(mine, holding),
            None => holding.clone(),
        };
        joined.holding.insert(*local, merged);
    }
    for (local, loan) in &b.loans {
        match joined.loans.get_mut(local) {
            Some(mine) => {
                if mine.spoiled.is_none() {
                    mine.spoiled = loan.spoiled;
                }
            }
            None => {
                joined.loans.insert(*local, loan.clone());
            }
        }
    }
    joined
}

/// Runs the ownership checks over a function's graph.
#[must_use]
pub fn check(graph: &Graph) -> Vec<Diagnostic> {
    let mut variables = BTreeMap::new();
    for node in &graph.nodes {
        for event in &node.events {
            if let Event::Bind { local, name, declared, must_give, origin } = event {
                variables.insert(
                    *local,
                    Variable { name: name.clone(), declared: *declared, must_give: *must_give, origin: *origin },
                );
            }
        }
    }
    // Every block's state on entry, found by going round until nothing changes. The lattice is finite, since every
    // span in it comes from the function, so this ends.
    let mut entry: Vec<Option<State>> = vec![None; graph.nodes.len()];
    if !graph.nodes.is_empty() {
        entry[0] = Some(State::default());
    }
    let mut work: BTreeSet<usize> = BTreeSet::from([0]);
    let mut quiet = Checker { variables: &variables, errors: Vec::new(), report: false };
    while let Some(index) = work.pop_first() {
        let Some(state) = entry[index].clone() else { continue };
        let out = quiet.run(&graph.nodes[index].events, state);
        for &next in &graph.nodes[index].next {
            let merged = match &entry[next] {
                Some(old) => join(old, &out),
                None => out.clone(),
            };
            if entry[next].as_ref() != Some(&merged) {
                entry[next] = Some(merged);
                work.insert(next);
            }
        }
    }
    let mut checker = Checker { variables: &variables, errors: Vec::new(), report: true };
    for (index, node) in graph.nodes.iter().enumerate() {
        if let Some(state) = entry[index].clone() {
            checker.run(&node.events, state);
        }
    }
    let mut seen = BTreeSet::new();
    checker.errors.retain(|error| seen.insert((error.span.start, error.span.end, error.what.clone())));
    checker.errors
}

struct Checker<'a> {
    variables: &'a BTreeMap<LocalId, Variable>,
    errors: Vec<Diagnostic>,
    report: bool,
}

impl Checker<'_> {
    fn name(&self, local: LocalId) -> String {
        self.variables.get(&local).map_or_else(String::new, |variable| variable.name.clone())
    }

    fn say(&mut self, diagnostic: Diagnostic) {
        if self.report {
            self.errors.push(diagnostic);
        }
    }

    fn run(&mut self, events: &[Event], mut state: State) -> State {
        for event in events {
            self.event(event, &mut state);
        }
        state
    }

    fn event(&mut self, event: &Event, state: &mut State) {
        match event {
            Event::Bind { local, .. } => {
                state.holding.insert(*local, Holding::Alive);
            }
            Event::Read { place, spot } => {
                if self.usable(state, place, spot, false) {
                    self.spoil(state, place.local, spot, true);
                }
            }
            Event::LendMut { place, spot } | Event::Change { place, spot } => {
                if self.usable(state, place, spot, false) {
                    self.spoil(state, place.local, spot, false);
                }
            }
            Event::Give { place, spot } => {
                if self.usable(state, place, spot, false) {
                    self.spoil(state, place.local, spot, false);
                    let holding = if place.path.is_empty() {
                        Holding::Gone { finished: false, at: spot.marker, maybe: false }
                    } else {
                        Holding::Partial { at: spot.marker }
                    };
                    state.holding.insert(place.local, holding);
                }
            }
            Event::Finish { local, spot } => {
                let place = Place { local: *local, path: Vec::new() };
                if self.usable(state, &place, spot, true) {
                    self.spoil(state, *local, spot, false);
                    state.holding.insert(*local, Holding::Gone { finished: true, at: spot.site, maybe: false });
                }
            }
            Event::Borrow { borrower, sources, mutable, held } => {
                state.loans.insert(
                    *borrower,
                    Loan { sources: sources.clone(), mutable: *mutable, held: *held, spoiled: None },
                );
            }
            Event::ScopeEnd { locals } => {
                for local in locals {
                    state.loans.remove(local);
                }
            }
            Event::Leave { span, locals } => {
                self.leave(state, *span, locals.as_deref());
                // What a `break` or `instead` leaves behind is gone after it.
                for local in locals.iter().flatten() {
                    state.holding.remove(local);
                }
            }
            Event::Maybe(_) => {}
        }
    }

    /// Whether the place can be used here, saying why not if it can't.
    fn usable(&mut self, state: &State, place: &Place, spot: &Spot, finishing: bool) -> bool {
        let name = self.name(place.local);
        if let Some(loan) = state.loans.get(&place.local)
            && let Some(at) = loan.spoiled
        {
            self.say(
                Diagnostic::refused(
                    spot.variable,
                    format!("`{name}` can't be used after what it borrows was used"),
                    BORROW_WHY,
                )
                .with_because(at, "what it borrows was used here"),
            );
            return false;
        }
        match state.holding.get(&place.local) {
            Some(Holding::Gone { finished: false, at, .. }) => {
                let span = if finishing { spot.site } else { spot.site };
                self.say(
                    Diagnostic::refused(span, format!("`{name}` was already given away"), GIVEN_WHY)
                        .with_because(*at, format!("`{name}` was given away here")),
                );
                false
            }
            Some(Holding::Gone { finished: true, at, .. }) => {
                if finishing {
                    self.say(
                        Diagnostic::refused(
                            spot.site,
                            format!("`{name}` was already finished"),
                            "a `must_Give` duty ends once (D2, D129)",
                        )
                        .with_because(*at, format!("`{name}` was finished here")),
                    );
                } else {
                    self.say(Diagnostic::not_designed(
                        spot.site,
                        format!("using `{name}` after `finish`"),
                        "not decided yet, and the standard doesn't say",
                    ));
                }
                false
            }
            Some(Holding::Partial { at }) if place.path.is_empty() && !finishing => {
                self.say(
                    Diagnostic::refused(
                        spot.variable,
                        format!(
                            "a field of `{name}` was moved out, so all that's left is moving out another, or `finish`"
                        ),
                        PARTIAL_WHY,
                    )
                    .with_because(*at, "it was moved out here"),
                );
                false
            }
            _ => true,
        }
    }

    /// A use of `local` spoils the stored borrows of it that it conflicts with: any use, for one lent to change,
    /// and anything but a read, for one lent to read.
    fn spoil(&mut self, state: &mut State, local: LocalId, spot: &Spot, reading: bool) {
        let borrowers: Vec<LocalId> = state
            .loans
            .iter()
            .filter(|(_, loan)| loan.sources.contains(&local) && loan.spoiled.is_none() && (loan.mutable || !reading))
            .map(|(borrower, _)| *borrower)
            .collect();
        for borrower in borrowers {
            let Some(loan) = state.loans.get_mut(&borrower) else { continue };
            if loan.held {
                let holder = self.name(borrower);
                let source = self.name(local);
                let declared = self.variables.get(&borrower).map_or(spot.place, |variable| variable.declared);
                self.say(
                    Diagnostic::refused(
                        spot.place,
                        format!("`{holder}` holds `{source}` until the end of its block"),
                        HELD_WHY,
                    )
                    .with_because(declared, format!("`{holder}` holds it from here")),
                );
            } else {
                loan.spoiled = Some(spot.place);
            }
        }
    }

    /// Leaving the function: every `must_Give` duty has to be done on every path.
    fn leave(&mut self, state: &State, span: Span, locals: Option<&[LocalId]>) {
        for (local, variable) in self.variables {
            if !variable.must_give || !matches!(variable.origin, Origin::Local | Origin::Given) {
                continue;
            }
            if locals.is_some_and(|locals| !locals.contains(local)) {
                continue;
            }
            let name = &variable.name;
            match state.holding.get(local) {
                Some(Holding::Alive | Holding::Partial { .. }) => {
                    let note = if variable.origin == Origin::Given {
                        format!("`{name}` was given to this function here")
                    } else {
                        format!("`{name}` was made here")
                    };
                    self.say(
                        Diagnostic::refused(
                            span,
                            format!("this leaves with `{name}` never given away or finished"),
                            DUTY_WHY,
                        )
                        .with_because(variable.declared, note),
                    );
                }
                Some(Holding::Gone { at, maybe: true, .. }) => {
                    self.say(
                        Diagnostic::refused(
                            variable.declared,
                            format!("`{name}` isn't given away or finished on every path"),
                            DUTY_WHY,
                        )
                        .with_because(*at, format!("`{name}` went here, on only one path")),
                    );
                }
                _ => {}
            }
        }
    }
}
